use phanerite::state::StorageRegistry;

#[test]
fn empty_registry_has_no_default() {
    let registry = StorageRegistry::new();
    assert!(registry.default().is_none());
}
