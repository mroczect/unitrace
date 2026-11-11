#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Schema};

fn schema_kind(name: &str, kind: ColumnKind) -> Schema {
    let col = ColumnDef::try_new(name.to_string(), kind).unwrap();
    Schema::try_new(vec![col]).unwrap()
}

#[test]
fn length_mismatch_is_rejected() {
    let schema = schema_kind("id", ColumnKind::Text);
    let result = schema.validate_field_types(&["a".to_string(), "b".to_string()]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::RecordLengthMismatch {
            expected: 1,
            found: 2
        }
    ));
}

#[test]
fn text_accepts_any_non_empty_value() {
    let schema = schema_kind("name", ColumnKind::Text);
    assert!(schema.validate_field_types(&["Alice".to_string()]).is_ok());
    assert!(
        schema
            .validate_field_types(&["Budi Santoso".to_string()])
            .is_ok()
    );
    assert!(schema.validate_field_types(&["héllo".to_string()]).is_ok());
    assert!(schema.validate_field_types(&["🚗".to_string()]).is_ok());
}

#[test]
fn empty_value_is_accepted_for_all_kinds() {
    for kind in [
        ColumnKind::Text,
        ColumnKind::Integer,
        ColumnKind::Decimal,
        ColumnKind::Date,
        ColumnKind::Boolean,
    ] {
        let schema = schema_kind("col", kind);
        assert!(
            schema.validate_field_types(&[String::new()]).is_ok(),
            "empty rejected for {kind:?}"
        );
    }
}

#[test]
fn whitespace_only_value_is_accepted_for_all_kinds() {
    for kind in [
        ColumnKind::Text,
        ColumnKind::Integer,
        ColumnKind::Decimal,
        ColumnKind::Date,
        ColumnKind::Boolean,
    ] {
        let schema = schema_kind("col", kind);
        assert!(
            schema.validate_field_types(&["   ".to_string()]).is_ok(),
            "whitespace rejected for {kind:?}"
        );
    }
}

#[test]
fn integer_accepts_valid_values() {
    let schema = schema_kind("n", ColumnKind::Integer);
    for value in ["0", "42", "-1", "9999999"] {
        assert!(
            schema.validate_field_types(&[value.to_string()]).is_ok(),
            "rejected valid integer {value}"
        );
    }
}

#[test]
fn integer_rejects_invalid_values() {
    let schema = schema_kind("n", ColumnKind::Integer);
    for value in ["abc", "1.5", "1,5", "١٢٣", "12a"] {
        let result = schema.validate_field_types(&[value.to_string()]);
        assert!(result.is_err(), "accepted invalid integer {value}");
        assert!(matches!(
            result.unwrap_err(),
            Error::FieldTypeMismatch { .. }
        ));
    }
}

#[test]
fn decimal_accepts_valid_values() {
    let schema = schema_kind("d", ColumnKind::Decimal);
    for value in ["0", "1.5", "-3.14", "1000000.0"] {
        assert!(
            schema.validate_field_types(&[value.to_string()]).is_ok(),
            "rejected valid decimal {value}"
        );
    }
}

#[test]
fn decimal_rejects_invalid_values() {
    let schema = schema_kind("d", ColumnKind::Decimal);
    for value in ["abc", "1,5", "12a"] {
        assert!(schema.validate_field_types(&[value.to_string()]).is_err());
    }
}

#[test]
fn date_accepts_valid_formats() {
    let schema = schema_kind("d", ColumnKind::Date);
    for value in ["2025-11-11", "2025/11/11", "11-11-2025", "11/11/2025"] {
        assert!(
            schema.validate_field_types(&[value.to_string()]).is_ok(),
            "rejected valid date {value}"
        );
    }
}

#[test]
fn date_rejects_invalid_formats() {
    let schema = schema_kind("d", ColumnKind::Date);
    for value in [
        "2025-11",
        "2025-11-11 ",
        "20251111",
        "abcd-ef-ghij",
        "2025.11.11",
    ] {
        assert!(
            schema.validate_field_types(&[value.to_string()]).is_err(),
            "accepted invalid date {value}"
        );
    }
}

#[test]
fn boolean_accepts_valid_values() {
    let schema = schema_kind("b", ColumnKind::Boolean);
    for value in ["true", "false", "TRUE", "FALSE", "True", "False", "1", "0"] {
        assert!(
            schema.validate_field_types(&[value.to_string()]).is_ok(),
            "rejected valid boolean {value}"
        );
    }
}

#[test]
fn boolean_rejects_invalid_values() {
    let schema = schema_kind("b", ColumnKind::Boolean);
    for value in ["yes", "no", "2", "t", "f", "tru"] {
        assert!(
            schema.validate_field_types(&[value.to_string()]).is_err(),
            "accepted invalid boolean {value}"
        );
    }
}

#[test]
fn multiple_columns_validates_each() {
    let id = ColumnDef::try_new("id".to_string(), ColumnKind::Integer).unwrap();
    let name = ColumnDef::try_new("name".to_string(), ColumnKind::Text).unwrap();
    let schema = Schema::try_new(vec![id, name]).unwrap();
    assert!(
        schema
            .validate_field_types(&["1".to_string(), "Alice".to_string()])
            .is_ok()
    );
    let result = schema.validate_field_types(&["abc".to_string(), "Alice".to_string()]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::FieldTypeMismatch { ref column, .. } if column == "id"
    ));
}
