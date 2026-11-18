use aidoku::{
    Chapter, DynamicListings, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga,
    MangaPageResult, Page, PageContent, PageContext, Source,
    imports::{defaults::defaults_get, error::AidokuError, net::Request, std::sleep},
    prelude::*,
};
use alloc::{string::String, vec::Vec};

use crate::{
    api::{Api, Method, REQUEST_TIMEOUT_SECONDS, RequestSpec, Response, Transport},
    config::Config,
    error::{Error, Result},
    models::{FileId, MangaKey, SearchPage, listing_library, merged_tags, title_or_default},
    reader,
};

impl From<Error> for AidokuError {
    fn from(value: Error) -> Self {
        Self::message(value)
    }
}

fn config() -> Result<Config> {
    Config::new(
        &defaults_get::<String>("server_url").unwrap_or_default(),
        &defaults_get::<String>("api_key").unwrap_or_default(),
    )
}

fn make_request(spec: &RequestSpec<'_>) -> Result<Request> {
    let request = match spec.method {
        Method::Get => Request::get(&spec.url),
        Method::Post => Request::post(&spec.url),
    }
    .map_err(|_| Error::InvalidUrl)?;
    let (name, value) = spec.key.header();
    let mut request = request
        .header(name, value.as_str())
        .header("Accept", "application/json")
        .timeout(REQUEST_TIMEOUT_SECONDS);
    if let Some(body) = &spec.body {
        request = request
            .header("Content-Type", "application/json")
            .body(body);
    }
    Ok(request)
}

struct AidokuTransport;

impl Transport for AidokuTransport {
    fn execute(&mut self, spec: &RequestSpec<'_>) -> Result<Response> {
        let response = make_request(spec)?.send().map_err(|_| Error::Network)?;
        let status = u16::try_from(response.status_code()).map_err(|_| Error::Network)?;
        let retry_after = response.get_header("Retry-After");
        let final_url = response.get_url();
        let body = if (200..300).contains(&status) {
            response.get_data().map_err(|_| Error::Network)?
        } else {
            Vec::new()
        };
        Ok(Response {
            status,
            body,
            retry_after,
            final_url,
        })
    }

    fn sleep(&mut self, seconds: u32) {
        sleep(seconds as i32);
    }
}

struct BookOrbit;

fn manga_page(result: SearchPage) -> MangaPageResult {
    MangaPageResult {
        entries: result
            .entries
            .into_iter()
            .map(|entry| Manga {
                key: entry.key.key(),
                title: entry.title,
                cover: entry.cover,
                authors: Some(entry.authors),
                tags: Some(entry.tags),
                ..Default::default()
            })
            .collect(),
        has_next_page: result.has_next_page,
    }
}

impl Source for BookOrbit {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> aidoku::Result<MangaPageResult> {
        let config = config()?;
        let result = Api::new(&config, &mut AidokuTransport).search(query.as_deref(), page)?;
        Ok(manga_page(result))
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> aidoku::Result<Manga> {
        if !needs_details && !needs_chapters {
            return Ok(manga);
        }
        let config = config()?;
        let key = MangaKey::parse(&manga.key)?;
        let mut transport = AidokuTransport;
        let mut api = Api::new(&config, &mut transport);
        let (detail, chapters, series_title) = match key {
            MangaKey::Book(id) => {
                let detail = api.book(id)?;
                let chapters = if needs_chapters {
                    reader::chapters(&detail)?
                } else {
                    Vec::new()
                };
                (detail, chapters, None)
            }
            MangaKey::Series(scope) => {
                let series = api.series(scope)?;
                (series.detail, series.chapters, Some(series.title))
            }
        };
        if needs_chapters {
            manga.chapters = Some(
                chapters
                    .into_iter()
                    .map(|chapter| Chapter {
                        key: if matches!(key, MangaKey::Series(_)) {
                            chapter.series_key()
                        } else {
                            chapter.file_id.key()
                        },
                        title: Some(chapter.title),
                        chapter_number: Some(chapter.number as f32),
                        ..Default::default()
                    })
                    .collect(),
            );
        }
        if needs_details {
            manga.title = series_title
                .unwrap_or_else(|| title_or_default(detail.title.as_deref(), detail.id));
            manga.tags = Some(merged_tags(&detail.genres, &detail.tags));
            let mut description = detail.description.unwrap_or_default();
            for (label, value) in [("출판사", detail.publisher), ("언어", detail.language)] {
                if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
                    if !description.is_empty() {
                        description.push_str("\n\n");
                    }
                    description.push_str(&format!("{label}: {value}"));
                }
            }
            manga.description = (!description.is_empty()).then_some(description);
            manga.authors = Some(
                detail
                    .authors
                    .into_iter()
                    .map(|author| author.name)
                    .collect(),
            );
            manga.cover = detail
                .cover_source
                .map(|_| config.endpoint(&format!("books/{}/cover", detail.id.value())));
        }
        Ok(manga)
    }

    fn get_page_list(&self, manga: Manga, chapter: Chapter) -> aidoku::Result<Vec<Page>> {
        let config = config()?;
        let mut transport = AidokuTransport;
        let mut api = Api::new(&config, &mut transport);
        let pages = match MangaKey::parse(&manga.key)? {
            MangaKey::Book(id) => api.pages(id, FileId::parse(&chapter.key)?)?,
            MangaKey::Series(scope) => {
                let (book, file) = reader::parse_series_chapter(&chapter.key)?;
                api.series_pages(scope, book, file)?
            }
        };
        Ok(pages
            .into_iter()
            .map(|url| Page {
                content: PageContent::url(url),
                ..Default::default()
            })
            .collect())
    }
}

impl ListingProvider for BookOrbit {
    fn get_manga_list(&self, listing: Listing, page: i32) -> aidoku::Result<MangaPageResult> {
        let library = listing_library(&listing.id)?;
        let config = config()?;
        Ok(manga_page(
            Api::new(&config, &mut AidokuTransport).search_library(None, page, library)?,
        ))
    }
}

impl DynamicListings for BookOrbit {
    fn get_dynamic_listings(&self) -> aidoku::Result<Vec<Listing>> {
        let config = config()?;
        Ok(Api::new(&config, &mut AidokuTransport)
            .libraries_with_comics()?
            .into_iter()
            .map(|library| Listing {
                id: format!("library:{}", library.id.value()),
                name: library.name,
                ..Default::default()
            })
            .collect())
    }
}

impl ImageRequestProvider for BookOrbit {
    fn get_image_request(
        &self,
        url: String,
        _context: Option<PageContext>,
    ) -> aidoku::Result<Request> {
        let config = config()?;
        let url = config.image_url(&url)?;
        let (name, value) = config.key.header();
        Ok(Request::get(url)
            .map_err(|_| Error::InvalidUrl)?
            .header(name, value.as_str())
            .header("Accept", "image/*")
            .timeout(REQUEST_TIMEOUT_SECONDS))
    }
}

register_source!(
    BookOrbit,
    ListingProvider,
    DynamicListings,
    ImageRequestProvider
);
