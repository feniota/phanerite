use crate::error::{Error, Result};
use crate::storage::Storage;
use async_fs::Metadata;
use futures::StreamExt;
use tracing::trace;

/// 清理共享储存桶中的孤立文件
impl Storage {
    pub async fn clean_hardlink(&self) -> Result<()> {
        let bucket = self.share_dir();

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
            .for_each_concurrent(16, async |path| {
                trace!("Cleaning orphan file: {}", path.display());
                if let Err(e) = async_fs::remove_file(path).await {
                    tracing::warn!(?e, "failed to remove orphan");
                }
            })
            .await;

        Ok(())
    }
}

#[cfg(target_family = "unix")]
#[inline]
fn ref_count(meta: &Metadata) -> Result<u64> {
    use async_fs::unix::MetadataExt;
    Ok(meta.nlink())
}

#[cfg(target_os = "windows")]
#[inline]
fn ref_count(meta: &Metadata) -> Result<u64> {
    use async_fs::windows::MetadataExt;
    match meta.number_of_links() {
        None => Err(Error::other("Unknown reference count of hard link")),
        Some(v) => Ok(v as u64),
    }
}
