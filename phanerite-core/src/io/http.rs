//! HTTP client abstraction and body types.

use std::collections::BTreeMap;
use std::sync::Mutex;

use super::fs::AsyncFile;
use super::{Error, Result};

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
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// An HTTP response.
pub struct HttpResponse<B> {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: B,
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
        Err(Error::Other)
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
}

impl<C: AsyncChunkReader> StreamingBody<C> {
    pub fn new(reader: C, content_length: Option<u64>) -> Self {
        Self {
            inner: Mutex::new(Inner { reader, pos: 0 }),
            content_length,
        }
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }
}

impl<C: AsyncChunkReader> AsyncFile for StreamingBody<C> {
    #[allow(clippy::await_holding_lock)]
    async fn read_at(&self, offset: u64, mut buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let mut inner = self.inner.lock().unwrap();

        while inner.pos < offset {
            let chunk = match inner.reader.read_chunk().await? {
                Some(c) => c,
                None => return Ok((0, buf)),
            };
            let chunk_len = chunk.len() as u64;
            let needed = offset - inner.pos;
            if needed < chunk_len {
                let start = needed as usize;
                let rem = &chunk[start..];
                let n = rem.len().min(buf.capacity());
                if buf.len() < n {
                    buf.resize(n, 0);
                }
                buf[..n].copy_from_slice(&rem[..n]);
                inner.pos += n as u64;
                return Ok((n, buf));
            }
            inner.pos += chunk_len;
        }

        let chunk = match inner.reader.read_chunk().await? {
            Some(c) => c,
            None => return Ok((0, buf)),
        };
        let n = chunk.len().min(buf.capacity());
        if buf.len() < n {
            buf.resize(n, 0);
        }
        buf[..n].copy_from_slice(&chunk[..n]);
        inner.pos += n as u64;
        Ok((n, buf))
    }

    async fn write_at(&self, _offset: u64, _buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        Err(Error::Other)
    }

    async fn size(&self) -> Result<u64> {
        self.content_length.ok_or(Error::Other)
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}

// ── Trait ─────────────────────────────────────────────────────────

pub trait HttpClient {
    /// Buffered body type.
    type Body: AsyncFile;
    /// Streaming body type.
    type StreamingBody: AsyncFile;

    /// Execute, buffering the full body in memory.
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse<Self::Body>>;

    /// Execute, returning a streaming body backed by the live connection.
    async fn execute_streaming(
        &self,
        request: HttpRequest,
    ) -> Result<HttpResponse<Self::StreamingBody>>;
}
