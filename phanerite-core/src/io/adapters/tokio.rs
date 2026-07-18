//! Adapter that wraps a [`tokio::fs::File`] as an [`AsyncFile`].
//!
//! Enabled via the `tokio` feature gate.
//!
//! ## Implementation note
//!
//! tokio uses cursor-based I/O (`seek` → `read` / `write`), which
//! requires `&mut self`.  The [`AsyncFile`] trait exposes `&self`
//! methods for concurrent-friendly positional access, so this adapter
//! wraps the tokio [`File`](tokio::fs::File) in a [`std::sync::Mutex`].
//!
//! Despite the Mutex, reads and writes operate directly on the caller's
//! [`Vec<u8>`] buffer — no intermediate copy is introduced.

use crate::io::{AsyncFile, FileSystem, FileType, Metadata, Result};
use std::io::SeekFrom;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tracing::{instrument, trace};

/// Positional [`AsyncFile`] backed by a [`tokio::fs::File`].
pub struct TokioFile {
    inner: Mutex<tokio::fs::File>,
}

impl TokioFile {
    pub fn new(file: tokio::fs::File) -> Self {
        Self {
            inner: Mutex::new(file),
        }
    }
}

impl AsyncFile for TokioFile {
    #[instrument(skip(self, buf), fields(buf_len = buf.len()))]
    async fn read_at(&self, offset: u64, mut buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let mut file = self.inner.lock().await;
        file.seek(SeekFrom::Start(offset)).await?;
        let n = file.read(&mut buf[..]).await?;
        trace!(offset, n, buf_len = buf.len(), "tokio read_at");
        Ok((n, buf))
    }

    #[instrument(skip(self, buf), fields(buf_len = buf.len()))]
    async fn write_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let mut file = self.inner.lock().await;
        file.seek(SeekFrom::Start(offset)).await?;
        let n = file.write(&buf).await?;
        let rest = split_off_suffix(buf, n);
        trace!(offset, n, remaining = rest.len(), "tokio write_at");
        Ok((n, rest))
    }

    #[instrument(skip_all)]
    async fn size(&self) -> Result<u64> {
        let file = self.inner.lock().await;
        let meta = file.metadata().await?;
        let sz = meta.len();
        trace!(size = sz, "tokio size");
        Ok(sz)
    }

    #[instrument(skip_all)]
    async fn flush(&self) -> Result<()> {
        let mut file = self.inner.lock().await;
        file.flush().await?;
        trace!("tokio flush");
        Ok(())
    }
}

// ── FileSystem adapter ────────────────────────────────────────────

/// A [`FileSystem`] backed by [`tokio::fs`].
pub struct TokioFs;

impl FileSystem for TokioFs {
    type File = TokioFile;

    #[instrument(skip_all)]
    async fn open(&self, path: &Path) -> Result<Self::File> {
        let file = tokio::fs::File::open(path).await?;
        trace!(path = %path.display(), "tokio open");
        Ok(TokioFile::new(file))
    }

    #[instrument(skip_all)]
    async fn create(&self, path: &Path) -> Result<Self::File> {
        let file = tokio::fs::File::create(path).await?;
        trace!(path = %path.display(), "tokio create");
        Ok(TokioFile::new(file))
    }

    #[instrument(skip_all)]
    async fn remove(&self, path: &Path) -> Result<()> {
        tokio::fs::remove_file(path).await?;
        trace!(path = %path.display(), "tokio remove");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        tokio::fs::rename(from, to).await?;
        trace!(from = %from.display(), to = %to.display(), "tokio rename");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        tokio::fs::create_dir_all(path).await?;
        trace!(path = %path.display(), "tokio create_dir_all");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn symlink(&self, original: &Path, link: &Path) -> Result<()> {
        tokio::fs::symlink(original, link).await?;
        trace!(original = %original.display(), link = %link.display(), "tokio symlink");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn hard_link(&self, original: &Path, link: &Path) -> Result<()> {
        tokio::fs::hard_link(original, link).await?;
        trace!(original = %original.display(), link = %link.display(), "tokio hard_link");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn metadata(&self, path: &Path) -> Result<Metadata> {
        let meta = tokio::fs::metadata(path).await?;
        let md = Metadata {
            file_type: if meta.is_dir() {
                FileType::Directory
            } else if meta.is_symlink() {
                FileType::Symlink
            } else {
                FileType::File
            },
            size: meta.len(),
            modified: meta.modified().ok(),
        };
        trace!(path = %path.display(), size = md.size, "tokio metadata");
        Ok(md)
    }
}

// ── helpers ───────────────────────────────────────────────────────

fn split_off_suffix(mut buf: Vec<u8>, n: usize) -> Vec<u8> {
    if n >= buf.len() {
        Vec::new()
    } else {
        buf.split_off(n)
    }
}
