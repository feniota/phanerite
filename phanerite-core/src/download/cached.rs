use crate::download::Downloader;
use crate::download::task::{DownloadTask, Target};
use crate::error::Result;
use crate::utils::{Hash, hash_file};
use http::{HeaderMap, StatusCode};
use std::path::PathBuf;
use url::Url;

pub struct DownloaderWithCache<'downloader, D: Downloader> {
    downloader: &'downloader D,
    get_cache: moka::future::Cache<Url, Vec<u8>>,
    head_cache: moka::future::Cache<Url, HeaderMap>,
    bucket_cache: moka::future::Cache<Hash, PathBuf>,
}

impl<'a, D: Downloader> DownloaderWithCache<'a, D> {
    pub(crate) fn new(downloader: &'a D, get_bytes: u64, head_bytes: u64) -> Self {
        Self {
            downloader,
            get_cache: moka::future::Cache::builder()
                .max_capacity(get_bytes)
                .weigher(|_, value: &Vec<u8>| value.len().div_ceil(1024) as u32)
                .build(),
            head_cache: moka::future::Cache::builder()
                .max_capacity(head_bytes)
                .weigher(|_, headers: &HeaderMap| {
                    headers
                        .iter()
                        .map(|(name, value)| (name.as_str().len() + value.as_bytes().len()) as u32)
                        .sum()
                })
                .build(),
            bucket_cache: moka::future::Cache::builder().build(),
        }
    }
}

impl<D: Downloader> Downloader for DownloaderWithCache<'_, D> {
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Vec<u8>> {
        self.get_cache
            .try_get_with(url.clone(), async {
                self.downloader.fetch(url, hash).await
            })
            .await
            .map_err(|e| e.into())
    }
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<(StatusCode, Vec<u8>)> {
        self.downloader.post_json(url, body).await
    }
    async fn head(&self, url: Url) -> Result<HeaderMap> {
        self.head_cache
            .try_get_with(url.clone(), async { self.downloader.head(url).await })
            .await
            .map_err(|e| e.into())
    }
    async fn download(&self, task: DownloadTask) -> Result<()> {
        // 空 Hash 和压缩包无法缓存
        let (hash, dst) = match (&task.file_hash, &task.target) {
            (hash, Target::File(dst)) if !hash.is_empty() => (hash, dst),
            _ => {
                self.downloader.download(task).await?;
                return Ok(());
            }
        };

        // 命中缓存
        if let Some(src) = self.bucket_cache.get(hash).await {
            if hash_file(&src, hash).await.is_ok() {
                async_fs::copy(&src, dst).await?;
                return Ok(());
            }
            self.bucket_cache.invalidate(hash).await;
        }

        // 未命中缓存
        let dst = dst.clone();
        self.bucket_cache
            .try_get_with(hash.clone(), async {
                self.downloader.download(task).await?;
                Ok(dst)
            })
            .await?;
        Ok(())
    }
    fn concurrency(&self) -> usize {
        self.downloader.concurrency()
    }
}
