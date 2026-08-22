use crate::error::Result;
use crate::storage::Storage;
use async_channel::Sender;
use async_executor::Executor;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// 文件类型，用于决定清理方式
/// File type, used to decide how to clean up
#[derive(Clone, Copy)]
enum FileType {
    File,
    Directory,
}

// 临时路径的包装，释放时创建清理任务
/// Wrapper around a temporary path that spawns a cleanup task when dropped
pub struct TempGuard<'storage> {
    // 临时路径
    /// The temporary path
    path: PathBuf,
    // 文件类型
    /// File type
    file_type: FileType,
    // 清理器
    /// The cleaner
    ex: &'storage Executor<'static>,
}

impl TempGuard<'_> {
    pub fn as_path(&self) -> &Path {
        &self.path
    }
    // 放弃自动清理
    /// Gives up automatic cleanup
    #[deprecated(note = "leaking a temporary file is usually a bug")]
    pub fn persist(mut self) -> PathBuf {
        std::mem::take(&mut self.path)
    }
}

impl AsRef<Path> for TempGuard<'_> {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl Deref for TempGuard<'_> {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl<'storage> Storage {
    // 创建临时文件
    /// Creates a temporary file
    pub async fn temp_file(&'storage self) -> Result<TempGuard<'storage>> {
        let path = self.cache_dir.join(Uuid::now_v7().to_string());
        async_fs::File::create(&path).await?;
        Ok(TempGuard {
            path,
            file_type: FileType::File,
            ex: &self.cleaner,
        })
    }
    // 创建临时目录
    /// Creates a temporary directory
    pub async fn temp_dir(&'storage self) -> Result<TempGuard<'storage>> {
        let path = self.cache_dir.join(Uuid::now_v7().to_string());
        async_fs::create_dir_all(&path).await?;
        Ok(TempGuard {
            path,
            file_type: FileType::Directory,
            ex: &self.cleaner,
        })
    }
    // 创建临时文件（阻塞 IO）
    /// Creates a temporary file (blocking IO)
    pub fn temp_file_blocking(&'storage self) -> Result<TempGuard<'storage>> {
        let path = self.cache_dir.join(Uuid::now_v7().to_string());
        std::fs::File::create(&path)?;
        Ok(TempGuard {
            path,
            file_type: FileType::File,
            ex: &self.cleaner,
        })
    }
    // 创建临时目录（阻塞 IO）
    /// Creates a temporary directory (blocking IO)
    pub fn temp_dir_blocking(&'storage self) -> Result<TempGuard<'storage>> {
        let path = self.cache_dir.join(Uuid::now_v7().to_string());
        std::fs::create_dir_all(&path)?;
        Ok(TempGuard {
            path,
            file_type: FileType::Directory,
            ex: &self.cleaner,
        })
    }
    // 仅当清理器工作时，临时文件能够最及时地清理
    // 用法示例
    // ```
    // use phanerite_core::storage::Storage;
    // let storage = Storage::new(".minecraft").await?;
    // // 请勿丢弃 `_shutdown`，否则会导致清理线程停止
    // let (cleaner, _shutdown) = storage.run_cleaner();
    // smol::spawn(cleaner).detach();
    // ```
    // 优雅停机：只需要让 `_shutdown` 离开作用域即可
    // ```
    // drop(_shutdown);
    // ```
    /// Temporary files are only cleaned up promptly while the cleaner is
    /// running
    /// Usage example
    /// ```
    /// use phanerite_core::storage::Storage;
    /// let storage = Storage::new(".minecraft").await?;
    /// // Do not drop `_shutdown`, or the cleaner thread will stop
    /// let (cleaner, _shutdown) = storage.run_cleaner();
    /// smol::spawn(cleaner).detach();
    /// ```
    /// Graceful shutdown: just let `_shutdown` go out of scope
    /// ```
    /// drop(_shutdown);
    /// ```
    pub fn run_cleaner(&self) -> (impl Future<Output = ()> + 'static, ShutdownGuard) {
        let (tx, rx) = async_channel::bounded(1);
        let cleaner = self.cleaner.clone();
        let task = async move {
            cleaner
                .run(async move {
                    let _ = rx.recv().await;
                })
                .await;
        };
        (task, ShutdownGuard { _guard: tx })
    }
}

// 用于控制清理任务的生命周期
/// Controls the lifetime of the cleanup task
pub struct ShutdownGuard {
    _guard: Sender<()>,
}

impl Drop for TempGuard<'_> {
    fn drop(&mut self) {
        if self.path.as_os_str().is_empty() {
            return;
        }

        let path = std::mem::take(&mut self.path);
        let file_type = self.file_type;

        self.ex
            .spawn(async move {
                let _ = match file_type {
                    FileType::File => async_fs::remove_file(path).await,
                    FileType::Directory => async_fs::remove_dir_all(path).await,
                };
            })
            .detach();
    }
}
