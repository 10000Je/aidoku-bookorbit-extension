use alloc::{format, string::String};
use url::Url;

use crate::{
    auth::ApiKey,
    error::{Error, Result},
    models::positive_id,
};

#[derive(Debug, Clone)]
pub struct Config {
    base: Url,
    pub key: ApiKey,
}

fn ambiguous_url(input: &str) -> bool {
    input.bytes().any(|b| b.is_ascii_control() || b == b'\\')
        || input.split('/').any(|part| {
            let lower = part.to_ascii_lowercase();
            part == "."
                || part == ".."
                || lower.contains("%2e")
                || lower.contains("%2f")
                || lower.contains("%5c")
        })
}

impl Config {
    pub fn new(server: &str, key: &str) -> Result<Self> {
        let server = server.trim();
        if server.is_empty() {
            return Err(Error::MissingConfig);
        }
        if ambiguous_url(server) || !server.contains("://") {
            return Err(Error::InvalidUrl);
        }
        let mut base = Url::parse(server).map_err(|_| Error::InvalidUrl)?;
        if !matches!(base.scheme(), "http" | "https")
            || base.host_str().is_none()
            || !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
        {
            return Err(Error::InvalidUrl);
        }
        let path = String::from(base.path().trim_end_matches('/'));
        if path.ends_with("/api/v1") || path.ends_with("/opds") {
            return Err(Error::InvalidUrl);
        }
        base.set_path(&path);
        Ok(Self {
            base,
            key: ApiKey::new(key)?,
        })
    }

    pub fn base_url(&self) -> &str {
        self.base.as_str().trim_end_matches('/')
    }

    pub fn endpoint(&self, path: &str) -> String {
        format!("{}/api/v1/{path}", self.base_url())
    }

    /// URL 파싱 후 origin, 프록시 경로, 허용 이미지 라우트를 모두 검사한다.
    pub fn image_url(&self, input: &str) -> Result<String> {
        if ambiguous_url(input) {
            return Err(Error::UnsafeImage);
        }
        let url = Url::parse(input).map_err(|_| Error::UnsafeImage)?;
        if url.origin() != self.base.origin()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(Error::UnsafeImage);
        }
        let prefix = format!("{}/api/v1/", self.base.path().trim_end_matches('/'));
        let route = url.path().strip_prefix(&prefix).ok_or(Error::UnsafeImage)?;
        let parts: alloc::vec::Vec<_> = route.split('/').collect();
        let allowed = match parts.as_slice() {
            ["books", id, "thumbnail" | "cover"] => positive_id(id).is_ok(),
            ["cbz", "files", id, "pages", index] => {
                positive_id(id).is_ok()
                    && !index.is_empty()
                    && index.bytes().all(|b| b.is_ascii_digit())
                    && index.parse::<u32>().is_ok()
            }
            _ => false,
        };
        if !allowed {
            return Err(Error::UnsafeImage);
        }
        Ok(url.into())
    }
}
