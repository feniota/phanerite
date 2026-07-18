use crate::utils::Sha1Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

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

#[derive(Deserialize, Serialize)]
pub struct AssetIndexList {
    pub objects: BTreeMap<String, AssetObject>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct AssetObject {
    hash: Sha1Hash,
    size: usize,
}

impl AssetIndex {
    pub fn index_file_name(&self) -> &str {
        self.url.split('/').last().expect("Incorrect URL format")
    }
}

pub struct DownloadObject {
    pub url: String,
    pub sha1: Sha1Hash,
    pub path: PathBuf,
}
