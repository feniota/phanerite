//! Caching utilities for [`Downloader`].
//!
//! This module provides a generic and powerful [`Downloader`] wrapper,
//! [`CachedDownloader`], which can save resources by caching requests.

use crate::download::Downloader;
use crate::download::task::{DownloadTask, Target};
use crate::error::Result;
use crate::utils::{Hash, hash_file};
use bytes::Bytes;
use http::{Request, Response};
use std::borrow::Borrow;
use std::future::Future;
use std::ops::Deref;
use std::path::PathBuf;
use url::Url;

/// A file registry for a [`Storage`](crate::storage::Storage) that keeps track
/// of what is currently stored.
///
/// Note that this registry is aware of its [`Storage`](crate::storage::Storage).
/// For different [`Storage`](crate::storage::Storage)s, you should create
/// separate [`StorageRegistry`]s that do not share data. **Passing multiple
/// [`Storages`](crate::storage::Storage) to the same [`CachedDownloader`]
/// (and therefore the same `StorageRegistry`) is a severe logic error.**
// TODO: change the cache key: Hash -> Hash+Storage
//       In this way this would no longer be Storage-aware.
#[allow(async_fn_in_trait)]
pub trait StorageRegistry: Send + Sync {
    async fn query(&self, key: &Hash) -> Option<impl Deref<Target = PathBuf>>;
    async fn insert(&self, key: Hash, val: PathBuf);
}

impl StorageRegistry for scc::HashMap<Hash, PathBuf> {
    async fn query(&self, key: &Hash) -> Option<impl Deref<Target = PathBuf>> {
        self.get_async(key).await
    }

    async fn insert(&self, key: Hash, val: PathBuf) {
        let _ = self.insert_async(key, val).await;
    }
}

/// A replaceable cache for [`fetch`](Downloader::fetch) requests.
///
/// # `fetch()` only
///
/// Implementors of this trait handle only the caching of
/// [`fetch`](Downloader::fetch) requests. For [`DownloadTask`] requests, see
/// [`StorageRegistry`] instead.
///
/// See [`CachedDownloader`] for more details.
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
/// For each incoming request, it checks the provided cache buckets. If the
/// cache is hit, it **short-circuits the request** and returns the
/// cached data directly. If the cache is missing, it retrieves the data from
/// the underlying [`Downloader`], writes it to the cache, and returns it.
///
/// Note that this wrapper is aware of its [`Storage`](crate::storage::Storage).
/// For different [`Storage`](crate::storage::Storage)s, you should create
/// separate [`StorageRegistry`]s that do not share data. **Passing multiple
/// [`Storages`](crate::storage::Storage) to the same [`CachedDownloader`]
/// (and therefore the same `StorageRegistry`) is a severe logic error.**
///
/// # Caching approach
///
/// The caching approaches are completely different for tasks passed to
/// [`download`](Downloader::download) with a [`DownloadTask`] and for direct
/// [`fetch`](Downloader::fetch) calls.
///
/// ## [`DownloadTask`] requests
///
/// Caching of these requests is handled by [`StorageRegistry`] implementors.
///
/// The "cache" here is actually a registry of already-downloaded files.
///
/// Minecraft assets commonly contain duplicates. Like the Minecraft asset
/// server, [`StorageRegistry`] records the files in the corresponding
/// [`Storage`](crate::storage::Storage) and their hashes. When a duplicate file
/// is requested, [`CachedDownloader`] can determine that the same file has
/// already been downloaded by querying its hash in the [`StorageRegistry`]. It
/// then skips the download and creates a new file that points to the existing
/// file, typically by creating a hard link. (See
/// [`linker`](crate::storage::Storage::linker) for more details.) This ensures
/// that the same data is stored only once and avoids unnecessary Web requests.
///
/// Only regular files are cached. Compressed archives, i.e. tasks with
/// [`Target::Extract`], are deliberately ignored.
///
/// ## [`fetch()`](Downloader::fetch) requests
///
/// Caching of these requests is handled by [`FetchCache`] implementors.
///
/// Data is distinguished solely by its URL, and only `GET` requests are cached.
///
/// The default value is `()`, which does nothing and passes requests through
/// directly. If you want actual caching capabilities, consider backing this
/// with a dedicated caching library such as
/// [moka](https://docs.rs/moka/latest/moka) or
/// [foyer](https://foyer-rs.github.io/foyer/).
pub struct CachedDownloader<
    D: Downloader,
    B: Borrow<D> + Send + Sync,
    S: StorageRegistry,
    F: FetchCache = (),
> {
    fetch_cache: F,
    storage_registry: S,
    downloader: B,
    _marker: std::marker::PhantomData<D>,
}

#[cfg(feature = "moka")]
impl<D: Downloader, B: Borrow<D> + Send + Sync, S: StorageRegistry>
    CachedDownloader<D, B, S, moka::future::Cache<Url, Bytes>>
{
    /// Create a new [`CachedDownloader`] whose fetch cache is backed by
    /// [`moka::future::Cache`].
    ///
    /// # Parameters
    ///
    /// - `downloader`: The inner downloader.
    /// - `get_bytes`: The maximum capacity of the in-memory cache.
    /// - `storage_cache`: The storage registry. See the documentation for
    ///   [`CachedDownloader`] for more information.
    ///
    /// # Returns
    ///
    /// A new [`CachedDownloader`] built with these parameters.
    pub fn new_moka(downloader: B, get_bytes: u64, storage_registry: S) -> Self {
        let fetch_cache = moka::future::Cache::builder()
            .max_capacity(get_bytes)
            .weigher(|_, value: &Bytes| value.len().div_ceil(1024) as u32)
            .build();
        Self::new(downloader, storage_registry, fetch_cache)
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: StorageRegistry, F: FetchCache>
    CachedDownloader<D, B, R, F>
{
    /// Create a new [`CachedDownloader`] instance.
    pub fn new(downloader: B, storage_registry: R, fetch_cache: F) -> Self {
        Self {
            fetch_cache,
            storage_registry,
            downloader,
            _marker: Default::default(),
        }
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync>
    CachedDownloader<D, B, scc::HashMap<Hash, PathBuf>>
{
    /// Create a [`CachedDownloader`] with the default parameters.
    ///
    /// The storage registry is an in-memory [`scc::HashMap`] that loses all
    /// its state when the program restarts. The fetch cache is disabled.
    ///
    /// Use this only for quick validation or demonstrations. **Not
    /// recommended for actual use.**
    pub fn default(downloader: B) -> CachedDownloader<D, B, scc::HashMap<Hash, PathBuf>, ()> {
        Self::new(downloader, scc::HashMap::new(), ())
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: StorageRegistry, F: FetchCache> Downloader
    for CachedDownloader<D, B, R, F>
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
        let (hash, dst, share) = match (&task.file_hash, &task.target, &task.share) {
            (hash, Target::File(dst), Some(share)) if !hash.is_empty() => {
                (hash, dst, share.clone())
            }
            _ => {
                self.downloader.borrow().download(task).await?;
                return Ok(());
            }
        };

        if let Some(src) = self.storage_registry.query(hash).await
            && hash_file(&src, hash).await.is_ok()
        {
            task.context.storage.linker()(src.deref(), dst).await?;
            return Ok(());
        }

        let hash = hash.clone();
        self.downloader.borrow().download(task).await?;
        if let Some(path) = share.get() {
            self.storage_registry.insert(hash, path.to_owned()).await;
        }
        Ok(())
    }

    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
}
