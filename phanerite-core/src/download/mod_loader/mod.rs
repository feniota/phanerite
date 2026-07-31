use crate::download::task::DownloadTask;
use crate::instance::Instance;
use crate::storage::Storage;

pub mod fabric;

impl Instance {
    pub(crate) fn extra_downloads(&self, storage: &Storage) -> impl Iterator<Item = DownloadTask> {
        self.fabric_downloads(storage)
    }
}
