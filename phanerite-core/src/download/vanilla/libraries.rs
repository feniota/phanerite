use crate::download::vanilla::version_info::Rule;
use crate::utils::Sha1Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Library
#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,

    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,

    pub sha1: Sha1Hash,

    pub size: u64,

    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}
