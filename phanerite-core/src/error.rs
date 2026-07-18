#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Http(nyquest::Error),
    SerdeJson(serde_json::Error),
    Cancelled,
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Http(status) => write!(f, "HTTP {status}"),
            Error::SerdeJson(e) => write!(f, "Serde JSON error: {e}"),
            Error::Cancelled => write!(f, "Operation cancelled"),
            Error::Other(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

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

impl From<nyquest::Error> for Error {
    fn from(e: nyquest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
