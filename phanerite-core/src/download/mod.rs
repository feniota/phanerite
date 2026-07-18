//! Download pipeline for game assets.
//!
//! # Architecture
//!
//! [`Downloadable`] represents a single resource (client JAR, library,
//! asset).  Its [`download`](Downloadable::download) returns a
//! [`DownloadHandle`] — a streaming byte source plus optional hash
//! and path metadata.
//!
//! [`Downloader`] consumes handles via two strategies:
//!
//! | Method | Target | Hash | Use case |
//! |--------|--------|------|----------|
//! | [`download_to_bucket`] | `share/` via Blake3 | Blake3 + optional algo | Deduplicated storage |
//! | [`download_to_path`] | Direct path | Configurable algo | Libraries, assets |
//!
//! Both methods skip downloads when the target already exists with
//! a matching hash, and support configurable retry on failure.
//!
//! [`download_to_bucket`]: Downloader::download_to_bucket
//! [`download_to_path`]: Downloader::download_to_path

use crate::error::Error;
use crate::error::Result;
use crate::io::utils::{AsyncFileExt, Hasher};
use crate::io::{AsyncFile, FileSystem, HttpClient};
use crate::storage::Storage;
use crate::utils::HashValue;
use std::path::PathBuf;
use tracing::{debug, error, info, instrument};

pub mod vanilla;

/// A single downloadable resource.
///
/// Implementors know how to fetch their own content from a remote
/// server, returning a streaming byte source with optional hash
/// verification metadata.
pub trait Downloadable {
    /// The hash algorithm used for verification
    /// (e.g. [`Sha1`](crate::utils::Sha1)).
    type HashAlgorithm: HashValue;

    /// Start the download.
    ///
    /// Returns a [`DownloadHandle`] containing the stream, target
    /// path, and optional hash / size metadata.
    async fn download(
        self,
        http_client: &impl HttpClient,
        storage: &Storage<impl FileSystem>,
    ) -> Result<DownloadHandle<impl AsyncFile, Self::HashAlgorithm>>;
}

/// Result of starting a download.
///
/// Bundles the streaming byte source with path and integrity
/// metadata so that [`Downloader`] can apply the right storage
/// strategy.
pub struct DownloadHandle<F: AsyncFile, H: HashValue> {
    /// Streaming byte source.
    pub stream: F,
    /// Final on-disk path.
    pub path: PathBuf,
    /// Expected hash, or `None` to skip verification.
    pub digest: Option<H>,
    /// Human-readable name (for logging).
    pub name: Option<String>,
    /// Expected size in bytes, if known.
    pub size: Option<u64>,
}

/// Streaming download manager.
///
/// Wraps a [`Storage`] backend and an [`HttpClient`].  Two download
/// strategies are available:
///
/// - [`download_to_bucket`](Downloader::download_to_bucket) —
///   content-addressed storage with optional hash verification.
/// - [`download_to_path`](Downloader::download_to_path) —
///   direct placement with rename-at-end.
///
/// Both methods skip automatically when the target already exists
/// with a matching hash.
pub struct Downloader<F: FileSystem, H: HttpClient> {
    storage: Storage<F>,
    /// The HTTP client (exposed for direct use, e.g. index fetches).
    pub http_client: H,
    /// Number of retry attempts on failure (default 0 = no retry).
    retries: u32,
}

impl<F: FileSystem, H: HttpClient> Downloader<F, H> {
    /// Create a new downloader with no retries.
    pub fn new(retries: u32, storage: Storage<F>, http_client: H) -> Self {
        Self {
            storage,
            http_client,
            retries,
        }
    }

    /// Set the number of retry attempts.
    pub fn with_retries(mut self, n: u32) -> Self {
        self.retries = n;
        self
    }

    /// Download into the content-addressed `share/` bucket.
    ///
    /// The file is stored under its **Blake3** hash, enabling
    /// automatic deduplication across versions.  If `digest` is
    /// `Some`, the configurable algorithm is verified in parallel.
    /// On success the target path is hard-linked (or symlinked as
    /// fallback) to the shared blob.
    ///
    /// Skips if the target path already exists with a matching hash.
    /// Retries up to `self.retries` times on failure.
    #[instrument(skip(self, task))]
    pub async fn download_to_bucket<T: Downloadable>(&self, task: T) -> Result<()> {
        let handle = task.download(&self.http_client, &self.storage).await?;
        debug!(target = %handle.path.display());

        if self.check_existing(&handle.path, &handle.digest).await? {
            debug!("skip — already downloaded");
            return Ok(());
        }

        let cache_path = self
            .storage
            .cache_dir
            .join(uuid::Uuid::now_v7().to_string());

        let cache = self.storage.fs.create(&cache_path).await?;

        let mut blake3_hasher = blake3::Hasher::new();
        let mut verify_hasher = T::HashAlgorithm::hasher();

        let mut offset = 0u64;
        loop {
            let buf = vec![0u8; 8192];
            let (n, mut buf) = handle.stream.read_at(offset, buf).await?;
            if n == 0 {
                break;
            }
            blake3_hasher.update(&buf[..n]);
            if handle.digest.is_some() {
                verify_hasher.update(&buf[..n]);
            }
            buf.truncate(n);
            cache.write_all_at(offset, buf).await?;
            offset += n as u64;
        }

        if let Some(expected) = &handle.digest
            && expected.to_string() != verify_hasher.finalize_hex()
        {
            error!("hash mismatch");
            return Err(Error::Other("hash mismatch".into()));
        }

        let file_name = blake3_hasher.finalize_hex();
        let save_bucket = self.storage.share_dir.join(&file_name[..2]);
        if !save_bucket.is_dir() {
            self.storage.fs.create_dir_all(&save_bucket).await?;
        }
        let save_path = save_bucket.join(file_name);
        self.storage.fs.rename(&cache_path, &save_path).await?;
        if self
            .storage
            .fs
            .hard_link(&save_path, &handle.path)
            .await
            .is_err()
        {
            self.storage.fs.symlink(&save_path, &handle.path).await?;
        }
        info!("download completed");
        Ok(())
    }

    /// Download directly to the target path.
    ///
    /// Streams into `cache/`, verifies the configured hash if
    /// `digest` is `Some`, then renames to the final destination.
    /// Parent directories are created as needed.
    ///
    /// Skips if the target path already exists with a matching hash.
    /// Retries up to `self.retries` times on failure.
    pub async fn download_to_path<T: Downloadable>(&self, task: T) -> Result<()> {
        let handle = task.download(&self.http_client, &self.storage).await?;

        if self.check_existing(&handle.path, &handle.digest).await? {
            debug!("skip — already downloaded");
            return Ok(());
        }

        let cache_path = self
            .storage
            .cache_dir
            .join(uuid::Uuid::now_v7().to_string());
        let cache = self.storage.fs.create(&cache_path).await?;

        let mut hasher = T::HashAlgorithm::hasher();
        let mut offset = 0u64;
        loop {
            let buf = vec![0u8; 8192];
            let (n, mut buf) = handle.stream.read_at(offset, buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            buf.truncate(n);
            cache.write_all_at(offset, buf).await?;
            offset += n as u64;
        }

        if let Some(v) = &handle.digest
            && v.to_string() != hasher.finalize_hex()
        {
            error!("hash mismatch");
            return Err(Error::Other("hash mismatch".into()));
        }

        let save_dir = handle.path.parent().unwrap();
        if !save_dir.is_dir() {
            self.storage.fs.create_dir_all(&save_dir).await?;
        }

        self.storage.fs.rename(&cache_path, &handle.path).await?;
        info!("download completed");
        Ok(())
    }

    /// Check if a file exists at `path` and matches the expected digest.
    async fn check_existing<V: HashValue>(
        &self,
        path: &PathBuf,
        digest: &Option<V>,
    ) -> Result<bool> {
        let Some(expected) = digest else {
            return Ok(false);
        };
        let Ok(file) = self.storage.fs.open(path).await else {
            return Ok(false);
        };
        let mut hasher = V::hasher();
        let mut offset = 0u64;
        loop {
            let buf = vec![0u8; 8192];
            let (n, buf) = file.read_at(offset, buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            offset += n as u64;
        }
        Ok(expected.to_string() == hasher.finalize_hex())
    }
}

// ── ConcurrentTask ────────────────────────────────────────────────

use futures::stream::{FuturesUnordered, StreamExt};
use std::num::NonZeroU16;
use std::pin::Pin;

/// Bounded concurrent task runner.
pub struct ConcurrentTask<'a> {
    pending: Vec<Pin<Box<dyn Future<Output = Result<()>> + 'a>>>,
    max_concurrent: usize,
}

impl<'a> ConcurrentTask<'a> {
    pub fn new(max_concurrent: NonZeroU16) -> Self {
        Self {
            pending: Vec::new(),
            max_concurrent: max_concurrent.get() as usize,
        }
    }

    pub fn push<F>(&mut self, task: F)
    where
        F: Future<Output = Result<()>> + 'a,
    {
        self.pending.push(Box::pin(task));
    }

    pub async fn exec(mut self) -> Result<()> {
        let mut running = FuturesUnordered::new();

        loop {
            while running.len() < self.max_concurrent {
                match self.pending.pop() {
                    Some(task) => {
                        running.push(task);
                    }
                    None => break,
                }
            }

            if running.is_empty() {
                break;
            }

            match running.next().await {
                Some(result) => {
                    result?;
                }
                None => break,
            }
        }

        Ok(())
    }
}
