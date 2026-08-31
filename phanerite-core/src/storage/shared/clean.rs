use crate::error::Result;
use crate::storage::Storage;
use async_fs::Metadata;
use futures::{Stream, StreamExt};
use std::path::PathBuf;
use tracing::trace;

impl Storage {
    fn list_current_bucket(&self) -> impl Stream<Item = PathBuf> {
        futures::stream::once(async_fs::read_dir(self.share_dir()))
            .filter_map(async |dir| dir.ok())
            .flat_map(|dir| {
                dir.filter_map(async |entry| entry.ok())
                    // 拉平一层目录
                    .filter_map(async |entry| async_fs::read_dir(entry.path()).await.ok())
                    .flatten()
                    .filter_map(async |entry| entry.ok())
                    .map(|entry| entry.path())
            })
    }

    // 清理共享储存桶中的孤立硬链接和空目录
    /// Cleans up orphaned hard links and empty directories in the shared
    /// bucket
    pub async fn clean_hardlink(&self) -> Result<()> {
        const CONCURRENT: usize = 16;

        // 删除孤立文件
        self.list_current_bucket()
            // 过滤孤立文件
            .filter_map(async |x| {
                async_fs::metadata(&x)
                    .await
                    .ok()
                    .and_then(|t| t.is_file().then_some(t))
                    .and_then(|t| ref_count(&t).ok())
                    .map(|n| n < 2)
                    .unwrap_or(false)
                    .then_some(x)
            })
            // 执行删除
            .for_each_concurrent(CONCURRENT, async |path| {
                trace!("Cleaning orphan file: {}", path.display());
                if let Err(e) = async_fs::remove_file(path).await {
                    tracing::warn!(?e, "failed to remove orphan");
                }
            })
            .await;

        // 删除空目录
        async_fs::read_dir(self.share_dir())
            .await?
            .filter_map(async |x| x.ok())
            .map(|x| x.path())
            .filter_map(async |x| {
                async_fs::read_dir(&x)
                    .await
                    .ok()?
                    .next()
                    .await
                    .is_none()
                    .then_some(x)
            })
            .for_each_concurrent(CONCURRENT, |path| async move {
                trace!("Cleaning empty directory: {}", path.to_string_lossy());
                if let Err(e) = async_fs::remove_dir(path).await {
                    tracing::warn!(?e, "failed to remove empty");
                }
            })
            .await;

        Ok(())
    }
}

// 硬链接引用计数
/// Hard link reference count
#[cfg(target_family = "unix")]
#[inline]
fn ref_count(meta: &Metadata) -> Result<u64> {
    use async_fs::unix::MetadataExt;
    Ok(meta.nlink())
}

// 硬链接引用计数
/// Hard link reference count
#[cfg(target_os = "windows")]
#[inline]
fn ref_count(meta: &Metadata) -> Result<u64> {
    use crate::error::Error;
    use async_fs::windows::MetadataExt;
    match meta.number_of_links() {
        None => Err(Error::other("Unknown reference count of hard link")),
        Some(v) => Ok(v as u64),
    }
}
