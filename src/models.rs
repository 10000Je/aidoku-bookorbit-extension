use alloc::{format, string::String, string::ToString, vec::Vec};
use serde::{Deserialize, Deserializer};

use crate::{
    config::Config,
    error::{Error, Result},
};

pub const PAGE_SIZE: usize = 40;
pub const MAX_QUERY_OFFSET: usize = 1_000_000;

pub fn positive_id(value: &str) -> Result<u64> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidId);
    }
    let id = value.parse::<u64>().map_err(|_| Error::InvalidId)?;
    if id == 0 || id > i32::MAX as u64 {
        return Err(Error::InvalidId);
    }
    Ok(id)
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            pub fn parse(key: &str) -> Result<Self> {
                positive_id(key).map(Self)
            }

            pub fn value(self) -> u64 {
                self.0
            }

            pub fn key(self) -> String {
                self.0.to_string()
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(
                deserializer: D,
            ) -> core::result::Result<Self, D::Error> {
                let id = u64::deserialize(deserializer)?;
                if id == 0 || id > i32::MAX as u64 {
                    return Err(serde::de::Error::custom("invalid identifier"));
                }
                Ok(Self(id))
            }
        }
    };
}

id_type!(BookId);
id_type!(FileId);
id_type!(LibraryId);
id_type!(SeriesId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeriesKey {
    pub library_id: LibraryId,
    pub series_id: SeriesId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MangaKey {
    Book(BookId),
    Series(SeriesKey),
}

impl MangaKey {
    pub fn parse(key: &str) -> Result<Self> {
        if let Some(key) = key.strip_prefix("series:") {
            let (library, series) = key.split_once(':').ok_or(Error::InvalidId)?;
            Ok(Self::Series(SeriesKey {
                library_id: LibraryId::parse(library)?,
                series_id: SeriesId::parse(series)?,
            }))
        } else {
            BookId::parse(key).map(Self::Book)
        }
    }

    pub fn key(self) -> String {
        match self {
            Self::Book(id) => id.key(),
            Self::Series(scope) => format!(
                "series:{}:{}",
                scope.library_id.value(),
                scope.series_id.value()
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub id: LibraryId,
    pub name: String,
    #[serde(default)]
    pub display_order: i32,
}

pub fn listing_library(key: &str) -> Result<Option<LibraryId>> {
    match key {
        "comics" => Ok(None),
        _ => key
            .strip_prefix("library:")
            .ok_or(Error::InvalidId)
            .and_then(LibraryId::parse)
            .map(Some),
    }
}

fn null_default<'de, D, T>(deserializer: D) -> core::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
pub struct BooksPage {
    pub items: Vec<BookCard>,
    pub total: usize,
    pub page: usize,
    pub size: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookCard {
    pub id: BookId,
    pub library_id: Option<LibraryId>,
    pub series_id: Option<SeriesId>,
    pub series_name: Option<String>,
    pub status: String,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub authors: Vec<String>,
    pub files: Vec<BookFile>,
    #[serde(default)]
    pub has_cover: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub genres: Vec<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookDetail {
    pub id: BookId,
    pub library_id: Option<LibraryId>,
    pub series_id: Option<SeriesId>,
    #[serde(default, deserialize_with = "null_default")]
    pub series_memberships: Vec<SeriesMembership>,
    pub status: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub cover_source: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub authors: Vec<BookAuthor>,
    pub files: Vec<BookFile>,
    #[serde(default, deserialize_with = "null_default")]
    pub genres: Vec<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesMembership {
    pub series_id: SeriesId,
}

impl BookDetail {
    pub fn belongs_to(&self, scope: SeriesKey) -> bool {
        self.library_id == Some(scope.library_id)
            && (self.series_id == Some(scope.series_id)
                || self
                    .series_memberships
                    .iter()
                    .any(|member| member.series_id == scope.series_id))
    }
}

#[derive(Debug, Deserialize)]
pub struct SeriesInfo {
    pub id: SeriesId,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesBooksPage {
    #[serde(flatten)]
    pub books: BooksPage,
    pub series_info: SeriesInfo,
}

#[derive(Debug, Deserialize)]
pub struct BookAuthor {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BookFile {
    pub id: FileId,
    pub format: Option<String>,
    pub role: String,
    pub filename: Option<String>,
}

impl BookFile {
    pub fn is_comic(&self) -> bool {
        matches!(self.role.as_str(), "primary" | "content")
            && self.format.as_deref().is_some_and(|format| {
                format.eq_ignore_ascii_case("cbz") || format.eq_ignore_ascii_case("cbr")
            })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageCount {
    pub page_count: usize,
}

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub key: MangaKey,
    pub book_id: BookId,
    pub title: String,
    pub cover: Option<String>,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct SearchPage {
    pub entries: Vec<Entry>,
    pub has_next_page: bool,
}

pub fn title_or_default(title: Option<&str>, id: BookId) -> String {
    title
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("도서 {}", id.value()))
}

pub fn merged_tags(genres: &[String], tags: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for tag in genres.iter().chain(tags) {
        if !tag.trim().is_empty() && !result.contains(tag) {
            result.push(tag.clone());
            if result.len() == 255 {
                break;
            }
        }
    }
    result
}

impl BooksPage {
    pub fn validate(&self, requested_page: usize) -> Result<()> {
        let offset = requested_page
            .checked_mul(PAGE_SIZE)
            .ok_or(Error::InvalidPage)?;
        if self.page != requested_page
            || self.size != PAGE_SIZE
            || self.items.len() > PAGE_SIZE
            || (!self.items.is_empty() && offset.saturating_add(self.items.len()) > self.total)
            || (self.items.is_empty() && offset < self.total)
        {
            return Err(Error::InvalidResponse);
        }
        let mut ids = alloc::collections::BTreeSet::new();
        if self.items.iter().any(|card| !ids.insert(card.id)) {
            return Err(Error::InvalidResponse);
        }
        Ok(())
    }

    pub fn into_search(self, config: &Config, requested_page: usize) -> Result<SearchPage> {
        self.validate(requested_page)?;
        let mut ids = Vec::new();
        let mut entries = Vec::new();
        for card in self.items {
            let key = match (card.library_id, card.series_id) {
                (Some(library_id), Some(series_id)) => MangaKey::Series(SeriesKey {
                    library_id,
                    series_id,
                }),
                _ => MangaKey::Book(card.id),
            };
            if ids.contains(&key) {
                return Err(Error::InvalidResponse);
            }
            ids.push(key);
            if card.status != "present" || !card.files.iter().any(BookFile::is_comic) {
                continue;
            }
            entries.push(Entry {
                key,
                book_id: card.id,
                title: if matches!(key, MangaKey::Series(_)) {
                    card.series_name
                        .filter(|name| !name.trim().is_empty())
                        .ok_or(Error::InvalidResponse)?
                } else {
                    title_or_default(card.title.as_deref(), card.id)
                },
                cover: card
                    .has_cover
                    .then(|| config.endpoint(&format!("books/{}/thumbnail", card.id.value()))),
                authors: card.authors,
                tags: merged_tags(&card.genres, &card.tags),
            });
        }
        Ok(SearchPage {
            entries,
            has_next_page: requested_page.saturating_add(1).saturating_mul(PAGE_SIZE) < self.total,
        })
    }
}
