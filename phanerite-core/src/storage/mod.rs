use crate::error::Result;
use crate::io::utils::AsyncFileExt;
use crate::io::{AsyncFile, FileSystem};
use std::path::{Path, PathBuf};

pub struct AssetsStore<T: FileSystem> {
    fs: T,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    storage_dir: PathBuf,
}

impl<T: FileSystem> AssetsStore<T> {
    pub async fn new(fs: T, root: &Path) -> Result<Self> {
        let data_dir = root.join("data");
        let cache_dir = data_dir.join("cache");
        let storage_dir = data_dir.join("storage");

        if !data_dir.is_dir() {
            fs.create_dir_all(&data_dir)
                .await
                .expect("Failed to create data directory");
        }
        if !cache_dir.is_dir() {
            fs.create_dir_all(&cache_dir)
                .await
                .expect("Failed to create cache directory");
        }
        if !storage_dir.is_dir() {
            fs.create_dir_all(&storage_dir)
                .await
                .expect("Failed to create storage directory");
        }
        Ok(Self {
            fs,
            data_dir,
            cache_dir,
            storage_dir,
        })
    }
    pub async fn download_to(&self, stream: &impl AsyncFile, path: &Path) -> Result<()> {
        let cache_path = self.cache_dir.join(uuid::Uuid::now_v7().to_string());
        let cache = self.fs.create(&cache_path).await?;
        let mut hasher = blake3::Hasher::new();

        // 分片下载，逐块更新 hash
        let mut offset: u64 = 0;
        loop {
            let buf = vec![0u8; 8192];
            let (n, mut buf) = stream.read_at(offset, buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            buf.truncate(n);
            cache.write_all_at(offset, buf).await?;
            offset += n as u64;
        }

        let file_name = hasher.finalize().to_string();
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
}
