//! Storage registry and persistence context for app-managed data.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use phanerite_core::storage::Storage;

use crate::route::StorageId;

pub struct StorageEntry {
    pub id: StorageId,
    pub root: PathBuf,
    pub storage: Arc<Storage>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StorageRegistryError {
    DuplicateRoot,
    UnknownId,
    InvalidReplacement,
    ReplacementRequired,
}

pub struct StorageRegistry {
    entries: Vec<StorageEntry>,
    default: Option<StorageId>,
    next_id: u64,
}

impl Default for StorageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            default: None,
            next_id: 0,
        }
    }

    pub fn is_default(&self, id: StorageId) -> bool {
        self.default == Some(id)
    }

    pub fn default(&self) -> Option<&StorageEntry> {
        self.default.and_then(|id| self.get(id))
    }

    pub fn get(&self, id: StorageId) -> Option<&StorageEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn add(
        &mut self,
        root: impl AsRef<Path>,
        storage: Arc<Storage>,
    ) -> Result<StorageId, StorageRegistryError> {
        let root = root.as_ref().to_path_buf();
        if self.entries.iter().any(|entry| entry.root == root) {
            return Err(StorageRegistryError::DuplicateRoot);
        }
        let id = StorageId::new(self.next_id);
        self.next_id += 1;
        self.entries.push(StorageEntry { id, root, storage });
        if self.default.is_none() {
            self.default = Some(id);
        }
        Ok(id)
    }

    pub fn set_default(&mut self, id: StorageId) -> Result<(), StorageRegistryError> {
        if self.get(id).is_none() {
            return Err(StorageRegistryError::UnknownId);
        }
        self.default = Some(id);
        Ok(())
    }

    pub fn remove(
        &mut self,
        id: StorageId,
        replacement: Option<StorageId>,
    ) -> Result<StorageEntry, StorageRegistryError> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.id == id)
            .ok_or(StorageRegistryError::UnknownId)?;
        if self.default == Some(id) {
            if self.entries.len() == 1 {
                if replacement.is_some() {
                    return Err(StorageRegistryError::InvalidReplacement);
                }
            } else {
                let replacement = replacement.ok_or(StorageRegistryError::ReplacementRequired)?;
                if replacement == id || self.get(replacement).is_none() {
                    return Err(StorageRegistryError::InvalidReplacement);
                }
                self.default = Some(replacement);
            }
        } else if replacement.is_some() {
            return Err(StorageRegistryError::InvalidReplacement);
        }
        let removed = self.entries.remove(index);
        if self.entries.is_empty() {
            self.default = None;
        }
        Ok(removed)
    }
}
