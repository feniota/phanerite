use crate::storage::symlink_async;
use crate::utils::walkdir::WalkDir;
use futures::StreamExt;
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
    WalkDir::new(root)
        .dir_mode()
        .map(DirCapability::check_path)
        .buffer_unordered(32)
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

impl DirCapability {
    // 检查目录能力
    /// Probes the capabilities of a directory
    async fn check_path(path: impl AsRef<Path>) -> DirCapability {
        let current = path.as_ref();

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
}
