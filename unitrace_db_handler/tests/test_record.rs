#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Record, Schema};

fn make_schema_with_identity() -> Arc<Schema> {
    let cols = vec![
        ColumnDef::try_new("id".to_string(), ColumnKind::Integer)
            .unwrap()
            .with_identity(),
        ColumnDef::try_new("name".to_string(), ColumnKind::Text).unwrap(),
    ];
    Arc::new(Schema::try_new(cols).unwrap())
}

fn make_schema_without_identity() -> Arc<Schema> {
    let cols = vec![
        ColumnDef::try_new("name".to_string(), ColumnKind::Text).unwrap(),
        ColumnDef::try_new("role".to_string(), ColumnKind::Text).unwrap(),
    ];
    Arc::new(Schema::try_new(cols).unwrap())
}

#[test]
fn test_record_try_new_valid_with_identity() {
    let schema = make_schema_with_identity();
    let fields = vec!["42".to_string(), "Alice".to_string()];
    let result = Record::try_new(42, fields, schema);
    assert!(result.is_ok());
    let record = result.unwrap();
    assert_eq!(record.id(), 42);
    assert_eq!(record.fields().len(), 2);
    assert_eq!(record.at(0), Some("42"));
    assert_eq!(record.field("name"), Some("Alice"));
    assert_eq!(record.field("missing"), None);
}

#[test]
fn test_record_try_new_valid_without_identity() {
    let schema = make_schema_without_identity();
    let fields = vec!["Alice".to_string(), "Admin".to_string()];
    let result = Record::try_new(1, fields, schema);
    assert!(result.is_ok());
}

#[test]
fn test_record_try_new_length_mismatch() {
    let schema = make_schema_with_identity();
    let fields = vec!["42".to_string()];
    let result = Record::try_new(42, fields, schema);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::RecordLengthMismatch {
            expected: 2,
            found: 1
        }
    ));
}

#[test]
fn test_record_try_new_identity_mismatch_value() {
    let schema = make_schema_with_identity();
    let fields = vec!["99".to_string(), "Alice".to_string()];
    let result = Record::try_new(42, fields, schema);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), Error::IdentityMismatch { id: 42, column_value } if column_value == "99")
    );
}

#[test]
fn test_record_try_new_invalid_integer_field() {
    let schema = make_schema_with_identity();
    let fields = vec!["not_a_number".to_string(), "Alice".to_string()];
    let result = Record::try_new(42, fields, schema);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            Error::FieldTypeMismatch { ref column, ref actual, .. }
                if column == "id" && actual == "not_a_number"
        ),
        "expected FieldTypeMismatch, got {err:?}"
    );
}
