use crate::storage::Storage;
use std::path::PathBuf;

pub struct BuildInRuntime {
    runtime_dir: PathBuf,
}

impl BuildInRuntime {
    pub fn new(storage: &Storage) -> Self {
        Self {
            runtime_dir: storage.runtime_dir().to_path_buf(),
        }
    }
}
