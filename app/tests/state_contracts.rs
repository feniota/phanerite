use phanerite::route::{CrashRef, InstanceRef, StorageId};

#[test]
fn references_include_storage_context() {
    let storage_id = StorageId::for_test(7);
    assert_eq!(InstanceRef::new(storage_id, "instance").storage_id, storage_id);
    assert_eq!(CrashRef::new(storage_id, "crash").storage_id, storage_id);
}
