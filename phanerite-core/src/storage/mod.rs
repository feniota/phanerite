use crate::error::Result;
use std::path::{Path, PathBuf};
use tracing::debug;

pub struct Storage {
    pub root_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub versions_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub share_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub assets_objects: PathBuf,
    pub assets_indexes: PathBuf,
}

impl Storage {
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        debug!("creating storage dirs");
        let root_dir = std::path::absolute(root.as_ref())?;
        let cache_dir = root_dir.join("cache");
        let versions_dir = root_dir.join("versions");
        let runtime_dir = root_dir.join("runtime");
        let share_dir = root_dir.join("share");
        let libraries_dir = root_dir.join("libraries");
        let assets_dir = root_dir.join("assets");
        let assets_objects = assets_dir.join("objects");
        let assets_indexes = assets_dir.join("indexes");

        if !root_dir.is_dir() {
            async_fs::create_dir_all(&root_dir).await?;
        }
        if !cache_dir.is_dir() {
            async_fs::create_dir_all(&cache_dir).await?;
        }
        if !versions_dir.is_dir() {
            async_fs::create_dir_all(&versions_dir).await?
        }
        if !runtime_dir.is_dir() {
            async_fs::create_dir_all(&runtime_dir).await?
        }
        if !share_dir.is_dir() {
            async_fs::create_dir_all(&share_dir).await?;
        }
        if !libraries_dir.is_dir() {
            async_fs::create_dir_all(&libraries_dir).await?;
        }
        if !assets_dir.is_dir() {
            async_fs::create_dir_all(&assets_dir).await?;
        }
        if !assets_objects.is_dir() {
            async_fs::create_dir_all(&assets_objects).await?;
        }
        if !assets_indexes.is_dir() {
            async_fs::create_dir_all(&assets_indexes).await?;
        }

        Ok(Self {
            root_dir,
            cache_dir,
            versions_dir,
            runtime_dir,
            share_dir,
            libraries_dir,
            assets_dir,
            assets_objects,
            assets_indexes,
        })
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.cache_dir);
    }
}
