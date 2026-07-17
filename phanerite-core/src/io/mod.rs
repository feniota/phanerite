//! Async I/O abstractions for phanerite.
//!
//! ## Module layout
//!
//! | Module        | Content                                      |
//! |---------------|----------------------------------------------|
//! | [`fs`]        | `AsyncFile` / `FileSystem` traits, `Metadata` |
//! | [`http`]      | `HttpClient` trait, request/response types    |
//! | [`adapters`]  | Backend implementations (tokio, compio, …)     |
//! | [`utils`]     | `AsyncFileExt` extension methods               |

pub mod adapters;
pub mod fs;
pub mod http;
pub mod utils;

// ── Common types ──────────────────────────────────────────────────

#[derive(Debug)]
pub enum Error {
    /// Underlying I/O error.
    Io(std::io::Error),
    /// HTTP protocol error.
    Http { status: u16, message: String },
    /// Catch-all for other failures.
    Other,
}

impl Error {
    pub(crate) fn other() -> Self {
        Error::Other
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

// ── Re-exports ────────────────────────────────────────────────────

pub use fs::{AsyncFile, FileSystem, FileType, Metadata};
pub use http::{
    AsyncChunkReader, HttpClient, HttpRequest, HttpResponse, InMemoryBody, Method, StreamingBody,
};
