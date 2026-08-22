use crate::download::cache::{BucketRecorder, DownloaderWithCache};
use crate::download::group::DownloadGroup;
use crate::download::mirror::{DownloaderWithMirror, Mirror};
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::utils::Hash;
use futures::{Stream, StreamExt};
use http::{Request, Response};
use std::path::PathBuf;
use url::Url;

pub mod authlib_injector;
pub mod cache;
pub mod downloader;
pub mod extract;
pub mod group;
pub mod java;
pub mod mirror;
pub mod task;
pub mod vanilla;

#[allow(async_fn_in_trait)]
pub trait Downloader {
    // 下载到内存（GET）
    /// Downloads into memory (GET)
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Vec<u8>>;
    // 封装 POST
    /// Wraps POST
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Vec<u8>>>;
    // 封装 HEAD，仅保证响应头与状态码有效
    /// Wraps HEAD; only the response headers and status code are guaranteed to
    /// be meaningful
    async fn head(&self, url: Url) -> Result<Response<()>>;
    // 发送自定义请求，用于需要额外请求头或表单编码的 API
    //
    // 优先使用 `fetch()`/`post_json()`/`head()`，仅在它们无法表达请求时使用
    /// Sends a custom request, for APIs that need extra headers or form
    /// encoding
    ///
    /// Prefer `fetch()`/`post_json()`/`head()`; use this only when they cannot
    /// express the request
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>>;
    // 下载文件到储存
    /// Downloads a file into storage
    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()>;

    // 并发量
    /// Concurrency
    fn concurrency(&self) -> usize;
    // 并发下载文件到储存
    /// Downloads files into storage concurrently
    fn download_concurrent<'cx>(
        &self,
        tasks: impl IntoIterator<Item = DownloadTask<'cx>>,
    ) -> impl Stream<Item = Result<()>> {
        futures::stream::iter(tasks)
            .map(async |task| self.download(task).await)
            .buffer_unordered(self.concurrency())
    }
}

pub trait DownloaderExt: Downloader + Sized {
    // 获取适合读取进度的下载任务组
    /// Gets a download task group suited for reading progress
    fn with_group(&self) -> DownloadGroup<'_, Self> {
        DownloadGroup::new(self)
    }
    // 获得带有镜像的下载器
    /// Gets a downloader backed by a mirror
    fn with_mirror<M: Mirror>(&self, mirror: M) -> DownloaderWithMirror<'_, Self, M> {
        DownloaderWithMirror::new(self, mirror)
    }

    // 默认 `Downloader::fetch()` 的最大缓存字节数
    /// Default maximum number of bytes `Downloader::fetch()` will cache
    const DEFAULT_GET_CACHE_BYTE: u64 = 5 * 1024 * 1024; // 5 MiB
    // 获得带有缓存的下载器
    /// Gets a downloader backed by a cache
    fn with_cache<R: BucketRecorder>(
        &self,
        get_bytes: u64,
        bucket_recorder: R,
    ) -> DownloaderWithCache<'_, Self, R> {
        DownloaderWithCache::new(self, get_bytes, bucket_recorder)
    }
    // 获得带有缓存的下载器（默认缓存大小）
    /// Gets a downloader backed by a cache (default cache size)
    fn with_cache_default(&self) -> DownloaderWithCache<'_, Self, scc::HashMap<Hash, PathBuf>> {
        DownloaderWithCache::new(self, Self::DEFAULT_GET_CACHE_BYTE, scc::HashMap::new())
    }
}

impl<D: Downloader> DownloaderExt for D {}
