//! Crate-wide error and result types.
//!
//! All fallible operations in `phanerite-core` return [`Result<T>`],
//! which aliases `std::result::Result<T, Error>`.

/// Unified error type for all phanerite operations.
///
/// # Variants
///
/// | Variant | Meaning |
/// |---------|---------|
/// | [`Io`](Error::Io) | Underlying filesystem or network I/O failure (chainable via `source()`) |
/// | [`Http`](Error::Http) | Non-2xx HTTP status code received |
/// | [`Other`](Error::Other) | Catch-all for domain-specific failures |
///
/// # Conversion
///
/// [`std::io::Error`] converts automatically:
///
/// ```rust
/// # use phanerite_core::error::Result;
/// fn example() -> Result<()> {
///     std::fs::read_to_string("/nonexistent")?; // io::Error → Error::Io
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub enum Error {
    /// Underlying I/O error (wraps [`std::io::Error`]).
    ///
    /// Chained via [`std::error::Error::source`].
    Io(std::io::Error),
    /// HTTP status-code error.
    ///
    /// Carries the numeric status code (e.g. `404`).
    Http(u16),
    /// Catch-all for other failures, with a human-readable message.
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Http(status) => write!(f, "HTTP {status}"),
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
    /// Convenience constructor for [`Error::Other`].
    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Shorthand for `std::result::Result<T, Error>`.
pub type Result<T> = core::result::Result<T, Error>;
