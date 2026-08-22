use crate::runtime::RuntimeScanPath;
use crate::storage::Storage;
use crate::utils::container::{Container, Guard};

pub type MultiStorage = Container<Storage>;

// 带插件的 Storage
// 可以存 DownloaderWithCache，清理线程的 Shutdown 等需要与 Storage 相同生命周期的内容
/// `Storage` with a plugin
/// Can hold things that need the same lifetime as `Storage`, such as
/// `DownloaderWithCache` or the cleaner thread's shutdown guard
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
impl AsRef<Storage> for Guard<'_, Storage> {
    fn as_ref(&self) -> &Storage {
        self
    }
}
impl<P> AsRef<Storage> for Guard<'_, StorageWithPlugin<P>> {
    fn as_ref(&self) -> &Storage {
        &self.storage
    }
}
pub type MultiStorageWithPlugin<Plugin> = Container<StorageWithPlugin<Plugin>>;

impl RuntimeScanPath for MultiStorage {
    type Provider<'a> = Guard<'a, Storage>;

    fn storages(&self) -> impl Iterator<Item = Self::Provider<'_>> + '_ {
        self.snapshot().into_iter()
    }
}

impl<P> RuntimeScanPath for MultiStorageWithPlugin<P> {
    type Provider<'a>
        = Guard<'a, StorageWithPlugin<P>>
    where
        P: 'a;

    fn storages(&self) -> impl Iterator<Item = Self::Provider<'_>> + '_ {
        self.snapshot().into_iter()
    }
}
