pub mod bucket;

use crate::error::Result;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct Storage {
    root_dir: PathBuf,
    cache_dir: OnceLock<PathBuf>,
    versions_dir: OnceLock<PathBuf>,
    runtime_dir: OnceLock<PathBuf>,
    share_dir: OnceLock<PathBuf>,
    libraries_dir: OnceLock<PathBuf>,
    assets_dir: OnceLock<PathBuf>,
    assets_objects: OnceLock<PathBuf>,
    assets_indexes: OnceLock<PathBuf>,
}

impl Storage {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root_dir = std::path::absolute(root.as_ref())?;
        if !root_dir.exists() {
            std::fs::create_dir_all(&root_dir).ok();
        }
        Ok(Self {
            root_dir,
            cache_dir: OnceLock::new(),
            versions_dir: OnceLock::new(),
            runtime_dir: OnceLock::new(),
            share_dir: OnceLock::new(),
            libraries_dir: OnceLock::new(),
            assets_dir: OnceLock::new(),
            assets_objects: OnceLock::new(),
            assets_indexes: OnceLock::new(),
        })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn cache_dir(&self) -> &Path {
        self.cache_dir.get_or_init(|| dir(&self.root_dir, "cache"))
    }
    pub fn versions_dir(&self) -> &Path {
        self.versions_dir
            .get_or_init(|| dir(&self.root_dir, "versions"))
    }
    pub fn runtime_dir(&self) -> &Path {
        self.runtime_dir
            .get_or_init(|| dir(&self.root_dir, "runtime"))
    }
    pub fn share_dir(&self) -> &Path {
        self.share_dir.get_or_init(|| dir(&self.root_dir, "share"))
    }
    pub fn libraries_dir(&self) -> &Path {
        self.libraries_dir
            .get_or_init(|| dir(&self.root_dir, "libraries"))
    }
    pub fn assets_dir(&self) -> &Path {
        self.assets_dir
            .get_or_init(|| dir(&self.root_dir, "assets"))
    }
    pub fn assets_objects(&self) -> &Path {
        self.assets_objects
            .get_or_init(|| dir(&self.root_dir, "assets/objects"))
    }
    pub fn assets_indexes(&self) -> &Path {
        self.assets_indexes
            .get_or_init(|| dir(&self.root_dir, "assets/indexes"))
    }
}

fn dir(root: &Path, name: &str) -> PathBuf {
    let p = root.join(name);
    let _ = std::fs::create_dir_all(&p);
    p
}

impl Drop for Storage {
    fn drop(&mut self) {
        if let Some(cache) = self.cache_dir.get() {
            let _ = std::fs::remove_dir_all(cache);
        }
    }
}
