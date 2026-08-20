use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LauncherProfiles {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,

    #[serde(default)]
    pub settings: LauncherSettings,

    #[serde(default)]
    pub version: u32,

    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub game_dir: Option<String>,

    #[serde(default)]
    pub java_dir: Option<String>,

    #[serde(default)]
    pub java_args: Option<String>,

    #[serde(default)]
    pub last_version_id: Option<String>,

    #[serde(default)]
    pub icon: Option<String>,

    #[serde(default)]
    pub type_: Option<String>,

    #[serde(default)]
    pub created: Option<String>,

    #[serde(default)]
    pub last_used: Option<String>,

    #[serde(default)]
    pub resolution: Option<Resolution>,

    #[serde(default)]
    pub custom_resolution: Option<bool>,

    #[serde(default)]
    pub quick_play_realms: Option<Vec<String>>,

    #[serde(default)]
    pub quick_play_singleplayer: Option<Vec<String>>,

    #[serde(default)]
    pub quick_play_multiplayer: Option<Vec<String>>,

    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Resolution {
    #[serde(default)]
    pub width: Option<u32>,

    #[serde(default)]
    pub height: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LauncherSettings {
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}
