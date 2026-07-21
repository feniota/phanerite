use crate::download::task::DownloadTask;
use crate::instance::instance_info::{Action, Rule};
use crate::storage::Storage;
use crate::utils::Sha1Hash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

impl Library {
    pub fn into_task(
        self,
        storage: &Storage,
        features: &HashSet<&'static str>,
    ) -> Option<DownloadTask> {
        if let Some(d) = self.downloads
            && let Some(a) = d.artifact
            && self.rules.is_none_or(|x| {
                x.iter().fold(false, |b, rule| {
                    rule.evaluate(features)
                        .map_or(b, |a| matches!(a, Action::Allow))
                })
            })
        {
            Some(
                DownloadTask::builder()
                    .url(a.url)
                    .to_library(a.path, storage)
                    .file_name(self.name)
                    .file_size(a.size)
                    .hash(a.sha1)
                    .build(),
            )
        } else {
            None
        }
    }
}

/// Library
#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,

    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,

    pub sha1: Sha1Hash,

    pub size: u64,

    pub url: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}
