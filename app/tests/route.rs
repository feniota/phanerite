use phanerite::route::{
    CrashRef, InstanceRef, Navigation, Route, StorageId,
};

#[test]
fn back_from_root_stays_on_play() {
    let mut nav = Navigation::new(Route::Play);
    nav.back();
    assert_eq!(nav.current(), &Route::Play);
}

#[test]
fn push_does_not_duplicate_current_route() {
    let mut nav = Navigation::new(Route::Play);
    nav.push(Route::Instances);
    nav.push(Route::Instances);
    nav.back();
    assert_eq!(nav.current(), &Route::Play);
}

#[test]
fn history_is_bounded_to_root_plus_thirty_one_entries() {
    let mut nav = Navigation::new(Route::Play);
    for _ in 0..40 {
        nav.push(Route::Settings);
    }
    assert!(nav.history_len() <= 32);
}

#[test]
fn replace_does_not_add_history() {
    let mut nav = Navigation::new(Route::Play);
    nav.push(Route::Instances);
    nav.replace(Route::Settings);
    nav.back();
    assert_eq!(nav.current(), &Route::Play);
}

#[test]
fn equal_instance_ids_in_different_storage_are_distinct() {
    let first = InstanceRef::for_test(StorageId::for_test(1), "shared");
    let second = InstanceRef::for_test(StorageId::for_test(2), "shared");
    assert_ne!(first, second);
}

#[test]
fn equal_crash_ids_in_different_storage_are_distinct() {
    let first = CrashRef::for_test(StorageId::for_test(1), "report");
    let second = CrashRef::for_test(StorageId::for_test(2), "report");
    assert_ne!(first, second);
}
