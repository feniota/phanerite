use crate::storage::symlink_async;
use futures::{Stream, StreamExt};
use std::ops::BitAnd;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// 目录能力
/// Directory capabilities
#[derive(Clone, Copy, Debug)]
pub struct DirCapability {
    pub read: bool,
    pub write: bool,
    pub hardlink: bool,
    pub symlink: bool,
}

impl BitAnd for DirCapability {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            read: self.read && rhs.read,
            write: self.write && rhs.write,
            hardlink: self.hardlink && rhs.hardlink,
            symlink: self.symlink && rhs.symlink,
        }
    }
}

// 遍历检查目录能力
/// Walks the tree and probes the directory capabilities
pub(super) async fn probe_tree(root: PathBuf) -> DirCapability {
    walk_dirs(root)
        .map(async |dir| probe_dir(&dir).await)
        .buffer_unordered(16)
        .fold(
            DirCapability {
                read: true,
                write: true,
                hardlink: true,
                symlink: true,
            },
            async |a, b| a & b,
        )
        .await
}

// 检查目录能力
/// Probes the capabilities of a directory
async fn probe_dir(current: &Path) -> DirCapability {
    let test_file = current.join(format!(".test-{}", Uuid::now_v7()));
    let test_link = current.join(format!(".test-link-{}", Uuid::now_v7()));

    let read = async_fs::read_dir(current).await.is_ok();

    let write = async_fs::File::create(&test_file).await.is_ok();

    let hardlink = async_fs::hard_link(&test_file, &test_link).await.is_ok();

    let symlink = symlink_async(&test_file, &test_link).await.is_ok();

    let _ = async_fs::remove_file(&test_file).await;
    let _ = async_fs::remove_file(&test_link).await;

    DirCapability {
        read,
        write,
        hardlink,
        symlink,
    }
}

// 遍历目录
/// Walks the directory tree
fn walk_dirs(root: PathBuf) -> impl Stream<Item = PathBuf> {
    futures::stream::unfold(vec![root], async |mut stack| {
        let dir = stack.pop()?;

        let mut entries = match async_fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => return Some((dir, stack)),
        };

        while let Some(entry) = entries.next().await {
            if let Ok(entry) = entry
                && entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false)
            {
                stack.push(entry.path());
            }
        }

        Some((dir, stack))
    })
}
