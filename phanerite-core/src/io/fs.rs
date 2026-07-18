//! File-system abstraction — positional async file I/O.
//!
//! The two core traits:
//!
//! - [`AsyncFile`] — a file handle with positional read / write.
//! - [`FileSystem`] — opens, creates, and manages files and directories.
//!
//! # Design
//!
//! Every data operation carries its own **offset** (`read_at`, `write_at`),
//! enabling multiple concurrent operations on the same handle without a
//! shared file cursor.  This maps directly to io_uring / IOCP semantics
//! and avoids the need for internal synchronisation in the common case.

use std::path::Path;
use std::time::SystemTime;

use crate::error::Result;

// ── Types ─────────────────────────────────────────────────────────

/// File metadata.
pub struct Metadata {
    /// Whether this is a regular file, directory, or symlink.
    pub file_type: FileType,
    /// Total size in bytes (`0` for directories).
    pub size: u64,
    /// Last modification time, if available.
    pub modified: Option<SystemTime>,
}

/// Type of a filesystem entry.
pub enum FileType {
    File,
    Directory,
    Symlink,
}

// ── Traits ────────────────────────────────────────────────────────

/// Positional async file handle — every operation carries its own offset.
///
/// Buffers are **owned** ([`Vec<u8>`]) rather than borrowed (`&mut [u8]`),
/// so completion-based backends (io_uring / IOCP) can submit the buffer
/// directly to the kernel without intermediate copies.
///
/// # Buffer contract
///
/// - [`read_at`](AsyncFile::read_at): returns `(bytes_read, buf)` where
///   `buf[..bytes_read]` contains the data read.  0 = EOF.
/// - [`write_at`](AsyncFile::write_at): returns `(bytes_written, rest)`
///   where `rest` is the **unwritten suffix**.  An empty `rest` means
///   every byte was accepted.
///
/// # Example
///
/// ```ignore
/// use phanerite_core::io::AsyncFile;
///
/// async fn copy(from: &impl AsyncFile, to: &impl AsyncFile, len: u64) -> Result<()> {
///     let buf = vec![0u8; 8192];
///     let (n, buf) = from.read_at(0, buf).await?;
///     to.write_at(0, buf).await?;
///     Ok(())
/// }
/// ```
pub trait AsyncFile {
    /// Read into `buf` starting at `offset`.
    ///
    /// Returns `(bytes_read, buf)` where `buf[..bytes_read]` is the data
    /// and `buf[bytes_read..]` retains the original spare capacity.
    /// `0` bytes indicates EOF.
    async fn read_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)>;

    /// Write from `buf` starting at `offset`.
    ///
    /// Returns `(bytes_written, rest)` where `rest` is whatever portion
    /// of `buf` was **not** accepted.  An empty `rest` means complete.
    async fn write_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)>;

    /// Return the total size in bytes.
    ///
    /// For files this is the file length; for directories it is `0`.
    async fn size(&self) -> Result<u64>;

    /// Flush buffered writes to the storage medium.
    async fn flush(&self) -> Result<()>;
}

/// Abstract filesystem — create, open, and query files / directories.
///
/// Backends implement this trait to provide concrete I/O.  See
/// the [`adapters`](super::adapters) module for tokio and compio
/// implementations.
pub trait FileSystem {
    /// The concrete file handle type returned by [`open`](FileSystem::open)
    /// and [`create`](FileSystem::create).
    type File: AsyncFile;

    /// Open an existing file for reading and writing.
    async fn open(&self, path: &Path) -> Result<Self::File>;

    /// Create a new file (truncating if it already exists) for writing.
    async fn create(&self, path: &Path) -> Result<Self::File>;

    /// Remove a file.
    async fn remove(&self, path: &Path) -> Result<()>;

    /// Atomically rename / move a file or directory.
    async fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    /// Recursively create a directory and all its parents.
    async fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Create a symbolic link at `link` pointing to `original`.
    async fn symlink(&self, original: &Path, link: &Path) -> Result<()>;

    /// Create a hard link at `link` pointing to `original`.
    ///
    /// Returns an error on filesystems / platforms that do not
    /// support hard links.
    async fn hard_link(&self, original: &Path, link: &Path) -> Result<()>;

    /// Query metadata for a path.
    async fn metadata(&self, path: &Path) -> Result<Metadata>;
}
