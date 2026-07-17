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

pub use crate::error::{Error, Result};

// ── Re-exports ────────────────────────────────────────────────────

pub use fs::{AsyncFile, FileSystem, FileType, Metadata};
pub use http::{
    AsyncChunkReader, HttpClient, HttpRequest, HttpResponse, InMemoryBody, Method, StreamingBody,
};
