//! Content-addressed storage layer.
//!
//! [`Storage`] manages a directory tree for deduplicated asset
//! storage: downloads land in `cache/`, are renamed into `share/`
//! under their Blake3 hash, and hard-linked or symlinked to their
//! final paths.

use crate::error::Result;
use crate::io::FileSystem;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

/// Content-addressed file store.
///
/// Manages a directory layout:
///
/// ```text
/// {root}/
///   data/
///     cache/      ← temp download files
///     share/      ← blake3-named deduplicated blobs
///     libraries/  ← library JARs
///     assets/     ← game assets
///   version/      ← per-version JSON + client JAR symlinks
/// ```
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
    #[instrument(skip(fs))]
    pub async fn new(root: &Path, fs: F) -> Result<Self> {
        debug!("creating storage dirs");
        let root_dir = root.to_owned();
        let cache_dir = root_dir.join("cache");
        let versions_dir = root_dir.join("version");
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
