use crate::error::Result;
use crate::storage::Storage;
use crate::utils::walkdir::WalkDir;
use async_fs::Metadata;
use futures::{Stream, StreamExt};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::trace;

const CONCURRENT: usize = 32;

impl Storage {
    // 列出当前 Storage 所有共享文件
    async fn list_current_bucket(&self) -> Result<impl Stream<Item = PathBuf>> {
        Ok(async_fs::read_dir(self.share_dir())
            .await?
            .filter_map(async |entry| entry.ok())
            // 拉平一层目录
            .filter_map(async |entry| async_fs::read_dir(entry.path()).await.ok())
            .flatten()
            .filter_map(async |entry| entry.ok())
            .filter_map(async |entry| std::path::absolute(entry.path()).ok()))
    }

    // 删除共享桶的空目录
    async fn remove_empty_dir(&self) -> Result<()> {
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
            .for_each_concurrent(CONCURRENT, async |path| {
                trace!("Cleaning empty directory: {}", path.to_string_lossy());
                if let Err(e) = async_fs::remove_dir(path).await {
                    tracing::warn!(?e, "failed to remove empty");
                }
            })
            .await;
        Ok(())
    }

    // 清理共享储存桶中的孤立硬链接和空目录
    /// Cleans up orphaned hard links and empty directories in the shared
    /// bucket
    pub async fn clean_hardlink(&self) -> Result<()> {
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

        self.list_current_bucket()
            .await?
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

        self.remove_empty_dir().await?;

        Ok(())
    }

    // 清理共享储存桶中的孤立符号链接和空目录
    /// Cleans up orphaned symbol links and empty directories in the shared
    /// bucket
    pub async fn clean_symlink(&self) -> Result<()> {
        // 列出被引用的文件
        let paths = WalkDir::new(self.versions_dir())
            .filter_map(async |x| x.is_symlink().then_some(x))
            .map(async_fs::read_link)
            .buffer_unordered(CONCURRENT)
            .filter_map(async |x| x.and_then(std::path::absolute).ok())
            .collect::<HashSet<_>>()
            .await;

        self.list_current_bucket()
            .await?
            // 过滤孤立文件
            .filter_map(async |x| paths.contains(&x).then_some(x))
            // 执行删除
            .for_each_concurrent(CONCURRENT, async |path| {
                trace!("Cleaning orphan file: {}", path.display());
                if let Err(e) = async_fs::remove_file(path).await {
                    tracing::warn!(?e, "failed to remove orphan");
                }
            })
            .await;

        self.remove_empty_dir().await?;

        Ok(())
    }
}
