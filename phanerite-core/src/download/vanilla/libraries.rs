use crate::download::extract::ExtractTask;
use crate::download::task::DownloadTask;
use crate::instance::instance_info::{Action, Artifact, Extract, Library};
use crate::storage::Storage;
use std::collections::HashSet;
use std::path::Path;

impl Library {
    pub fn to_task(
        &self,
        storage: &Storage,
        features: &HashSet<&'static str>,
    ) -> Option<DownloadTask> {
        if !self.allowed_env(features) {
            return None;
        }
        let a = self.downloads.as_ref()?.artifact.clone()?;
        Some(
            DownloadTask::builder()
                .url(a.url)
                .to_library(a.path, storage)
                .file_name(self.name.clone())
                .file_size(a.size)
                .hash(a.sha1)
                .build(),
        )
    }

    pub fn to_native_task(
        &self,
        features: &HashSet<&'static str>,
        native_dir: &Path,
    ) -> Option<DownloadTask> {
        if !self.allowed_env(features) {
            return None;
        }

        let os_key = match std::env::consts::OS {
            "macos" => "osx",
            other => other,
        };

        // ── v1: natives map + classifiers ────────────────
        if let Some(natives_map) = &self.natives {
            let classifiers = self.downloads.as_ref()?.classifiers.as_ref()?;
            let classifier_key = natives_map.get(os_key)?;
            let artifact = classifiers.get(classifier_key)?;
            return Some(native_download(
                artifact,
                native_dir,
                &format!("{}-{}", self.name, classifier_key),
                self.extract.as_ref(),
            ));
        }

        // ── v2: classifier suffix in Maven name ──────────
        // e.g. `org.lwjgl:lwjgl:3.4.1:natives-windows`
        if let Some(classifier) = self.name.split(':').nth(3)
            && classifier.starts_with("natives-")
        {
            // v2 rules alone don't filter by OS — the classifier name acts
            // as the platform gate. Skip natives that aren't for this OS.
            let target_os = &classifier["natives-".len()..];
            let current_os = match std::env::consts::OS {
                "macos" => "macos",
                "windows" => "windows",
                "linux" => "linux",
                _ => return None,
            };
            if !target_os.starts_with(current_os) {
                return None;
            }
            let a = self.downloads.as_ref()?.artifact.clone()?;
            return Some(native_download(
                &a,
                native_dir,
                &self.name,
                self.extract.as_ref(),
            ));
        }

        None
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

fn native_download(
    artifact: &Artifact,
    native_dir: &Path,
    file_name: &str,
    extract: Option<&Extract>,
) -> DownloadTask {
    let mut builder = ExtractTask::builder().target(native_dir).zip().flatten();
    if let Some(ex) = extract
        && let Some(ref patterns) = ex.exclude
    {
        builder = builder.exclude(patterns.iter().cloned());
    }
    DownloadTask::builder()
        .url(artifact.url.clone())
        .extract_to(builder.build())
        .file_name(file_name)
        .file_size(artifact.size)
        .hash(artifact.sha1.clone())
        .share()
        .build()
}
