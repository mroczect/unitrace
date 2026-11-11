use zeroize as _;

use unitrace_db_handler::Error;

#[test]
fn test_error_display_empty_key() {
    let err = Error::EmptyKey;
    assert_eq!(format!("{}", err), "encryption key must not be empty");
}

#[test]
fn test_error_display_key_too_short() {
    let err = Error::KeyTooShort { min: 12, got: 5 };
    assert_eq!(
        format!("{}", err),
        "encryption key too short: minimum 12 characters, got 5"
    );
}

#[test]
fn test_error_display_invalid_column_name() {
    let err = Error::InvalidColumnName("1bad".to_string());
    let display = format!("{}", err);
    assert!(display.contains("1bad"));
    assert!(display.contains("invalid column name"));
}

#[test]
fn test_error_display_duplicate_column() {
    let err = Error::DuplicateColumn("name".to_string());
    assert_eq!(format!("{}", err), "duplicate column: name");
}

#[test]
fn test_error_display_schema_version_mismatch() {
    let err = Error::SchemaVersionMismatch {
        expected: 1,
        found: 2,
    };
    assert_eq!(
        format!("{}", err),
        "schema version mismatch: expected 1, found 2"
    );
}

#[test]
fn test_error_display_record_length_mismatch() {
    let err = Error::RecordLengthMismatch {
        expected: 3,
        found: 2,
    };
    assert_eq!(
        format!("{}", err),
        "record length mismatch: expected 3, found 2"
    );
}
