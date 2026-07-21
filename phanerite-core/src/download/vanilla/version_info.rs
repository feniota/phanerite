use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::download::vanilla::assets::{AssetIndex, AssetIndexList};
use crate::download::vanilla::libraries::Library;
use crate::download::vanilla::version_index::{Version, VersionType};
use crate::error::{Error, Result};
use crate::instance::instance_info::{Arguments, JavaVersion, Logging};
use crate::storage::Storage;
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Add;
use std::path::{Path, PathBuf};

impl VersionInfo {
    pub async fn get(version: &Version, downloader: &Downloader) -> Result<Self> {
        let body = downloader
            .fetch(version.url.clone(), Some(version.sha1.clone().into()))
            .await?;
        let json = serde_json::from_slice(&body)?;
        Ok(json)
    }
    pub async fn build_all_task(
        self,
        client_path: PathBuf,
        native_path: &Path,
        features: &HashSet<&'static str>,
        storage: &Storage,
        downloader: &Downloader,
    ) -> Result<(impl Iterator<Item = DownloadTask>, AssetIndexList)> {
        let client = self.build_client_task(client_path, storage);

        let assets_list = AssetIndexList::get(&self.asset_index, downloader).await?;
        let assets = assets_list.clone().build_assets_task(storage);

        let library = self.build_libraries_task(storage, native_path, features);

        let chain = if let Some(c) = client {
            library.chain(assets).chain(std::iter::once(c))
        } else {
            return Err(Error::other("No downloadable client"));
        };

        Ok((chain, assets_list))
    }
    pub fn build_libraries_task(
        self,
        storage: &Storage,
        native_dir: &Path,
        features: &HashSet<&'static str>,
    ) -> impl Iterator<Item = DownloadTask> {
        self.libraries
            .into_iter()
            .flat_map(|x| {
                [
                    x.to_task(storage, features),
                    x.to_native_task(features, native_dir),
                ]
            })
            .flatten()
    }
    pub fn build_client_task(&self, path: PathBuf, storage: &Storage) -> Option<DownloadTask> {
        self.downloads.client.as_ref().map(|c| {
            DownloadTask::builder()
                .url(c.url.clone())
                .to_path(path)
                .share(storage)
                .file_name(self.id.clone().add(".jar"))
                .file_size(c.size)
                .hash(c.sha1.clone())
                .build()
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
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
    pub url: String,
}
