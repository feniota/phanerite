use super::Storage;
use std::path::PathBuf;

/// A type representing a [`Storage`] which is relatively cheap to clone.
///
/// [`Storage`] has 13 fields, making it expensive to clone. By converting
/// a [`Storage`] to this type, we can identify a [`Storage`] without cloning
/// all 13 fields.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StorageIdent {
    pub root_dir: PathBuf,
}

impl From<&Storage> for StorageIdent {
    fn from(value: &Storage) -> Self {
        Self {
            root_dir: value.root_dir.clone(),
        }
    }
}

impl PartialEq<Storage> for StorageIdent {
    fn eq(&self, other: &Storage) -> bool {
        self.root_dir == other.root_dir
    }
}

impl PartialEq<StorageIdent> for Storage {
    fn eq(&self, other: &StorageIdent) -> bool {
        self.root_dir == other.root_dir
    }
}
