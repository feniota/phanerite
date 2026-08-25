//! Integration tests for the Turso-backed storage registry.

use phanerite::db::{Database, migration::apply_pending, storage_registry::StorageReg};
use phanerite_core::{
    download::dedup::StorageRegistry,
    storage::StorageIdent,
    utils::{Blake3Hash, Hash, HashValue},
};
use std::path::Path;

fn storage(root: impl AsRef<Path>) -> StorageIdent {
    StorageIdent {
        root_dir: root.as_ref().to_owned(),
    }
}

fn hash(byte: u8) -> Hash {
    Hash::Blake3(Blake3Hash::from_bytes(&[byte; 32]).unwrap())
}

fn setup() -> (Database, tempfile::TempDir) {
    let root = tempfile::tempdir().unwrap();
    let db = Database::new();
    gpui::block_on(apply_pending(&db)).unwrap();
    (db, root)
}

#[test]
fn missing_entry_is_a_cache_miss() {
    let (db, root) = setup();
    let registry = gpui::block_on(StorageReg::new(db));

    let key = (storage(root.path().join("missing")), hash(1));
    let result = gpui::block_on(registry.query(&key));

    assert!(result.is_none());
}

#[test]
fn inserted_entry_can_be_queried_with_the_same_key() {
    let (db, root) = setup();
    let registry = gpui::block_on(StorageReg::new(db));
    let key = (storage(root.path().join("storage")), hash(2));
    let value = root.path().join("share/asset.bin");

    gpui::block_on(registry.insert(key.clone(), value.clone()));

    let result = gpui::block_on(registry.query(&key)).unwrap();
    assert_eq!(result.as_path(), value.as_path());
}

#[test]
fn storage_and_hash_are_both_part_of_the_key() {
    let (db, root) = setup();
    let registry = gpui::block_on(StorageReg::new(db));
    let storage_a = storage(root.path().join("a"));
    let storage_b = storage(root.path().join("b"));
    let value_a = root.path().join("share/a.bin");
    let value_b = root.path().join("share/b.bin");
    let value_c = root.path().join("share/c.bin");

    gpui::block_on(registry.insert((storage_a.clone(), hash(3)), value_a.clone()));
    gpui::block_on(registry.insert((storage_b.clone(), hash(3)), value_b.clone()));
    gpui::block_on(registry.insert((storage_a.clone(), hash(4)), value_c.clone()));

    assert_eq!(
        gpui::block_on(registry.query(&(storage_a.clone(), hash(3))))
            .unwrap()
            .as_path(),
        value_a.as_path()
    );
    assert_eq!(
        gpui::block_on(registry.query(&(storage_b, hash(3))))
            .unwrap()
            .as_path(),
        value_b.as_path()
    );
    assert_eq!(
        gpui::block_on(registry.query(&(storage_a, hash(4))))
            .unwrap()
            .as_path(),
        value_c.as_path()
    );
}

#[test]
fn data_is_visible_through_a_new_registry_on_the_same_database() {
    let (db, root) = setup();
    let first = gpui::block_on(StorageReg::new(db.clone()));
    let key = (storage(root.path().join("storage")), hash(5));
    let value = root.path().join("share/persistent.bin");

    gpui::block_on(first.insert(key.clone(), value.clone()));

    let second = gpui::block_on(StorageReg::new(db));
    assert_eq!(
        gpui::block_on(second.query(&key)).unwrap().as_path(),
        value.as_path()
    );
}

#[test]
fn duplicate_key_does_not_replace_the_existing_path() {
    let (db, root) = setup();
    let registry = gpui::block_on(StorageReg::new(db));
    let key = (storage(root.path().join("storage")), hash(6));
    let first = root.path().join("share/first.bin");
    let second = root.path().join("share/second.bin");

    gpui::block_on(registry.insert(key.clone(), first.clone()));
    gpui::block_on(registry.insert(key.clone(), second));

    assert_eq!(
        gpui::block_on(registry.query(&key)).unwrap().as_path(),
        first.as_path()
    );
}

#[test]
fn duplicate_path_does_not_create_a_second_entry() {
    let (db, root) = setup();
    let registry = gpui::block_on(StorageReg::new(db));
    let path = root.path().join("share/shared.bin");
    let first_key = (storage(root.path().join("first")), hash(7));
    let second_key = (storage(root.path().join("second")), hash(8));

    gpui::block_on(registry.insert(first_key.clone(), path.clone()));
    gpui::block_on(registry.insert(second_key.clone(), path.clone()));

    assert!(gpui::block_on(registry.query(&first_key)).is_some());
    assert!(gpui::block_on(registry.query(&second_key)).is_none());
}

#[test]
fn concurrent_inserts_and_queries_keep_entries_isolated() {
    let (db, root) = setup();
    let registry = gpui::block_on(StorageReg::new(db));
    let entries: Vec<_> = (0..16_u8)
        .map(|i| {
            (
                (storage(root.path().join(format!("storage-{i}"))), hash(i)),
                root.path().join(format!("share/{i}.bin")),
            )
        })
        .collect();
    let expected: Vec<_> = entries
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();

    std::thread::scope(|scope| {
        for (key, value) in entries {
            let registry = &registry;
            scope.spawn(move || {
                gpui::block_on(registry.insert(key, value));
            });
        }
    });

    for (key, value) in expected {
        assert_eq!(
            gpui::block_on(registry.query(&key)).unwrap().as_path(),
            value.as_path()
        );
    }
}
