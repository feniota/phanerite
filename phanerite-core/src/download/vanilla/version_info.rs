use crate::download::downloader::Downloader;
use crate::download::vanilla::version_index::{Version, VersionType};
use crate::error::Result;
use crate::instance::instance_info::{Arguments, JavaVersion, Logging};
use crate::instance::instance_info::{AssetIndex, Library};
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

impl VersionManifest {
    pub async fn get(version: &Version, downloader: &Downloader) -> Result<Self> {
        let body = downloader
            .fetch(version.url.clone(), Some(version.sha1.clone().into()))
            .await?;
        let json = serde_json::from_slice(&body)?;
        Ok(json)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub id: String,

    #[serde(rename = "type")]
    pub version_type: VersionType,

    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,

    pub main_class: String,

    pub minimum_launcher_version: Option<u32>,

    pub arguments: Option<Arguments>,

    pub asset_index: AssetIndex,

    pub downloads: Downloads,

    pub java_version: Option<JavaVersion>,

    pub libraries: Vec<Library>,

    pub logging: Option<Logging>,

    // 旧版本字段
    pub minecraft_arguments: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 游戏下载
#[derive(Clone, Deserialize, Serialize)]
pub struct Downloads {
    pub client: Option<Download>,
    pub server: Option<Download>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Download {
    pub sha1: Sha1Hash,
    pub size: u64,
    pub url: Url,
}
