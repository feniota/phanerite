use crate::download::Downloader;
use crate::download::task::{DownloadProcess, DownloadTask, Target};
use crate::error::{Error, Result};
use crate::storage::temp::TempGuard;
use crate::utils::Hash;
use async_channel::{Receiver, Sender};
use futures::{AsyncReadExt, AsyncWriteExt};
use http::{Request, Response};
use isahc::config::{Configurable, RedirectPolicy};
use isahc::{AsyncReadResponseExt, HttpClient};
use std::mem::forget;
use std::num::NonZeroU8;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tracing::{debug, error, warn};
use url::Url;

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
    /// UA
    user_agent: &'static str,
}

impl Default for DownloaderBuilder {
    fn default() -> Self {
        Self {
            retries: 3,
            concurrency: 32,
            threshold: 2 * 1024 * 1024,
            large_parallelism: 4,
            small_parallelism: 16,
            large_buffer: 512 * 1024,
            small_buffer: 128 * 1024,
            // 符合 Modrinth 规范的 User Agent
            user_agent: concat!(
                "feniota/phanerite/",
                env!("CARGO_PKG_VERSION"),
                " (",
                env!("CARGO_PKG_HOMEPAGE"),
                ")"
            ),
        }
    }
}

impl DownloaderBuilder {
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
    /// 设置 User-Agent
    pub fn user_agent(mut self, ua: &'static str) -> Self {
        self.user_agent = ua;
        self
    }

    pub async fn build(self) -> Result<RawDownloader> {
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
        Ok(RawDownloader {
            retries: self.retries,
            concurrency: self.concurrency,
            threshold: self.threshold,
            client: HttpClient::builder()
                .default_header("User-Agent", self.user_agent)
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

pub struct RawDownloader {
    retries: usize,
    concurrency: usize,
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

impl Downloader for RawDownloader {
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Vec<u8>> {
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
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Vec<u8>>> {
        let req = Request::post(url.as_str())
            .header("Content-Type", "application/json")
            .body(body.as_ref().as_bytes().to_vec())
            .expect("building a request from a valid URL should never fail");
        self.send(req).await
    }
    async fn head(&self, url: Url) -> Result<Response<()>> {
        let req = Request::head(url.as_str())
            .body(Vec::new())
            .expect("building a request from a valid URL should never fail");
        let (parts, _) = self.send(req).await?.into_parts();
        Ok(Response::from_parts(parts, ()))
    }
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>> {
        let mut res = self.client.send_async(req).await?;
        let body = res.bytes().await?;
        let (parts, _) = res.into_parts();
        Ok(Response::from_parts(parts, body))
    }
    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()> {
        /// 用于发送失败信号
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

        // 下载文件
        let mut last_res = Err(Error::other("Unreachable"));
        for _ in 0..=self.retries {
            match self.retry_body(&task).await {
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
        }

        // 保存或解压
        self.post_download(&task, last_res?).await?;

        task.process.finish();
        forget(guard);
        Ok(())
    }

    fn concurrency(&self) -> usize {
        self.concurrency
    }
}

impl RawDownloader {
    pub fn builder() -> DownloaderBuilder {
        DownloaderBuilder::default()
    }

    /// 重试体
    async fn retry_body<'cx>(
        &self,
        task: &DownloadTask<'cx>,
    ) -> Result<(TempGuard<'cx>, Option<blake3::Hash>)> {
        // 创建临时文件
        let cache = task.context.storage.temp_file().await?;

        // 共享储存桶 Hasher
        let mut bucket_hasher = match task.target {
            Target::File(_) => task.share.is_some().then_some(blake3::Hasher::new()),
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
            Ok((cache, bucket_hasher.map(|t| blake3::Hasher::finalize(&t))))
        } else {
            Err(Error::other(format!(
                "hash mismatch: {}",
                task.process.name().unwrap_or("Unknow file")
            )))
        }
    }
    /// 下载后的保存和解压
    async fn post_download<'cx>(
        &self,
        task: &DownloadTask<'cx>,
        (cache, bucket_hash): (TempGuard<'cx>, Option<blake3::Hash>),
    ) -> Result<()> {
        match &task.target {
            // 直接保存
            Target::File(path) => {
                // 若 task.share.is_some() 此值可以 unwrap()
                let bucket_path = bucket_hash.map(|hash| {
                    let file_name = hash.to_string();
                    task.context
                        .storage
                        .share_dir()
                        .join(&file_name[..2])
                        .join(file_name)
                });

                // 确定保存路径
                let save_path = if task.share.is_some() {
                    bucket_path.as_ref().unwrap()
                } else {
                    path
                };

                // 确保父目录存在
                let parent = save_path.parent().expect("invalid save path");
                if !parent.is_dir() {
                    async_fs::create_dir_all(parent).await?;
                }

                // 移动到目标位置，共享桶存在文件则不执行操作
                if task.share.is_some() || !save_path.exists() {
                    async_fs::rename(&cache, &save_path).await?
                };

                // 如果有 bucket，将共享文件链接到 task.target
                if let Some(record) = &task.share {
                    task.context.storage.linker()(save_path, path).await?;

                    // 初始化 Task 的共享位置（用于外部记录）
                    let _ = record.set(bucket_path.unwrap()).await;
                }
            }
            // 解压缩
            Target::Extract(extract) => {
                task.process.extracting();
                extract
                    .exec(&cache, task.share.is_some(), task.context.storage)
                    .await?
            }
        }

        Ok(())
    }
    /// 申请下载缓存，限制总并行度
    async fn alloc_buf(&self, size: Option<u64>) -> impl DerefMut<Target = [u8]> {
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
