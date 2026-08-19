use crate::error::Result;
use crate::storage::Storage;
use async_fs::Metadata;
use futures::StreamExt;
use tracing::trace;

impl Storage {
    /// 清理共享储存桶中的孤立硬链接和空目录
    pub async fn clean_hardlink(&self) -> Result<()> {
        let bucket = self.share_dir();
        const CONCURRENT: usize = 16;

        // 删除孤立文件
        async_fs::read_dir(bucket)
            .await?
            .filter_map(async |x| x.ok())
            // 拉平一层目录
            .filter_map(async |x| async_fs::read_dir(x.path()).await.ok())
            .flatten()
            .filter_map(async |x| x.ok())
            .map(|x| x.path())
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
        async_fs::read_dir(bucket)
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

/// 硬链接引用计数
#[cfg(target_family = "unix")]
#[inline]
fn ref_count(meta: &Metadata) -> Result<u64> {
    use async_fs::unix::MetadataExt;
    Ok(meta.nlink())
}

/// 硬链接引用计数
#[cfg(target_os = "windows")]
#[inline]
fn ref_count(meta: &Metadata) -> Result<u64> {
    use async_fs::windows::MetadataExt;
    match meta.number_of_links() {
        None => Err(Error::other("Unknown reference count of hard link")),
        Some(v) => Ok(v as u64),
    }
}
