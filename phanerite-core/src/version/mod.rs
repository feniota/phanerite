use crate::download::vanilla::version_info::VersionInfo;
use crate::error::Result;
use crate::io::FileSystem;
use std::path::{Path, PathBuf};

pub struct VersionsManager<F: FileSystem> {
    fs: F,
    versions_dir: PathBuf,
}

impl<F: FileSystem> VersionsManager<F> {
    pub async fn new(root: &Path, fs: F) -> Result<Self> {
        let versions_dir = root.join("version");
        if !versions_dir.is_dir() {
            fs.create_dir_all(&versions_dir).await?
        }
        Ok(Self { fs, versions_dir })
    }
    pub async fn creat_version(&self, name: &str, version: &VersionInfo) -> Result<PathBuf> {
        let version_path = self.versions_dir.join(name);
        self.fs.create_dir_all(&version_path).await?;

        todo!()
    }
}
