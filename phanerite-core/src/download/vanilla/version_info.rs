use crate::download::vanilla::assets::AssetIndex;
use crate::download::vanilla::libraries::Library;
use crate::download::vanilla::version_index::{Version, VersionType};
use crate::error::{Error, Result};
use crate::io::utils::{AsyncFileExt, Hasher};
use crate::io::{AsyncFile, FileSystem, HttpClient, HttpRequest, Method};
use crate::utils::{HashValue, Sha1};
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::slice::Iter;

#[derive(Deserialize)]
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
#[derive(Deserialize)]
pub struct Arguments {
    pub game: Option<Vec<Argument>>,
    pub jvm: Option<Vec<Argument>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Simple(String),

    Complex {
        rules: Option<Vec<Rule>>,
        value: Value,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<Os>,
}

#[derive(Deserialize)]
pub struct Os {
    pub name: Option<String>,
    pub arch: Option<String>,
}

/// 游戏下载
#[derive(Deserialize)]
pub struct Downloads {
    pub client: Option<Download>,
    pub server: Option<Download>,
}

#[derive(Deserialize)]
pub struct Download {
    pub sha1: Sha1,
    pub size: u64,
    pub url: String,
}

/// Java版本要求
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

/// 日志配置
#[derive(Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Deserialize)]
pub struct LoggingClient {
    pub argument: String,

    pub file: LoggingFile,
}

#[derive(Deserialize)]
pub struct LoggingFile {
    pub id: String,

    pub sha1: Sha1,

    pub size: u64,

    pub url: String,
}

impl VersionInfo {
    async fn fetch(version: &Version, http_client: &impl HttpClient) -> Result<Self> {
        let request = HttpRequest {
            method: Method::Get,
            url: &version.url,
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

        if Sha1::from_hex(hash) != version.sha1 {
            return Err(Error::Other("hash mismatch".to_string()));
        }

        let json = serde_json::from_slice(&body).map_err(|e| Error::Other(e.to_string()))?;
        Ok(json)
    }
    async fn local(path: &Path, fs: &impl FileSystem) -> Result<Self> {
        let file = fs.open(path).await?.read_all().await?;
        let json = serde_json::from_slice(&file).map_err(|e| Error::Other(e.to_string()))?;
        Ok(json)
    }
    async fn download_client<H: HttpClient>(
        &self,
        http_client: &H,
    ) -> Result<(impl AsyncFile, impl HashValue)> {
        if let Some(download) = &self.downloads.client {
            let request = HttpRequest {
                method: Method::Get,
                url: &download.url,
                headers: Default::default(),
                body: None,
            };
            let response = http_client.execute_streaming(request).await?;
            if response.status < 200 || response.status >= 300 {
                Err(Error::Http(response.status))
            } else {
                Ok((response.body, download.sha1.clone()))
            }
        } else {
            Err(Error::Other("No download link".to_string()))
        }
    }
}
