use std::sync::Arc;

use phanerite::{route::StorageId, state::*};

#[test]
fn instance_equal_mutations_do_not_notify() {
    let id = StorageId::for_test(1);
    let mut store = InstanceStore::new(phanerite::seed::seed_instances(id));
    store.set_storage_context(id);
    assert!(!store.set_mod_enabled(id, "inst-fog", "m-sodium", true));
    assert!(store.set_mod_enabled(id, "inst-fog", "m-sodium", false));
    assert!(!store.set_mod_enabled(id, "inst-fog", "m-sodium", false));
}

#[test]
fn stale_storage_results_are_ignored() {
    let one = StorageId::for_test(1);
    let two = StorageId::for_test(2);
    let mut store = InstanceStore::new(Vec::new());
    store.set_storage_context(two);
    assert!(!store.apply_for_storage(
        &StorageRegistry::new(),
        one,
        phanerite::seed::seed_instances(one)
    ));
    assert!(store.all().is_empty());
}

#[test]
fn settings_equal_values_do_not_notify() {
    let mut store = SettingsStore::default();
    assert!(!store.set_accent("emerald"));
    assert!(store.set_accent("gold"));
    assert!(!store.set_accent("gold"));
}

#[test]
fn all_mutable_stores_ignore_equal_values() {
    let mut accounts = AccountStore::new(phanerite::seed::seed_accounts());
    assert!(!accounts.set_active_profile("acc-enita", "profile-enita"));
    assert_eq!(accounts.revision(), 0);

    let mut launch = LaunchStore::default();
    assert!(!launch.set_job(None));
    assert_eq!(launch.revision(), 0);

    let mut sessions = SessionStore::default();
    let session = SessionSummary {
        id: SessionId::from("session-a"),
        instance_id: "inst-fog".into(),
        started_at: "now".into(),
        exit_code: None,
        running: true,
    };
    assert!(sessions.start(session.clone()));
    assert!(!sessions.start(session));
    assert!(!sessions.finish("missing", 1));
    assert_eq!(sessions.revision(), 1);

    let root = tempfile::tempdir().unwrap();
    let storage =
        Arc::new(pollster::block_on(phanerite_core::storage::Storage::new(root.path())).unwrap());
    let mut registry = StorageRegistry::new();
    let storage_id = registry.add(root.path(), storage).unwrap();
    let mut crashes = CrashStore::default();
    crashes.set_storage_context(storage_id);
    assert!(!crashes.apply_for_storage(&registry, storage_id, vec![]));
    assert_eq!(crashes.revision(), 0);
}

#[test]
fn storage_registry_and_context_both_guard_late_results() {
    let root_a = tempfile::tempdir().unwrap();
    let root_b = tempfile::tempdir().unwrap();
    let storage_a =
        Arc::new(pollster::block_on(phanerite_core::storage::Storage::new(root_a.path())).unwrap());
    let storage_b =
        Arc::new(pollster::block_on(phanerite_core::storage::Storage::new(root_b.path())).unwrap());
    let mut registry = StorageRegistry::new();
    let a = registry.add(root_a.path(), storage_a).unwrap();
    let b = registry.add(root_b.path(), storage_b).unwrap();
    registry.set_default(b).unwrap();
    let mut store = InstanceStore::new(Vec::new());
    store.set_storage_context(a);
    assert!(store.apply_for_storage(&registry, a, phanerite::seed::seed_instances(a)));
    assert_eq!(store.all()[0].storage_id, a);
    store.set_storage_context(b);
    assert!(!store.apply_for_storage(&registry, a, phanerite::seed::seed_instances(a)));
    assert_eq!(store.all()[0].storage_id, a);
    assert!(store.apply_for_storage(&registry, b, phanerite::seed::seed_instances(b)));
    assert_eq!(store.all()[0].storage_id, b);
}
