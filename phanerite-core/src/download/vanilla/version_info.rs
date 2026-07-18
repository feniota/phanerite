use crate::download::vanilla::assets::AssetIndex;
use crate::download::vanilla::libraries::Library;
use crate::download::vanilla::version_index::VersionType;
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
