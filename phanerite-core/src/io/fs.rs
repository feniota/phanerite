//! File-system abstraction — positional async file I/O.

use std::path::Path;
use std::time::SystemTime;

use crate::error::Result;

// ── Types ─────────────────────────────────────────────────────────

pub struct Metadata {
    pub file_type: FileType,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

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
/// directly to the kernel with zero extra copies.
///
/// # Buffer contract
///
/// - [`read_at`](AsyncFile::read_at): returns `(bytes_read, buf)` where
///   `buf[..bytes_read]` contains the data read.  0 = EOF.
/// - [`write_at`](AsyncFile::write_at): returns `(bytes_written, rest)`
///   where `rest` is the unwritten suffix.  Empty `rest` = all data
///   accepted.
pub trait AsyncFile {
    /// Read into `buf` starting at `offset`.
    async fn read_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)>;

    /// Write from `buf` starting at `offset`.
    async fn write_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)>;

    /// Return the total size in bytes.
    async fn size(&self) -> Result<u64>;

    /// Flush buffered writes to the storage medium.
    async fn flush(&self) -> Result<()>;
}

pub trait FileSystem {
    type File: AsyncFile;

    async fn open(&self, path: &Path) -> Result<Self::File>;

    async fn create(&self, path: &Path) -> Result<Self::File>;

    async fn remove(&self, path: &Path) -> Result<()>;

    async fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    async fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Create a symbolic link at `link` pointing to `original`.
    async fn symlink(&self, original: &Path, link: &Path) -> Result<()>;

    /// Create a hard link at `link` pointing to `original`.
    async fn hard_link(&self, original: &Path, link: &Path) -> Result<()>;

    async fn metadata(&self, path: &Path) -> Result<Metadata>;
}
