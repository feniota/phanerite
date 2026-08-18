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
    assert!(!store.apply_for_storage(one, phanerite::seed::seed_instances(one)));
    assert!(store.all().is_empty());
}

#[test]
fn settings_equal_values_do_not_notify() {
    let mut store = SettingsStore::default();
    assert!(!store.set_accent("emerald"));
    assert!(store.set_accent("gold"));
    assert!(!store.set_accent("gold"));
}
