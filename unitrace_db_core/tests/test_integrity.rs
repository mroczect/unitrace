#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use csv as _;
use hex as _;
use rusqlite as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile::tempdir;

use unitrace_db_core::checksum::checksum_hex;
use unitrace_db_core::integrity::DefaultIntegrity;
use unitrace_db_handler::traits::Integrity;
use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Metadata, Schema, schema};

fn sample_schema() -> Arc<Schema> {
    let cols = vec![
        ColumnDef::try_new("id".to_string(), ColumnKind::Integer)
            .unwrap()
            .with_identity(),
        ColumnDef::try_new("name".to_string(), ColumnKind::Text).unwrap(),
    ];
    Arc::new(Schema::try_new(cols).unwrap())
}

fn metadata_with_checksum(checksum: String) -> Metadata {
    Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1.0.0".to_string(),
        100,
        checksum,
        "2025-11-11T10:00:00Z".to_string(),
        sample_schema(),
    )
    .unwrap()
}

#[test]
fn verify_passes_for_matching_checksum() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let payload = b"some database content";
    std::fs::write(&db_path, payload).unwrap();

    let metadata = metadata_with_checksum(checksum_hex(payload));
    let integrity = DefaultIntegrity::new();
    integrity.verify(&db_path, &metadata).unwrap();
}

#[test]
fn verify_accepts_uppercase_hex_in_metadata() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let payload = b"some database content";
    std::fs::write(&db_path, payload).unwrap();

    let metadata = metadata_with_checksum(checksum_hex(payload).to_uppercase());
    let integrity = DefaultIntegrity::new();
    integrity.verify(&db_path, &metadata).unwrap();
}

#[test]
fn verify_fails_on_tampered_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let original = b"original content";
    std::fs::write(&db_path, original).unwrap();

    let metadata = metadata_with_checksum(checksum_hex(original));
    std::fs::write(&db_path, b"tampered content").unwrap();

    let integrity = DefaultIntegrity::new();
    let result = integrity.verify(&db_path, &metadata);
    assert!(matches!(
        result.unwrap_err(),
        Error::ChecksumMismatch { .. }
    ));
}

#[test]
fn verify_fails_on_missing_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("does-not-exist.db");
    let metadata = metadata_with_checksum("a".repeat(64));

    let integrity = DefaultIntegrity::new();
    let result = integrity.verify(&db_path, &metadata);
    assert!(matches!(result.unwrap_err(), Error::DatabaseNotFound(_)));
}

#[test]
fn verify_fails_on_empty_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("empty.db");
    std::fs::write(&db_path, b"").unwrap();

    let metadata = metadata_with_checksum(checksum_hex(b"nonempty"));
    let integrity = DefaultIntegrity::new();
    let result = integrity.verify(&db_path, &metadata);
    assert!(matches!(
        result.unwrap_err(),
        Error::ChecksumMismatch { .. }
    ));
}

#[test]
fn integrity_is_zero_sized() {
    assert_eq!(size_of::<DefaultIntegrity>(), 0);
}
