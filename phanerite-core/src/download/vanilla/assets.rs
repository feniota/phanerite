use crate::utils::Sha1;
use serde::Deserialize;

/// 资源索引
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: Sha1,
    pub size: u64,
    pub total_size: Option<u64>,
    pub url: String,
}
