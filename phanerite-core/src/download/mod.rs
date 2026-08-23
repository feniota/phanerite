// 下载
//
// [`Downloader`] 是这个模块对外的唯一接口，能力靠包装逐层叠加：
// [`downloader::RawDownloader`] 负责真正的 HTTP，其余包装器都只持有内层
// 的引用并同样实现 `Downloader`，因此可以自由组合、就地丢弃。
//
// ```text
// RawDownloader              连接池、重试、缓冲池、流式落盘
//   └ with_cache()           GET 结果的内存缓存；共享桶命中则直接链接，不发请求
//       └ with_mirror()      按镜像规则改写 URL
//           └ with_group()   把任务登记进 Monitor，用于读进度
// ```
//
// 顺序不是随意的。`with_group` 只看得见经过它的任务，因此要套在最外层，
// 否则被缓存短路掉的任务不会计入进度。生命周期也不同：内侧几层通常与
// 程序或 `Storage` 同寿，而 [`group::DownloadGroup`] 应当一次性使用——
// 它的 [`group::Monitor`] 只增不减，复用会让进度越算越大。
//
// [`DownloaderExt::with_cache`] 使用内存记录器，进程重启后认不
// 出桶里已有的文件，会重新下载一遍。要持久化就自己实现
// [`cache::BucketRecorder`] 并传给 [`DownloaderExt::with_cache`]。
//
// # 任务
//
// [`task::DownloadTask`] 用 typestate builder 构造，URL 和目标缺一不可，
// 编译期就能保证。目标可以是一个文件，也可以是 [`extract::ExtractTask`]
// ——下载完直接解压，解压跑在独立线程上，不占执行器。
//
// 每个任务自带一份 [`task::DownloadProcess`]：进度和状态都用原子量维护，
// 可以在任务执行期间从任何地方读，也可以用它取消任务。
//
// 共享桶的粒度是「最终落盘的文件」：直接下载的文件按整个响应的 Blake3
// 入桶；解压任务则按解压出来的每个条目分别入桶，压缩包本身不保留。
// 空 Hash 的任务和解压任务不经过 [`cache`] 层。
//
// # 数据源
//
// [`vanilla`] 是官方的版本索引、资源与库，[`java`] 是 Java 运行时，
// [`authlib_injector`] 是第三方登录要用的注入器，[`mirror`] 是国内镜像。
//! Downloading
//!
//! [`Downloader`] is this module's only public interface; capabilities are
//! layered on by wrapping. [`downloader::RawDownloader`] does the actual
//! HTTP, and every other wrapper merely holds a reference to the layer
//! inside it and implements `Downloader` as well, so they compose freely and
//! can be dropped on the spot.
//!
//! ```text
//! RawDownloader            connection pool, retries, buffer pool, streaming to disk
//!   └ with_cache()         in-memory cache of GET results; a share-bucket hit links
//!                          directly and sends no request
//!       └ with_mirror()    rewrites URLs according to the mirror's rules
//!           └ with_group() registers tasks with a Monitor so progress can be read
//! ```
//!
//! The order is not arbitrary. `with_group` only sees the tasks that pass
//! through it, so it belongs on the outside; otherwise a task short-circuited
//! by the cache never counts towards progress. The lifetimes differ too: the
//! inner layers usually live as long as the program or as `Storage`, whereas
//! [`group::DownloadGroup`] should be used once and thrown away — its
//! [`group::Monitor`] only ever grows, so reusing it makes the progress
//! figures drift upwards.
//!
//! [`DownloaderExt::with_cache`] uses an in-memory recorder, which
//! will not recognise files already in the bucket after a restart and
//! downloads them again. To persist that state, implement
//! [`cache::StorageRegistry`] yourself and pass it to
//! [`DownloaderExt::with_cache`].
//!
//! # Tasks
//!
//! [`task::DownloadTask`] is built with a typestate builder, so the compiler
//! guarantees that neither the URL nor the target is missing. The target can
//! be a file or an [`extract::ExtractTask`] — extracted as soon as it is
//! downloaded, on a dedicated thread rather than on the executor.
//!
//! Every task carries its own [`task::DownloadProcess`]: progress and state
//! are kept in atomics, so they can be read from anywhere while the task
//! runs, and the same handle cancels it.
//!
//! The share bucket works at the granularity of "the file that ends up on
//! disk": a plain download enters the bucket under the Blake3 hash of the
//! whole response, while an extraction task enters each extracted entry
//! separately and keeps no copy of the archive itself. Tasks with an empty
//! hash, and extraction tasks, bypass the [`cache`] layer.
//!
//! # Sources
//!
//! [`vanilla`] is the official version index, assets and libraries; [`java`]
//! is the Java runtime; [`authlib_injector`] is the injector required for
//! third-party login; [`mirror`] holds the Chinese mirrors.

use crate::download::cache::{CachedDownloader, StorageRegistry};
use crate::download::group::DownloadGroup;
use crate::download::mirror::{DownloaderWithMirror, Mirror};
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::utils::Hash;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use http::{Request, Response};
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
pub trait Downloader: Send + Sync {
    // 下载到内存（GET）
    /// Downloads into memory (GET)
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Bytes>;
    // 封装 POST
    /// Wraps POST
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Bytes>>;
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
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>>;
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
    fn with_group(&self) -> DownloadGroup<Self, &Self> {
        DownloadGroup::new(self)
    }
    // 获得带有镜像的下载器
    /// Gets a downloader backed by a mirror
    fn with_mirror<M: Mirror>(&self, mirror: M) -> DownloaderWithMirror<Self, &Self, M> {
        DownloaderWithMirror::new(self, mirror)
    }

    // 默认 `Downloader::fetch()` 的最大缓存字节数
    /// Default maximum number of bytes `Downloader::fetch()` will cache
    const DEFAULT_GET_CACHE_BYTE: u64 = 5 * 1024 * 1024; // 5 MiB

    // 获得带有缓存的下载器
    /// Gets a downloader backed by a cache
    fn with_cache<S: StorageRegistry>(&self, storage_cache: S) -> CachedDownloader<Self, &Self, S> {
        CachedDownloader::new(self, storage_cache, ())
    }
}

impl<D: Downloader> DownloaderExt for D {}
