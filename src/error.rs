use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

/// 외부 응답 본문이나 자격 증명을 오류에 포함하지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidUrl,
    MissingConfig,
    InvalidKey,
    InvalidId,
    InvalidPage,
    QueryTooLong,
    InvalidResponse,
    EmptyComic,
    TooManyPages,
    TooManyBooks,
    Unauthorized,
    Forbidden,
    NotFound,
    CorruptArchive,
    RateLimited,
    Http(u16),
    Network,
    UnsafeImage,
    Redirect,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidUrl => "서버 기본 URL을 확인하세요. 인증 정보·쿼리·API 경로는 제외하세요.",
            Self::MissingConfig => {
                "BookOrbit 소스 설정에서 서버 주소와 읽기 전용 API 키를 입력하세요."
            }
            Self::InvalidKey => "API 키에 공백이나 잘못된 문자가 있습니다.",
            Self::InvalidId => "잘못된 도서 또는 파일 ID입니다. 작품을 다시 불러오세요.",
            Self::InvalidPage => "요청한 목록 페이지가 유효 범위를 벗어났습니다.",
            Self::QueryTooLong => "검색어는 200자 이내로 입력하세요.",
            Self::InvalidResponse => {
                "BookOrbit 응답 형식이 올바르지 않습니다. 서버 호환성을 확인하세요."
            }
            Self::EmptyComic => "이 파일에는 읽을 수 있는 이미지 페이지가 없습니다.",
            Self::TooManyPages => "페이지 수가 안전 한도(20,000)를 초과했습니다.",
            Self::TooManyBooks => "시리즈 도서 수가 안전 한도(20,000)를 초과했습니다.",
            Self::Unauthorized => {
                "BookOrbit 인증 실패(401). API 키의 만료·철회·활성화를 확인하세요."
            }
            Self::Forbidden => {
                "BookOrbit 접근 거부(403). 키의 라이브러리와 읽기 권한을 확인하세요."
            }
            Self::NotFound => "도서·파일·페이지가 없거나 접근할 수 없습니다(404/410).",
            Self::CorruptArchive => {
                "서버에서 만화 파일을 읽을 수 없습니다(422). 파일을 확인하세요."
            }
            Self::RateLimited => "서버 요청 제한(429). 잠시 후 다시 시도하세요.",
            Self::Http(_) => "BookOrbit 서버 오류입니다. 서버 상태와 API 호환성을 확인하세요.",
            Self::Network => "서버에 연결할 수 없습니다. 주소·네트워크·TLS 인증서를 확인하세요.",
            Self::UnsafeImage => "설정된 서버의 만화 이미지 경로에만 인증 정보를 전송합니다.",
            Self::Redirect => {
                "서버 요청이 다른 주소로 이동했습니다. 리다이렉트 없는 최종 기본 URL을 입력하세요."
            }
        })
    }
}

impl core::error::Error for Error {}

pub fn check_status(status: u16) -> Result<()> {
    match status {
        200..=299 => Ok(()),
        300..=399 => Err(Error::Redirect),
        401 => Err(Error::Unauthorized),
        403 => Err(Error::Forbidden),
        404 | 410 => Err(Error::NotFound),
        422 => Err(Error::CorruptArchive),
        429 => Err(Error::RateLimited),
        _ => Err(Error::Http(status)),
    }
}
