use async_fs::{self, ReadDir};
use futures::{Stream, StreamExt, TryStreamExt};
use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};

type ReadDirFuture = Pin<Box<dyn Future<Output = std::io::Result<ReadDir>> + Send>>;

pub struct WalkDir {
    root: PathBuf,
    stack: Vec<PathBuf>,
    current: Option<ReadDir>,
    reading: Option<ReadDirFuture>,
}

impl WalkDir {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();

        Self {
            stack: vec![root.clone()],
            root,
            current: None,
            reading: None,
        }
    }

    // 移动整个目录，并跳过已有文件
    /// Moves a whole directory, skipping files that already exist
    pub async fn merge_move(self, dst: impl Into<PathBuf>) -> std::io::Result<()> {
        let root = self.root.clone();
        let dst = dst.into();

        async_fs::create_dir_all(&dst).await?;

        self.map(async |src| {
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

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // 当前目录正在读取
            if let Some(current) = &mut self.current {
                match Pin::new(current).poll_next(cx) {
                    Poll::Pending => return Poll::Pending,

                    Poll::Ready(Some(Ok(entry))) => {
                        let path = entry.path();

                        if path.is_dir() {
                            self.stack.push(path);
                            continue;
                        }

                        return Poll::Ready(Some(path));
                    }

                    Poll::Ready(Some(Err(_))) => {
                        // 忽略单个目录项错误
                        continue;
                    }

                    Poll::Ready(None) => {
                        self.current = None;
                        continue;
                    }
                }
            }

            // 正在异步打开下一个目录
            if let Some(reading) = &mut self.reading {
                match reading.as_mut().poll(cx) {
                    Poll::Pending => return Poll::Pending,

                    Poll::Ready(Ok(read_dir)) => {
                        self.reading = None;
                        self.current = Some(read_dir);
                        continue;
                    }

                    Poll::Ready(Err(_)) => {
                        self.reading = None;
                        continue;
                    }
                }
            }

            // 没有正在读取的目录，尝试从 DFS 栈中取一个
            if let Some(path) = self.stack.pop() {
                self.reading = Some(Box::pin(async_fs::read_dir(path)));
                continue;
            }

            // 栈为空，遍历结束
            return Poll::Ready(None);
        }
    }
}
