//! [`HttpClient`] adapter backed by [`reqwest`].
//!
//! Enabled via the `reqwest` feature gate (implies `tokio`).

use std::collections::BTreeMap;

use crate::io::{
    AsyncChunkReader, Error, HttpClient, HttpRequest, HttpResponse, InMemoryBody, Method, Result,
    StreamingBody,
};

/// An [`HttpClient`] that delegates to a [`reqwest::Client`].
pub struct ReqwestClient {
    inner: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    pub fn from_client(client: reqwest::Client) -> Self {
        Self { inner: client }
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for ReqwestClient {
    type Body = InMemoryBody;
    type StreamingBody = StreamingBody<ReqwestChunkReader>;

    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse<InMemoryBody>> {
        let (status, headers, resp) = self.send_request(request).await?;
        let bytes = resp.bytes().await.map_err(|_| Error::other())?;
        Ok(HttpResponse {
            status,
            headers,
            body: InMemoryBody::new(bytes.to_vec()),
        })
    }

    async fn execute_streaming(
        &self,
        request: HttpRequest,
    ) -> Result<HttpResponse<StreamingBody<ReqwestChunkReader>>> {
        let (status, headers, resp) = self.send_request(request).await?;
        let content_length = resp.content_length();
        Ok(HttpResponse {
            status,
            headers,
            body: StreamingBody::new(ReqwestChunkReader { resp }, content_length),
        })
    }
}

impl ReqwestClient {
    async fn send_request(
        &self,
        request: HttpRequest,
    ) -> Result<(u16, BTreeMap<String, String>, reqwest::Response)> {
        let method = convert_method(&request.method);
        let mut req = self.inner.request(method, &request.url);

        for (k, v) in &request.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }

        let resp = req.send().await.map_err(|_| Error::other())?;
        let status = resp.status().as_u16();
        let headers = resp
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_ascii_lowercase(),
                    v.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();

        Ok((status, headers, resp))
    }
}

fn convert_method(m: &Method) -> reqwest::Method {
    match m {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Put => reqwest::Method::PUT,
        Method::Delete => reqwest::Method::DELETE,
        Method::Patch => reqwest::Method::PATCH,
        Method::Head => reqwest::Method::HEAD,
        Method::Options => reqwest::Method::OPTIONS,
        Method::Connect => reqwest::Method::CONNECT,
        Method::Trace => reqwest::Method::TRACE,
        Method::Custom(s) => {
            reqwest::Method::from_bytes(s.as_bytes()).unwrap_or(reqwest::Method::GET)
        }
    }
}

// ── Reqwest chunk reader ──────────────────────────────────────────

/// Adapts a [`reqwest::Response`] to implement [`AsyncChunkReader`].
pub struct ReqwestChunkReader {
    resp: reqwest::Response,
}

impl AsyncChunkReader for ReqwestChunkReader {
    async fn read_chunk(&mut self) -> Result<Option<Vec<u8>>> {
        match self.resp.chunk().await {
            Ok(Some(c)) => Ok(Some(c.to_vec())),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::other()),
        }
    }
}
