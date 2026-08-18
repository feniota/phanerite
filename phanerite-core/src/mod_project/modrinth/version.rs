use crate::download::task::DownloadTask;
use crate::error::{Error, Result};
use crate::mod_project::modrinth::serde::Version;
use crate::mod_project::ModVersion;
use std::fmt::Display;
use std::path::Path;

impl ModVersion for Version {
    fn version(&self) -> &str {
        &self.version_number
    }
    fn change_log(&self) -> Option<impl Display + '_> {
        self.changelog.as_ref()
    }
    fn download(&self, dir: impl AsRef<Path>) -> Result<DownloadTask> {
        let file = self
            .files
            .iter()
            .find(|file| file.primary)
            .or_else(|| self.files.first())
            .ok_or(Error::other("No available file"))?;
        Ok(DownloadTask::builder()
            .url(file.url.clone())
            .to_path(dir.as_ref().join(file.filename.clone()))
            .hash(file.hashes.sha1.clone())
            .file_name(file.filename.clone())
            .file_size(file.size)
            .share()
            .build())
    }
}
