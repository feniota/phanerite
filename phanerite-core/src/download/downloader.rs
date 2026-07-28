use crate::download::task::{DownloadProcess, DownloadTask, Target};
use crate::error::{Error, Result};
use crate::storage::{ShareStrategy, Storage};
use crate::utils::{Hash, Hasher};
use async_channel::{Receiver, Sender};
use futures::{AsyncReadExt, AsyncWriteExt, Stream, StreamExt};
use http::{HeaderMap, StatusCode};
use isahc::{AsyncReadResponseExt, HttpClient};
use std::mem::forget;
use std::num::NonZeroU8;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use tracing::{debug, error};
use uuid::Uuid;

pub struct DownloaderBuilder {
    /// 重试次数
    retries: usize,
    /// 最大并发数,
    max_concurrent: usize,
    /// 单个缓冲大小
    buffer_per_thread: usize,
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
            max_concurrent: 4,
            buffer_per_thread: 64 * 1024,
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
    /// 设置并发数
    pub fn concurrent(mut self, max: NonZeroU8) -> Self {
        self.max_concurrent = max.get() as usize;
        self
    }
    /// 设置下载缓存
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer_per_thread = buffer;
        self
    }
    pub async fn build(self) -> Result<Downloader> {
        let (pool_tx, pool_rx) = async_channel::bounded(self.max_concurrent);
        for _ in 0..self.max_concurrent {
            pool_tx
                .send(vec![0u8; self.buffer_per_thread])
                .await
                .unwrap()
        }
        Ok(Downloader {
            retries: self.retries,
            max_concurrent: self.max_concurrent,
            cache: self.cache,
            bucket: self.bucket,
            strategy: self.strategy,
            client: HttpClient::builder().build()?,
            pool_rx,
            pool_tx,
        })
    }
}

pub struct Downloader {
    retries: usize,
    pub max_concurrent: usize,
    cache: PathBuf,
    bucket: PathBuf,
    strategy: ShareStrategy,

    /// HTTP 客户端
    client: HttpClient,
    /// 获取缓冲
    pool_rx: Receiver<Vec<u8>>,
    /// 归还缓冲
    pool_tx: Sender<Vec<u8>>,
}

impl Downloader {
    pub fn builder(storage: &Storage) -> DownloaderBuilder {
        DownloaderBuilder::new(storage)
    }
    /// 下载到内存（GET）
    pub async fn fetch(&self, url: impl AsRef<str>, hash: Option<Hash>) -> Result<Vec<u8>> {
        for _ in 0..self.retries {
            let res = self.client.get_async(url.as_ref()).await?.bytes().await?;
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
        url: impl AsRef<str>,
        body: impl AsRef<str>,
    ) -> Result<(StatusCode, Vec<u8>)> {
        let req = isahc::Request::post(url.as_ref())
            .header("Content-Type", "application/json")
            .body(body.as_ref())
            .unwrap();
        let mut res = self.client.send_async(req).await?;
        Ok((res.status(), res.bytes().await?))
    }
    /// 封装 HEAD
    pub async fn head(&self, url: impl AsRef<str>) -> Result<HeaderMap> {
        Ok(self
            .client
            .head_async(url.as_ref())
            .await?
            .headers()
            .clone())
    }
    /// 下载文件到储存
    pub async fn download(&self, task: DownloadTask) -> Result<()> {
        // 申请缓存并等待
        let mut buf = self.alloc_buf().await;
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
        let mut retry_body = async || {
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

            // 校验文件
            if task.file_hash == hasher.finalize() {
                Ok(bucket_hasher)
            } else {
                Err(Error::other("hash mismatch"))
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
                        &mut buf,
                    )
                    .await?
            }
        } // Save or Extract

        task.process.finish();
        forget(guard);
        Ok(())
    }
    /// 校验文件 Hash，不检验压缩包
    pub async fn hash_file(&self, task: &DownloadTask) -> Result<()> {
        let Target::File(path) = &task.target else {
            return Ok(());
        };

        let mut buf = self.alloc_buf().await;
        let mut hasher = task.file_hash.hasher();
        let mut file = async_fs::File::open(path).await?;

        loop {
            let n = file.read(&mut buf).await?;

            if n == 0 {
                break;
            }

            hasher.update(&buf[..n]);
        }

        if hasher.finalize() == task.file_hash {
            Ok(())
        } else {
            Err(Error::other("Hash mismatch"))
        }
    }
    /// 并发下载文件到储存
    pub async fn download_concurrent(
        &self,
        tasks: impl Iterator<Item = DownloadTask>,
    ) -> impl Stream<Item = Error> {
        futures::stream::iter(tasks)
            .map(async |task| self.download(task).await)
            .buffer_unordered(self.max_concurrent)
            .filter_map(async |res| res.err())
    }
    /// 申请下载缓存，限制总并发量
    async fn alloc_buf(&self) -> BufferGuard {
        BufferGuard {
            buf: Some(self.pool_rx.recv().await.expect("Failed to alloc buffer")),
            pool: self.pool_tx.clone(),
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
                | Err(async_channel::TrySendError::Closed(_)) => {}
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
