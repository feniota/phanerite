use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::utils::Hash;
use bytes::Bytes;
use http::{HeaderMap, Response};
use std::borrow::Borrow;
use std::marker::PhantomData;
use url::Url;

// 默认 `Downloader::get_check()` 的最大缓存字节数
/// Default maximum number of bytes `Downloader::get_check()` will cache
const DEFAULT_GET_CACHE_BYTE: u64 = 5 * 1024 * 1024; // 5 MiB

pub struct CachedDownloader<D: Downloader, B: Borrow<D> + Send + Sync> {
    get_cache: Option<moka::future::Cache<(Url, Hash), Bytes>>,

    downloader: B,
    _marker: PhantomData<fn() -> D>,
}

impl<D: Downloader, B: Borrow<D> + Send + Sync> CachedDownloader<D, B> {
    pub fn new(downloader: B, get_bytes: u64) -> Self {
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

            downloader,
            _marker: Default::default(),
        }
    }
    pub fn new_default(downloader: B) -> CachedDownloader<D, B> {
        CachedDownloader {
            get_cache: Some(
                moka::future::Cache::builder()
                    .max_capacity(DEFAULT_GET_CACHE_BYTE)
                    .weigher(|_, value: &Bytes| value.len().div_ceil(1024) as u32)
                    .build(),
            ),

            downloader,
            _marker: Default::default(),
        }
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync> Downloader for CachedDownloader<D, B> {
    async fn get(&self, url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        self.downloader.borrow().get(url, headers).await
    }
    async fn get_check(&self, url: Url, hash: Hash) -> Result<Bytes> {
        if let Some(cache) = &self.get_cache {
            cache
                .try_get_with((url.clone(), hash.clone()), async {
                    self.downloader.borrow().get_check(url, hash).await
                })
                .await
                .map_err(|e| e.into())
        } else {
            self.downloader.borrow().get_check(url, hash).await
        }
    }
    async fn post(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().post(url, headers, body).await
    }
    async fn put(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().put(url, headers, body).await
    }
    async fn patch(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().patch(url, headers, body).await
    }
    async fn delete(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().delete(url, headers, body).await
    }
    async fn head(&self, url: Url, headers: HeaderMap) -> Result<Response<()>> {
        self.downloader.borrow().head(url, headers).await
    }
    async fn options(&self, url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        self.downloader.borrow().options(url, headers).await
    }
    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()> {
        self.downloader.borrow().download(task).await
    }

    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
}
