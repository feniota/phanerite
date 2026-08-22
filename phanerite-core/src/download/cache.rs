use crate::download::Downloader;
use crate::download::task::{DownloadTask, Target};
use crate::error::Result;
use crate::utils::{Hash, hash_file};
use http::{Request, Response};
use std::ops::Deref;
use std::path::PathBuf;
use url::Url;

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

pub struct DownloaderWithCache<'downloader, D: Downloader, R: BucketRecorder> {
    downloader: &'downloader D,
    get_cache: Option<moka::future::Cache<Url, Vec<u8>>>,
    bucket_cache: R,
}

impl<'a, D: Downloader, R: BucketRecorder> DownloaderWithCache<'a, D, R> {
    pub(crate) fn new(downloader: &'a D, get_bytes: u64, bucket_recorder: R) -> Self {
        Self {
            downloader,
            get_cache: if get_bytes == 0 {
                None
            } else {
                Some(
                    moka::future::Cache::builder()
                        .max_capacity(get_bytes)
                        .weigher(|_, value: &Vec<u8>| value.len().div_ceil(1024) as u32)
                        .build(),
                )
            },
            bucket_cache: bucket_recorder,
        }
    }
}

impl<D: Downloader, R: BucketRecorder> Downloader for DownloaderWithCache<'_, D, R> {
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Vec<u8>> {
        if let Some(cache) = &self.get_cache {
            cache
                .try_get_with(url.clone(), async {
                    self.downloader.fetch(url, hash).await
                })
                .await
                .map_err(|e| e.into())
        } else {
            self.downloader.fetch(url, hash).await
        }
    }
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Vec<u8>>> {
        self.downloader.post_json(url, body).await
    }
    async fn head(&self, url: Url) -> Result<Response<()>> {
        self.downloader.head(url).await
    }
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>> {
        // 自定义请求可能携带凭据，不缓存
        self.downloader.send(req).await
    }
    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()> {
        // 空 Hash 和压缩包无法缓存，仅缓存共享的文件
        let (hash, dst, share) = match (&task.file_hash, &task.target, &task.share) {
            (hash, Target::File(dst), Some(share)) if !hash.is_empty() => {
                (hash, dst, share.clone())
            }
            _ => {
                self.downloader.download(task).await?;
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
        self.downloader.download(task).await?;
        // 记录地址
        if let Some(path) = share.get() {
            self.bucket_cache.insert(hash, path.to_owned()).await;
        };

        Ok(())
    }

    fn concurrency(&self) -> usize {
        self.downloader.concurrency()
    }
}
