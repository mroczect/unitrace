#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Schema};

fn make_col(name: &str) -> ColumnDef {
    ColumnDef::try_new(name.to_string(), ColumnKind::Text).unwrap()
}

#[test]
fn schema_with_single_column_is_valid() {
    let schema = Schema::try_new(vec![make_col("id")]).unwrap();
    assert_eq!(schema.len(), 1);
}

#[test]
fn schema_with_no_identity_is_valid() {
    let schema = Schema::try_new(vec![make_col("a"), make_col("b")]).unwrap();
    assert_eq!(schema.identity_index(), None);
    assert!(schema.identity_indices().is_empty());
}

#[test]
fn schema_with_one_identity_is_valid() {
    let schema = Schema::try_new(vec![make_col("id").with_identity(), make_col("name")]).unwrap();
    assert_eq!(schema.identity_index(), Some(0));
}

#[test]
fn schema_rejects_unicode_column_name() {
    let result = ColumnDef::try_new("naïve".to_string(), ColumnKind::Text);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidColumnName(_)));
}

#[test]
fn schema_accepts_leading_underscore() {
    let schema = Schema::try_new(vec![make_col("_internal")]).unwrap();
    assert_eq!(schema.len(), 1);
}

#[test]
fn schema_rejects_column_with_space() {
    let result = ColumnDef::try_new("first name".to_string(), ColumnKind::Text);
    assert!(result.is_err());
}

#[test]
fn schema_accepts_arc_clone() {
    let schema = Arc::new(Schema::try_new(vec![make_col("id")]).unwrap());
    let cloned = Arc::clone(&schema);
    assert_eq!(cloned.len(), schema.len());
}
