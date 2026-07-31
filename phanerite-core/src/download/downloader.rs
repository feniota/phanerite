use crate::download::task::{DownloadProcess, DownloadTask, Target};
use crate::error::{Error, Result};
use crate::storage::{ShareStrategy, Storage};
use crate::utils::{Hash, Hasher};
use async_channel::{Receiver, Sender};
use futures::{AsyncReadExt, AsyncWriteExt, Stream, StreamExt};
use http::{HeaderMap, StatusCode};
use isahc::config::{Configurable, RedirectPolicy};
use isahc::{AsyncReadResponseExt, HttpClient};
use std::mem::forget;
use std::num::NonZeroU8;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, error, warn};
use url::Url;
use uuid::Uuid;

pub struct DownloaderBuilder {
    /// 重试次数
    retries: usize,
    /// 最大并发数,
    concurrency: usize,
    /// 大文件阈值
    threshold: u64,
    /// 大文件并行度
    large_parallelism: usize,
    /// 小文件并行度
    small_parallelism: usize,
    /// 大文件缓冲大小
    large_buffer: usize,
    /// 小文件缓冲大小
    small_buffer: usize,
    /// 缓存目录
    cache: PathBuf,
    /// 共享储存目录
    bucket: PathBuf,
    /// 共享储存策略
    strategy: ShareStrategy,
}

impl DownloaderBuilder {
    /// 构建默认下载器
    fn new(storage: &Storage) -> Self {
        Self {
            retries: 3,
            concurrency: 32,
            threshold: 2 * 1024 * 1024,
            large_parallelism: 4,
            small_parallelism: 16,
            large_buffer: 512 * 1024,
            small_buffer: 128 * 1024,
            cache: storage.cache_dir().to_path_buf(),
            bucket: storage.share_dir().to_path_buf(),
            strategy: storage.share_strategy,
        }
    }
    /// 设置重试次数
    pub fn retries(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }
    /// 设置并发度
    pub fn concurrency(mut self, max: NonZeroU8) -> Self {
        self.concurrency = max.get() as usize;
        self
    }
    /// 设置大文件阈值
    pub fn threshold(mut self, threshold: u64) -> Self {
        self.threshold = threshold;
        self
    }
    /// 设置大文件并行度
    pub fn large_parallelism(mut self, max: NonZeroU8) -> Self {
        self.large_parallelism = max.get() as usize;
        self
    }
    /// 设置小文件并行度
    pub fn small_parallelism(mut self, max: NonZeroU8) -> Self {
        self.small_parallelism = max.get() as usize;
        self
    }
    /// 设置大文件下载缓冲
    pub fn large_buffer(mut self, buffer: usize) -> Self {
        self.large_buffer = buffer;
        self
    }
    /// 设置大文件下载缓冲
    pub fn small_buffer(mut self, buffer: usize) -> Self {
        self.small_buffer = buffer;
        self
    }
    pub async fn build(self) -> Result<Downloader> {
        let (large_tx, large_rx) = async_channel::bounded(self.large_parallelism);
        for _ in 0..self.large_parallelism {
            large_tx.send(vec![0u8; self.large_buffer]).await.unwrap()
        }
        let (small_tx, small_rx) = async_channel::bounded(self.small_parallelism);
        for _ in 0..self.small_parallelism {
            small_tx.send(vec![0u8; self.small_buffer]).await.unwrap()
        }
        if self.small_parallelism + self.large_parallelism > self.concurrency {
            warn!(
                "The parallelism is greater than the concurrency, and thus the buffer cannot be fully utilized."
            )
        }
        Ok(Downloader {
            retries: self.retries,
            concurrency: self.concurrency,
            cache: self.cache,
            bucket: self.bucket,
            strategy: self.strategy,
            threshold: self.threshold,
            client: HttpClient::builder()
                .redirect_policy(RedirectPolicy::Limit(10))
                .tcp_keepalive(Duration::from_secs(60))
                .low_speed_timeout(1024, Duration::from_secs(30))
                .dns_cache(Duration::from_secs(300))
                .connection_cache_size(self.concurrency * 2)
                .tcp_nodelay()
                .build()?,
            large_tx,
            large_rx,
            small_tx,
            small_rx,
        })
    }
}

pub struct Downloader {
    retries: usize,
    pub(crate) concurrency: usize,
    cache: PathBuf,
    bucket: PathBuf,
    strategy: ShareStrategy,
    threshold: u64,

    /// HTTP 客户端
    client: HttpClient,
    /// 获取大缓冲
    large_rx: Receiver<Vec<u8>>,
    /// 归还大缓冲
    large_tx: Sender<Vec<u8>>,
    /// 获取小缓冲
    small_rx: Receiver<Vec<u8>>,
    /// 归还小缓冲
    small_tx: Sender<Vec<u8>>,
}

impl Downloader {
    pub fn builder(storage: &Storage) -> DownloaderBuilder {
        DownloaderBuilder::new(storage)
    }
    /// 下载到内存（GET）
    pub async fn fetch(&self, url: &Url, hash: Option<Hash>) -> Result<Vec<u8>> {
        for _ in 0..self.retries {
            let res = self.client.get_async(url.as_str()).await?.bytes().await?;
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
        Err(Error::other("download failed after retries"))
    }
    /// 封装 POST
    pub async fn post_json(
        &self,
        url: &Url,
        body: impl AsRef<str>,
    ) -> Result<(StatusCode, Vec<u8>)> {
        let req = isahc::Request::post(url.as_str())
            .header("Content-Type", "application/json")
            .body(body.as_ref())
            .unwrap();
        let mut res = self.client.send_async(req).await?;
        Ok((res.status(), res.bytes().await?))
    }
    /// 封装 HEAD
    pub async fn head(&self, url: &Url) -> Result<HeaderMap> {
        Ok(self
            .client
            .head_async(url.as_ref())
            .await?
            .headers()
            .clone())
    }
    /// 下载文件到储存
    pub async fn download(&self, task: DownloadTask) -> Result<()> {
        struct FailGuard<'a> {
            process: &'a DownloadProcess,
        }
        impl Drop for FailGuard<'_> {
            fn drop(&mut self) {
                self.process.fail()
            }
        }
        let guard = FailGuard {
            process: &task.process,
        };

        // 准备工作
        debug!(
            "downloading: {}",
            task.process.name().unwrap_or("unknown filename")
        );
        task.process.start();
        let cache = self.cache.join(Uuid::now_v7().to_string());

        // 下载文件
        let retry_body = async || {
            // 共享储存桶 Hasher
            let mut bucket_hasher = match task.target {
                Target::File(_) => task.share.then_some(blake3::Hasher::new()),
                // 解压需要单独计算文件 Hash
                Target::Extract(_) => None,
            };

            // 构造和发送请求
            let uri = task.url.as_str();
            let mut res = self.client.get_async(uri).await?;

            // 补充文件大小（如果不存在）
            if let Ok(len) = res
                .headers()
                .get("content-length")
                .and_then(|t| t.to_str().ok())
                .unwrap_or_default()
                .parse()
            {
                task.process.set_total(len);
            }

            // 创建文件
            let mut file = async_fs::File::create(&cache).await?;
            let reader = res.body_mut();
            let mut hasher = task.file_hash.hasher();

            // 申请缓存并等待
            let mut buf = self.alloc_buf(task.process.total()).await;
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
            file.flush().await?;
            drop(buf);
            drop(res);
            drop(file);

            // 校验文件
            if task.file_hash == hasher.finalize() {
                Ok(bucket_hasher)
            } else {
                Err(Error::other(format!(
                    "hash mismatch: {}",
                    task.process.name().unwrap_or("Unknow file")
                )))
            }
        }; // RETRY_BODY

        let mut last_res = Ok(None);
        for _ in 0..=self.retries {
            match retry_body().await {
                Ok(v) => {
                    last_res = Ok(v);
                    break;
                }
                Err(Error::Cancelled) => {
                    forget(guard);
                    return Err(Error::Cancelled);
                }
                Err(e) => last_res = Err(e),
            }
            let _ = async_fs::remove_file(&cache).await;
        }
        let bucket_hasher = last_res?;

        match &task.target {
            // 直接保存
            Target::File(path) => {
                // 确定保存路径
                let save_path = if task.share {
                    let hash = bucket_hasher.unwrap().finalize().to_string();
                    &self.bucket.join(&hash[..2]).join(hash)
                } else {
                    path
                };

                // 确保父目录存在
                let parent = save_path.parent().expect("invalid save path");
                if !parent.is_dir() {
                    async_fs::create_dir_all(parent).await?;
                }

                // 移动到目标位置，共享桶存在文件则删除缓存，不执行操作
                if task.share || !save_path.exists() {
                    async_fs::rename(&cache, &save_path).await?
                } else {
                    async_fs::remove_file(&cache).await?
                };

                // 如果有 bucket，将共享文件链接到 task.target
                if task.share {
                    link_file(save_path, path, self.strategy).await?
                }
            }
            // 解压缩
            Target::Extract(extract) => {
                task.process.extracting();
                extract
                    .exec(
                        &cache,
                        task.share.then_some(self.bucket.clone()),
                        self.strategy,
                    )
                    .await?
            }
        } // Save or Extract

        task.process.finish();
        forget(guard);
        Ok(())
    }
    /// 并发下载文件到储存
    pub async fn download_concurrent(
        &self,
        tasks: impl Iterator<Item = DownloadTask>,
    ) -> impl Stream<Item = Error> {
        futures::stream::iter(tasks)
            .map(async |task| self.download(task).await)
            .buffer_unordered(self.concurrency)
            .filter_map(async |res| res.err())
    }
    /// 申请下载缓存，限制总并行度
    async fn alloc_buf(&self, size: Option<u64>) -> BufferGuard {
        match size {
            Some(size) if size > self.threshold => BufferGuard {
                buf: Some(self.large_rx.recv().await.expect("Failed to alloc buffer")),
                pool: self.large_tx.clone(),
            }, // >=阈值
            Some(size) if size <= self.threshold => BufferGuard {
                buf: Some(self.small_rx.recv().await.expect("Failed to alloc buffer")),
                pool: self.small_tx.clone(),
            }, // <=阈值
            _ => BufferGuard {
                buf: Some(self.small_rx.recv().await.expect("Failed to alloc buffer")),
                pool: self.small_tx.clone(),
            }, // 默认小文件
        }
    }
}

struct BufferGuard {
    buf: Option<Vec<u8>>,
    pool: Sender<Vec<u8>>,
}

impl Deref for BufferGuard {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buf.as_deref().unwrap()
    }
}

impl DerefMut for BufferGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_deref_mut().unwrap()
    }
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        if let Some(buf) = self.buf.take() {
            match self.pool.try_send(buf) {
                Ok(_) => {}
                Err(async_channel::TrySendError::Full(_))
                | Err(async_channel::TrySendError::Closed(_)) => {
                    error!("Buffer loss")
                }
            }
        }
    }
}

async fn link_file(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    strategy: ShareStrategy,
) -> Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();

    match strategy {
        ShareStrategy::Off => {
            async_fs::rename(source, target).await?;
        }
        ShareStrategy::Prefer => {
            if async_fs::hard_link(source, target).await.is_err() {
                async_fs::rename(source, target).await?;
            }
        }
        ShareStrategy::Fallback => {
            if async_fs::hard_link(source, target).await.is_ok() {
                return Ok(());
            }
            #[cfg(target_family = "unix")]
            if async_fs::unix::symlink(source, target).await.is_ok() {
                return Ok(());
            }
            #[cfg(target_os = "windows")]
            if async_fs::windows::symlink_file(source, target)
                .await
                .is_ok()
            {
                return Ok(());
            }
            async_fs::rename(source, target).await?;
        }
        ShareStrategy::Force => {
            async_fs::hard_link(source, target).await?;
        }
    }
    Ok(())
}
