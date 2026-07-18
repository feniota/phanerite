use crate::download::concurrent::ConcurrentTask;
use crate::download::task::DownloadTask;
use crate::error::Error;
use crate::storage::Storage;
use crate::utils::Hasher;
use futures::{AsyncReadExt, AsyncWriteExt};
use nyquest::AsyncClient;
use std::num::NonZeroU8;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Downloader {
    retries: usize,
    concurrent: usize,
    buffer_per_thread: usize,
    client: AsyncClient,
    cache: PathBuf,
}

impl Downloader {
    pub async fn new(storage: &Storage) -> nyquest::Result<Self> {
        Ok(Self {
            retries: 3,
            concurrent: 4,
            buffer_per_thread: 64 * 1024,
            client: nyquest::client::ClientBuilder::default()
                .build_async()
                .await?,
            cache: storage.cache_dir.clone(),
        })
    }
    pub fn retries(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }
    pub fn concurrent(mut self, max: NonZeroU8) -> Self {
        self.concurrent = max.get() as usize;
        self
    }
    pub async fn download(&self, task: DownloadTask) -> crate::error::Result<()> {
        let cache = self.cache.join(Uuid::now_v7().to_string());
        for _ in 0..self.retries {
            // 构造和发送请求
            let req = nyquest::Request::get(task.url.clone());
            let res = self.client.request(req).await?;

            // 补充文件大小（如果不存在）
            if let Some(len) = res.content_length() {
                task.process.set_total(len);
            }

            // 创建文件
            let mut file = async_fs::File::create(&cache).await?;
            let mut buf = vec![0u8; self.buffer_per_thread];
            let mut reader = res.into_async_read();
            let mut hasher = task.file_hash.hasher();
            let mut bucket_hasher = task.bucket.as_ref().map(|_| blake3::Hasher::new());

            // 流式下载
            loop {
                let n = reader.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                if task.process.is_canceled() {
                    async_fs::remove_file(&cache).await?;
                    return Err(Error::Cancelled);
                }

                file.write_all(&buf[..n]).await?;

                hasher.update(&buf[..n]);
                if let Some(ref mut h) = bucket_hasher {
                    h.update(&buf[..n]);
                }
                task.process.step(n as u64);
            }

            // 校验文件
            if task.file_hash != hasher.finalize() {
                async_fs::remove_file(&cache).await?;
                continue;
            }

            // 保存位置
            let save_path = if let Some(b) = &task.bucket {
                let hash = bucket_hasher.unwrap().finalize().to_string();
                b.join(&hash[..2]).join(hash)
            } else {
                task.target.clone()
            };

            // 移动到目标
            let parent = save_path.parent().expect("Error path format");
            if !parent.is_dir() {
                async_fs::create_dir_all(parent).await?;
            }
            async_fs::rename(&cache, save_path).await?;
            return Ok(());
        }

        Err(Error::Other("download failed after retries".to_string()))
    }
    pub async fn download_concurrent(
        &self,
        tasks: impl Iterator<Item = DownloadTask>,
    ) -> crate::error::Result<()> {
        let mut executor = ConcurrentTask::new(self.concurrent);
        tasks.for_each(|x| executor.push(self.download(x)));

        executor.exec().await?;

        Ok(())
    }
}
