use crate::download::extract::ExtractTask;
use crate::download::task::DownloadTask;
use crate::instance::instance_info::{Action, Rule};
use crate::storage::Storage;
use crate::utils::Sha1Hash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

impl Library {
    pub fn to_task(
        &self,
        storage: &Storage,
        features: &HashSet<&'static str>,
    ) -> Option<DownloadTask> {
        if self.allowed_env(features)
            && let Some(a) = self.downloads.as_ref().and_then(|t| t.artifact.clone())
        {
            Some(
                DownloadTask::builder()
                    .url(a.url)
                    .to_library(a.path, storage)
                    .file_name(self.name.clone())
                    .file_size(a.size)
                    .hash(a.sha1)
                    .build(),
            )
        } else {
            None
        }
    }
    pub fn to_native_task(
        &self,
        features: &HashSet<&'static str>,
        native_dir: &Path,
    ) -> Option<DownloadTask> {
        if !self.allowed_env(features) {
            return None;
        }

        let natives_map = self.natives.as_ref()?;
        let classifiers = self.downloads.as_ref()?.classifiers.as_ref()?;

        let os_key = match std::env::consts::OS {
            "macos" => "osx",
            other => other,
        };

        let classifier_key = natives_map.get(os_key)?;
        let artifact = classifiers.get(classifier_key)?;

        let extract_task = ExtractTask::builder().target(native_dir).zip().build();

        Some(
            DownloadTask::builder()
                .url(artifact.url.clone())
                .extract_to(extract_task)
                .file_name(format!("{}-{}", self.name, classifier_key))
                .file_size(artifact.size)
                .hash(artifact.sha1.clone())
                .build(),
        )
    }
    fn allowed_env(&self, features: &HashSet<&'static str>) -> bool {
        self.rules.as_ref().is_none_or(|x| {
            x.iter().fold(true, |b, rule| {
                rule.evaluate(features)
                    .map_or(b, |a| matches!(a, Action::Allow))
            })
        })
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
