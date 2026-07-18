//! [`HttpClient`] adapter backed by [`ureq`].
//!
//! Enabled via the `ureq` feature gate.
//!
//! ureq is synchronous — calls inside `async fn` block the current
//! thread.

use std::collections::BTreeMap;
use std::io::Read;

use crate::error::Error;
use crate::io::{HttpClient, HttpRequest, HttpResponse, InMemoryBody, Method, Result};
use tracing::trace;

/// An [`HttpClient`] backed by a [`ureq::Agent`].
pub struct UreqClient {
    agent: ureq::Agent,
}

impl UreqClient {
    pub fn new() -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder().build().into();
        Self { agent }
    }

    pub fn from_agent(agent: ureq::Agent) -> Self {
        Self { agent }
    }
}

impl Default for UreqClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for UreqClient {
    type Body = InMemoryBody;
    type StreamingBody = InMemoryBody;

    async fn execute(&self, request: HttpRequest<'_>) -> Result<HttpResponse<InMemoryBody>> {
        let mut resp = self
            .send(
                &request.method,
                request.url,
                &request.headers,
                &request.body,
            )
            .map_err(|e| Error::Other(e.to_string()))?;

        let status = resp.status().as_u16();
        trace!(%status, method = %request.method.as_str(), url = request.url, "request");

        let headers: BTreeMap<_, _> = resp
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_ascii_lowercase(),
                    v.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();

        let mut body = Vec::new();
        resp.body_mut()
            .as_reader()
            .read_to_end(&mut body)
            .map_err(|e| Error::Other(e.to_string()))?;
        trace!(body_len = body.len(), "body read");

        Ok(HttpResponse {
            status,
            headers,
            body: InMemoryBody::new(body),
        })
    }

    async fn execute_streaming(
        &self,
        request: HttpRequest<'_>,
    ) -> Result<HttpResponse<InMemoryBody>> {
        self.execute(request).await
    }
}

impl UreqClient {
    fn send(
        &self,
        method: &Method,
        url: &str,
        headers: &BTreeMap<String, String>,
        body: &Option<Vec<u8>>,
    ) -> std::result::Result<http::Response<ureq::Body>, ureq::Error> {
        if body.is_some() || matches!(method, Method::Post | Method::Put | Method::Patch) {
            let mut req = match method {
                Method::Post => self.agent.post(url),
                Method::Put => self.agent.put(url),
                _ => self.agent.post(url),
            };
            for (k, v) in headers {
                req = req.header(k, v);
            }
            req.send(body.as_deref().unwrap_or(&[]))
        } else {
            let mut req = match method {
                Method::Get => self.agent.get(url),
                Method::Delete => self.agent.delete(url),
                Method::Head => self.agent.head(url),
                Method::Options => self.agent.options(url),
                _ => self.agent.get(url),
            };
            for (k, v) in headers {
                req = req.header(k, v);
            }
            req.call()
        }
    }
}
