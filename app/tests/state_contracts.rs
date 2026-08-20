//! Integration tests for state and storage-reference contracts.

use std::path::PathBuf;

use phanerite::{
    route::{CrashRef, InstanceRef, StorageId},
    state::JavaRuntimeSummary,
};

#[test]
fn references_include_storage_context() {
    let storage_id = StorageId::for_test(7);
    assert_eq!(
        InstanceRef::new(storage_id, "instance").storage_id,
        storage_id
    );
    assert_eq!(CrashRef::new(storage_id, "crash").storage_id, storage_id);
}

#[test]
fn seed_projections_are_deterministic_and_storage_scoped() {
    let storage = StorageId::for_test(7);
    let first = phanerite::seed::seed_instances(storage);
    let second = phanerite::seed::seed_instances(storage);

    assert_eq!(first, second);
    assert!(first.iter().all(|instance| instance.storage_id == storage));
    assert!(first.iter().all(|instance| !instance.icon_seed.is_empty()));
}

#[test]
fn java_runtime_projection_keeps_owned_executable_path() {
    let runtime = phanerite_core::runtime::java::JavaRuntime {
        name: "Temurin".into(),
        major: 21,
        version: "21.0.6".into(),
        path: PathBuf::from("/opt/java/bin/java"),
    };

    let projection = JavaRuntimeSummary::from_core(&runtime);
    assert_eq!(projection.name, "Temurin");
    assert_eq!(projection.version, 21);
    assert_eq!(projection.version_string, "21.0.6");
    assert_eq!(projection.path, PathBuf::from("/opt/java/bin/java"));
}

#[test]
fn crash_exports_redact_credentials_tokens_and_home_paths() {
    let input = concat!(
        "--accessToken secret-access ",
        "--clientToken client-secret ",
        "--session session-secret ",
        "jwt=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.signature ",
        "path=/home/alice/.minecraft/logs/latest.log"
    );

    let redacted = phanerite::state::redact(input);
    assert!(!redacted.contains("secret-access"));
    assert!(!redacted.contains("client-secret"));
    assert!(!redacted.contains("session-secret"));
    assert!(!redacted.contains("eyJhbGci"));
    assert!(!redacted.contains("/home/alice"));
    assert!(redacted.contains("~/.minecraft/logs/latest.log"));
}
