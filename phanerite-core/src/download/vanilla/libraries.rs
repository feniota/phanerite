use crate::download::vanilla::version_info::Rule;
use crate::download::{DownloadHandle, Downloadable};
use crate::error::{Error, Result};
use crate::io::{AsyncFile, FileSystem, HttpClient, HttpRequest, Method};
use crate::storage::Storage;
use crate::utils::{HashValue, Sha1};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

/// Library
#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,

    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,

    pub sha1: Sha1,

    pub size: u64,

    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

impl Downloadable for Library {
    type HashAlgorithm = Sha1;

    #[instrument(skip_all)]
    async fn download(
        self,
        http_client: &impl HttpClient,
        storage: &Storage<impl FileSystem>,
    ) -> Result<DownloadHandle<impl AsyncFile, Self::HashAlgorithm>> {
        if let Some(download) = self.downloads {
            if let Some(artifact) = download.artifact {
                let request = HttpRequest {
                    method: Method::Get,
                    url: &artifact.url,
                    headers: Default::default(),
                    body: None,
                };
                let response = http_client.execute_streaming(request).await?;
                response.ok()?;
                return Ok(DownloadHandle {
                    name: response.filename(),
                    size: response.size(),
                    stream: response.body,
                    path: storage.libraries_dir.join(artifact.path),
                    digest: Some(artifact.sha1),
                });
            }
        }

        Err(Error::Other("No download link".to_string()))
    }
}
