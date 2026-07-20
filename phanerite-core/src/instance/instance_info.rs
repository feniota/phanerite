// Remote types from Mojang API — aliased to avoid conflicts with local types.
use crate::download::vanilla::libraries::Library as RemoteLibrary;
use crate::download::vanilla::version_info::{
    self as remote, Arguments as RemoteArguments, JavaVersion as RemoteJavaVersion,
    Logging as RemoteLogging, VersionInfo,
};
use crate::error::Result;
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use futures::AsyncReadExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Top-level Minecraft version manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub id: String,
    pub arguments: Arguments,
    pub main_class: String,
    pub jar: String,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub compliance_level: u32,
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub downloads: Downloads,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,
    pub minimum_launcher_version: Option<u32>,
    pub root: Option<bool>,
    #[serde(default)]
    pub patches: Vec<Patch>,
    /// 旧版本 minecraftArguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A single patch entry (e.g. the `game` patch).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub id: String,
    pub version: String,
    pub priority: i32,
    pub arguments: Arguments,
    pub main_class: String,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub compliance_level: u32,
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub downloads: Downloads,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,
    pub minimum_launcher_version: Option<u32>,
}

/// JVM / game launch arguments split into two arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

/// A single argument entry: either a plain string or a conditional block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Conditional(ConditionalArgument),
}

/// A group of arguments guarded by OS / feature rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalArgument {
    pub rules: Vec<Rule>,
    pub value: Vec<String>,
}

/// A single rule: OS-based or feature-based allow/deny.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<OsInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Allow,
    Disallow,
}

/// OS constraint (name + optional architecture).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

/// Asset index metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub total_size: u64,
    pub id: String,
    pub url: String,
    pub sha1: Sha1Hash,
    pub size: u64,
}

/// Java version requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

/// A single library dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    /// Maven-style coordinate, e.g. `com.google.code.gson:gson:2.14.0`
    pub name: String,
    pub downloads: LibraryDownloads,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
}

/// Download information for a library artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDownloads {
    pub artifact: Artifact,
}

/// A downloadable file with a Maven-style path (used for libraries).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub path: String,
    pub url: String,
    pub sha1: Sha1Hash,
    pub size: u64,
}

/// A download without a path field (used for client / server jars).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadInfo {
    pub url: String,
    pub sha1: Sha1Hash,
    pub size: u64,
}

/// Client / server download URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<DownloadInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<DownloadInfo>,
}

/// Client-side logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingClient {
    pub file: LoggingFileInfo,
    pub argument: String,
    #[serde(rename = "type")]
    pub type_: String,
}

/// Logging configuration file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingFileInfo {
    pub id: String,
    pub url: String,
    pub sha1: Sha1Hash,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

impl VersionManifest {
    /// Build an instance manifest from the remote version metadata.
    pub fn from_remote(remote: VersionInfo) -> Self {
        let assets = remote.asset_index.id.clone();
        let id = remote.id;
        let mut manifest = Self {
            id: id.clone(),
            arguments: convert_arguments(remote.arguments),
            main_class: remote.main_class,
            jar: id,
            asset_index: convert_asset_index(remote.asset_index),
            assets,
            compliance_level: 1,
            java_version: convert_java_version(remote.java_version),
            libraries: remote
                .libraries
                .into_iter()
                .filter_map(convert_library)
                .collect(),
            downloads: Downloads {
                client: remote.downloads.client.map(|d| DownloadInfo {
                    url: d.url,
                    sha1: d.sha1,
                    size: d.size,
                }),
                server: remote.downloads.server.map(|d| DownloadInfo {
                    url: d.url,
                    sha1: d.sha1,
                    size: d.size,
                }),
            },
            logging: remote.logging.map(convert_logging),
            version_type: remote.version_type,
            time: remote.time,
            release_time: remote.release_time,
            minimum_launcher_version: remote.minimum_launcher_version,
            root: Some(true),
            patches: vec![],
            minecraft_arguments: remote.minecraft_arguments,
            extra: filter_extra(remote.extra),
        };
        manifest.patches = vec![Patch {
            id: "game".into(),
            version: manifest.id.clone(),
            priority: 0,
            arguments: manifest.arguments.clone(),
            main_class: manifest.main_class.clone(),
            asset_index: manifest.asset_index.clone(),
            assets: manifest.assets.clone(),
            compliance_level: manifest.compliance_level,
            java_version: manifest.java_version.clone(),
            libraries: manifest.libraries.clone(),
            downloads: manifest.downloads.clone(),
            logging: manifest.logging.clone(),
            version_type: manifest.version_type.clone(),
            time: manifest.time,
            release_time: manifest.release_time,
            minimum_launcher_version: manifest.minimum_launcher_version,
        }];
        manifest
    }

    pub async fn form_local(path: &Path) -> Result<Self> {
        let mut buf = vec![];
        async_fs::File::open(path)
            .await?
            .read_to_end(&mut buf)
            .await?;
        Ok(serde_json::from_slice(&buf)?)
    }

    /// Override `id` and `jar` with the given instance name.
    pub fn rename(mut self, name: String) -> Self {
        self.id = name.clone();
        self.jar = name;
        self
    }
}

// ── Argument conversion ──────────────────────────────────────────────

fn convert_arguments(args: Option<RemoteArguments>) -> Arguments {
    let (game, jvm) = match args {
        Some(a) => (
            a.game
                .map(|v| v.into_iter().map(convert_argument).collect())
                .unwrap_or_default(),
            a.jvm
                .map(|v| v.into_iter().map(convert_argument).collect())
                .unwrap_or_default(),
        ),
        None => (vec![], vec![]),
    };
    Arguments { game, jvm }
}

fn convert_argument(a: remote::Argument) -> Argument {
    match a {
        remote::Argument::Simple(s) => Argument::String(s),
        remote::Argument::Complex { rules, value } => {
            let converted_value = match value {
                remote::Value::Single(s) => vec![s],
                remote::Value::Multiple(v) => v,
            };
            Argument::Conditional(ConditionalArgument {
                rules: rules
                    .map(|v| v.into_iter().map(convert_rule).collect())
                    .unwrap_or_default(),
                value: converted_value,
            })
        }
    }
}

fn convert_rule(r: remote::Rule) -> Rule {
    let action = match r.action.as_str() {
        "allow" => Action::Allow,
        "disallow" => Action::Disallow,
        _ => Action::Allow,
    };
    Rule {
        action,
        os: r.os.map(|o| OsInfo {
            name: o.name.unwrap_or_default(),
            arch: o.arch,
        }),
        features: r.features,
    }
}

// ── Asset index conversion ───────────────────────────────────────────

fn convert_asset_index(ai: crate::download::vanilla::assets::AssetIndex) -> AssetIndex {
    AssetIndex {
        total_size: ai.total_size.unwrap_or(0),
        id: ai.id,
        url: ai.url,
        sha1: ai.sha1,
        size: ai.size,
    }
}

// ── Java version conversion ──────────────────────────────────────────

fn convert_java_version(jv: Option<RemoteJavaVersion>) -> JavaVersion {
    match jv {
        Some(j) => JavaVersion {
            component: j.component,
            major_version: j.major_version,
        },
        None => JavaVersion {
            component: "java-runtime-alpha".into(),
            major_version: 8,
        },
    }
}

// ── Library conversion ───────────────────────────────────────────────

fn convert_library(lib: RemoteLibrary) -> Option<Library> {
    let artifact = lib.downloads?.artifact?;
    Some(Library {
        name: lib.name,
        downloads: LibraryDownloads {
            artifact: Artifact {
                path: artifact.path,
                url: artifact.url,
                sha1: artifact.sha1,
                size: artifact.size,
            },
        },
        rules: lib.rules.map(|v| v.into_iter().map(convert_rule).collect()),
    })
}

// ── Logging conversion ───────────────────────────────────────────────

fn convert_logging(log: RemoteLogging) -> Logging {
    let file = log.client.file;
    Logging {
        client: LoggingClient {
            file: LoggingFileInfo {
                id: file.id,
                url: file.url,
                sha1: file.sha1,
                size: file.size,
            },
            argument: log.client.argument,
            // Remote LoggingClient does not include `type`; vanilla defaults to log4j2-xml.
            type_: "log4j2-xml".into(),
        },
    }
}

/// Strip keys that `VersionManifest` already owns from the remote `extra` map,
/// so serde won't emit duplicate fields.
fn filter_extra(
    mut extra: HashMap<String, serde_json::Value>,
) -> HashMap<String, serde_json::Value> {
    extra.remove("assets");
    extra.remove("complianceLevel");
    extra.remove("id");
    extra.remove("type");
    extra.remove("time");
    extra.remove("releaseTime");
    extra.remove("minimumLauncherVersion");
    extra.remove("mainClass");
    extra.remove("arguments");
    extra.remove("assetIndex");
    extra.remove("javaVersion");
    extra.remove("libraries");
    extra.remove("downloads");
    extra.remove("logging");
    extra.remove("jar");
    extra.remove("patches");
    extra.remove("root");
    extra.remove("minecraftArguments");
    extra
}
