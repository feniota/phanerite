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

    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse<InMemoryBody>> {
        self.do_execute(request).await
    }

    async fn execute_streaming(&self, request: HttpRequest) -> Result<HttpResponse<InMemoryBody>> {
        self.do_execute(request).await
    }
}

impl NyquestClient {
    async fn do_execute(&self, request: HttpRequest) -> Result<HttpResponse<InMemoryBody>> {
        let method = convert_method(&request.method);
        let mut req = Request::new(method, request.url);

        for (k, v) in &request.headers {
            req = req.with_header(k.clone(), v.clone());
        }

        if let Some(body) = &request.body {
            req = req.with_body(Body::binary_bytes(body.clone()));
        }

        let resp = self
            .inner
            .request(req)
            .await
            .map_err(|_| Error::other("request failed"))?;

        let status = resp.status().code();
        let bytes = resp
            .bytes()
            .await
            .map_err(|_| Error::other("read body failed"))?;

        Ok(HttpResponse {
            status,
            headers: BTreeMap::new(), // nyquest doesn't expose a full header iterator
            body: InMemoryBody::new(bytes),
        })
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
