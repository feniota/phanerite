use crate::io::{AsyncFile, Result};

/// Extension trait providing compound I/O methods on top of [`AsyncFile`].
///
/// These methods are built from the four primitives
/// ([`read_at`](AsyncFile::read_at), [`write_at`](AsyncFile::write_at),
/// [`size`](AsyncFile::size), [`flush`](AsyncFile::flush)).
///
/// Where possible they use the owned-buffer protocol to avoid copies,
/// though compound operations that span multiple [`read_at`] calls
/// necessarily concatenate buffers.
pub trait AsyncFileExt: AsyncFile {
    // ── Read utilities ──────────────────────────────────────────────

    /// Read exactly `len` bytes starting at `offset`.
    ///
    /// Returns an error if EOF is reached before `len` bytes have been
    /// accumulated.
    async fn read_at_exact(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let mut dst = Vec::with_capacity(len);
        let mut pos = offset;
        while dst.len() < len {
            let needed = len - dst.len();
            let buf = vec![0u8; needed];
            let (n, buf) = self.read_at(pos, buf).await?;
            if n == 0 {
                return Err(crate::io::Error::Other);
            }
            dst.extend_from_slice(&buf[..n]);
            pos += n as u64;
        }
        Ok(dst)
    }

    /// Read from `offset` to end-of-file.
    ///
    /// Uses [`size`](AsyncFile::size) to determine the requested range,
    /// then issues a single [`read_at`](AsyncFile::read_at).
    async fn read_all_at(&self, offset: u64) -> Result<Vec<u8>> {
        let file_size = self.size().await?;
        if offset >= file_size {
            return Ok(Vec::new());
        }
        let len = (file_size - offset) as usize;
        let buf = vec![0u8; len];
        let (n, buf) = self.read_at(offset, buf).await?;
        // Truncate in case of short read (file truncated concurrently).
        let mut buf = buf;
        buf.truncate(n);
        Ok(buf)
    }

    /// Read the entire file.
    async fn read_all(&self) -> Result<Vec<u8>> {
        self.read_all_at(0).await
    }

    /// Read the entire file and interpret as UTF-8.
    async fn read_to_string(&self) -> Result<String> {
        let bytes = self.read_all().await?;
        String::from_utf8(bytes).map_err(|_| crate::io::Error::Other)
    }

    // ── Write utilities ─────────────────────────────────────────────

    /// Write all of `buf` at `offset`, retrying on short writes.
    async fn write_all_at(&self, mut offset: u64, mut buf: Vec<u8>) -> Result<()> {
        while !buf.is_empty() {
            let (n, rest) = self.write_at(offset, buf).await?;
            if n == 0 {
                return Err(crate::io::Error::Other);
            }
            offset += n as u64;
            buf = rest;
        }
        Ok(())
    }

    // ── Combined ────────────────────────────────────────────────────

    /// Copy `len` bytes from `reader` at `src_offset` into this file at
    /// `dst_offset`.
    async fn copy_from<R: AsyncFile + ?Sized>(
        &self,
        dst_offset: u64,
        reader: &R,
        src_offset: u64,
        len: u64,
    ) -> Result<u64>
    where
        Self: Sized,
    {
        const CHUNK: usize = 8192;
        let mut total = 0u64;
        let mut src_off = src_offset;
        let mut dst_off = dst_offset;
        let mut remaining = len;
        while remaining > 0 {
            let cap = (CHUNK as u64).min(remaining) as usize;
            let buf = vec![0u8; cap];
            let (n, buf) = reader.read_at(src_off, buf).await?;
            if n == 0 {
                break;
            }
            let n = n as u64;
            self.write_all_at(dst_off, buf).await?;
            total += n;
            src_off += n;
            dst_off += n;
            remaining -= n;
        }
        Ok(total)
    }

    /// Copy from `reader` at `src_offset` until EOF into this file at
    /// `dst_offset`.
    ///
    /// Unlike [`copy_from`](AsyncFileExt::copy_from), this method does
    /// not require knowing the total length upfront — it reads until
    /// [`read_at`](AsyncFile::read_at) returns 0 bytes.
    async fn copy_all<R: AsyncFile + ?Sized>(
        &self,
        dst_offset: u64,
        reader: &R,
        src_offset: u64,
    ) -> Result<u64>
    where
        Self: Sized,
    {
        const CHUNK: usize = 8192;
        let mut total = 0u64;
        let mut src_off = src_offset;
        let mut dst_off = dst_offset;
        loop {
            let buf = vec![0u8; CHUNK];
            let (n, buf) = reader.read_at(src_off, buf).await?;
            if n == 0 {
                break;
            }
            let n = n as u64;
            self.write_all_at(dst_off, buf).await?;
            total += n;
            src_off += n;
            dst_off += n;
        }
        Ok(total)
    }
}

impl<T: AsyncFile> AsyncFileExt for T {}
