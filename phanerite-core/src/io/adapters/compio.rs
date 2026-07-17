//! Adapter that wraps a [`compio::fs::File`] as an [`AsyncFile`].
//!
//! Enabled via the `compio` feature gate.
//!
//! ## Zero-copy
//!
//! compio's [`File`](compio::fs::File) implements
//! [`AsyncReadAt`](compio::io::AsyncReadAt) /
//! [`AsyncWriteAt`](compio::io::AsyncWriteAt) which accept owned
//! buffers.  [`AsyncFile`] uses owned [`Vec<u8>`] as well, so this
//! adapter is a **pass-through** — the kernel writes directly into the
//! caller's buffer with no intermediate copy.

use crate::io::{AsyncFile, FileSystem, FileType, Metadata, Result};
use compio::buf::BufResult;
use compio::io::{AsyncReadAt, AsyncWriteAt};
use std::path::Path;

/// Positional [`AsyncFile`] backed by a [`compio::fs::File`].
pub struct CompioFile {
    inner: compio::fs::File,
}

impl CompioFile {
    pub fn new(file: compio::fs::File) -> Self {
        Self { inner: file }
    }
}

impl AsyncFile for CompioFile {
    async fn read_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let BufResult(result, buf) = self.inner.read_at(buf, offset).await;
        Ok((result?, buf))
    }

    async fn write_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        // AsyncWriteAt for &File takes &mut self (i.e. &mut &File).
        let mut file_ref = &self.inner;
        let BufResult(result, buf) = AsyncWriteAt::write_at(&mut file_ref, buf, offset).await;
        Ok((result?, buf))
    }

    async fn size(&self) -> Result<u64> {
        let meta = self.inner.metadata().await?;
        Ok(meta.len())
    }

    async fn flush(&self) -> Result<()> {
        self.inner.sync_all().await?;
        Ok(())
    }
}

// ── FileSystem adapter ────────────────────────────────────────────

/// A [`FileSystem`] backed by [`compio::fs`].
pub struct CompioFs;

impl FileSystem for CompioFs {
    type File = CompioFile;

    async fn open(&self, path: &Path) -> Result<Self::File> {
        let file = compio::fs::File::open(path).await?;
        Ok(CompioFile::new(file))
    }

    async fn create(&self, path: &Path) -> Result<Self::File> {
        let file = compio::fs::File::create(path).await?;
        Ok(CompioFile::new(file))
    }

    async fn remove(&self, path: &Path) -> Result<()> {
        compio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn metadata(&self, path: &Path) -> Result<Metadata> {
        let meta = compio::fs::metadata(path).await?;
        Ok(Metadata {
            file_type: if meta.is_dir() {
                FileType::Directory
            } else if meta.is_symlink() {
                FileType::Symlink
            } else {
                FileType::File
            },
            size: meta.len(),
            modified: meta.modified().ok(),
        })
    }
}
