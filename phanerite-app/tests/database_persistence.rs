//! Integration test for the debug database persistence override.

use phanerite::db::{Database, migration::apply_pending, storage_registry::StorageReg};
use phanerite_core::{
    storage::{StorageIdent, shared::MultiRegistry},
    utils::{Blake3Hash, Hash, HashValue},
};

#[test]
fn database_path_override_persists_registry_entries() {
    gpui::block_on(async {
        let root = tempfile::tempdir().unwrap();
        let storage = StorageIdent {
            root_dir: root.path().join("storage"),
        };
        let hash = Hash::Blake3(Blake3Hash::from_bytes(&[42; 32]).unwrap());
        let value = root.path().join("share/asset.bin");

        // This integration test is compiled in debug mode, where the override
        // is intentionally supported. The environment is private to this test
        // binary, which avoids affecting the other integration-test binaries.
        unsafe { std::env::set_var("PHANERITE_DB_PATH", root.path()) };

        let first = Database::new();
        apply_pending(&first).await.unwrap();
        let registry = StorageReg::new(first).await;
        registry
            .insert((&storage, hash.clone()), value.clone())
            .await
            .unwrap();
        drop(registry);

        let second = Database::new();
        apply_pending(&second).await.unwrap();
        let registry = StorageReg::new(second).await;
        let persisted = registry.query((&storage, &hash)).await.unwrap();

        assert_eq!(persisted.as_path(), value.as_path());
        assert!(root.path().join("database").exists());

        unsafe { std::env::remove_var("PHANERITE_DB_PATH") };
    });
}
