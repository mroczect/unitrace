#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Record, Schema};

fn schema_with_identity() -> Arc<Schema> {
    let cols = vec![
        ColumnDef::try_new("id".to_string(), ColumnKind::Integer)
            .unwrap()
            .with_identity(),
        ColumnDef::try_new("name".to_string(), ColumnKind::Text).unwrap(),
    ];
    Arc::new(Schema::try_new(cols).unwrap())
}

#[test]
fn record_with_negative_id_matches_column() {
    let schema = schema_with_identity();
    let fields = vec!["-1".to_string(), "Alice".to_string()];
    let record = Record::try_new(-1, fields, schema).unwrap();
    assert_eq!(record.id(), -1);
    assert_eq!(record.at(0), Some("-1"));
}

#[test]
fn record_with_leading_zero_id_rejected() {
    let schema = schema_with_identity();
    let fields = vec!["007".to_string(), "Alice".to_string()];
    let result = Record::try_new(7, fields, schema);
    assert!(result.is_err());
}

#[test]
fn record_empty_string_identity_rejected() {
    let schema = schema_with_identity();
    let fields = vec![String::new(), "Alice".to_string()];
    let result = Record::try_new(0, fields, schema);
    assert!(result.is_err());
}
