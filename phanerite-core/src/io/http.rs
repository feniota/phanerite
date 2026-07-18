//! HTTP client abstraction and body types.
//!
//! # Core types
//!
//! | Type | Role |
//! |------|------|
//! | [`Method`] | HTTP method (GET, POST, …) |
//! | [`HttpRequest`] | Request descriptor (method, url, headers, body) |
//! | [`HttpResponse`] | Response (status, headers, body) |
//! | [`HttpClient`] | Trait — execute requests |
//! | [`InMemoryBody`] | Fully buffered response body |
//! | [`StreamingBody`] | Chunked response body backed by [`AsyncChunkReader`] |
//! | [`AsyncChunkReader`] | Trait — produces chunks for streaming |

use std::collections::BTreeMap;
use std::sync::Mutex;

use super::fs::AsyncFile;
use crate::error::{Error, Result};

// ── Types ─────────────────────────────────────────────────────────

/// HTTP request method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Connect,
    Trace,
    Custom(String),
}

impl Method {
    /// Return the uppercase string representation (e.g. `"GET"`).
    pub fn as_str(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
            Method::Custom(s) => s.as_str(),
        }
    }
}

/// An HTTP request descriptor.
///
/// Header keys should be lowercase for portability across HTTP/1.1
/// and HTTP/2 backends.
pub struct HttpRequest<'a> {
    pub method: Method,
    pub url: &'a str,
    pub headers: BTreeMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// An HTTP response.
///
/// ## Convenience methods
///
/// - [`ok`](HttpResponse::ok) — check for 2xx status
/// - [`get_header`](HttpResponse::get_header) — case-insensitive lookup
/// - [`filename`](HttpResponse::filename) — extract from `Content-Disposition`
/// - [`size`](HttpResponse::size) — parse `Content-Length`
pub struct HttpResponse<B> {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: B,
}

impl<B> HttpResponse<B> {
    /// Return `Ok(())` for 2xx, `Err(Http(status))` otherwise.
    pub fn ok(&self) -> Result<()> {
        if self.status < 200 || self.status >= 300 {
            Err(Error::Http(self.status))
        } else {
            Ok(())
        }
    }

    /// Case-insensitive header lookup.
    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Extract a filename from `Content-Disposition`.
    pub fn filename(&self) -> Option<String> {
        let disposition = self.get_header("content-disposition")?;
        disposition.split(';').find_map(|part| {
            let part = part.trim();
            part.strip_prefix("filename=")
                .map(|v| v.trim_matches('"').to_string())
        })
    }

    /// Parse `Content-Length` as `u64`.
    pub fn size(&self) -> Option<u64> {
        self.get_header("content-length")
            .and_then(|v| v.parse().ok())
    }
}

// ── In-memory body ────────────────────────────────────────────────

/// A read-only [`AsyncFile`] backed by an in-memory byte buffer.
pub struct InMemoryBody {
    data: Vec<u8>,
}

impl InMemoryBody {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl AsyncFile for InMemoryBody {
    async fn read_at(&self, offset: u64, mut buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let start = offset as usize;
        if start >= self.data.len() {
            return Ok((0, buf));
        }
        let available = self.data.len() - start;
        let n = available.min(buf.capacity());
        if buf.len() < n {
            buf.resize(n, 0);
        }
        buf[..n].copy_from_slice(&self.data[start..start + n]);
        Ok((n, buf))
    }

    async fn write_at(&self, _offset: u64, _buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        Err(Error::Other("InMemoryBody is read-only".into()))
    }

    async fn size(&self) -> Result<u64> {
        Ok(self.data.len() as u64)
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}

// ── Streaming body ────────────────────────────────────────────────

/// Produces the next chunk of data from a sequential byte source.
pub trait AsyncChunkReader {
    /// Read the next chunk.  `Some(data)` = chunk, `None` = EOF.
    async fn read_chunk(&mut self) -> Result<Option<Vec<u8>>>;
}

/// A streaming [`AsyncFile`] backed by any [`AsyncChunkReader`].
///
/// Data is pulled from the underlying reader one chunk at a time.
/// Only **sequential** reads are supported — the offset must not
/// jump backwards.  Forward seeks are handled by discarding bytes.
///
/// Peak memory is bounded by the largest chunk returned by the
/// reader, independent of the total data size.
pub struct StreamingBody<C> {
    inner: Mutex<Inner<C>>,
    content_length: Option<u64>,
}

struct Inner<C> {
    reader: C,
    pos: u64,
    /// Buffered remainder from the last oversized chunk.
    leftover: Vec<u8>,
}

impl<C: AsyncChunkReader> StreamingBody<C> {
    /// Wrap a chunk reader and optional content-length hint.
    pub fn new(reader: C, content_length: Option<u64>) -> Self {
        Self {
            inner: Mutex::new(Inner {
                reader,
                pos: 0,
                leftover: Vec::new(),
            }),
            content_length,
        }
    }

    /// `Content-Length` header value, if known.
    ///
    /// Returns `None` for chunked transfer encoding or when the
    /// header was absent.
    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }
}

impl<C: AsyncChunkReader> AsyncFile for StreamingBody<C> {
    #[allow(clippy::await_holding_lock)]
    async fn read_at(&self, offset: u64, mut buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let mut inner = self.inner.lock().unwrap();

        // Forward seek: discard leftover and fast-forward through chunks.
        while inner.pos < offset {
            inner.leftover.clear();
            let chunk = match inner.reader.read_chunk().await? {
                Some(c) => c,
                None => return Ok((0, buf)),
            };
            let chunk_len = chunk.len() as u64;
            let needed = offset - inner.pos;
            if needed < chunk_len {
                // The target offset falls inside this chunk.
                // Store the part after the target as leftover, return max buf bytes.
                let start = needed as usize;
                inner.leftover = chunk[start..].to_vec();
                inner.pos += needed;
                break;
            }
            inner.pos += chunk_len;
        }

        // Serve from leftover first.
        if !inner.leftover.is_empty() {
            let n = inner.leftover.len().min(buf.capacity());
            if buf.len() < n {
                buf.resize(n, 0);
            }
            buf[..n].copy_from_slice(&inner.leftover[..n]);
            if n < inner.leftover.len() {
                inner.leftover.drain(..n);
            } else {
                inner.leftover.clear();
            }
            inner.pos += n as u64;
            return Ok((n, buf));
        }

        // Read a fresh chunk.
        let chunk = match inner.reader.read_chunk().await? {
            Some(c) => c,
            None => return Ok((0, buf)),
        };
        let n = chunk.len().min(buf.capacity());
        if buf.len() < n {
            buf.resize(n, 0);
        }
        buf[..n].copy_from_slice(&chunk[..n]);
        // Store unread portion.
        if n < chunk.len() {
            inner.leftover = chunk[n..].to_vec();
        }
        inner.pos += n as u64;
        Ok((n, buf))
    }

    async fn write_at(&self, _offset: u64, _buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        Err(Error::Other("StreamingBody is read-only".into()))
    }

    async fn size(&self) -> Result<u64> {
        self.content_length
            .ok_or(Error::Other("content-length unknown".into()))
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}

// ── Trait ─────────────────────────────────────────────────────────

/// Abstract HTTP client.
///
/// Implementations dispatch requests to a concrete HTTP library
/// (reqwest, ureq, nyquest, …).
pub trait HttpClient {
    /// Buffered body type (full response in memory).
    type Body: AsyncFile;
    /// Streaming body type (chunked from the live connection).
    type StreamingBody: AsyncFile;

    /// Execute, buffering the full body in memory.
    async fn execute(&self, request: HttpRequest<'_>) -> Result<HttpResponse<Self::Body>>;

    /// Execute, returning a streaming body backed by the live connection.
    async fn execute_streaming(
        &self,
        request: HttpRequest<'_>,
    ) -> Result<HttpResponse<Self::StreamingBody>>;
}
