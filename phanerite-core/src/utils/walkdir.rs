use async_fs::{self, ReadDir};
use futures::{Stream, StreamExt, TryStreamExt};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

type ReadDirFuture = Pin<Box<dyn Future<Output = std::io::Result<ReadDir>> + Send>>;

pub enum WalkDirMode {
    Files,
    Directories,
    All,
}

// DFS 栈帧：一个已经打开、尚未遍历完的目录
/// A DFS stack frame: an opened directory whose entries are not exhausted yet.
struct Frame {
    path: PathBuf,
    read_dir: ReadDir,
}

// 后序深度优先遍历：目录总是在其全部内容之后产出，根目录自身不产出
/// Post-order depth-first traversal: a directory is always yielded after all of
/// its contents, and the root itself is never yielded.
pub struct WalkDir {
    root: PathBuf,
    stack: Vec<Frame>,
    reading: Option<(PathBuf, ReadDirFuture)>,
    mode: WalkDirMode,
}

impl WalkDir {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();

        Self {
            stack: Vec::new(),
            reading: Some((root.clone(), Box::pin(async_fs::read_dir(root.clone())))),
            root,
            mode: WalkDirMode::Files,
        }
    }

    /// Only yield files.
    pub fn file_mode(mut self) -> Self {
        self.mode = WalkDirMode::Files;
        self
    }

    /// Only yield directories.
    pub fn dir_mode(mut self) -> Self {
        self.mode = WalkDirMode::Directories;
        self
    }

    /// Yield both files and directories.
    pub fn all_mode(mut self) -> Self {
        self.mode = WalkDirMode::All;
        self
    }

    // 当前模式下目录自身是否需要产出：根目录不算遍历结果
    /// Whether the directory itself is a result in the current mode; the root never is.
    #[inline]
    fn yields_dir(&self, path: &Path) -> bool {
        matches!(self.mode, WalkDirMode::Directories | WalkDirMode::All)
            && path != self.root.as_path()
    }

    // 移动整个目录的文件，并跳过已有文件
    /// Merges the contents of this directory into `dst`.
    ///
    /// Existing files are skipped. Files are moved concurrently.
    pub async fn merge_move(self, dst: impl Into<PathBuf>) -> std::io::Result<()> {
        let root = self.root.clone();
        let dst = dst.into();

        async_fs::create_dir_all(&dst).await?;

        self.file_mode()
            .map(async |src| {
                let relative = src
                    .strip_prefix(&root)
                    .expect("WalkDir path must be under root");
                let target = dst.join(relative);

                if src.is_dir() {
                    if target.exists() {
                        // 目标目录已经存在：
                        // 不做任何事情，让 WalkDir 继续遍历里面的内容。
                        return Ok(());
                    }

                    // 目标目录不存在，整个目录直接搬过去。
                    async_fs::rename(src, target).await?;
                } else {
                    // 目标文件已经存在，跳过。
                    if target.exists() {
                        return Ok(());
                    }

                    if let Some(parent) = target.parent() {
                        async_fs::create_dir_all(parent).await?;
                    }

                    async_fs::rename(src, target).await?;
                }

                Ok(())
            })
            .buffer_unordered(32)
            .try_for_each(async |_| Ok(()))
            .await
    }
}

impl Stream for WalkDir {
    type Item = PathBuf;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // 正在异步打开下一个目录
            if let Some((_, reading)) = &mut this.reading {
                let polled = reading.as_mut().poll(cx);

                match polled {
                    Poll::Pending => return Poll::Pending,

                    Poll::Ready(Ok(read_dir)) => {
                        let (path, _) = this.reading.take().expect("reading checked above");
                        // 入栈，先遍历它的内容
                        this.stack.push(Frame { path, read_dir });
                        continue;
                    }

                    Poll::Ready(Err(_)) => {
                        // 打不开的目录没有内容可以先行产出，直接产出它自身
                        let (path, _) = this.reading.take().expect("reading checked above");

                        if this.yields_dir(&path) {
                            return Poll::Ready(Some(path));
                        }

                        continue;
                    }
                }
            }

            // 没有正在打开的目录，继续遍历栈顶目录
            let Some(frame) = this.stack.last_mut() else {
                // 栈为空，遍历结束
                return Poll::Ready(None);
            };

            let polled = Pin::new(&mut frame.read_dir).poll_next(cx);

            match polled {
                Poll::Pending => return Poll::Pending,

                Poll::Ready(Some(Ok(entry))) => {
                    let path = entry.path();

                    if path.is_dir() {
                        // 后序：先深入子目录，等它遍历完出栈时再产出它自身
                        this.reading = Some((path.clone(), Box::pin(async_fs::read_dir(path))));
                        continue;
                    }

                    match this.mode {
                        WalkDirMode::Files | WalkDirMode::All => {
                            return Poll::Ready(Some(path));
                        }

                        WalkDirMode::Directories => {
                            continue;
                        }
                    }
                }

                Poll::Ready(Some(Err(_))) => {
                    // 忽略单个目录项错误
                    continue;
                }

                Poll::Ready(None) => {
                    // 内容已经全部产出，后序产出目录自身
                    let frame = this.stack.pop().expect("stack top checked above");

                    if this.yields_dir(&frame.path) {
                        return Poll::Ready(Some(frame.path));
                    }

                    continue;
                }
            }
        }
    }
}
