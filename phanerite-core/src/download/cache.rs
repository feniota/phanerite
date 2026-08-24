//! Caching utilities for [`Downloader`].
//!
//! This module provides a generic and powerful [`Downloader`] wrapper,
//! [`CachedDownloader`], which can save resources by caching requests.

use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::utils::Hash;
use bytes::Bytes;
use http::{Request, Response};
use std::borrow::Borrow;
use std::future::Future;
use std::ops::Deref;
use url::Url;

/// A replaceable cache for [`fetch`](Downloader::fetch) requests.
///
/// # `fetch()` only
///
/// Implementors of this trait handle only the caching of
/// [`fetch`](Downloader::fetch) requests. For [`DownloadTask`] requests, see
/// [`StorageRegistry`](super::dedup::StorageRegistry) instead.
///
/// See [`CachedDownloader`] and [`DeduplicateDownloader`](super::dedup::DeduplicateDownloader)
/// for more details.
#[allow(async_fn_in_trait)]
pub trait FetchCache: Send + Sync {
    /// Try to resolve an entry from the cache.
    ///
    /// If the cache misses, `init` should be `.await`ed to retrieve the data from
    /// the internal [`Downloader`]. A cache entry should then be created.
    ///
    /// # Parameters
    ///
    /// - `url`: The URL of the GET request.
    /// - `init`: A `Future` that fetches data from the Web. It should only be
    ///   awaited when the cache misses or the entry has expired. Note that this is failable.
    async fn resolve(
        &self,
        url: Url,
        init: impl Future<Output = Result<Bytes>>,
    ) -> Result<impl Deref<Target = [u8]>>;
}

/// When you specify `()` as the fetch cache, [`CachedDownloader`] does nothing
/// but passes the requests through directly.
impl FetchCache for () {
    async fn resolve(
        &self,
        _: Url,
        init: impl Future<Output = Result<Bytes>>,
    ) -> Result<impl Deref<Target = [u8]>> {
        init.await
    }
}

#[cfg(feature = "moka")]
impl FetchCache for moka::future::Cache<Url, Bytes> {
    async fn resolve(
        &self,
        url: Url,
        init: impl Future<Output = Result<Bytes>>,
    ) -> Result<impl Deref<Target = [u8]>> {
        self.try_get_with(url, init)
            .await
            .map(|value| Bytes::clone(&value))
            .map_err(Into::into)
    }
}

/// A [`Downloader`] wrapper with caching capabilities.
///
/// For each incoming request, it checks the provided cache bucket. If the
/// cache is hit, it **short-circuits the request** and returns the
/// cached data directly. If the cache is missing, it retrieves the data from
/// the underlying [`Downloader`], writes it to the cache, and returns it.
///
/// # Caching approach
///
/// Caching is handled by [`FetchCache`] implementors.
///
/// Data is distinguished solely by its URL, and only `GET` requests are cached.
///
/// The default value is `()`, which does nothing and passes requests through
/// directly. If you want actual caching capabilities, consider backing this
/// with a dedicated caching library such as
/// [moka](https://docs.rs/moka/latest/moka) or
/// [foyer](https://foyer-rs.github.io/foyer/).
pub struct CachedDownloader<D: Downloader, B: Borrow<D> + Send + Sync, F: FetchCache = ()> {
    fetch_cache: F,
    downloader: B,
    _marker: std::marker::PhantomData<D>,
}

#[cfg(feature = "moka")]
impl<D: Downloader, B: Borrow<D> + Send + Sync>
    CachedDownloader<D, B, moka::future::Cache<Url, Bytes>>
{
    /// Create a new [`CachedDownloader`] whose cache is backed by
    /// [`moka::future::Cache`].
    ///
    /// # Parameters
    ///
    /// - `downloader`: The inner downloader.
    /// - `get_bytes`: The maximum capacity of the in-memory cache.
    ///
    /// # Returns
    ///
    /// A new [`CachedDownloader`] built with these parameters.
    pub fn new_moka(downloader: B, get_bytes: u64) -> Self {
        let fetch_cache = moka::future::Cache::builder()
            .max_capacity(get_bytes)
            .weigher(|_, value: &Bytes| value.len().div_ceil(1024) as u32)
            .build();
        Self::new(downloader, fetch_cache)
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, F: FetchCache> CachedDownloader<D, B, F> {
    /// Create a new [`CachedDownloader`] instance.
    pub fn new(downloader: B, fetch_cache: F) -> Self {
        Self {
            fetch_cache,
            downloader,
            _marker: Default::default(),
        }
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, F: FetchCache> Downloader
    for CachedDownloader<D, B, F>
{
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Bytes> {
        self.fetch_cache
            .resolve(url.clone(), self.downloader.borrow().fetch(url, hash))
            .await
            .map(|data| Bytes::copy_from_slice(data.deref()))
    }

    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Bytes>> {
        self.downloader.borrow().post_json(url, body).await
    }

    async fn head(&self, url: Url) -> Result<Response<()>> {
        self.downloader.borrow().head(url).await
    }

    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>> {
        self.downloader.borrow().send(req).await
    }

    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()> {
        self.downloader.borrow().download(task).await
    }

    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
}
