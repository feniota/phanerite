//! [`HttpClient`] adapter backed by [`nyquest`].
//!
//! Enabled via the `nyquest` feature gate.
//!
//! Both [`execute`] and [`execute_streaming`] currently buffer the full
//! body.  A streaming chunk reader can be added later.
//!
//! ## Header limitations
//!
//! nyquest's [`Response`](nyquest::r#async::Response) only exposes
//! individual headers via [`get_header`], not a full iterator.
//! Response headers are therefore returned as an empty map.

use std::collections::BTreeMap;

use crate::io::{Error, HttpClient, HttpRequest, HttpResponse, InMemoryBody, Method, Result};
use nyquest::AsyncClient;
use nyquest::r#async::{Body, Request};
use tracing::{trace, warn};

/// An [`HttpClient`] that delegates to a [`nyquest::AsyncClient`].
pub struct NyquestClient {
    inner: AsyncClient,
}

impl NyquestClient {
    /// Wrap an already-built [`AsyncClient`].
    pub fn new(client: AsyncClient) -> Self {
        Self { inner: client }
    }
}

impl HttpClient for NyquestClient {
    type Body = InMemoryBody;
    type StreamingBody = InMemoryBody; // TODO: StreamingBody + NyquestChunkReader

    async fn execute(&self, request: HttpRequest<'_>) -> Result<HttpResponse<InMemoryBody>> {
        self.do_execute(request).await
    }

    async fn execute_streaming(
        &self,
        request: HttpRequest<'_>,
    ) -> Result<HttpResponse<InMemoryBody>> {
        self.do_execute(request).await
    }
}

impl NyquestClient {
    async fn do_execute(&self, request: HttpRequest<'_>) -> Result<HttpResponse<InMemoryBody>> {
        let method = convert_method(&request.method);
        let url = request.url.to_owned();
        let headers = request.headers.clone();
        let body = request.body.clone();

        for attempt in 0..3 {
            let mut req = Request::new(method.clone(), url.clone());
            for (k, v) in &headers {
                req = req.with_header(k.clone(), v.clone());
            }
            if let Some(b) = &body {
                req = req.with_body(Body::binary_bytes(b.clone()));
            }

            let resp = match self.inner.request(req).await {
                Ok(r) => r,
                Err(e) if attempt < 2 => {
                    warn!(attempt, %e, "request failed, retrying");
                    continue;
                }
                Err(e) => return Err(Error::other(format!("request failed: {e}"))),
            };

            let status = resp.status().code();
            trace!(%status, method = %request.method.as_str(), url = %request.url, "request");

            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) if attempt < 2 => {
                    warn!(attempt, %e, "read body failed, retrying");
                    continue;
                }
                Err(e) => return Err(Error::other(format!("read body failed: {e}"))),
            };
            trace!(body_len = bytes.len(), "body read");

            return Ok(HttpResponse {
                status,
                headers: BTreeMap::new(),
                body: InMemoryBody::new(bytes),
            });
        }
        unreachable!()
    }
}

fn convert_method(m: &Method) -> nyquest::Method {
    match m {
        Method::Get => nyquest::Method::get(),
        Method::Post => nyquest::Method::post(),
        Method::Put => nyquest::Method::put(),
        Method::Delete => nyquest::Method::delete(),
        Method::Patch => nyquest::Method::patch(),
        Method::Head => nyquest::Method::head(),
        other => nyquest::Method::custom(other.as_str().to_owned()),
    }
}
