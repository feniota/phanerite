use crate::download::task::DownloadTask;
use crate::error::{Error, Result};
use crate::instance::Instance;
use crate::mod_project::ModVersion;
use crate::mod_project::modrinth::serde::Version;
use std::fmt::Display;

impl ModVersion for Version {
    fn version(&self) -> &str {
        &self.version_number
    }
    fn change_log(&self) -> Option<impl Display + '_> {
        self.changelog.as_ref()
    }
    fn download<'cx, R: Clone, C: Clone>(
        &self,
        instance: &Instance<'cx, R, C>,
    ) -> Result<DownloadTask<'cx>> {
        let file = self
            .files
            .iter()
            .find(|file| file.primary)
            .or_else(|| self.files.first())
            .ok_or(Error::other("No available file"))?;
        Ok(DownloadTask::builder()
            .url(file.url.clone())
            .to_path(instance.instance_dir.join("mods"), instance.storage)
            .hash(file.hashes.sha1.clone())
            .file_name(file.filename.clone())
            .file_size(file.size)
            .share()
            .build())
    }
}
