use std::sync::Arc;

use phanerite::state::{StorageRegistry, StorageRegistryError};
use phanerite_core::storage::Storage;
use tempfile::TempDir;

fn registry() -> (TempDir, StorageRegistry) {
    let roots = tempfile::tempdir().unwrap();
    let first = roots.path().join("first");
    let second = roots.path().join("second");
    let mut registry = StorageRegistry::new();
    let first_storage = Arc::new(pollster::block_on(Storage::new(&first)).unwrap());
    let second_storage = Arc::new(pollster::block_on(Storage::new(&second)).unwrap());
    registry.add(&first, first_storage).unwrap();
    registry.add(&second, second_storage).unwrap();
    (roots, registry)
}

#[test]
fn empty_registry_has_no_default() {
    let registry = StorageRegistry::new();
    assert!(registry.default().is_none());
}

#[test]
fn first_entry_becomes_default() {
    let roots = tempfile::tempdir().unwrap();
    let root = roots.path().join("first");
    let storage = Arc::new(pollster::block_on(Storage::new(&root)).unwrap());
    let mut registry = StorageRegistry::new();
    let id = registry.add(root, storage).unwrap();
    assert!(registry.is_default(id));
}

#[test]
fn duplicate_root_is_rejected() {
    let (roots, mut registry) = registry();
    let root = roots.path().join("first");
    let storage = Arc::new(pollster::block_on(Storage::new(&root)).unwrap());
    assert_eq!(
        registry.add(root, storage),
        Err(StorageRegistryError::DuplicateRoot)
    );
}

#[test]
fn adding_does_not_change_default() {
    let (_roots, registry) = registry();
    let first = registry.default().unwrap().id;
    assert!(registry.is_default(first));
}

#[test]
fn switching_default_changes_default() {
    let (_roots, mut registry) = registry();
    let second = registry
        .get(phanerite::route::StorageId::new(1))
        .unwrap()
        .id;
    registry.set_default(second).unwrap();
    assert!(registry.is_default(second));
}

#[test]
fn deleting_default_requires_and_uses_replacement() {
    let (_roots, mut registry) = registry();
    let first = registry.default().unwrap().id;
    let second = registry
        .get(phanerite::route::StorageId::new(1))
        .unwrap()
        .id;
    assert!(matches!(
        registry.remove(first, None),
        Err(StorageRegistryError::ReplacementRequired)
    ));
    registry.remove(first, Some(second)).unwrap();
    assert!(registry.is_default(second));
}

#[test]
fn deleting_only_entry_clears_default() {
    let roots = tempfile::tempdir().unwrap();
    let root = roots.path().join("only");
    let storage = Arc::new(pollster::block_on(Storage::new(&root)).unwrap());
    let mut registry = StorageRegistry::new();
    let id = registry.add(root, storage).unwrap();
    registry.remove(id, None).unwrap();
    assert!(registry.default().is_none());
}

#[test]
fn ids_are_stable_and_monotonic_after_removal() {
    let (roots, mut registry) = registry();
    let first = registry.default().unwrap().id;
    let second = registry
        .get(phanerite::route::StorageId::new(1))
        .unwrap()
        .id;
    registry.remove(first, Some(second)).unwrap();
    let third_root = roots.path().join("third");
    let third_storage = Arc::new(pollster::block_on(Storage::new(&third_root)).unwrap());
    let third = registry.add(third_root, third_storage).unwrap();
    assert_ne!(first, third);
    assert_ne!(second, third);
}
