use crate::storage::Storage;
use std::path::PathBuf;

pub mod buildin;
pub mod system;

#[cfg(target_os = "windows")]
const BIN_NAME: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
const BIN_NAME: &str = "java";

pub struct RuntimeManager {
    build_in_root: PathBuf,
}

impl RuntimeManager {
    fn new(storage: &Storage) -> Self {
        Self {
            build_in_root: storage.runtime_dir.clone(),
        }
    }
}

#[derive(Debug)]
pub struct JavaRuntime {
    name: String,
    major: usize,
    version: String,
    path: PathBuf,
}
