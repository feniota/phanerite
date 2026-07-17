use crate::download::vanilla::version_info::Rule;
use crate::error::{Error, Result};
use crate::io::{AsyncFile, HttpClient, HttpRequest, Method};
use crate::utils::{HashValue, Sha1};
use serde::Deserialize;
use std::collections::HashMap;

/// Library
#[derive(Deserialize)]
pub struct Library {
    pub name: String,

    pub downloads: Option<LibraryDownloads>,

    pub rules: Option<Vec<Rule>>,

    pub natives: Option<HashMap<String, String>>,

    pub extract: Option<Extract>,

    pub classifiers: Option<HashMap<String, Artifact>>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,

    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Deserialize)]
pub struct Artifact {
    pub path: String,

    pub sha1: Sha1,

    pub size: u64,

    pub url: String,
}

#[derive(Deserialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

impl Library {
    pub async fn download(
        &self,
        http_client: &impl HttpClient,
    ) -> Result<(impl AsyncFile, impl HashValue)> {
        if let Some(download) = &self.downloads {
            if let Some(artifact) = &download.artifact {
                let request = HttpRequest {
                    method: Method::Get,
                    url: &artifact.url,
                    headers: Default::default(),
                    body: None,
                };
                let response = http_client.execute_streaming(request).await?;
                if response.status < 200 || response.status >= 300 {
                    return Err(Error::Http(response.status));
                }
                return Ok((response.body, artifact.sha1.clone()));
            }
        }

        Err(Error::Other("No download link".to_string()))
    }
}
