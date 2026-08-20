use crate::download::cached::DownloaderWithCache;
use crate::download::group::DownloadGroup;
use crate::download::mirror::{DownloaderWithMirror, Mirror};
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::utils::Hash;
use futures::{Stream, StreamExt};
use http::{Request, Response};
use url::Url;

pub mod authlib_injector;
pub mod cached;
pub mod downloader;
pub mod extract;
pub mod group;
pub mod java;
pub mod mirror;
pub mod task;
pub mod vanilla;

#[allow(async_fn_in_trait)]
pub trait Downloader {
    /// 下载到内存（GET）
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Vec<u8>>;
    /// 封装 POST
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Vec<u8>>>;
    /// 封装 HEAD，仅保证响应头与状态码有效
    async fn head(&self, url: Url) -> Result<Response<()>>;
    /// 发送自定义请求，用于需要额外请求头或表单编码的 API
    ///
    /// 优先使用 `fetch()`/`post_json()`/`head()`，仅在它们无法表达请求时使用
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>>;
    /// 下载文件到储存
    async fn download(&self, task: DownloadTask) -> Result<()>;
    /// 并发量
    fn concurrency(&self) -> usize;
    /// 并发下载文件到储存
    fn download_concurrent(
        &self,
        tasks: impl IntoIterator<Item = DownloadTask>,
    ) -> impl Stream<Item = Result<()>> {
        futures::stream::iter(tasks)
            .map(async |task| self.download(task).await)
            .buffer_unordered(self.concurrency())
    }
}

pub trait DownloaderExt: Downloader + Sized {
    /// 获取适合读取进度的下载任务组
    fn with_group(&self) -> DownloadGroup<'_, Self> {
        DownloadGroup::new(self)
    }
    /// 获得带有镜像的下载器
    fn with_mirror<M: Mirror>(&self, mirror: M) -> DownloaderWithMirror<'_, Self, M> {
        DownloaderWithMirror::new(self, mirror)
    }

    /// 默认 `Downloader::fetch()` 的最大缓存字节数
    const MAX_GET_CACHE_BYTE: u64 = 5 * 1024 * 1024;
    /// 获得带有缓存的下载器
    fn with_cache(&self, get_bytes: u64) -> DownloaderWithCache<'_, Self> {
        DownloaderWithCache::new(self, get_bytes)
    }
    /// 获得带有缓存的下载器（默认缓存大小）
    fn with_cache_default(&self) -> DownloaderWithCache<'_, Self> {
        DownloaderWithCache::new(self, Self::MAX_GET_CACHE_BYTE)
    }
}

impl<D: Downloader> DownloaderExt for D {}
