//! File deduplication utilities for [`Downloader`].
//!
//! This module provides [`DeduplicateDownloader`], a wrapper that avoids
//! downloading the same shared file more than once. It records downloaded
//! files in a [`StorageRegistry`] keyed by their storage and content hash.
//! When a matching, valid file is found, the wrapper links it into the
//! requested path instead of downloading it again.
//!
//! Only regular files with a known hash and a share bucket participate in
//! deduplication. Extraction tasks and tasks without a hash are passed to the
//! wrapped downloader unchanged.

use crate::download::Downloader;
use crate::download::task::{DownloadTask, Target};
use crate::error::Result;
use crate::storage::StorageIdent;
use crate::utils::{Hash, hash_file};
use bytes::Bytes;
use http::{Request, Response};
use std::borrow::Borrow;
use std::ops::Deref;
use std::path::PathBuf;
use url::Url;

/// A registry for files that keeps track of what is currently stored.
///
/// Each [`Storage`](crate::storage::Storage) is converted to a [`StorageIdent`],
/// making it relatively cheap to clone.
///
/// <div class="warning">
///
/// Implementors must not let entries with different [`StorageIdent`]s return
/// the same path. Deduplication is implemented via linking, and if that happens,
/// files in different [`Storage`](crate::storage::Storage)s will point to the
/// same file. In case these `Storage`s are on different filesystems, this will
/// lead to fatals.
///
/// </div>
#[allow(async_fn_in_trait)]
pub trait StorageRegistry: Send + Sync {
    /// Looks up the path of a file identified by its storage and content hash.
    ///
    /// Returns a reference-like handle to the registered path when the file
    /// is present in the registry.
    async fn query(&self, key: &(StorageIdent, Hash)) -> Option<impl Deref<Target = PathBuf>>;

    /// Registers the path of a file identified by its storage and content hash.
    async fn insert(&self, key: (StorageIdent, Hash), val: PathBuf);
}

impl StorageRegistry for scc::HashMap<(StorageIdent, Hash), PathBuf> {
    async fn query(&self, key: &(StorageIdent, Hash)) -> Option<impl Deref<Target = PathBuf>> {
        self.get_async(key).await
    }

    async fn insert(&self, key: (StorageIdent, Hash), val: PathBuf) {
        let _ = self.insert_async(key, val).await;
    }
}

/// [`Downloader`] wrapper with deduplication.
///
/// Deduplication is handled by [`StorageRegistry`] implementors, which is a
/// registry of already-downloaded files.
///
/// Minecraft assets commonly contain duplicates. Like the Minecraft asset
/// server, [`StorageRegistry`] records the files in the corresponding
/// [`Storage`](crate::storage::Storage) and their hashes. When a duplicate file
/// is requested, [`DeduplicateDownloader`] can determine that the same file has
/// already been downloaded by querying its hash in the [`StorageRegistry`]. It
/// then skips the download and creates a new file that points to the existing
/// file, typically by creating a hard link. (See
/// [`linker`](crate::storage::Storage::linker) for more details.) This ensures
/// that the same data is stored only once and avoids unnecessary Web requests.
///
/// Only regular files are being deduplicated. Compressed archives, i.e. tasks with
/// [`Target::Extract`], or files whose hashes are not already known, are deliberately
/// ignored.
pub struct DeduplicateDownloader<D, B, R>
where
    D: Downloader,
    B: Borrow<D> + Send + Sync,
    R: StorageRegistry,
{
    _marker: std::marker::PhantomData<D>,
    downloader: B,
    registry: R,
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: StorageRegistry> DeduplicateDownloader<D, B, R> {
    pub fn new(downloader: B, registry: R) -> Self {
        Self {
            _marker: Default::default(),
            downloader,
            registry,
        }
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync>
    DeduplicateDownloader<D, B, scc::HashMap<(StorageIdent, Hash), PathBuf>>
{
    /// Create a [`DeduplicateDownloader`] with the default parameters.
    ///
    /// The storage registry is an in-memory [`scc::HashMap`] that loses all
    /// its state when the program restarts.
    ///
    /// Use this only for quick validation or demonstrations. **Not
    /// recommended for actual use.**
    pub fn default(
        downloader: B,
    ) -> DeduplicateDownloader<D, B, scc::HashMap<(StorageIdent, Hash), PathBuf>> {
        Self::new(downloader, scc::HashMap::new())
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: StorageRegistry> Downloader
    for DeduplicateDownloader<D, B, R>
{
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Bytes> {
        self.downloader.borrow().fetch(url, hash).await
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
        // ignore deduplicating for items without a hash
        let (hash, dst, share) = match (&task.file_hash, &task.target, &task.share) {
            (hash, Target::File(dst), Some(share)) if !hash.is_empty() => {
                (hash, dst, share.clone())
            }
            _ => {
                self.downloader.borrow().download(task).await?;
                return Ok(());
            }
        };

        let storage_ident: StorageIdent = task.context.storage.into();
        if let Some(src) = self
            .registry
            .query(&(storage_ident.clone(), hash.clone()))
            .await
            && hash_file(&src, hash).await.is_ok()
        {
            task.context.storage.linker()(src.deref(), dst).await?;
            return Ok(());
        }

        let hash = hash.clone();
        self.downloader.borrow().download(task).await?;
        if let Some(path) = share.get() {
            self.registry
                .insert((storage_ident, hash), path.to_owned())
                .await;
        }
        Ok(())
    }

    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
}
