use crate::download::vanilla::assets::AssetIndexList;
use crate::error::Result;
use crate::instance::Instance;
use crate::storage::Storage;
use crate::utils::walkdir::WalkDir;
use async_fs::Metadata;
use futures::{AsyncReadExt, Stream, StreamExt};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::trace;

const CONCURRENT: usize = 32;

impl Storage {
    // 删除共享储存桶的空目录
    async fn clean_empty_bucket(&self) -> Result<()> {
        async_fs::read_dir(self.share_dir())
            .await?
            .filter_map(async |x| x.ok())
            .map(|x| x.path())
            .filter_map(empty_dir)
            .inspect(|x| trace!("Cleaning empty directory: {}", x.to_string_lossy()))
            .for_each_concurrent(CONCURRENT, async |path| {
                if let Err(e) = async_fs::remove_dir(path).await {
                    tracing::warn!(?e, "failed to remove empty");
                }
            })
            .await;

        Ok(())
    }

    // 列出当前 Storage 所有共享文件
    async fn list_current_bucket(&self) -> Result<impl Stream<Item = PathBuf>> {
        Ok(async_fs::read_dir(self.share_dir())
            .await?
            .filter_map(async |entry| entry.ok())
            // 拉平一层目录
            .filter_map(async |entry| async_fs::read_dir(entry.path()).await.ok())
            .flatten()
            .filter_map(async |entry| entry.ok())
            // 保证绝对路径
            .filter_map(async |entry| std::path::absolute(entry.path()).ok()))
    }

    // 清理共享储存桶中的孤立硬链接
    /// Cleans up orphaned hard links in the shared bucket
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

        self.clean_empty_bucket().await?;

        Ok(())
    }

    // 清理共享储存桶中的孤立符号链接
    /// Cleans up orphaned symbol links and in the shared bucket
    pub async fn clean_symlink(&self) -> Result<()> {
        // 列出被引用的文件
        let paths = WalkDir::new(self.versions_dir())
            .file_mode()
            .filter_map(async |x| x.is_symlink().then_some(x))
            .map(async_fs::read_link)
            .buffer_unordered(CONCURRENT)
            // 保证绝对路径
            .filter_map(async |x| x.and_then(std::path::absolute).ok())
            .collect::<HashSet<_>>()
            .await;

        self.list_current_bucket()
            .await?
            // 过滤孤立文件
            .filter_map(async |x| (!paths.contains(&x)).then_some(x))
            // 执行删除
            .for_each_concurrent(CONCURRENT, async |path| {
                trace!("Cleaning orphan file: {}", path.display());
                if let Err(e) = async_fs::remove_file(path).await {
                    tracing::warn!(?e, "failed to remove orphan");
                }
            })
            .await;

        self.clean_empty_bucket().await?;

        Ok(())
    }

    pub async fn clean_assets(&self) -> Result<()> {
        // 清理索引

        /// 简化的版本清单
        #[derive(Deserialize)]
        struct Manifest {
            assets: String,
        }

        // 列出被引用的索引
        let indexes = async_fs::read_dir(self.versions_dir())
            .await?
            .filter_map(async |x| x.ok())
            // 拉平一层目录
            .filter_map(async |x| async_fs::read_dir(x.path()).await.ok())
            .flatten()
            .filter_map(async |x| x.ok())
            .filter_map(async |x| std::path::absolute(x.path()).ok())
            // 打开实例版本清单
            .filter_map(async |x| x.extension().is_some_and(|ext| ext == "json").then_some(x))
            .map(async |x| async_fs::File::open(x).await)
            .buffer_unordered(CONCURRENT)
            .filter_map(async |x| x.ok())
            .filter_map(async |mut x| {
                let mut buf = Vec::new();
                x.read_to_end(&mut buf).await.ok()?;
                serde_json::from_slice::<Manifest>(&buf).ok()
            })
            // 解析索引路径
            .map(|x| self.assets_indexes().join(format!("{}.json", x.assets)))
            // 保证绝对路径
            .filter_map(async |x| std::path::absolute(x).ok())
            .collect::<HashSet<_>>()
            .await;

        // 执行删除
        async_fs::read_dir(self.assets_indexes())
            .await?
            .filter_map(async |x| x.map(|t| t.path()).ok())
            // 筛选 json
            .filter_map(async |x| x.extension().is_some_and(|ext| ext == "json").then_some(x))
            // 保证绝对路径
            .filter_map(async |x| std::path::absolute(x).ok())
            // 筛选不被引用
            .filter_map(async |x| (!indexes.contains(&x)).then_some(x))
            .for_each_concurrent(CONCURRENT, async |path| {
                trace!("Cleaning assets index: {}", path.display());
                if let Err(e) = async_fs::remove_file(path).await {
                    tracing::warn!(?e, "failed to remove assets index");
                }
            })
            .await;

        // 清理对象

        // 列出被引用的文件
        let objects = async_fs::read_dir(self.assets_indexes())
            .await?
            .filter_map(async |x| x.ok())
            // 打开 index
            .map(async |x| async_fs::File::open(x.path()).await)
            .buffer_unordered(CONCURRENT)
            .filter_map(async |x| x.ok())
            // 反序列化 index
            .filter_map(async |mut x| {
                let mut buf = Vec::new();
                x.read_to_end(&mut buf).await.ok()?;
                serde_json::from_slice::<AssetIndexList>(&buf).ok()
            })
            // 解析为路径
            .flat_map(|x| {
                futures::stream::iter(x.objects.into_values().map(|x| {
                    let file_name = x.hash.to_string();
                    self.assets_objects().join(&file_name[..2]).join(file_name)
                }))
            })
            // 保证绝对路径
            .filter_map(async |x| std::path::absolute(x).ok())
            .collect::<HashSet<_>>()
            .await;

        // 执行删除
        async_fs::read_dir(self.assets_objects())
            .await?
            .filter_map(async |x| x.ok())
            // 拉平一层目录
            .filter_map(async |x| async_fs::read_dir(x.path()).await.ok())
            .flatten()
            .filter_map(async |x| x.ok())
            .filter_map(async |x| std::path::absolute(x.path()).ok())
            // 保证绝对路径
            .filter_map(async |x| std::path::absolute(x).ok())
            // 筛选不被引用
            .filter_map(async |x| (!objects.contains(&x)).then_some(x))
            .for_each_concurrent(CONCURRENT, async |path| {
                trace!("Cleaning assets index: {}", path.display());
                if let Err(e) = async_fs::remove_file(path).await {
                    tracing::warn!(?e, "failed to remove assets object");
                }
            })
            .await;

        // 清理空目录
        async_fs::read_dir(self.assets_objects())
            .await?
            .filter_map(async |x| x.ok())
            .map(|x| x.path())
            .filter_map(empty_dir)
            .inspect(|x| trace!("Cleaning empty directory: {}", x.to_string_lossy()))
            .for_each_concurrent(CONCURRENT, async |path| {
                if let Err(e) = async_fs::remove_dir(path).await {
                    tracing::warn!(?e, "failed to remove empty");
                }
            })
            .await;

        Ok(())
    }

    pub async fn clean_libraries(&self) -> Result<()> {
        // 列出被使用的库
        let paths = Instance::scan(self)
            .filter_map(async |x| x.ok())
            .flat_map(|x| futures::stream::iter(x.manifest.libraries))
            .map(|x| self.libraries_dir().join(x.name.path()))
            // 保证绝对路径
            .filter_map(async |x| std::path::absolute(x).ok())
            .collect::<HashSet<_>>()
            .await;

        // 执行删除
        WalkDir::new(self.libraries_dir())
            .file_mode()
            // 保证绝对路径
            .filter_map(async |x| std::path::absolute(x).ok())
            // 筛选不被引用
            .filter_map(async |x| (!paths.contains(&x)).then_some(x))
            .for_each_concurrent(CONCURRENT, async |path| {
                trace!("Cleaning assets index: {}", path.display());
                if let Err(e) = async_fs::remove_file(path).await {
                    tracing::warn!(?e, "failed to remove assets object");
                }
            })
            .await;

        // 清理空目录删除，必须串行：后序保证子目录先于父目录产出，
        // 但并发时父目录的检查会赶在子目录删除完成之前，嵌套空目录一趟删不干净
        WalkDir::new(self.libraries_dir())
            .dir_mode()
            .filter_map(empty_dir)
            .inspect(|x| trace!("Cleaning empty directory: {}", x.to_string_lossy()))
            .for_each(async |path| {
                if let Err(e) = async_fs::remove_dir(path).await {
                    tracing::warn!(?e, "failed to remove empty");
                }
            })
            .await;

        Ok(())
    }
}

// 只产出空目录
/// Yields the path only if it points to an empty directory
async fn empty_dir(path: PathBuf) -> Option<PathBuf> {
    async_fs::read_dir(&path)
        .await
        .ok()?
        .next()
        .await
        .is_none()
        .then_some(path)
}
