use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::download::vanilla::assets::{AssetIndex, AssetIndexList};
use crate::download::vanilla::libraries::Library;
use crate::download::vanilla::version_index::{Version, VersionType};
use crate::error::{Error, Result};
use crate::storage::Storage;
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Add;
use std::path::PathBuf;

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
        storage: &Storage,
        downloader: &Downloader,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        let client = self.build_client_task(client_path, storage);

        let assets_list = AssetIndexList::get(&self.asset_index, downloader).await?;
        let assets = assets_list.build_assets_task(storage);

        let library = self.build_libraries_task(storage);

        let chain = if let Some(c) = client {
            library.chain(assets).chain(std::iter::once(c))
        } else {
            return Err(Error::Other("No downloadable client".to_string()));
        };

        Ok(chain)
    }
    pub fn build_libraries_task(self, storage: &Storage) -> impl Iterator<Item = DownloadTask> {
        self.libraries
            .into_iter()
            .filter_map(|x| x.into_task(storage))
    }
    pub fn build_client_task(&self, path: PathBuf, storage: &Storage) -> Option<DownloadTask> {
        self.downloads.client.as_ref().map(|c| {
            DownloadTask::builder()
                .url(c.url.clone())
                .to_path(path, storage)
                .file_name(self.id.clone().add(".jar"))
                .file_size(c.size)
                .hash(c.sha1.clone())
                .build()
        })
    }
}

#[derive(Deserialize, Serialize)]
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

/// 启动参数
#[derive(Deserialize, Serialize)]
pub struct Arguments {
    pub game: Option<Vec<Argument>>,
    pub jvm: Option<Vec<Argument>>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
    Simple(String),

    Complex {
        rules: Option<Vec<Rule>>,
        value: Value,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Serialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<Os>,
}

#[derive(Deserialize, Serialize)]
pub struct Os {
    pub name: Option<String>,
    pub arch: Option<String>,
}

/// 游戏下载
#[derive(Deserialize, Serialize)]
pub struct Downloads {
    pub client: Option<Download>,
    pub server: Option<Download>,
}

#[derive(Deserialize, Serialize)]
pub struct Download {
    pub sha1: Sha1Hash,
    pub size: u64,
    pub url: String,
}

/// Java版本要求
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

/// 日志配置
#[derive(Deserialize, Serialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Deserialize, Serialize)]
pub struct LoggingClient {
    pub argument: String,

    pub file: LoggingFile,
}

#[derive(Deserialize, Serialize)]
pub struct LoggingFile {
    pub id: String,

    pub sha1: Sha1Hash,

    pub size: u64,

    pub url: String,
}
