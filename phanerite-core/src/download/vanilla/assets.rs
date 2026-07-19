use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::storage::Storage;
use crate::utils::Sha1Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

impl AssetIndexList {
    pub async fn get(index: &AssetIndex, downloader: &Downloader) -> Result<Self> {
        let body = downloader
            .fetch(index.url.clone(), Some(index.sha1.clone().into()))
            .await?;
        let json = serde_json::from_slice(&body)?;
        Ok(json)
    }
    pub fn build_assets_task(self, storage: &Storage) -> impl Iterator<Item = DownloadTask> {
        self.objects.into_iter().map(|(name, object)| {
            let hash = &object.hash.to_string();
            DownloadTask::builder()
                .url(format!("{}/{}/{}", RESOURCES_URL, &hash[..2], hash))
                .to_asset(&Path::new("objects").join(&hash[..2]).join(hash), storage)
                .file_name(name)
                .file_size(object.size)
                .hash(object.hash)
                .build()
        })
    }
}

/// 资源索引
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: Sha1Hash,
    pub size: u64,
    pub total_size: Option<u64>,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AssetIndexList {
    pub objects: BTreeMap<String, AssetObject>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AssetObject {
    hash: Sha1Hash,
    size: u64,
}

impl AssetIndex {
    pub fn index_file_name(&self) -> &str {
        self.url
            .split('/')
            .next_back()
            .expect("Incorrect URL format")
    }
}

pub struct DownloadObject {
    pub url: String,
    pub sha1: Sha1Hash,
    pub path: PathBuf,
}
