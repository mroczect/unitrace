#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Error, MAX_COLUMN_NAME_LEN};

#[test]
fn test_column_def_try_new_valid() {
    let result = ColumnDef::try_new("valid_name".to_string(), ColumnKind::Text);
    assert!(result.is_ok(), "Expected Ok for valid column name");
    let col = result.unwrap();
    assert_eq!(col.name(), "valid_name");
    assert_eq!(col.kind(), ColumnKind::Text);
    assert!(!col.is_searchable());
    assert!(!col.is_normalized());
    assert!(!col.is_identity());
}

#[test]
fn test_column_def_try_new_empty() {
    let result = ColumnDef::try_new(String::new(), ColumnKind::Text);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptyColumnName));
}

#[test]
fn test_column_def_try_new_too_long() {
    let long_name = "a".repeat(MAX_COLUMN_NAME_LEN.saturating_add(1));
    let result = ColumnDef::try_new(long_name, ColumnKind::Text);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::ColumnNameTooLong { .. }
    ));
}

#[test]
fn test_column_def_try_new_invalid_start() {
    let result = ColumnDef::try_new("1invalid".to_string(), ColumnKind::Text);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidColumnName(_)));
}

#[test]
fn test_column_def_try_new_invalid_char() {
    let result = ColumnDef::try_new("invalid-name".to_string(), ColumnKind::Text);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidColumnName(_)));
}

#[test]
fn test_column_def_builders() {
    let col = ColumnDef::try_new("id".to_string(), ColumnKind::Integer).unwrap();
    let enhanced = col.with_searchable().with_normalize().with_identity();
    assert!(enhanced.is_searchable());
    assert!(enhanced.is_normalized());
    assert!(enhanced.is_identity());
}
