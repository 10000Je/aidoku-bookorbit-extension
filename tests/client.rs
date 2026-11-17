use std::collections::VecDeque;

use bookorbit_aidoku_source::{
    api::{
        Api, MAX_JSON_BYTES, Method, RequestSpec, Response, Transport, parse_json, query_body,
        retry_delay,
    },
    auth::ApiKey,
    config::Config,
    error::{Error, Result, check_status},
    models::{
        BookDetail, BookId, BooksPage, FileId, LibraryId, MangaKey, PageCount, SeriesKey,
        listing_library, merged_tags,
    },
    reader::{
        MAX_PAGES, MAX_SERIES_BOOKS, chapters, page_urls, parse_series_chapter, series_chapters,
    },
};
use serde_json::{Value, json};

const PAGE: &str = include_str!("fixtures/books-page.json");
const DETAIL: &str = include_str!("fixtures/book-detail.json");
const COUNT: &str = include_str!("fixtures/page-count.json");
const EMPTY: &str = include_str!("fixtures/empty-page.json");
const TEST_KEY: &str = "bok_synthetic_test_value";
const SERIES: &str = include_str!("fixtures/series-books-page.json");
const CB7_DETAIL: &str = include_str!("fixtures/cb7-book-detail.json");

fn scope() -> SeriesKey {
    match MangaKey::parse("series:3:7").unwrap() {
        MangaKey::Series(scope) => scope,
        _ => unreachable!(),
    }
}

fn series_detail() -> String {
    let mut detail: Value = serde_json::from_str(DETAIL).unwrap();
    detail["libraryId"] = json!(3);
    detail["seriesId"] = json!(7);
    detail.to_string()
}

fn config() -> Config {
    Config::new("https://books.example.com/shelf/", TEST_KEY).unwrap()
}
fn book() -> BookId {
    BookId::parse("42").unwrap()
}
fn file() -> FileId {
    FileId::parse("501").unwrap()
}

struct Reply {
    status: u16,
    body: String,
    retry_after: Option<String>,
    final_url: Option<String>,
}

impl Reply {
    fn new(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.into(),
            retry_after: None,
            final_url: None,
        }
    }
    fn retry(mut self, value: &str) -> Self {
        self.retry_after = Some(value.into());
        self
    }
}

struct Recorded {
    method: Method,
    url: String,
    header: (String, String),
    body: Option<String>,
}

#[derive(Default)]
struct Mock {
    replies: VecDeque<Result<Reply>>,
    requests: Vec<Recorded>,
    sleeps: Vec<u32>,
}

impl Mock {
    fn replies(replies: Vec<Reply>) -> Self {
        Self {
            replies: replies.into_iter().map(Ok).collect(),
            ..Default::default()
        }
    }
}

impl Transport for Mock {
    fn execute(&mut self, request: &RequestSpec<'_>) -> Result<Response> {
        let (name, value) = request.key.header();
        self.requests.push(Recorded {
            method: request.method,
            url: request.url.clone(),
            header: (name.into(), value),
            body: request.body.clone(),
        });
        let reply = self
            .replies
            .pop_front()
            .expect("unexpected extra request")?;
        Ok(Response {
            status: reply.status,
            body: reply.body.into_bytes(),
            retry_after: reply.retry_after,
            final_url: Some(reply.final_url.unwrap_or_else(|| request.url.clone())),
        })
    }
    fn sleep(&mut self, seconds: u32) {
        self.sleeps.push(seconds);
    }
}

#[test]
fn normalizes_root_proxy_path_default_port_and_ipv6() {
    for (input, expected) in [
        (
            " https://BOOKS.example.com:443/// ",
            "https://books.example.com",
        ),
        (
            "https://books.example.com/shelf///",
            "https://books.example.com/shelf",
        ),
        ("http://[::1]:3000/library/", "http://[::1]:3000/library"),
        (
            "https://books.example.com/a%20b/",
            "https://books.example.com/a%20b",
        ),
    ] {
        let config = Config::new(input, TEST_KEY).unwrap();
        assert_eq!(config.base_url(), expected);
        assert_eq!(
            config.endpoint("books/42"),
            format!("{expected}/api/v1/books/42")
        );
    }
}

#[test]
fn rejects_invalid_or_ambiguous_server_urls() {
    for input in [
        "books.example.com",
        "https:/books.example.com",
        "ftp://books.example.com",
        "file:///tmp/books",
        "https://user:password@books.example.com",
        "https://books.example.com?key=secret",
        "https://books.example.com/#fragment",
        "https://books.example.com/api/v1/",
        "https://books.example.com/shelf/api/v1/opds",
        "https://books.example.com/opds/",
        "https://books.example.com/a/../b",
        "https://books.example.com/%2e%2e",
        "https://books.example.com/a%2fb",
        "https://books.example.com/a\\b",
        "https://books.exa\nmple.com",
        "https://books.example.com:99999",
    ] {
        assert!(Config::new(input, TEST_KEY).is_err(), "accepted {input:?}");
    }
}

#[test]
fn missing_settings_and_header_injection_fail_without_exposing_key() {
    assert_eq!(Config::new("", TEST_KEY).unwrap_err(), Error::MissingConfig);
    assert_eq!(
        Config::new("https://books.example.com", "").unwrap_err(),
        Error::MissingConfig
    );
    for key in ["secret\r\nX-Injected: value", "has space", "한글키"] {
        assert_eq!(ApiKey::new(key).unwrap_err(), Error::InvalidKey);
    }
    assert_eq!(
        ApiKey::new(&"x".repeat(513)).unwrap_err(),
        Error::InvalidKey
    );
    assert!(!format!("{:?}", config()).contains(TEST_KEY));
    let request = RequestSpec {
        method: Method::Get,
        url: config().endpoint("books/42"),
        key: &config().key,
        body: None,
    };
    assert!(!format!("{request:?}").contains(TEST_KEY));
}

#[test]
fn creates_exactly_one_bearer_header_and_trims_paste_whitespace() {
    let key = ApiKey::new(&format!(" {TEST_KEY} \n")).unwrap();
    assert_eq!(
        key.header(),
        ("Authorization", format!("Bearer {TEST_KEY}"))
    );
}

#[test]
fn image_guard_accepts_only_same_origin_and_mount_image_routes() {
    let config = config();
    for path in [
        "books/42/thumbnail",
        "books/42/cover",
        "cbz/files/501/pages/0",
        "cbz/files/501/pages/2",
    ] {
        let url = config.endpoint(path);
        assert_eq!(config.image_url(&url).unwrap(), url);
    }
    for url in [
        "https://evil.example/shelf/api/v1/books/42/cover",
        "https://books.example.com.evil.example/shelf/api/v1/books/42/cover",
        "https://books.example.com@evil.example/shelf/api/v1/books/42/cover",
        "https://user@books.example.com/shelf/api/v1/books/42/cover",
        "http://books.example.com/shelf/api/v1/books/42/cover",
        "https://books.example.com:444/shelf/api/v1/books/42/cover",
        "https://books.example.com/api/v1/books/42/cover",
        "https://books.example.com/shelf2/api/v1/books/42/cover",
        "https://books.example.com/shelf/api/v1/books/42/cover?token=secret",
        "https://books.example.com/shelf/api/v1/books/42/cover#fragment",
        "https://books.example.com/shelf/api/v1/books/42/../43/cover",
        "https://books.example.com/shelf/api/v1/books/%2e%2e/cover",
        "https://books.example.com/shelf/api/v1/auth/me",
        "https://books.example.com/shelf/api/v1/cbz/files/501/pages/-1",
        "https://books.example.com/shelf/api/v1/books/0/cover",
    ] {
        assert_eq!(
            config.image_url(url),
            Err(Error::UnsafeImage),
            "accepted {url}"
        );
    }
}

#[test]
fn search_preserves_query_and_adds_required_filters_and_zero_based_page() {
    let (page, body) = query_body(Some("  만화 \"제목\"  "), 2).unwrap();
    assert_eq!(page, 1);
    assert_eq!(body["q"], "만화 \"제목\"");
    assert_eq!(body["pagination"], json!({"page":1,"size":40}));
    assert_eq!(
        body["filter"]["rules"][0]["value"],
        json!(["cbz", "cbr", "cb7"])
    );
    assert_eq!(body["filter"]["rules"][1]["operator"], "isPresent");
    assert_eq!(body["collapseSeries"], true);
    assert_eq!(body["collapseLinked"], false);
    assert_eq!(query_body(None, 1).unwrap().1["q"], "");
}

#[test]
fn rejects_negative_zero_overflow_and_deep_pages() {
    for page in [i32::MIN, -1, 0, 25_002, i32::MAX] {
        assert_eq!(query_body(None, page), Err(Error::InvalidPage));
    }
    assert_eq!(query_body(None, 25_001).unwrap().0, 25_000);
}

#[test]
fn enforces_server_utf16_search_length_without_splitting_unicode() {
    assert!(query_body(Some(&"가".repeat(200)), 1).is_ok());
    assert_eq!(
        query_body(Some(&"가".repeat(201)), 1),
        Err(Error::QueryTooLong)
    );
    assert!(query_body(Some(&"📚".repeat(100)), 1).is_ok());
    assert_eq!(
        query_body(Some(&"📚".repeat(101)), 1),
        Err(Error::QueryTooLong)
    );
}

#[test]
fn converts_real_contract_list_shape_and_nullable_metadata() {
    let result = parse_json::<BooksPage>(PAGE.as_bytes())
        .unwrap()
        .into_search(&config(), 0)
        .unwrap();
    assert_eq!(result.entries.len(), 2);
    assert_eq!(result.entries[0].book_id.key(), "42");
    assert_eq!(result.entries[0].title, "테스트 만화");
    assert_eq!(result.entries[0].authors, ["가상 작가"]);
    assert_eq!(result.entries[0].tags, ["만화", "완결"]);
    assert_eq!(
        result.entries[0].cover.as_deref(),
        Some("https://books.example.com/shelf/api/v1/books/42/thumbnail")
    );
    assert_eq!(result.entries[1].title, "도서 43");
    assert!(result.entries[1].cover.is_none());
    assert!(result.entries[1].authors.is_empty());
    assert!(result.has_next_page);
}

#[test]
fn handles_empty_library_and_last_page() {
    let empty = parse_json::<BooksPage>(EMPTY.as_bytes())
        .unwrap()
        .into_search(&config(), 0)
        .unwrap();
    assert!(empty.entries.is_empty());
    assert!(!empty.has_next_page);
    let mut value: Value = serde_json::from_str(PAGE).unwrap();
    value["total"] = json!(2);
    assert!(
        !parse_json::<BooksPage>(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .into_search(&config(), 0)
            .unwrap()
            .has_next_page
    );
}

#[test]
fn pagination_uses_server_total_even_if_defensive_filter_excludes_entries() {
    let mut value: Value = serde_json::from_str(PAGE).unwrap();
    value["items"][0]["status"] = json!("missing");
    value["items"][1]["files"][0]["role"] = json!("sidecar");
    let result = parse_json::<BooksPage>(&serde_json::to_vec(&value).unwrap())
        .unwrap()
        .into_search(&config(), 0)
        .unwrap();
    assert!(result.entries.is_empty());
    assert!(result.has_next_page);
}

#[test]
fn rejects_inconsistent_list_envelopes_and_duplicate_ids() {
    for (field, bad) in [
        ("page", json!(1)),
        ("size", json!(20)),
        ("total", json!(0)),
        ("items", json!([])),
    ] {
        let mut value: Value = serde_json::from_str(PAGE).unwrap();
        value[field] = bad;
        assert_eq!(
            parse_json::<BooksPage>(&serde_json::to_vec(&value).unwrap())
                .unwrap()
                .into_search(&config(), 0),
            Err(Error::InvalidResponse)
        );
    }
    let mut value: Value = serde_json::from_str(PAGE).unwrap();
    value["items"][1]["id"] = json!(42);
    assert_eq!(
        parse_json::<BooksPage>(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .into_search(&config(), 0),
        Err(Error::InvalidResponse)
    );
}

#[test]
fn chapter_keys_are_file_ids_and_stable_when_server_order_changes() {
    let mut detail: BookDetail = parse_json(DETAIL.as_bytes()).unwrap();
    let expected = chapters(&detail).unwrap();
    assert_eq!(
        expected.iter().map(|c| c.file_id.key()).collect::<Vec<_>>(),
        ["502", "501"]
    );
    assert_eq!(
        expected.iter().map(|c| c.number).collect::<Vec<_>>(),
        [2, 1]
    );
    detail.files.reverse();
    assert_eq!(chapters(&detail).unwrap(), expected);
    assert_eq!(detail.id.key(), "42");
}

#[test]
fn chapter_filter_accepts_cbz_cbr_primary_content_only() {
    let mut detail: BookDetail = parse_json(DETAIL.as_bytes()).unwrap();
    detail.files[0].format = Some("CBR".into());
    detail.files[1].filename = None;
    let result = chapters(&detail).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[1].title, "만화 파일 501");
    detail.files.clear();
    assert_eq!(chapters(&detail), Err(Error::EmptyComic));
}

#[test]
fn duplicate_chapter_ids_fail_instead_of_creating_colliding_chapters() {
    let mut detail: Value = serde_json::from_str(DETAIL).unwrap();
    detail["files"][0]["id"] = json!(501);
    assert_eq!(
        chapters(&parse_json::<BookDetail>(&serde_json::to_vec(&detail).unwrap()).unwrap()),
        Err(Error::InvalidResponse)
    );
}

#[test]
fn first_and_last_pages_use_file_id_and_zero_based_order() {
    let count: PageCount = parse_json(COUNT.as_bytes()).unwrap();
    let pages = page_urls(&config(), file(), count.page_count).unwrap();
    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0], config().endpoint("cbz/files/501/pages/0"));
    assert_eq!(pages[2], config().endpoint("cbz/files/501/pages/2"));
    assert_eq!(page_urls(&config(), file(), 1).unwrap().len(), 1);
    assert_eq!(page_urls(&config(), file(), 0), Err(Error::EmptyComic));
    assert_eq!(
        page_urls(&config(), file(), MAX_PAGES + 1),
        Err(Error::TooManyPages)
    );
}

#[test]
fn rejects_malformed_json_and_changed_required_fields() {
    for body in [
        "<html>login</html>",
        "null",
        "{}",
        "{\"pageCount\":-1}",
        "{\"pageCount\":\"3\"}",
        "{\"pages\":3}",
    ] {
        assert!(
            parse_json::<PageCount>(body.as_bytes()).is_err(),
            "accepted {body}"
        );
    }
    for field in ["id", "status", "files"] {
        let mut detail: Value = serde_json::from_str(DETAIL).unwrap();
        detail.as_object_mut().unwrap().remove(field);
        assert!(parse_json::<BookDetail>(&serde_json::to_vec(&detail).unwrap()).is_err());
    }
    assert!(parse_json::<Value>(&vec![b' '; MAX_JSON_BYTES + 1]).is_err());
}

#[test]
fn validates_typed_ids_without_confusing_book_and_file_keys() {
    for id in ["", "0", "-1", "+1", "1/2", "1?key=x", "abc", "2147483648"] {
        assert_eq!(BookId::parse(id), Err(Error::InvalidId));
        assert_eq!(FileId::parse(id), Err(Error::InvalidId));
    }
    assert_eq!(BookId::parse("0042").unwrap(), book());
}

#[test]
fn mock_search_checks_method_authentication_body_and_proxy_path() {
    let mut mock = Mock::replies(vec![Reply::new(200, PAGE)]);
    let result = Api::new(&config(), &mut mock)
        .search(Some("만화"), 1)
        .unwrap();
    assert_eq!(result.entries.len(), 2);
    let request = &mock.requests[0];
    assert_eq!(request.method, Method::Post);
    assert_eq!(request.url, config().endpoint("books/query"));
    assert_eq!(
        request.header,
        ("Authorization".into(), format!("Bearer {TEST_KEY}"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(request.body.as_deref().unwrap()).unwrap()["q"],
        "만화"
    );
    assert!(!request.url.contains(TEST_KEY));
}

#[test]
fn page_flow_validates_book_file_relationship_before_count_request() {
    let mut mock = Mock::replies(vec![Reply::new(200, DETAIL), Reply::new(200, COUNT)]);
    assert_eq!(
        Api::new(&config(), &mut mock)
            .pages(book(), file())
            .unwrap()
            .len(),
        3
    );
    assert_eq!(mock.requests.len(), 2);
    assert_eq!(mock.requests[0].url, config().endpoint("books/42"));
    assert_eq!(
        mock.requests[1].url,
        config().endpoint("cbz/files/501/pages")
    );
    assert!(
        mock.requests
            .iter()
            .all(|r| r.method == Method::Get && r.body.is_none())
    );
    let mut mock = Mock::replies(vec![Reply::new(200, DETAIL)]);
    assert_eq!(
        Api::new(&config(), &mut mock).pages(book(), FileId::parse("999").unwrap()),
        Err(Error::NotFound)
    );
    assert_eq!(mock.requests.len(), 1);
}

#[test]
fn changed_detail_id_and_missing_books_fail() {
    let mut value: Value = serde_json::from_str(DETAIL).unwrap();
    value["id"] = json!(43);
    let mut mock = Mock::replies(vec![Reply::new(200, &value.to_string())]);
    assert!(matches!(
        Api::new(&config(), &mut mock).book(book()),
        Err(Error::InvalidResponse)
    ));
    value["id"] = json!(42);
    value["status"] = json!("missing");
    let mut mock = Mock::replies(vec![Reply::new(200, &value.to_string())]);
    assert!(matches!(
        Api::new(&config(), &mut mock).book(book()),
        Err(Error::NotFound)
    ));
}

#[test]
fn errors_map_without_echoing_server_body_or_credentials() {
    for (status, error) in [
        (401, Error::Unauthorized),
        (403, Error::Forbidden),
        (404, Error::NotFound),
        (410, Error::NotFound),
        (422, Error::CorruptArchive),
        (500, Error::Http(500)),
        (302, Error::Redirect),
    ] {
        let mut mock = Mock::replies(vec![Reply::new(status, TEST_KEY)]);
        assert_eq!(
            Api::new(&config(), &mut mock).search(None, 1),
            Err(error.clone())
        );
        assert_eq!(mock.requests.len(), 1);
        assert!(mock.sleeps.is_empty());
        assert!(!error.to_string().contains(TEST_KEY));
        assert_eq!(check_status(status), Err(error));
    }
}

#[test]
fn rate_limit_retries_twice_with_backoff_then_recovers() {
    let mut mock = Mock::replies(vec![
        Reply::new(429, ""),
        Reply::new(429, "").retry("3"),
        Reply::new(200, EMPTY),
    ]);
    assert!(
        Api::new(&config(), &mut mock)
            .search(None, 1)
            .unwrap()
            .entries
            .is_empty()
    );
    assert_eq!(mock.sleeps, [1, 3]);
    assert_eq!(mock.requests.len(), 3);
    assert!(mock.requests.windows(2).all(|p| p[0].body == p[1].body));
}

#[test]
fn rate_limit_exhaustion_is_bounded() {
    let mut mock = Mock::replies(vec![
        Reply::new(429, ""),
        Reply::new(429, ""),
        Reply::new(429, ""),
    ]);
    assert_eq!(
        Api::new(&config(), &mut mock).search(None, 1),
        Err(Error::RateLimited)
    );
    assert_eq!(mock.requests.len(), 3);
    assert_eq!(mock.sleeps, [1, 2]);
}

#[test]
fn respects_retry_after_without_early_or_unbounded_retry() {
    assert_eq!(retry_delay(0, Some("0")), Some(1));
    assert_eq!(retry_delay(1, Some("1")), Some(2));
    assert_eq!(retry_delay(0, Some("10")), Some(10));
    assert_eq!(retry_delay(2, None), None);
    for header in [
        "11",
        "600",
        "-1",
        "invalid",
        "Wed, 18 Nov 2026 00:00:00 GMT",
    ] {
        let mut mock = Mock::replies(vec![Reply::new(429, "").retry(header)]);
        assert_eq!(
            Api::new(&config(), &mut mock).search(None, 1),
            Err(Error::RateLimited)
        );
        assert_eq!(mock.requests.len(), 1);
        assert!(mock.sleeps.is_empty());
    }
}

#[test]
fn network_and_parsing_failures_do_not_retry() {
    let mut mock = Mock {
        replies: VecDeque::from([Err(Error::Network)]),
        ..Default::default()
    };
    assert_eq!(
        Api::new(&config(), &mut mock).search(None, 1),
        Err(Error::Network)
    );
    assert_eq!(mock.requests.len(), 1);
    let mut mock = Mock::replies(vec![Reply::new(200, "{\"key\":\"private\"}")]);
    assert_eq!(
        Api::new(&config(), &mut mock).search(None, 1),
        Err(Error::InvalidResponse)
    );
    assert_eq!(mock.requests.len(), 1);
}

#[test]
fn detects_redirected_response_without_retrying_credentials() {
    let mut reply = Reply::new(200, PAGE);
    reply.final_url = Some("https://evil.example/api/v1/books/query".into());
    let mut mock = Mock::replies(vec![reply]);
    assert_eq!(
        Api::new(&config(), &mut mock).search(None, 1),
        Err(Error::Redirect)
    );
    assert_eq!(mock.requests.len(), 1);
}

#[test]
fn invalid_query_makes_no_network_request() {
    let mut mock = Mock::default();
    assert_eq!(
        Api::new(&config(), &mut mock).search(None, -1),
        Err(Error::InvalidPage)
    );
    assert!(mock.requests.is_empty());
}

#[test]
fn genre_tags_are_deduplicated_and_bounded_to_sdk_limit() {
    let tags: Vec<_> = (0..300).map(|n| n.to_string()).collect();
    assert_eq!(merged_tags(&tags, &tags).len(), 255);
}

#[test]
fn libraries_keep_server_names_and_display_order_and_use_bearer_auth() {
    let mut mock = Mock::replies(vec![Reply::new(
        200,
        include_str!("fixtures/libraries.json"),
    )]);
    let libraries = Api::new(&config(), &mut mock).libraries().unwrap();
    assert_eq!(
        libraries
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["만화", "텍스트 소설"]
    );
    assert_eq!(mock.requests[0].method, Method::Get);
    assert_eq!(mock.requests[0].url, config().endpoint("libraries"));
    assert_eq!(mock.requests[0].header.1, format!("Bearer {TEST_KEY}"));
    assert_eq!(listing_library("comics"), Ok(None));
    assert_eq!(
        listing_library("library:3"),
        Ok(Some(LibraryId::parse("3").unwrap()))
    );
    for key in ["", "library:0", "library:3/4", "unknown", "library:3?x=1"] {
        assert_eq!(listing_library(key), Err(Error::InvalidId));
    }
}

#[test]
fn duplicate_or_blank_libraries_fail_and_empty_libraries_are_supported() {
    for body in [
        r#"[{"id":3,"name":"만화"},{"id":3,"name":"소설"}]"#,
        r#"[{"id":3,"name":" "}]"#,
    ] {
        let mut mock = Mock::replies(vec![Reply::new(200, body)]);
        assert!(matches!(
            Api::new(&config(), &mut mock).libraries(),
            Err(Error::InvalidResponse)
        ));
    }
    let mut mock = Mock::replies(vec![Reply::new(200, "[]")]);
    assert!(
        Api::new(&config(), &mut mock)
            .libraries()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn library_listing_uses_id_scoped_endpoint_and_collapsed_pagination() {
    let mut value: Value = serde_json::from_str(SERIES).unwrap();
    value["items"].as_array_mut().unwrap().truncate(1);
    value["total"] = json!(41);
    let mut mock = Mock::replies(vec![Reply::new(200, &value.to_string())]);
    let result = Api::new(&config(), &mut mock)
        .search_library(Some("드래곤"), 1, Some(scope().library_id))
        .unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].key.key(), "series:3:7");
    assert_eq!(result.entries[0].title, "드래곤");
    assert!(result.has_next_page);
    assert_eq!(mock.requests[0].url, config().endpoint("libraries/3/books"));
    let body: Value = serde_json::from_str(mock.requests[0].body.as_deref().unwrap()).unwrap();
    assert_eq!(body["collapseSeries"], true);
    assert_eq!(body["q"], "드래곤");
    value["items"][0]["libraryId"] = json!(9);
    let mut mock = Mock::replies(vec![Reply::new(200, &value.to_string())]);
    assert_eq!(
        Api::new(&config(), &mut mock).search_library(None, 1, Some(scope().library_id)),
        Err(Error::InvalidResponse)
    );
}

#[test]
fn series_keys_survive_representative_changes_and_separate_libraries() {
    let mut page: Value = serde_json::from_str(SERIES).unwrap();
    page["items"][1]["libraryId"] = json!(9);
    let result = parse_json::<BooksPage>(&serde_json::to_vec(&page).unwrap())
        .unwrap()
        .into_search(&config(), 0)
        .unwrap();
    assert_eq!(
        result
            .entries
            .iter()
            .map(|e| e.key.key())
            .collect::<Vec<_>>(),
        ["series:3:7", "series:9:7"]
    );
    page["items"].as_array_mut().unwrap().remove(0);
    page["total"] = json!(1);
    page["items"][0]["libraryId"] = json!(3);
    let result = parse_json::<BooksPage>(&serde_json::to_vec(&page).unwrap())
        .unwrap()
        .into_search(&config(), 0)
        .unwrap();
    assert_eq!(result.entries[0].key.key(), "series:3:7");
    for key in [
        "series:3",
        "series:3:0",
        "series:3:7:8",
        "series:0:7",
        "series:3:7?key=x",
    ] {
        assert_eq!(MangaKey::parse(key), Err(Error::InvalidId));
    }
    assert_eq!(MangaKey::parse("42").unwrap().key(), "42");
    let legacy = parse_json::<BooksPage>(PAGE.as_bytes())
        .unwrap()
        .into_search(&config(), 0)
        .unwrap();
    assert_eq!(legacy.entries[0].key.key(), "42");
}

#[test]
fn series_uses_server_volume_order_instead_of_file_ids_or_title_sort() {
    let mut mock = Mock::replies(vec![
        Reply::new(200, SERIES),
        Reply::new(200, &series_detail()),
    ]);
    let result = Api::new(&config(), &mut mock).series(scope()).unwrap();
    assert_eq!(result.title, "드래곤");
    assert_eq!(
        result
            .chapters
            .iter()
            .map(|c| (c.series_key(), c.number, c.title.as_str()))
            .collect::<Vec<_>>(),
        [
            ("43:499".into(), 2, "드래곤 c02"),
            ("42:501".into(), 1, "드래곤 c01")
        ]
    );
    assert_eq!(
        mock.requests[0].url,
        config().endpoint("series/7/books?libraryId=3&page=0&size=40&sort=seriesIndex&order=asc")
    );
    assert!(
        mock.requests
            .iter()
            .all(|r| r.method == Method::Get && r.body.is_none())
    );
    assert_eq!(parse_series_chapter("42:501"), Ok((book(), file())));
    for key in ["501", "42:0", "42:501:2", "42:501/2"] {
        assert_eq!(parse_series_chapter(key), Err(Error::InvalidId));
    }
}

fn paged_series() -> (Value, Value) {
    let original: Value = serde_json::from_str(SERIES).unwrap();
    let mut first = original.clone();
    first["total"] = json!(41);
    first["items"] = Value::Array(
        (0..40)
            .map(|i| {
                let mut card = original["items"][0].clone();
                card["id"] = json!(42 + i);
                card["files"][0]["id"] = json!(501 + i);
                card
            })
            .collect(),
    );
    let mut last = original.clone();
    last["page"] = json!(1);
    last["total"] = json!(41);
    last["items"] = json!([original["items"][0].clone()]);
    last["items"][0]["id"] = json!(82);
    last["items"][0]["files"][0]["id"] = json!(541);
    (first, last)
}

#[test]
fn series_loads_all_pages_before_building_chapters() {
    let (first, last) = paged_series();
    let mut mock = Mock::replies(vec![
        Reply::new(200, &first.to_string()),
        Reply::new(200, &last.to_string()),
        Reply::new(200, &series_detail()),
    ]);
    let result = Api::new(&config(), &mut mock).series(scope()).unwrap();
    assert_eq!(result.chapters.len(), 41);
    assert_eq!(result.chapters[0].series_key(), "82:541");
    assert_eq!(result.chapters.last().unwrap().series_key(), "42:501");
    assert!(mock.requests[1].url.contains("page=1&size=40"));
}

#[test]
fn series_rejects_duplicates_changed_totals_and_truncated_pages() {
    let (first, last) = paged_series();
    for change in ["duplicate", "total", "library", "series", "info", "empty"] {
        let mut last = last.clone();
        match change {
            "duplicate" => last["items"][0]["id"] = json!(42),
            "total" => last["total"] = json!(42),
            "library" => last["items"][0]["libraryId"] = json!(9),
            "series" => last["items"][0]["seriesId"] = json!(8),
            "info" => last["seriesInfo"]["id"] = json!(8),
            "empty" => last["items"] = json!([]),
            _ => unreachable!(),
        }
        let mut mock = Mock::replies(vec![
            Reply::new(200, &first.to_string()),
            Reply::new(200, &last.to_string()),
        ]);
        assert!(
            matches!(
                Api::new(&config(), &mut mock).series(scope()),
                Err(Error::InvalidResponse)
            ),
            "{change}"
        );
    }
    let mut first = first;
    first["items"].as_array_mut().unwrap().truncate(39);
    let mut mock = Mock::replies(vec![Reply::new(200, &first.to_string())]);
    assert!(matches!(
        Api::new(&config(), &mut mock).series(scope()),
        Err(Error::InvalidResponse)
    ));
}

#[test]
fn series_limits_are_bounded_and_network_failures_do_not_return_partial_chapters() {
    let (mut first, _) = paged_series();
    let mut mock = Mock::replies(vec![
        Reply::new(200, &first.to_string()),
        Reply::new(403, ""),
    ]);
    assert!(matches!(
        Api::new(&config(), &mut mock).series(scope()),
        Err(Error::Forbidden)
    ));
    first["total"] = json!(MAX_SERIES_BOOKS + 1);
    let mut mock = Mock::replies(vec![Reply::new(200, &first.to_string())]);
    assert!(matches!(
        Api::new(&config(), &mut mock).series(scope()),
        Err(Error::TooManyBooks)
    ));
    let mut empty: Value = serde_json::from_str(SERIES).unwrap();
    empty["items"] = json!([]);
    empty["total"] = json!(0);
    let mut mock = Mock::replies(vec![Reply::new(200, &empty.to_string())]);
    assert!(matches!(
        Api::new(&config(), &mut mock).series(scope()),
        Err(Error::EmptyComic)
    ));
}

#[test]
fn series_chapters_exclude_epub_missing_and_sidecar_and_keep_file_keys() {
    let mut page: Value = serde_json::from_str(SERIES).unwrap();
    page["items"][1]["files"][0]["format"] = json!("epub");
    page["items"][0]["files"] = serde_json::from_str::<Value>(DETAIL).unwrap()["files"].clone();
    let books: BooksPage = parse_json(&serde_json::to_vec(&page).unwrap()).unwrap();
    let expected = series_chapters(&books.items).unwrap();
    assert_eq!(
        expected.iter().map(|c| c.series_key()).collect::<Vec<_>>(),
        ["42:502", "42:501"]
    );
    page["items"][0]["files"].as_array_mut().unwrap().reverse();
    let books: BooksPage = parse_json(&serde_json::to_vec(&page).unwrap()).unwrap();
    assert_eq!(series_chapters(&books.items).unwrap(), expected);
    page["items"][0]["status"] = json!("missing");
    let books: BooksPage = parse_json(&serde_json::to_vec(&page).unwrap()).unwrap();
    assert_eq!(series_chapters(&books.items), Err(Error::EmptyComic));
    page["items"][0]["status"] = json!("present");
    page["items"][0]["files"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": 501, "format": "cbz", "role": "primary"}));
    let books: BooksPage = parse_json(&serde_json::to_vec(&page).unwrap()).unwrap();
    assert_eq!(series_chapters(&books.items), Err(Error::InvalidResponse));
}

#[test]
fn series_pages_check_library_membership_and_file_ownership_before_image_requests() {
    let mut mock = Mock::replies(vec![
        Reply::new(200, &series_detail()),
        Reply::new(200, COUNT),
    ]);
    assert_eq!(
        Api::new(&config(), &mut mock)
            .series_pages(scope(), book(), file())
            .unwrap()
            .len(),
        3
    );
    for change in ["library", "series", "file"] {
        let mut detail: Value = serde_json::from_str(&series_detail()).unwrap();
        match change {
            "library" => detail["libraryId"] = json!(9),
            "series" => detail["seriesId"] = json!(8),
            "file" => detail["files"] = json!([]),
            _ => unreachable!(),
        }
        let mut mock = Mock::replies(vec![Reply::new(200, &detail.to_string())]);
        assert_eq!(
            Api::new(&config(), &mut mock).series_pages(scope(), book(), file()),
            Err(Error::NotFound)
        );
        assert_eq!(mock.requests.len(), 1);
    }
    let mut detail: Value = serde_json::from_str(&series_detail()).unwrap();
    detail["seriesId"] = json!(8);
    detail["seriesMemberships"] = json!([{"seriesId": 7}]);
    let mut mock = Mock::replies(vec![
        Reply::new(200, &detail.to_string()),
        Reply::new(200, COUNT),
    ]);
    assert!(
        Api::new(&config(), &mut mock)
            .series_pages(scope(), book(), file())
            .is_ok()
    );
}

#[test]
fn cb7_books_appear_in_global_search_and_library_listings() {
    let mut card: Value = serde_json::from_str(CB7_DETAIL).unwrap();
    card.as_object_mut().unwrap().remove("seriesId");
    let page = json!({"items": [card], "total": 1, "page": 0, "size": 40}).to_string();
    for library in [None, Some(scope().library_id)] {
        let mut mock = Mock::replies(vec![Reply::new(200, &page)]);
        let result = Api::new(&config(), &mut mock)
            .search_library(Some("드래곤"), 1, library)
            .unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].key.key(), "142");
        assert!(!result.has_next_page);
        let body: Value = serde_json::from_str(mock.requests[0].body.as_deref().unwrap()).unwrap();
        assert_eq!(
            body["filter"]["rules"][0]["value"],
            json!(["cbz", "cbr", "cb7"])
        );
        assert_eq!(
            mock.requests[0].url,
            config().endpoint(if library.is_some() {
                "libraries/3/books"
            } else {
                "books/query"
            })
        );
    }
}

#[test]
fn cb7_chapters_keep_ids_and_exclude_sidecars_and_plain_7z_archives() {
    let mut detail: BookDetail = parse_json(CB7_DETAIL.as_bytes()).unwrap();
    let expected = chapters(&detail).unwrap();
    assert_eq!(
        expected
            .iter()
            .map(|c| (c.file_id.key(), c.number))
            .collect::<Vec<_>>(),
        [("702".into(), 2), ("701".into(), 1)]
    );
    assert_eq!(expected[1].title, "드래곤 c03.cb7");
    detail.files.reverse();
    assert_eq!(chapters(&detail).unwrap(), expected);
    let mut file = serde_json::from_str::<Value>(CB7_DETAIL).unwrap()["files"][1].clone();
    for role in ["cover", "sidecar", "unknown"] {
        file["role"] = json!(role);
        assert!(
            !parse_json::<bookorbit_aidoku_source::models::BookFile>(
                &serde_json::to_vec(&file).unwrap()
            )
            .unwrap()
            .is_comic()
        );
    }
}

#[test]
fn cb7_series_use_existing_stable_keys_and_server_page_order() {
    let card: Value = serde_json::from_str(CB7_DETAIL).unwrap();
    let page = json!({"items": [card], "total": 1, "page": 0, "size": 40, "seriesInfo": {"id":7,"name":"드래곤"}}).to_string();
    let listing = parse_json::<BooksPage>(page.as_bytes())
        .unwrap()
        .into_search(&config(), 0)
        .unwrap();
    assert_eq!(listing.entries[0].key.key(), "series:3:7");
    let mut mock = Mock::replies(vec![Reply::new(200, &page), Reply::new(200, CB7_DETAIL)]);
    let series = Api::new(&config(), &mut mock).series(scope()).unwrap();
    assert_eq!(
        series
            .chapters
            .iter()
            .map(|c| c.series_key())
            .collect::<Vec<_>>(),
        ["142:702", "142:701"]
    );
    let book = BookId::parse("142").unwrap();
    let file = FileId::parse("701").unwrap();
    for is_series in [false, true] {
        let mut mock = Mock::replies(vec![Reply::new(200, CB7_DETAIL), Reply::new(200, COUNT)]);
        let cb7_config = config();
        let mut api = Api::new(&cb7_config, &mut mock);
        let pages = if is_series {
            api.series_pages(scope(), book, file)
        } else {
            api.pages(book, file)
        }
        .unwrap();
        assert_eq!(
            pages,
            (0..3)
                .map(|index| config().endpoint(&format!("cbz/files/701/pages/{index}")))
                .collect::<Vec<_>>()
        );
        for url in &pages {
            assert_eq!(config().image_url(url), Ok(url.clone()));
        }
        assert_eq!(
            mock.requests[1].url,
            config().endpoint("cbz/files/701/pages")
        );
    }
    let mut mock = Mock::replies(vec![Reply::new(200, CB7_DETAIL)]);
    assert_eq!(
        Api::new(&config(), &mut mock).pages(book, FileId::parse("703").unwrap()),
        Err(Error::NotFound)
    );
    assert_eq!(mock.requests.len(), 1);
}

#[test]
fn cb7_archive_failures_keep_existing_error_and_page_count_guards() {
    for (status, body, expected) in [
        (422, "", Error::CorruptArchive),
        (404, "", Error::NotFound),
        (200, r#"{"pageCount":0}"#, Error::EmptyComic),
    ] {
        let mut mock = Mock::replies(vec![Reply::new(200, CB7_DETAIL), Reply::new(status, body)]);
        assert_eq!(
            Api::new(&config(), &mut mock)
                .pages(BookId::parse("142").unwrap(), FileId::parse("701").unwrap()),
            Err(expected)
        );
    }
}
