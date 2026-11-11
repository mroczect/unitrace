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

use ed25519_dalek as _;
use ed25519_dalek as _;
use unitrace_db_core::metadata_io::{read_metadata, verify_checksum, write_metadata};
use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Metadata, Schema, schema};

fn sample_schema() -> Arc<Schema> {
    let cols = vec![
        ColumnDef::try_new("id".to_string(), ColumnKind::Integer)
            .unwrap()
            .with_identity(),
        ColumnDef::try_new("no_mesin".to_string(), ColumnKind::Text)
            .unwrap()
            .with_searchable()
            .with_normalize(),
        ColumnDef::try_new("tenor".to_string(), ColumnKind::Integer).unwrap(),
        ColumnDef::try_new("tgl".to_string(), ColumnKind::Date).unwrap(),
        ColumnDef::try_new("aktif".to_string(), ColumnKind::Boolean).unwrap(),
    ];
    Arc::new(Schema::try_new(cols).unwrap())
}

fn sample_metadata() -> Metadata {
    Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1.0.0".to_string(),
        8072,
        "a".repeat(64),
        "2025-11-11T10:00:00Z".to_string(),
        sample_schema(),
    )
    .unwrap()
}

#[test]
fn round_trip_preserves_all_fields() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let original = sample_metadata();

    write_metadata(&path, &original).unwrap();
    let restored = read_metadata(&path).unwrap();

    assert_eq!(restored.schema_version(), original.schema_version());
    assert_eq!(restored.data_version(), original.data_version());
    assert_eq!(restored.row_count(), original.row_count());
    assert_eq!(restored.checksum_sha256(), original.checksum_sha256());
    assert_eq!(restored.built_at(), original.built_at());
    assert_eq!(
        restored.schema().columns().len(),
        original.schema().columns().len()
    );
}

#[test]
fn round_trip_preserves_column_flags() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let original = sample_metadata();

    write_metadata(&path, &original).unwrap();
    let restored = read_metadata(&path).unwrap();

    for (orig, rest) in original
        .schema()
        .columns()
        .iter()
        .zip(restored.schema().columns().iter())
    {
        assert_eq!(orig.name(), rest.name());
        assert_eq!(orig.kind(), rest.kind());
        assert_eq!(orig.is_searchable(), rest.is_searchable());
        assert_eq!(orig.is_normalized(), rest.is_normalized());
        assert_eq!(orig.is_identity(), rest.is_identity());
    }
}

#[test]
fn round_trip_preserves_kind_enum() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let original = sample_metadata();

    write_metadata(&path, &original).unwrap();
    let restored = read_metadata(&path).unwrap();

    let kinds: Vec<_> = restored
        .schema()
        .columns()
        .iter()
        .map(ColumnDef::kind)
        .collect();
    assert!(kinds.contains(&ColumnKind::Integer));
    assert!(kinds.contains(&ColumnKind::Text));
    assert!(kinds.contains(&ColumnKind::Date));
    assert!(kinds.contains(&ColumnKind::Boolean));
}

#[test]
fn read_missing_file_returns_metadata_not_found() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("does-not-exist.json");
    let result = read_metadata(&path);
    assert!(matches!(result.unwrap_err(), Error::MetadataNotFound(_)));
}

#[test]
fn read_corrupted_json_returns_json_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    std::fs::write(&path, b"not valid json {{{").unwrap();
    let result = read_metadata(&path);
    assert!(matches!(result.unwrap_err(), Error::Json(_)));
}

#[test]
fn read_empty_file_returns_json_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    std::fs::write(&path, b"").unwrap();
    let result = read_metadata(&path);
    assert!(matches!(result.unwrap_err(), Error::Json(_)));
}

#[test]
fn read_json_with_bad_checksum_returns_invalid_checksum_hex() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let corrupt = r#"{
        "schema_version": 1,
        "data_version": "v1",
        "row_count": 0,
        "checksum_sha256": "too-short",
        "built_at": "2025-11-11",
        "schema": {
            "columns": [
                {"name": "id", "kind": "integer", "searchable": false, "normalize": false, "identity": true}
            ]
        }
    }"#;
    std::fs::write(&path, corrupt).unwrap();
    let result = read_metadata(&path);
    assert!(matches!(result.unwrap_err(), Error::InvalidChecksumHex));
}

#[test]
fn read_json_with_wrong_schema_version_returns_mismatch() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let corrupt = r#"{
        "schema_version": 99,
        "data_version": "v1",
        "row_count": 0,
        "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "built_at": "2025-11-11",
        "schema": {
            "columns": [
                {"name": "id", "kind": "integer", "searchable": false, "normalize": false, "identity": true}
            ]
        }
    }"#;
    std::fs::write(&path, corrupt).unwrap();
    let result = read_metadata(&path);
    assert!(matches!(
        result.unwrap_err(),
        Error::SchemaVersionMismatch {
            expected: 1,
            found: 99
        }
    ));
}

#[test]
fn read_json_with_duplicate_columns_returns_duplicate_column() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let corrupt = r#"{
        "schema_version": 1,
        "data_version": "v1",
        "row_count": 0,
        "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "built_at": "2025-11-11",
        "schema": {
            "columns": [
                {"name": "id", "kind": "integer", "searchable": false, "normalize": false, "identity": false},
                {"name": "ID", "kind": "text", "searchable": false, "normalize": false, "identity": false}
            ]
        }
    }"#;
    std::fs::write(&path, corrupt).unwrap();
    let result = read_metadata(&path);
    assert!(matches!(result.unwrap_err(), Error::DuplicateColumn(_)));
}

#[test]
fn read_json_with_two_identity_columns_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let corrupt = r#"{
        "schema_version": 1,
        "data_version": "v1",
        "row_count": 0,
        "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "built_at": "2025-11-11",
        "schema": {
            "columns": [
                {"name": "a", "kind": "integer", "searchable": false, "normalize": false, "identity": true},
                {"name": "b", "kind": "integer", "searchable": false, "normalize": false, "identity": true}
            ]
        }
    }"#;
    std::fs::write(&path, corrupt).unwrap();
    let result = read_metadata(&path);
    assert!(matches!(
        result.unwrap_err(),
        Error::MultipleIdentityColumns { count: 2 }
    ));
}

#[test]
fn write_creates_file_that_did_not_exist() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let original = sample_metadata();
    assert!(!path.exists());
    write_metadata(&path, &original).unwrap();
    assert!(path.exists());
}

#[test]
fn write_does_not_leave_tmp_file_on_success() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let original = sample_metadata();
    write_metadata(&path, &original).unwrap();

    let tmp = dir.path().join("data.meta.json.tmp");
    assert!(!tmp.exists(), "tmp file was not cleaned up");
}

#[test]
fn write_overwrites_existing_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.meta.json");
    let original = sample_metadata();
    write_metadata(&path, &original).unwrap();

    let updated = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v2.0.0".to_string(),
        9000,
        "b".repeat(64),
        "2025-12-01T10:00:00Z".to_string(),
        sample_schema(),
    )
    .unwrap();
    write_metadata(&path, &updated).unwrap();

    let restored = read_metadata(&path).unwrap();
    assert_eq!(restored.data_version(), "v2.0.0");
    assert_eq!(restored.row_count(), 9000);
    assert_eq!(restored.checksum_sha256(), "b".repeat(64));
}

#[test]
fn verify_checksum_passes_for_matching_content() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let payload = b"some database bytes";
    std::fs::write(&db_path, payload).unwrap();

    let checksum = unitrace_db_core::checksum::checksum_hex(payload);
    let metadata = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1".to_string(),
        1,
        checksum,
        "2025-11-11".to_string(),
        sample_schema(),
    )
    .unwrap();

    verify_checksum(&db_path, &metadata).unwrap();
}

#[test]
fn verify_checksum_accepts_uppercase_hex() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let payload = b"some database bytes";
    std::fs::write(&db_path, payload).unwrap();

    let checksum = unitrace_db_core::checksum::checksum_hex(payload).to_uppercase();
    let metadata = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1".to_string(),
        1,
        checksum,
        "2025-11-11".to_string(),
        sample_schema(),
    )
    .unwrap();

    verify_checksum(&db_path, &metadata).unwrap();
}

#[test]
fn verify_checksum_fails_on_content_mismatch() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    std::fs::write(&db_path, b"tampered content").unwrap();

    let metadata = sample_metadata();
    let result = verify_checksum(&db_path, &metadata);
    assert!(matches!(
        result.unwrap_err(),
        Error::ChecksumMismatch { .. }
    ));
}

#[test]
fn verify_checksum_fails_on_missing_db() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("missing.db");
    let metadata = sample_metadata();
    let result = verify_checksum(&db_path, &metadata);
    assert!(matches!(result.unwrap_err(), Error::DatabaseNotFound(_)));
}

#[test]
fn verify_checksum_fails_on_empty_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("empty.db");
    std::fs::write(&db_path, b"").unwrap();

    let metadata = sample_metadata();
    let result = verify_checksum(&db_path, &metadata);
    assert!(matches!(
        result.unwrap_err(),
        Error::ChecksumMismatch { .. }
    ));
}
