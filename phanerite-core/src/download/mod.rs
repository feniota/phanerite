use crate::error::Error;
use crate::io::utils::{AsyncFileExt, Hasher};
use crate::io::{AsyncFile, FileSystem, HttpClient};
use crate::storage::Storage;
use crate::utils::HashValue;
use std::path::PathBuf;

pub mod vanilla;

pub trait Downloadable {
    type HashAlgorithm: HashValue;
    async fn download(
        self,
        http_client: &impl HttpClient,
        storage: &Storage<impl FileSystem>,
    ) -> crate::error::Result<(impl AsyncFile, Option<Self::HashAlgorithm>, PathBuf)>;
}

pub struct Downloader<F: FileSystem, H: HttpClient> {
    storage: Storage<F>,
    pub http_client: H,
}

impl<F: FileSystem, H: HttpClient> Downloader<F, H> {
    pub fn new(storage: Storage<F>, http_client: H) -> Self {
        Self {
            storage,
            http_client,
        }
    }
    /// 下载到共享储存桶，并链接到目标位置
    pub async fn download_to_bucket<T: Downloadable>(&self, task: T) -> crate::error::Result<()> {
        let (stream, digest, path) = task.download(&self.http_client, &self.storage).await?;

        let cache_path = self
            .storage
            .cache_dir
            .join(uuid::Uuid::now_v7().to_string());
        let cache = self.storage.fs.create(&cache_path).await?;

        let mut blake3_hasher = blake3::Hasher::new();
        let mut verify_hasher = T::HashAlgorithm::hasher();

        let mut offset = 0u64;
        loop {
            let buf = vec![0u8; 8192];
            let (n, mut buf) = stream.read_at(offset, buf).await?;
            if n == 0 {
                break;
            }
            blake3_hasher.update(&buf[..n]);
            if digest.is_some() {
                verify_hasher.update(&buf[..n]);
            }
            buf.truncate(n);
            cache.write_all_at(offset, buf).await?;
            offset += n as u64;
        }

        if let Some(expected) = digest
            && expected.to_string() != verify_hasher.finalize_hex()
        {
            return Err(Error::Other("hash mismatch".into()));
        }

        let file_name = blake3_hasher.finalize_hex();
        let save_bucket = self.storage.share_dir.join(&file_name[..2]);
        if !save_bucket.is_dir() {
            self.storage.fs.create_dir_all(&save_bucket).await?;
        }
        let save_path = save_bucket.join(file_name);
        self.storage.fs.rename(&cache_path, &save_path).await?;
        if self.storage.fs.hard_link(&save_path, &path).await.is_err() {
            self.storage.fs.symlink(&save_path, &path).await?;
        }
        Ok(())
    }
    /// 直接下载到目标位置
    pub async fn download_to_path<T: Downloadable>(&self, task: T) -> crate::error::Result<()> {
        let (stream, digest, path) = task.download(&self.http_client, &self.storage).await?;

        let cache_path = self
            .storage
            .cache_dir
            .join(uuid::Uuid::now_v7().to_string());
        let cache = self.storage.fs.create(&cache_path).await?;

        let (_, hash) = cache
            .copy_all_with_hasher(0, &stream, 0, T::HashAlgorithm::hasher())
            .await?;

        if let Some(v) = digest
            && v != T::HashAlgorithm::from_hex(hash)
        {
            return Err(Error::Other("hash mismatch".into()));
        }

        let save_dir = path.parent().unwrap();

        if !save_dir.is_dir() {
            self.storage.fs.create_dir_all(&save_dir).await?;
        }

        self.storage.fs.rename(&cache_path, &path).await?;

        Ok(())
    }
}
