use alloc::{format, string::String, vec::Vec};

use crate::{
    config::Config,
    error::{Error, Result},
    models::{BookCard, BookDetail, BookId, FileId, title_or_default},
};

pub const MAX_PAGES: usize = 20_000;
pub const MAX_SERIES_BOOKS: usize = 20_000;

#[derive(Debug, PartialEq)]
pub struct ComicChapter {
    pub book_id: BookId,
    pub file_id: FileId,
    pub title: String,
    pub number: usize,
}

pub fn chapters(detail: &BookDetail) -> Result<Vec<ComicChapter>> {
    let mut files: Vec<_> = detail.files.iter().filter(|file| file.is_comic()).collect();
    files.sort_by_key(|file| file.id);
    if files.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(Error::InvalidResponse);
    }
    if files.is_empty() {
        return Err(Error::EmptyComic);
    }
    let mut result: Vec<_> = files
        .into_iter()
        .enumerate()
        .map(|(index, file)| ComicChapter {
            book_id: detail.id,
            file_id: file.id,
            title: file
                .filename
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| format!("만화 파일 {}", file.id.value())),
            number: index + 1,
        })
        .collect();
    result.reverse();
    Ok(result)
}

impl ComicChapter {
    pub fn series_key(&self) -> String {
        format!("{}:{}", self.book_id.value(), self.file_id.value())
    }
}

pub fn parse_series_chapter(key: &str) -> Result<(BookId, FileId)> {
    let (book, file) = key.split_once(':').ok_or(Error::InvalidId)?;
    Ok((BookId::parse(book)?, FileId::parse(file)?))
}

/// 도서 순서는 서버의 seriesIndex 정렬을 유지하고, 한 도서 안의 파일은 ID로 정렬한다.
pub fn series_chapters(books: &[BookCard]) -> Result<Vec<ComicChapter>> {
    let mut chapters = Vec::new();
    let mut file_ids = alloc::collections::BTreeSet::new();
    for book in books {
        if book.status != "present" {
            continue;
        }
        let mut files: Vec<_> = book.files.iter().filter(|file| file.is_comic()).collect();
        files.sort_by_key(|file| file.id);
        let title = title_or_default(book.title.as_deref(), book.id);
        let multiple = files.len() > 1;
        for (index, file) in files.into_iter().enumerate() {
            if !file_ids.insert(file.id) {
                return Err(Error::InvalidResponse);
            }
            chapters.push(ComicChapter {
                book_id: book.id,
                file_id: file.id,
                title: if multiple {
                    format!("{title} · 파일 {}", index + 1)
                } else {
                    title.clone()
                },
                number: chapters.len() + 1,
            });
        }
    }
    if chapters.is_empty() {
        return Err(Error::EmptyComic);
    }
    chapters.reverse();
    Ok(chapters)
}

pub fn verify_chapter(detail: &BookDetail, file_id: FileId) -> Result<()> {
    if detail
        .files
        .iter()
        .any(|file| file.id == file_id && file.is_comic())
    {
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}

/// 서버의 정렬 결과를 나타내는 0..N 인덱스만 사용한다. 이미지 바이트는 가져오지 않는다.
pub fn page_urls(config: &Config, file_id: FileId, count: usize) -> Result<Vec<String>> {
    match count {
        0 => Err(Error::EmptyComic),
        n if n > MAX_PAGES => Err(Error::TooManyPages),
        n => Ok((0..n)
            .map(|index| config.endpoint(&format!("cbz/files/{}/pages/{index}", file_id.value())))
            .collect()),
    }
}
