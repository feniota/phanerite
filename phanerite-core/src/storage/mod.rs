use crate::error::{Error, Result};
use crate::io::utils::{AsyncFileExt, Hasher};
use crate::io::{AsyncFile, FileSystem};
use crate::utils::HashValue;
use std::path::{Path, PathBuf};

pub struct Storage<F: FileSystem> {
    pub fs: F,
    pub root_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub versions_dir: PathBuf,
    pub share_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub assets_dir: PathBuf,
}

impl<F: FileSystem> Storage<F> {
    pub async fn new(root: &Path, fs: F) -> Result<Self> {
        let root_dir = root.join("data");
        let cache_dir = root_dir.join("cache");
        let versions_dir = root.join("version");
        let share_dir = root_dir.join("share");
        let libraries_dir = root_dir.join("libraries");
        let assets_dir = root_dir.join("assets");

        if !root_dir.is_dir() {
            fs.create_dir_all(&root_dir).await?;
        }
        if !cache_dir.is_dir() {
            fs.create_dir_all(&cache_dir).await?;
        }
        if !versions_dir.is_dir() {
            fs.create_dir_all(&versions_dir).await?
        }
        if !share_dir.is_dir() {
            fs.create_dir_all(&share_dir).await?;
        }
        if !libraries_dir.is_dir() {
            fs.create_dir_all(&libraries_dir).await?;
        }
        if !assets_dir.is_dir() {
            fs.create_dir_all(&assets_dir).await?;
        }

        Ok(Self {
            fs,
            root_dir,
            cache_dir,
            versions_dir,
            share_dir,
            libraries_dir,
            assets_dir,
        })
    }
}
