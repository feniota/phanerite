use crate::download::vanilla::assets::AssetIndex;
use crate::download::vanilla::libraries::Library;
use crate::download::vanilla::version_index::{Version, VersionType};
use crate::download::{DownloadHandle, Downloadable};
use crate::error::{Error, Result};
use crate::io::utils::{AsyncFileExt, Hasher};
use crate::io::{AsyncFile, FileSystem, HttpClient, HttpRequest, Method};
use crate::storage::Storage;
use crate::utils::{HashValue, Sha1};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

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
    pub sha1: Sha1,
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

    pub sha1: Sha1,

    pub size: u64,

    pub url: String,
}

impl VersionInfo {
    #[instrument(skip(http_client))]
    pub async fn fetch(version: &Version, http_client: &impl HttpClient) -> Result<Self> {
        debug!(url = %version.url, "fetching version info");
        let request = HttpRequest {
            method: Method::Get,
            url: &version.url,
            headers: Default::default(),
            body: None,
        };

        let response = http_client.execute(request).await?;

        response.ok()?;

        let body = response.body.read_all().await?;

        let hash = {
            let mut hasher = sha1::Sha1::default();
            hasher.update(&body);
            hasher.finalize_hex()
        };

        if Sha1::from_hex(hash) != version.sha1 {
            return Err(Error::Other("hash mismatch".to_string()));
        }

        let json = serde_json::from_slice(&body).map_err(|e| Error::Other(e.to_string()))?;
        Ok(json)
    }
    pub async fn local(path: &Path, fs: &impl FileSystem) -> Result<Self> {
        let file = fs.open(path).await?.read_all().await?;
        let json = serde_json::from_slice(&file).map_err(|e| Error::Other(e.to_string()))?;
        Ok(json)
    }
    #[instrument(skip(self))]
    pub fn get_client(&self, path: PathBuf) -> Result<ClientDownload> {
        if let Some(client) = &self.downloads.client {
            Ok(ClientDownload {
                url: client.url.to_string(),
                sha1: client.sha1.clone(),
                path,
            })
        } else {
            Err(Error::Other("No download link".to_string()))
        }
    }
}

#[derive(Debug)]
pub struct ClientDownload {
    pub url: String,
    pub sha1: Sha1,
    pub path: PathBuf,
}

impl Downloadable for ClientDownload {
    type HashAlgorithm = Sha1;

    #[instrument(skip(http_client, _storage))]
    async fn download(
        self,
        http_client: &impl HttpClient,
        _storage: &Storage<impl FileSystem>,
    ) -> Result<DownloadHandle<impl AsyncFile, Self::HashAlgorithm>> {
        debug!("downloading client");
        let request = HttpRequest {
            method: Method::Get,
            url: &self.url,
            headers: Default::default(),
            body: None,
        };
        let response = http_client.execute_streaming(request).await?;
        response.ok()?;
        Ok(DownloadHandle {
            name: response.filename(),
            size: response.size(),
            stream: response.body,
            path: self.path,
            digest: Some(self.sha1),
        })
    }
}
