use crate::download::concurrent::ConcurrentTask;
use crate::download::task::DownloadTask;
use crate::error::Error;
use crate::error::Result;
use crate::storage::Storage;
use crate::utils::{Hash, Hasher};
use futures::{AsyncReadExt, AsyncWriteExt};
use nyquest::AsyncClient;
use std::borrow::Cow;
use std::num::NonZeroU8;
use std::path::PathBuf;
use tracing::{debug, error};
use uuid::Uuid;

pub struct Downloader {
    retries: usize,
    concurrent: usize,
    buffer_per_thread: usize,
    client: AsyncClient,
    cache: PathBuf,
}

impl Downloader {
    /// 构建默认下载器
    pub async fn new(storage: &Storage) -> Result<Self> {
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
    /// 设置重试次数
    pub fn retries(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }
    /// 设置并发数
    pub fn concurrent(mut self, max: NonZeroU8) -> Self {
        self.concurrent = max.get() as usize;
        self
    }
    /// 设置下载缓存
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer_per_thread = buffer;
        self
    }
    /// 下载到内存
    pub async fn fetch(
        &self,
        url: impl Into<Cow<'static, str>>,
        hash: Option<Hash>,
    ) -> Result<Vec<u8>> {
        let url = url.into();
        for _ in 0..self.retries {
            let req = nyquest::Request::get(url.clone());
            let res = self.client.request(req).await?.bytes().await?;
            if let Some(h) = &hash {
                let mut hasher = h.hasher();
                hasher.update(&res);
                let digest = hasher.finalize();
                if digest != *h {
                    error!("hash mismatch");
                    continue;
                }
            }
            return Ok(res);
        }
        Err(Error::Other("download failed after retries".to_string()))
    }
    /// 下载文件到储存
    pub async fn download(&self, task: DownloadTask) -> Result<()> {
        self.do_download(task, vec![0u8; self.buffer_per_thread])
            .await?;
        Ok(())
    }
    /// 并发下载文件到储存
    pub async fn download_concurrent(
        &self,
        tasks: impl Iterator<Item = DownloadTask>,
    ) -> Result<()> {
        let (pool_tx, pool_rx) = async_channel::bounded(self.concurrent);
        for _ in 0..self.concurrent {
            pool_tx
                .send(vec![0u8; self.buffer_per_thread])
                .await
                .unwrap();
        }

        let mut executor = ConcurrentTask::new(self.concurrent);
        tasks.for_each(|x| {
            executor.push(async {
                let mut buf = self.do_download(x, pool_rx.recv().await.unwrap()).await?;
                buf.resize(self.buffer_per_thread, 0);
                pool_tx.send(buf).await.unwrap();
                Ok(())
            })
        });

        executor.exec().await?;

        Ok(())
    }
    /// 执行下载
    // 允许不可到达代码（用于条件编译）
    #[allow(unreachable_code)]
    async fn do_download(&self, task: DownloadTask, mut buf: Vec<u8>) -> Result<Vec<u8>> {
        debug!(
            "downloading: {}",
            task.process.name().unwrap_or("unknown filename")
        );
        task.process.start();
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
                error!("hash mismatch");
                continue;
            }

            // 确定保存路径
            let save_path = if let Some(b) = &task.bucket {
                let hash = bucket_hasher.unwrap().finalize().to_string();
                b.join(&hash[..2]).join(hash)
            } else {
                task.target.clone()
            };

            // 确保父目录存在（先建目录再移动）
            let parent = save_path.parent().expect("invalid save path");
            if !parent.is_dir() {
                async_fs::create_dir_all(parent).await?;
            }

            // 移动到目标位置
            async_fs::rename(&cache, &save_path).await?;

            // 如果有 bucket，将共享文件链接到 task.target
            if task.bucket.is_some()
                && let Err(_e) = async_fs::hard_link(&save_path, &task.target).await
            {
                #[cfg(target_family = "unix")]
                {
                    async_fs::unix::symlink(save_path, task.target).await?;
                    break;
                }
                #[cfg(target_os = "windows")]
                {
                    async_fs::windows::symlink_file(save_path, task.target).await?;
                    break;
                }

                return Err(Error::Io(_e));
            }
            task.process.finish();
            return Ok(buf);
        }

        let _ = async_fs::remove_file(&cache).await;
        Err(Error::Other("download failed after retries".to_string()))
    }
}
