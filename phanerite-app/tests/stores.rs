//! Integration tests for application store mutation and notification behavior.

use phanerite::state::*;
#[cfg(feature = "seed")]
use phanerite_core::storage::{Storage, StorageIdent as CoreStorageIdent, multi::MultiStorage};

#[cfg(feature = "seed")]
#[test]
fn instance_equal_mutations_do_not_notify() {
    let id = phanerite::seed::storage_ident(1);
    let mut store = InstanceStore::new(phanerite::seed::seed_instances(id.clone()));
    store.set_storage_context(id.clone());
    let reference = phanerite::route::InstanceRef::new(id, "inst-fog");
    assert!(!store.set_mod_enabled(&reference, "m-sodium", true));
    assert!(store.set_mod_enabled(&reference, "m-sodium", false));
    assert!(!store.set_mod_enabled(&reference, "m-sodium", false));
}

#[cfg(feature = "seed")]
#[test]
fn stale_storage_results_are_ignored() {
    let one = phanerite::seed::storage_ident(1);
    let two = phanerite::seed::storage_ident(2);
    let mut store = InstanceStore::new(Vec::new());
    store.set_storage_context(two.clone());
    let storages = MultiStorage::new();
    let root = tempfile::tempdir().unwrap();
    let storage = gpui_kit::block_on(Storage::new(root.path())).unwrap();
    gpui_kit::block_on(storages.insert(one.clone(), storage)).unwrap();
    assert!(!store.apply_for_storage(
        &storages,
        one,
        phanerite::seed::seed_instances(phanerite::seed::storage_ident(1))
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

#[cfg(feature = "seed")]
#[test]
fn all_mutable_stores_ignore_equal_values() {
    let mut accounts = AccountStore::new(phanerite::seed::seed_accounts());
    assert!(!accounts.set_active_profile("acc-enita", "profile-enita"));
    assert_eq!(accounts.revision(), 0);

    let mut launch = LaunchStore::default();
    let reference =
        phanerite::route::InstanceRef::new(phanerite::seed::storage_ident(1), "inst-fog");
    let job = LaunchJob::new(reference.clone(), "The Fog", "the-fog", Loader::Fabric);
    assert!(launch.start(job.clone()));
    assert!(!launch.start(job));
    assert!(launch.finish(&reference));
    assert_eq!(launch.revision(), 2);

    let mut sessions = SessionStore::default();
    let session = SessionSummary {
        id: SessionId::from("session-a"),
        instance: reference,
        started_at: "now".into(),
        exit_code: None,
        running: true,
    };
    assert!(sessions.start(session.clone()));
    assert!(!sessions.start(session));
    assert!(!sessions.finish(&SessionId::from("missing"), 1));
    assert_eq!(sessions.revision(), 1);

    let root = tempfile::tempdir().unwrap();
    let storage = gpui_kit::block_on(Storage::new(root.path())).unwrap();
    let storage = CoreStorageIdent::from(&storage);
    let storages = MultiStorage::new();
    let storage_value = gpui_kit::block_on(Storage::new(root.path())).unwrap();
    gpui_kit::block_on(storages.insert(storage.clone(), storage_value)).unwrap();
    let mut crashes = CrashStore::default();
    crashes.set_storage_context(storage.clone());
    assert!(!crashes.apply_for_storage(&storages, storage, vec![]));
    assert_eq!(crashes.revision(), 0);
}

#[cfg(feature = "seed")]
#[test]
fn storage_registry_and_context_both_guard_late_results() {
    let root_a = tempfile::tempdir().unwrap();
    let root_b = tempfile::tempdir().unwrap();
    let storage_a = gpui_kit::block_on(Storage::new(root_a.path())).unwrap();
    let storage_b = gpui_kit::block_on(Storage::new(root_b.path())).unwrap();
    let a = CoreStorageIdent::from(&storage_a);
    let b = CoreStorageIdent::from(&storage_b);
    let storages = MultiStorage::new();
    gpui_kit::block_on(storages.insert(a.clone(), storage_a)).unwrap();
    gpui_kit::block_on(storages.insert(b.clone(), storage_b)).unwrap();
    let mut store = InstanceStore::new(Vec::new());
    store.set_storage_context(a.clone());
    assert!(store.apply_for_storage(
        &storages,
        a.clone(),
        phanerite::seed::seed_instances(a.clone())
    ));
    assert_eq!(store.all()[0].storage, a);
    store.set_storage_context(b.clone());
    assert!(!store.apply_for_storage(
        &storages,
        a.clone(),
        phanerite::seed::seed_instances(a.clone())
    ));
    assert_eq!(store.all()[0].storage, a);
    assert!(store.apply_for_storage(
        &storages,
        b.clone(),
        phanerite::seed::seed_instances(b.clone())
    ));
    assert_eq!(store.all()[0].storage, b);
}

#[cfg(feature = "seed")]
#[test]
fn preview_and_offline_account_selection_survives_profile_changes_and_removal() {
    let mut accounts = AccountStore::default();
    let offline = accounts
        .add("Steve", AccountType::Offline, None, vec![], None)
        .unwrap();
    let profile = |id: &str| PlayerProfileSummary {
        id: id.into(),
        name: id.into(),
        skin_url: String::new(),
        is_slim: false,
    };
    let preview = accounts
        .add_preview(
            "Player".into(),
            AccountType::Aphanite,
            Some("https://play.example.org".into()),
            vec![profile("survival"), profile("builder")],
            "survival".into(),
        )
        .unwrap();
    assert_eq!(accounts.active_id().as_ref(), Some(&preview));
    assert!(accounts.set_active_profile(&preview, "builder"));
    assert_eq!(accounts.active_profile().unwrap().id, "builder");
    let revision = accounts.revision();
    assert!(!accounts.set_active_profile(&preview, "missing"));
    assert_eq!(accounts.revision(), revision);

    assert!(accounts.set_active(&offline));
    assert_eq!(
        accounts.active().unwrap().account_type,
        AccountType::Offline
    );
    assert!(accounts.remove(&offline));
    assert_eq!(accounts.active_id().as_ref(), Some(&preview));
    assert!(accounts.remove(&preview));
    assert!(accounts.active().is_none());
    assert!(accounts.is_empty());
}

#[cfg(feature = "seed")]
#[test]
fn preview_accounts_are_scoped_to_the_authentication_server() {
    let mut accounts = AccountStore::default();
    let profile = PlayerProfileSummary {
        id: "player".into(),
        name: "Player".into(),
        skin_url: String::new(),
        is_slim: false,
    };
    let mut add = |server: &str| {
        accounts.add_preview(
            "Player".into(),
            AccountType::Yggdrasil,
            Some(server.into()),
            vec![profile.clone()],
            "player".into(),
        )
    };
    let first = add("https://one.example.org").unwrap();
    assert!(add("https://one.example.org").is_none());
    let second = add("https://two.example.org").unwrap();
    assert_ne!(first, second);
    assert_eq!(accounts.len(), 2);
}
