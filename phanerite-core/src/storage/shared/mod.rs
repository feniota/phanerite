pub mod clean;
pub mod dedup;

use crate::error::Result;
use crate::utils::Hash;

// 全局的，只增的 Hash 注册表
// 以 blake3 为索引共享文件
// 将下载前可获取的 Hash 与本地的 blake3 关联
// 用于避免重复下载和清理共享文件
// 后端实现应该让读取操作等待已有的写入操作，避免重复下载
/// A global, append-only registry of hashes associated with shared files.
///
/// BLAKE3 is used to identify shared files, while other hashes that are
/// available before downloading are associated with the corresponding local
/// BLAKE3 hash. This allows existing files to be reused instead of being
/// downloaded again, and helps locate shared files during cleanup.
///
/// Backend implementations should ensure that read operations wait for
/// in-progress writes when necessary, so that concurrent requests for the
/// same hash do not trigger duplicate downloads.
#[allow(async_fn_in_trait)]
pub trait HashRegistry: Send + Sync {
    async fn insert(&self, blake3: blake3::Hash, other: Hash) -> Result<()>;
    async fn get(&self, other: &Hash) -> Option<blake3::Hash>;
}
