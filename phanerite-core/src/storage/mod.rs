use crate::error::{Error, Result};
use crate::io::utils::{AsyncFileExt, Hasher};
use crate::io::{AsyncFile, FileSystem};
use crate::utils::HashValue;
use std::path::{Path, PathBuf};

pub struct SharedStorage<F: FileSystem> {
    fs: F,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    storage_dir: PathBuf,
    libraries_dir: PathBuf,
    assets_dir: PathBuf,
}

impl<F: FileSystem> SharedStorage<F> {
    pub async fn new(root: &Path, fs: F) -> Result<Self> {
        let data_dir = root.join("data");
        let cache_dir = data_dir.join("cache");
        let storage_dir = data_dir.join("storage");
        let libraries_dir = data_dir.join("libraries");
        let assets_dir = data_dir.join("assets");

        if !data_dir.is_dir() {
            fs.create_dir_all(&data_dir).await?;
        }
        if !cache_dir.is_dir() {
            fs.create_dir_all(&cache_dir).await?;
        }
        if !storage_dir.is_dir() {
            fs.create_dir_all(&storage_dir).await?;
        }
        if !libraries_dir.is_dir() {
            fs.create_dir_all(&libraries_dir).await?;
        }
        if !assets_dir.is_dir() {
            fs.create_dir_all(&assets_dir).await?;
        }

        Ok(Self {
            fs,
            data_dir,
            cache_dir,
            storage_dir,
            libraries_dir,
            assets_dir,
        })
    }
    pub async fn download_to<H: HashValue>(
        &self,
        stream: &impl AsyncFile,
        digest: Option<H>,
        path: &Path,
    ) -> Result<()> {
        let cache_path = self.cache_dir.join(uuid::Uuid::now_v7().to_string());
        let cache = self.fs.create(&cache_path).await?;

        let mut blake3_hasher = blake3::Hasher::new();
        let mut verify_hasher = digest.as_ref().map(|_| H::hasher());

        let mut offset = 0u64;
        loop {
            let buf = vec![0u8; 8192];
            let (n, mut buf) = stream.read_at(offset, buf).await?;
            if n == 0 {
                break;
            }
            blake3_hasher.update(&buf[..n]);
            if let Some(ref mut h) = verify_hasher {
                h.update(&buf[..n]);
            }
            buf.truncate(n);
            cache.write_all_at(offset, buf).await?;
            offset += n as u64;
        }

        if let (Some(expected), Some(hasher)) = (digest, verify_hasher)
            && expected.to_string() != hasher.finalize_hex()
        {
            return Err(Error::Other("hash mismatch".into()));
        }

        let file_name = blake3_hasher.finalize_hex();
        let save_bucket = self.storage_dir.join(&file_name[..2]);
        if !save_bucket.is_dir() {
            self.fs.create_dir_all(&save_bucket).await?;
        }
        let save_path = save_bucket.join(file_name);
        self.fs.rename(&cache_path, &save_path).await?;
        if self.fs.hard_link(&save_path, path).await.is_err() {
            self.fs.symlink(&save_path, path).await?;
        }
        Ok(())
    }
    pub async fn download_library<H: HashValue>(
        &self,
        stream: &impl AsyncFile,
        digest: Option<H>,
        path: &Path,
    ) -> Result<()> {
        let cache_path = self.cache_dir.join(uuid::Uuid::now_v7().to_string());
        let cache = self.fs.create(&cache_path).await?;

        let (_, hash) = cache
            .copy_all_with_hasher(0, stream, 0, H::hasher())
            .await?;

        if let Some(v) = digest
            && v != H::from_hex(hash)
        {
            return Err(Error::Other("hash mismatch".into()));
        }

        let save_dir = self
            .libraries_dir
            .join(&path.parent().unwrap_or("".as_ref()));

        if !save_dir.is_dir() {
            self.fs.create_dir_all(&save_dir).await?;
        }

        self.fs
            .rename(&cache_path, &self.libraries_dir.join(&path))
            .await?;

        Ok(())
    }
}
