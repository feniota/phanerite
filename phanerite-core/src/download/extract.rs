use crate::error::Result;
use std::path::{Path, PathBuf};

pub enum ArchiveFormat {
    Zip,
    Tar,
}

pub struct ExtractTask {
    path: PathBuf,
    format: ArchiveFormat,
}

impl ExtractTask {
    pub async fn exec(&self, archive: &Path, bucket: Option<PathBuf>) -> Result<()> {
        todo!()
    }
}
