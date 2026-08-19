use crate::download::Downloader;
use crate::download::task::{DownloadTask, Target};
use crate::error::{Error, Result};
use crate::utils::{Hash, hash_file};
use http::{HeaderMap, Request, Response, StatusCode};
use std::path::PathBuf;
use url::Url;

pub struct DownloaderWithCache<'downloader, D: Downloader> {
    downloader: &'downloader D,
    get_cache: moka::future::Cache<Url, Vec<u8>>,
    head_cache: moka::future::Cache<Url, (StatusCode, HeaderMap)>,
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
                .weigher(|_, (_, headers): &(StatusCode, HeaderMap)| {
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
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Vec<u8>>> {
        self.downloader.post_json(url, body).await
    }
    async fn head(&self, url: Url) -> Result<Response<()>> {
        // Response 与 Parts 都不是 Clone（Extensions 无法克隆），不能直接进缓存，
        // 因此只缓存 HEAD 真正有意义的状态码与响应头，命中后手动还原 Response。
        // 代价是 Extensions（isahc 的 metrics、EffectiveUri 等）在缓存命中时丢失。
        let (status, headers) = self
            .head_cache
            .try_get_with(url.clone(), async {
                let (parts, _) = self.downloader.head(url).await?.into_parts();
                Ok::<_, Error>((parts.status, parts.headers))
            })
            .await?;

        let mut res = Response::new(());
        *res.status_mut() = status;
        *res.headers_mut() = headers;
        Ok(res)
    }
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>> {
        // 自定义请求可能携带凭据，不缓存
        self.downloader.send(req).await
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
