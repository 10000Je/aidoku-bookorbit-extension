use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::{
    auth::ApiKey,
    config::Config,
    error::{Error, Result, check_status},
    models::{
        BookDetail, BookId, BooksPage, FileId, Library, LibraryId, MAX_QUERY_OFFSET, PAGE_SIZE,
        PageCount, SearchPage, SeriesBooksPage, SeriesKey,
    },
    reader,
};

pub const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_RETRIES: usize = 2;
pub const REQUEST_TIMEOUT_SECONDS: f64 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug)]
pub struct RequestSpec<'a> {
    pub method: Method,
    pub url: String,
    pub key: &'a ApiKey,
    pub body: Option<String>,
}

pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
    pub retry_after: Option<String>,
    pub final_url: Option<String>,
}

/// Aidoku의 호스트 호출을 순수 로직에서 분리하여 fixture로 재시도까지 검증한다.
pub trait Transport {
    fn execute(&mut self, request: &RequestSpec<'_>) -> Result<Response>;
    fn sleep(&mut self, seconds: u32);
}

pub fn query_body(query: Option<&str>, aidoku_page: i32) -> Result<(usize, Value)> {
    let page = aidoku_page
        .checked_sub(1)
        .filter(|p| *p >= 0)
        .ok_or(Error::InvalidPage)? as usize;
    if page
        .checked_mul(PAGE_SIZE)
        .is_none_or(|offset| offset > MAX_QUERY_OFFSET)
    {
        return Err(Error::InvalidPage);
    }
    let query = query.unwrap_or_default().trim();
    // 서버는 JavaScript 문자열 길이(UTF-16 코드 단위)를 사용한다.
    if query.encode_utf16().count() > 200 {
        return Err(Error::QueryTooLong);
    }
    Ok((
        page,
        json!({
            "pagination": {"page": page, "size": PAGE_SIZE},
            "sort": [{"field": "title", "dir": "asc"}],
            "collapseSeries": true,
            "collapseLinked": false,
            "filter": {"type": "group", "join": "AND", "rules": [
                {"type": "rule", "field": "format", "operator": "includesAny", "value": ["cbz", "cbr"]},
                {"type": "rule", "field": "fileAvailability", "operator": "isPresent"}
            ]},
            "q": query
        }),
    ))
}

/// 과도하게 기다리거나 서버가 지정한 시간보다 먼저 재시도하지 않는다.
/// HTTP-date, 잘못된 값, 10초 초과의 Retry-After는 사용자에게 제한 오류를 반환한다.
pub fn retry_delay(attempt: usize, retry_after: Option<&str>) -> Option<u32> {
    if attempt >= MAX_RETRIES {
        return None;
    }
    let backoff = 1 << attempt;
    match retry_after {
        None => Some(backoff),
        Some(value) => {
            let delay = value.trim().parse::<u32>().ok()?;
            (delay <= 10).then_some(delay.max(backoff))
        }
    }
}

pub fn parse_json<T: DeserializeOwned>(body: &[u8]) -> Result<T> {
    if body.len() > MAX_JSON_BYTES {
        return Err(Error::InvalidResponse);
    }
    serde_json::from_slice(body).map_err(|_| Error::InvalidResponse)
}

pub struct Api<'a, T> {
    config: &'a Config,
    transport: &'a mut T,
}

impl<'a, T: Transport> Api<'a, T> {
    pub fn new(config: &'a Config, transport: &'a mut T) -> Self {
        Self { config, transport }
    }

    fn json<R: DeserializeOwned>(
        &mut self,
        method: Method,
        path: &str,
        body: Option<String>,
    ) -> Result<R> {
        let request = RequestSpec {
            method,
            url: self.config.endpoint(path),
            key: &self.config.key,
            body,
        };
        for attempt in 0..=MAX_RETRIES {
            let response = self.transport.execute(&request)?;
            // 사후 검사이며 호스트의 자동 리다이렉트를 차단하는 기능은 아니다.
            let final_url = response
                .final_url
                .as_deref()
                .and_then(|value| url::Url::parse(value).ok());
            if final_url != url::Url::parse(&request.url).ok() {
                return Err(Error::Redirect);
            }
            if response.status == 429 {
                if let Some(delay) = retry_delay(attempt, response.retry_after.as_deref()) {
                    self.transport.sleep(delay);
                    continue;
                }
            }
            check_status(response.status)?;
            return parse_json(&response.body);
        }
        Err(Error::RateLimited)
    }

    pub fn search(&mut self, query: Option<&str>, page: i32) -> Result<SearchPage> {
        self.search_library(query, page, None)
    }

    pub fn libraries(&mut self) -> Result<Vec<Library>> {
        let mut libraries: Vec<Library> = self.json(Method::Get, "libraries", None)?;
        let mut ids = alloc::collections::BTreeSet::new();
        if libraries
            .iter()
            .any(|library| !ids.insert(library.id) || library.name.trim().is_empty())
        {
            return Err(Error::InvalidResponse);
        }
        libraries.sort_by_key(|library| (library.display_order, library.id));
        Ok(libraries)
    }

    pub fn search_library(
        &mut self,
        query: Option<&str>,
        page: i32,
        library: Option<LibraryId>,
    ) -> Result<SearchPage> {
        let (page, body) = query_body(query, page)?;
        let path = library
            .map(|id| format!("libraries/{}/books", id.value()))
            .unwrap_or_else(|| "books/query".into());
        let result: BooksPage = self.json(Method::Post, &path, Some(body.to_string()))?;
        if let Some(id) = library {
            if result.items.iter().any(|card| card.library_id != Some(id)) {
                return Err(Error::InvalidResponse);
            }
        }
        result.into_search(self.config, page)
    }

    /// 전 페이지를 확인한 뒤 반환하여 시리즈가 일부 권만 있는 것처럼 보이지 않게 한다.
    pub fn series(&mut self, scope: SeriesKey) -> Result<ComicSeries> {
        let mut books = Vec::new();
        let mut ids = alloc::collections::BTreeSet::new();
        let mut total = None;
        let mut title = None;
        for page in 0.. {
            let result: SeriesBooksPage = self.json(
                Method::Get,
                &format!(
                    "series/{}/books?libraryId={}&page={page}&size={PAGE_SIZE}&sort=seriesIndex&order=asc",
                    scope.series_id.value(),
                    scope.library_id.value()
                ),
                None,
            )?;
            result.books.validate(page)?;
            if result.books.total > reader::MAX_SERIES_BOOKS {
                return Err(Error::TooManyBooks);
            }
            if result.series_info.id != scope.series_id
                || result.series_info.name.trim().is_empty()
                || total.is_some_and(|total| total != result.books.total)
                || title
                    .as_ref()
                    .is_some_and(|title| title != &result.series_info.name)
                || result.books.items.iter().any(|card| {
                    card.library_id != Some(scope.library_id)
                        || card.series_id != Some(scope.series_id)
                        || !ids.insert(card.id)
                })
            {
                return Err(Error::InvalidResponse);
            }
            let has_next = (page + 1) * PAGE_SIZE < result.books.total;
            // 중간 페이지가 짧으면 offset 기반 조회에서 도서가 누락될 수 있다.
            if has_next && result.books.items.len() != PAGE_SIZE {
                return Err(Error::InvalidResponse);
            }
            total = Some(result.books.total);
            title = Some(result.series_info.name);
            books.extend(result.books.items);
            if !has_next {
                if Some(books.len()) != total {
                    return Err(Error::InvalidResponse);
                }
                break;
            }
        }
        let chapters = reader::series_chapters(&books)?;
        let first = chapters.last().ok_or(Error::EmptyComic)?;
        let detail = self.book(first.book_id)?;
        if !detail.belongs_to(scope) {
            return Err(Error::InvalidResponse);
        }
        Ok(ComicSeries {
            title: title.ok_or(Error::InvalidResponse)?,
            detail,
            chapters,
        })
    }

    pub fn book(&mut self, id: BookId) -> Result<BookDetail> {
        let detail: BookDetail = self.json(Method::Get, &format!("books/{}", id.value()), None)?;
        if detail.id != id {
            return Err(Error::InvalidResponse);
        }
        if detail.status != "present" {
            return Err(Error::NotFound);
        }
        Ok(detail)
    }

    pub fn pages(&mut self, book: BookId, file: FileId) -> Result<Vec<String>> {
        let detail = self.book(book)?;
        self.file_pages(&detail, file)
    }

    pub fn series_pages(
        &mut self,
        scope: SeriesKey,
        book: BookId,
        file: FileId,
    ) -> Result<Vec<String>> {
        let detail = self.book(book)?;
        if !detail.belongs_to(scope) {
            return Err(Error::NotFound);
        }
        self.file_pages(&detail, file)
    }

    fn file_pages(&mut self, detail: &BookDetail, file: FileId) -> Result<Vec<String>> {
        reader::verify_chapter(detail, file)?;
        let count: PageCount = self.json(
            Method::Get,
            &format!("cbz/files/{}/pages", file.value()),
            None,
        )?;
        reader::page_urls(self.config, file, count.page_count)
    }
}

pub struct ComicSeries {
    pub title: String,
    pub detail: BookDetail,
    pub chapters: Vec<reader::ComicChapter>,
}
