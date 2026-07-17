use crate::download::Downloadable;
use crate::error::{Error, Result};
use crate::io::utils::{AsyncFileExt, Hasher};
use crate::io::{AsyncFile, FileSystem, HttpClient, HttpRequest, Method};
use crate::storage::Storage;
use crate::utils::{HashValue, Sha1};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

/// 资源索引
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: Sha1,
    pub size: u64,
    pub total_size: Option<u64>,
    pub url: String,
}

#[derive(Deserialize)]
pub struct AssetIndexList {
    pub objects: BTreeMap<String, AssetObject>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct AssetObject {
    hash: Sha1,
    size: usize,
}

impl AssetIndex {
    pub fn index_file_name(&self) -> &str {
        self.url.split('/').last().expect("Incorrect URL format")
    }
}

pub struct DownloadObject {
    pub url: String,
    pub sha1: Sha1,
    pub path: PathBuf,
}

impl AssetIndexList {
    pub async fn fetch(asset_index: &AssetIndex, http_client: &impl HttpClient) -> Result<Self> {
        let request = HttpRequest {
            method: Method::Get,
            url: &asset_index.url,
            headers: Default::default(),
            body: None,
        };

        let response = http_client.execute(request).await?;

        if response.status < 200 || response.status >= 300 {
            return Err(Error::Http(response.status));
        }

        let body = response.body.read_all().await?;

        let hash = {
            let mut hasher = sha1::Sha1::default();
            hasher.update(&body);
            hasher.finalize_hex()
        };

        if Sha1::from_hex(hash) != asset_index.sha1 {
            return Err(Error::Other("hash mismatch".to_string()));
        }

        let json = serde_json::from_slice(&body).map_err(|e| Error::Other(e.to_string()))?;
        Ok(json)
    }
    pub fn iter_downloadable(&self) -> impl Iterator<Item = DownloadObject> {
        self.objects.iter().map(|x| {
            let hash = x.1.hash.to_string();
            DownloadObject {
                url: format!("{}/{}/{}", RESOURCES_URL, &hash[..2], hash),
                sha1: x.1.hash.clone(),
                path: PathBuf::new().join("object").join(&hash[..2]).join(hash),
            }
        })
    }
}

impl Downloadable for DownloadObject {
    type HashAlgorithm = Sha1;

    async fn download(
        self,
        http_client: &impl HttpClient,
        storage: &Storage<impl FileSystem>,
    ) -> Result<(impl AsyncFile, Option<Self::HashAlgorithm>, PathBuf)> {
        let request = HttpRequest {
            method: Method::Get,
            url: &self.url,
            headers: Default::default(),
            body: None,
        };
        let response = http_client.execute_streaming(request).await?;
        if response.status < 200 || response.status >= 300 {
            return Err(Error::Http(response.status));
        }
        Ok((
            response.body,
            Some(self.sha1),
            storage.assets_dir.join(self.path),
        ))
    }
}
