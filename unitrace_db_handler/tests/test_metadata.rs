#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Metadata, Schema, schema};

fn make_valid_schema() -> Arc<Schema> {
    let col = ColumnDef::try_new("id".to_string(), ColumnKind::Text).unwrap();
    Arc::new(Schema::try_new(vec![col]).unwrap())
}

#[test]
fn test_metadata_try_new_valid() {
    let schema = make_valid_schema();
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1.0".to_string(),
        100,
        "a".repeat(64),
        "2023-10-27T10:00:00Z".to_string(),
        schema,
    );
    assert!(result.is_ok());
    let meta = result.unwrap();
    assert_eq!(meta.schema_version(), schema::SCHEMA_VERSION);
    assert_eq!(meta.data_version(), "v1.0");
    assert_eq!(meta.row_count(), 100);
    assert_eq!(meta.checksum_sha256().len(), 64);
    assert!(meta.built_at().contains('-'));
}

#[test]
fn test_metadata_try_new_schema_version_mismatch() {
    let schema = make_valid_schema();
    let result = Metadata::try_new(
        99,
        "v1.0".to_string(),
        100,
        "a".repeat(64),
        "2023-10-27".to_string(),
        schema,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::SchemaVersionMismatch {
            expected: 1,
            found: 99
        }
    ));
}

#[test]
fn test_metadata_try_new_empty_data_version() {
    let schema = make_valid_schema();
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        String::new(),
        100,
        "a".repeat(64),
        "2023-10-27".to_string(),
        schema,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptyDataVersion));
}

#[test]
fn test_metadata_try_new_invalid_checksum() {
    let schema = make_valid_schema();
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1.0".to_string(),
        100,
        "not-a-valid-hex-string-of-64-chars-length!!".to_string(),
        "2023-10-27".to_string(),
        schema,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidChecksumHex));
}

#[test]
fn test_metadata_try_new_invalid_built_at() {
    let schema = make_valid_schema();
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1.0".to_string(),
        100,
        "a".repeat(64),
        "no-digits-here".to_string(),
        schema,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidBuiltAt));
}
