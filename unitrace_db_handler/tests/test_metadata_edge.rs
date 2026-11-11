#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Metadata, Schema, schema};

fn schema_one() -> Arc<Schema> {
    let col = ColumnDef::try_new("id".to_string(), ColumnKind::Text).unwrap();
    Arc::new(Schema::try_new(vec![col]).unwrap())
}

#[test]
fn metadata_row_count_zero_is_valid() {
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v0".to_string(),
        0,
        "a".repeat(64),
        "2023-10-27T10:00:00Z".to_string(),
        schema_one(),
    );
    assert!(result.is_ok());
}

#[test]
fn metadata_checksum_uppercase_hex_accepted() {
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1".to_string(),
        1,
        "A".repeat(64),
        "2023-10-27".to_string(),
        schema_one(),
    );
    assert!(result.is_ok());
}

#[test]
fn metadata_checksum_with_non_hex_rejected() {
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1".to_string(),
        1,
        "g".repeat(64),
        "2023-10-27".to_string(),
        schema_one(),
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidChecksumHex));
}

#[test]
fn metadata_checksum_65_chars_rejected() {
    let result = Metadata::try_new(
        schema::SCHEMA_VERSION,
        "v1".to_string(),
        1,
        "a".repeat(65),
        "2023-10-27".to_string(),
        schema_one(),
    );
    assert!(result.is_err());
}
