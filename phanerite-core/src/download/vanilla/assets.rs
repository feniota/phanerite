use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::instance::manifest::AssetIndex;
use crate::storage::Storage;
use crate::utils::Sha1Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use url::Url;

static RESOURCES_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://resources.download.minecraft.net").unwrap());

impl AssetIndexList {
    pub async fn get(index: &AssetIndex, downloader: &Downloader) -> Result<Self> {
        let body = downloader
            .fetch(&index.url, Some(index.sha1.clone().into()))
            .await?;
        let json = serde_json::from_slice(&body)?;
        Ok(json)
    }
    pub fn build_assets_task(self, storage: &Storage) -> impl Iterator<Item = DownloadTask> {
        self.objects.into_iter().map(|(name, object)| {
            let hash = &object.hash.to_string();
            let url = RESOURCES_URL
                .join(&format!("{}/{}", &hash[..2], hash))
                .expect("Failed to parse url");
            DownloadTask::builder()
                .url(url)
                .to_asset(Path::new(&hash[..2]).join(hash), storage)
                .file_name(name)
                .file_size(object.size)
                .hash(object.hash)
                .build()
        })
    }
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
            .as_str()
            .split('/')
            .next_back()
            .expect("Incorrect URL format")
    }
}

pub struct DownloadObject {
    pub url: Url,
    pub sha1: Sha1Hash,
    pub path: PathBuf,
}
