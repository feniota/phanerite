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
use tracing::{instrument, trace};

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
    #[instrument(skip(self, buf), fields(buf_len = buf.len()))]
    async fn read_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let BufResult(result, buf) = self.inner.read_at(buf, offset).await;
        let (n, buf) = (result?, buf);
        trace!(offset, n, buf_len = buf.len(), "compio read_at");
        Ok((n, buf))
    }

    #[instrument(skip(self, buf), fields(buf_len = buf.len()))]
    async fn write_at(&self, offset: u64, buf: Vec<u8>) -> Result<(usize, Vec<u8>)> {
        let mut file_ref = &self.inner;
        let BufResult(result, buf) = AsyncWriteAt::write_at(&mut file_ref, buf, offset).await;
        let n = result?;
        let rest = split_off_suffix(buf, n);
        trace!(offset, n, remaining = rest.len(), "compio write_at");
        Ok((n, rest))
    }

    #[instrument(skip_all)]
    async fn size(&self) -> Result<u64> {
        let meta = self.inner.metadata().await?;
        let sz = meta.len();
        trace!(size = sz, "compio size");
        Ok(sz)
    }

    #[instrument(skip_all)]
    async fn flush(&self) -> Result<()> {
        self.inner.sync_all().await?;
        trace!("compio flush");
        Ok(())
    }
}

// ── FileSystem adapter ────────────────────────────────────────────

/// A [`FileSystem`] backed by [`compio::fs`].
pub struct CompioFs;

impl FileSystem for CompioFs {
    type File = CompioFile;

    #[instrument(skip_all)]
    async fn open(&self, path: &Path) -> Result<Self::File> {
        let file = compio::fs::File::open(path).await?;
        trace!(path = %path.display(), "compio open");
        Ok(CompioFile::new(file))
    }

    #[instrument(skip_all)]
    async fn create(&self, path: &Path) -> Result<Self::File> {
        let file = compio::fs::File::create(path).await?;
        trace!(path = %path.display(), "compio create");
        Ok(CompioFile::new(file))
    }

    #[instrument(skip_all)]
    async fn remove(&self, path: &Path) -> Result<()> {
        compio::fs::remove_file(path).await?;
        trace!(path = %path.display(), "compio remove");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        compio::fs::rename(from, to).await?;
        trace!(from = %from.display(), to = %to.display(), "compio rename");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        compio::fs::create_dir_all(path).await?;
        trace!(path = %path.display(), "compio create_dir_all");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn symlink(&self, original: &Path, link: &Path) -> Result<()> {
        compio::fs::symlink(original, link).await?;
        trace!(original = %original.display(), link = %link.display(), "compio symlink");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn hard_link(&self, original: &Path, link: &Path) -> Result<()> {
        compio::fs::hard_link(original, link).await?;
        trace!(original = %original.display(), link = %link.display(), "compio hard_link");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn metadata(&self, path: &Path) -> Result<Metadata> {
        let meta = compio::fs::metadata(path).await?;
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
        trace!(path = %path.display(), size = md.size, "compio metadata");
        Ok(md)
    }
}

fn split_off_suffix(mut buf: Vec<u8>, n: usize) -> Vec<u8> {
    if n >= buf.len() {
        Vec::new()
    } else {
        buf.split_off(n)
    }
}
