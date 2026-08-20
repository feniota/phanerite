use crate::runtime::RuntimeScanPath;
use crate::storage::Storage;
use crate::utils::container::Container;
use std::path::PathBuf;

pub type MultiStorage = Container<Storage>;

/// 带插件的 Storage
/// 可以存 DownloaderWithCache，清理线程的 Shutdown 等需要与 Storage 相同生命周期的内容
pub struct StorageWithPlugin<P> {
    pub storage: Storage,
    pub plugin: P,
}
impl<P> PartialEq for StorageWithPlugin<P> {
    fn eq(&self, other: &Self) -> bool {
        self.storage == other.storage
    }
}
impl<P> Eq for StorageWithPlugin<P> {}
pub type MultiStorageWithPlugin<Plugin> = Container<StorageWithPlugin<Plugin>>;

impl RuntimeScanPath for MultiStorage {
    fn paths(&self) -> impl Iterator<Item = PathBuf> {
        self.iter(|iter| {
            iter.map(|(_, s)| s.runtime_dir().to_owned())
                .collect::<Vec<_>>()
        })
        .into_iter()
    }
}

impl<P> RuntimeScanPath for MultiStorageWithPlugin<P> {
    fn paths(&self) -> impl Iterator<Item = PathBuf> {
        self.iter(|iter| {
            iter.map(|(_, s)| s.storage.runtime_dir().to_owned())
                .collect::<Vec<_>>()
        })
        .into_iter()
    }
}
