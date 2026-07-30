use crate::auth::yggdrasil::YggdrasilError;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Http(isahc::Error),
    SerdeJson(serde_json::Error),
    Zip(zip::result::ZipError),
    Yggdrasil(YggdrasilError),
    UrlParseErr(url::ParseError),
    Cancelled,
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Http(status) => write!(f, "HTTP {status}"),
            Error::SerdeJson(e) => write!(f, "Serde JSON error: {e}"),
            Error::Yggdrasil(e) => write!(f, "{}", e),
            Error::Cancelled => write!(f, "Operation cancelled"),
            Error::Zip(e) => write!(f, "ZIP error: {e}"),
            Error::UrlParseErr(e) => write!(f, "Url parse error: {e}"),
            Error::Other(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<isahc::Error> for Error {
    fn from(e: isahc::Error) -> Self {
        Error::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        Error::Zip(e)
    }
}

impl From<YggdrasilError> for Error {
    fn from(e: YggdrasilError) -> Self {
        Error::Yggdrasil(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::UrlParseErr(e)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
