use alloc::{format, string::String};
use core::fmt;

use crate::error::{Error, Result};

#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(input: &str) -> Result<Self> {
        let input = input.trim();
        if input.is_empty() {
            return Err(Error::MissingConfig);
        }
        if input.len() > 512 || !input.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(Error::InvalidKey);
        }
        Ok(Self(String::from(input)))
    }

    /// BookOrbit 공식 API 키 문서에서 허용하는 단일 인증 헤더.
    pub fn header(&self) -> (&'static str, String) {
        ("Authorization", format!("Bearer {}", self.0))
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey([REDACTED])")
    }
}
