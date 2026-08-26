//! Integration tests for the Turso-backed storage registry.

use phanerite::db::{Database, migration::apply_pending, storage_registry::StorageReg};
use phanerite_core::{
    storage::{StorageIdent, shared::MultiRegistry},
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

async fn setup() -> (Database, tempfile::TempDir) {
    let root = tempfile::tempdir().unwrap();
    let db = Database::new();
    apply_pending(&db).await.unwrap();
    (db, root)
}

#[test]
fn missing_entry_is_a_cache_miss() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let registry = StorageReg::new(db).await;

        let storage = storage(root.path().join("missing"));
        let result = registry.query((&storage, &hash(1))).await;

        assert!(result.is_none());
    });
}

#[test]
fn inserted_entry_can_be_queried_with_the_same_key() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let registry = StorageReg::new(db).await;
        let storage = storage(root.path().join("storage"));
        let key = hash(2);
        let value = root.path().join("share/asset.bin");

        registry
            .insert((&storage, key.clone()), value.clone())
            .await
            .unwrap();

        let result = registry.query((&storage, &key)).await.unwrap();
        assert_eq!(result.as_path(), value.as_path());
    });
}

#[test]
fn query_and_increase_and_decrease_update_reference_count() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let registry = StorageReg::new(db).await;
        let storage = storage(root.path().join("storage"));
        let key = hash(9);
        let value = root.path().join("share/counts.bin");

        registry
            .insert((&storage, key.clone()), value.clone())
            .await
            .unwrap();

        assert_eq!(
            registry
                .query_and_increase((&storage, &key))
                .await
                .unwrap()
                .unwrap()
                .as_path(),
            value.as_path()
        );
        assert_eq!(registry.decrease((&storage, &key)).await.unwrap(), 1);
        assert_eq!(registry.decrease((&storage, &key)).await.unwrap(), 0);
        assert!(registry.query((&storage, &key)).await.is_none());
    });
}

#[test]
fn storage_and_hash_are_both_part_of_the_key() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let registry = StorageReg::new(db).await;
        let storage_a = storage(root.path().join("a"));
        let storage_b = storage(root.path().join("b"));
        let value_a = root.path().join("share/a.bin");
        let value_b = root.path().join("share/b.bin");
        let value_c = root.path().join("share/c.bin");

        registry
            .insert((&storage_a, hash(3)), value_a.clone())
            .await
            .unwrap();
        registry
            .insert((&storage_b, hash(3)), value_b.clone())
            .await
            .unwrap();
        registry
            .insert((&storage_a, hash(4)), value_c.clone())
            .await
            .unwrap();

        assert_eq!(
            registry
                .query((&storage_a, &hash(3)))
                .await
                .unwrap()
                .as_path(),
            value_a.as_path()
        );
        assert_eq!(
            registry
                .query((&storage_b, &hash(3)))
                .await
                .unwrap()
                .as_path(),
            value_b.as_path()
        );
        assert_eq!(
            registry
                .query((&storage_a, &hash(4)))
                .await
                .unwrap()
                .as_path(),
            value_c.as_path()
        );
    });
}

#[test]
fn data_is_visible_through_a_new_registry_on_the_same_database() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let first = StorageReg::new(db.clone()).await;
        let storage = storage(root.path().join("storage"));
        let key = hash(5);
        let value = root.path().join("share/persistent.bin");

        first
            .insert((&storage, key.clone()), value.clone())
            .await
            .unwrap();

        let second = StorageReg::new(db).await;
        assert_eq!(
            second.query((&storage, &key)).await.unwrap().as_path(),
            value.as_path()
        );
    });
}

#[test]
fn duplicate_key_does_not_replace_the_existing_path() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let registry = StorageReg::new(db).await;
        let storage = storage(root.path().join("storage"));
        let key = hash(6);
        let first = root.path().join("share/first.bin");
        let second = root.path().join("share/second.bin");

        registry
            .insert((&storage, key.clone()), first.clone())
            .await
            .unwrap();
        let _ = registry.insert((&storage, key.clone()), second).await;

        assert_eq!(
            registry.query((&storage, &key)).await.unwrap().as_path(),
            first.as_path()
        );
    });
}

#[test]
fn duplicate_path_does_not_create_a_second_entry() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let registry = StorageReg::new(db).await;
        let path = root.path().join("share/shared.bin");
        let first_storage = storage(root.path().join("first"));
        let second_storage = storage(root.path().join("second"));
        let first_key = hash(7);
        let second_key = hash(8);

        registry
            .insert((&first_storage, first_key.clone()), path.clone())
            .await
            .unwrap();
        let _ = registry
            .insert((&second_storage, second_key.clone()), path.clone())
            .await;

        assert!(registry.query((&first_storage, &first_key)).await.is_some());
        assert!(
            registry
                .query((&second_storage, &second_key))
                .await
                .is_none()
        );
    });
}

#[test]
fn concurrent_inserts_and_queries_keep_entries_isolated() {
    gpui::block_on(async {
        let (db, root) = setup().await;
        let registry = StorageReg::new(db).await;
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
                    gpui::block_on(registry.insert((&key.0, key.1.clone()), value)).unwrap();
                });
            }
        });

        for (key, value) in expected {
            assert_eq!(
                registry.query((&key.0, &key.1)).await.unwrap().as_path(),
                value.as_path()
            );
        }
    });
}
