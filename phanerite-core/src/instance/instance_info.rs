use crate::download::vanilla::version_info::VersionManifest;
use crate::error::Result;
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use futures::AsyncReadExt;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use strum::{Display, EnumString};

/// Top-level Minecraft version manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceManifest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
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
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
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
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(deserialize_with = "deser_value_string_or_vec")]
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

impl Rule {
    /// Evaluate this rule against the current OS and a set of enabled features.
    ///
    /// Returns `Some(action)` if the rule's OS **and** feature conditions
    /// are all met, or `None` if this rule does not apply.
    pub fn evaluate(&self, features: &HashSet<&'static str>) -> Option<Action> {
        if let Some(ref os) = self.os
            && !os.matches_current()
        {
            return None;
        }
        if let Some(ref feats) = self.features {
            for (k, v) in feats {
                if features.contains(k.as_str()) != *v {
                    return None;
                }
            }
        }
        Some(self.action)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Allow,
    Disallow,
}

impl Action {
    pub fn allow(&self) -> bool {
        match self {
            Action::Allow => true,
            Action::Disallow => false,
        }
    }
}

/// OS constraint (name + optional architecture).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsInfo {
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

impl OsInfo {
    /// Check whether the current system is included by this OS constraint.
    ///
    /// - `name` empty → matches any OS.
    /// - `name` "osx" matches macOS (`std::env::consts::OS == "macos"`).
    /// - `arch` `None` → matches any architecture.
    pub fn matches_current(&self) -> bool {
        if !self.name.is_empty() {
            let current_os = std::env::consts::OS;
            let mapped = match current_os {
                "macos" => "osx",
                other => other,
            };
            if self.name != mapped {
                return false;
            }
        }
        if let Some(ref arch) = self.arch
            && arch != std::env::consts::ARCH
        {
            return false;
        }
        true
    }
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Library {
    pub name: String,

    pub downloads: Option<LibraryDownloads>,

    pub rules: Option<Vec<Rule>>,

    pub natives: Option<HashMap<String, String>>,

    pub extract: Option<Extract>,

    pub classifiers: Option<HashMap<String, Artifact>>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,

    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,

    pub sha1: Sha1Hash,

    pub size: u64,

    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
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

fn default_logging_type() -> String {
    "log4j2-xml".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingClient {
    pub file: LoggingFileInfo,
    pub argument: String,
    #[serde(rename = "type", default = "default_logging_type")]
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

#[derive(Debug, Clone, Copy, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    #[strum(to_string = "release")]
    Release,
    #[strum(to_string = "snapshot")]
    Snapshot,
    #[strum(to_string = "old_beta")]
    OldBeta,
    #[strum(to_string = "old_alpha")]
    OldAlpha,
}

impl InstanceManifest {
    /// Build an instance manifest from the remote version metadata.
    pub fn from_remote(remote: VersionManifest) -> Self {
        let assets = remote.asset_index.id.clone();
        let id = remote.id;
        let mut manifest = Self {
            id: id.clone(),
            arguments: remote.arguments,
            main_class: remote.main_class,
            jar: id,
            asset_index: convert_asset_index(remote.asset_index),
            assets,
            compliance_level: 1,
            java_version: remote.java_version.unwrap_or(JavaVersion {
                component: "java-runtime-alpha".into(),
                major_version: 8,
            }),
            libraries: remote.libraries,
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
            logging: remote.logging,
            version_type: remote.version_type,
            time: remote.time,
            release_time: remote.release_time,
            minimum_launcher_version: remote.minimum_launcher_version,
            root: Some(true),
            patches: vec![],
            minecraft_arguments: remote.minecraft_arguments,
            extra: filter_extra(remote.extra),
        };
        manifest.patches = manifest.arguments.clone().map_or(vec![], |args| {
            vec![Patch {
                id: "game".into(),
                version: manifest.id.clone(),
                priority: 0,
                arguments: args,
                main_class: manifest.main_class.clone(),
                asset_index: manifest.asset_index.clone(),
                assets: manifest.assets.clone(),
                compliance_level: manifest.compliance_level,
                java_version: manifest.java_version.clone(),
                libraries: manifest.libraries.clone(),
                downloads: manifest.downloads.clone(),
                logging: manifest.logging.clone(),
                version_type: manifest.version_type,
                time: manifest.time,
                release_time: manifest.release_time,
                minimum_launcher_version: manifest.minimum_launcher_version,
            }]
        });
        manifest
    }

    pub async fn form_local(path: impl AsRef<Path>) -> Result<Self> {
        let mut buf = vec![];
        async_fs::File::open(path)
            .await?
            .read_to_end(&mut buf)
            .await?;
        Ok(serde_json::from_slice(&buf)?)
    }

    /// Override `id` and `jar` with the given instance name.
    pub fn rename(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.id = name.clone();
        self.jar = name;
        self
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

// ── Custom deserializer: accept both `"str"` and `["a","b"]` ──────

fn deser_value_string_or_vec<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    struct StringOrVec;
    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or a sequence of strings")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
            Ok(vec![v.to_owned()])
        }

        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                v.push(s);
            }
            Ok(v)
        }
    }
    d.deserialize_any(StringOrVec)
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
