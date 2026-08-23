use crate::download::Downloader;
use crate::download::task::{DownloadTask, Target};
use crate::error::Result;
use crate::utils::{Hash, hash_file};
use bytes::Bytes;
use http::{Request, Response};
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::PathBuf;
use url::Url;

// 默认 `Downloader::fetch()` 的最大缓存字节数
/// Default maximum number of bytes `Downloader::fetch()` will cache
const DEFAULT_GET_CACHE_BYTE: u64 = 5 * 1024 * 1024; // 5 MiB

// 可用于记录共享储存桶 Hash 的记录器
// 必须区分 Storage 保存
/// A recorder for the hashes in the shared bucket
/// Must be kept separate per `Storage`
#[allow(async_fn_in_trait)]
pub trait BucketRecorder: Send + Sync {
    async fn query(&self, key: &Hash) -> Option<impl Deref<Target = PathBuf>>;
    async fn insert(&self, key: Hash, val: PathBuf);
}

impl BucketRecorder for scc::HashMap<Hash, PathBuf> {
    async fn query(&self, key: &Hash) -> Option<impl Deref<Target = PathBuf>> {
        self.get_async(key).await
    }
    async fn insert(&self, key: Hash, val: PathBuf) {
        let _ = self.insert_async(key, val).await;
    }
}

pub struct CachedDownloader<D: Downloader, B: Borrow<D> + Send + Sync, R: BucketRecorder> {
    get_cache: Option<moka::future::Cache<Url, Bytes>>,
    bucket_cache: R,

    downloader: B,
    _marker: PhantomData<fn() -> D>,
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: BucketRecorder> CachedDownloader<D, B, R> {
    pub fn new(downloader: B, get_bytes: u64, bucket_recorder: R) -> Self {
        Self {
            get_cache: if get_bytes == 0 {
                None
            } else {
                Some(
                    moka::future::Cache::builder()
                        .max_capacity(get_bytes)
                        .weigher(|_, value: &Bytes| value.len().div_ceil(1024) as u32)
                        .build(),
                )
            },
            bucket_cache: bucket_recorder,

            downloader,
            _marker: Default::default(),
        }
    }
    pub fn new_default(downloader: B) -> CachedDownloader<D, B, scc::HashMap<Hash, PathBuf>> {
        CachedDownloader {
            get_cache: Some(
                moka::future::Cache::builder()
                    .max_capacity(DEFAULT_GET_CACHE_BYTE)
                    .weigher(|_, value: &Bytes| value.len().div_ceil(1024) as u32)
                    .build(),
            ),
            bucket_cache: scc::HashMap::new(),

            downloader,
            _marker: Default::default(),
        }
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: BucketRecorder> Downloader
    for CachedDownloader<D, B, R>
{
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Bytes> {
        if let Some(cache) = &self.get_cache {
            cache
                .try_get_with(url.clone(), async {
                    self.downloader.borrow().fetch(url, hash).await
                })
                .await
                .map_err(|e| e.into())
        } else {
            self.downloader.borrow().fetch(url, hash).await
        }
    }
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Bytes>> {
        self.downloader.borrow().post_json(url, body).await
    }
    async fn head(&self, url: Url) -> Result<Response<()>> {
        self.downloader.borrow().head(url).await
    }
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>> {
        // 自定义请求可能携带凭据，不缓存
        self.downloader.borrow().send(req).await
    }
    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()> {
        // 空 Hash 和压缩包无法缓存，仅缓存共享的文件
        let (hash, dst, share) = match (&task.file_hash, &task.target, &task.share) {
            (hash, Target::File(dst), Some(share)) if !hash.is_empty() => {
                (hash, dst, share.clone())
            }
            _ => {
                self.downloader.borrow().download(task).await?;
                return Ok(());
            }
        };

        // 命中缓存
        if let Some(src) = self.bucket_cache.query(hash).await
            && hash_file(&src, hash).await.is_ok()
        {
            task.context.storage.linker()(src.deref(), dst).await?;
            return Ok(());
        }

        // 未命中缓存
        let hash = hash.clone();
        self.downloader.borrow().download(task).await?;
        // 记录地址
        if let Some(path) = share.get() {
            self.bucket_cache.insert(hash, path.to_owned()).await;
        };

        Ok(())
    }

    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
}
