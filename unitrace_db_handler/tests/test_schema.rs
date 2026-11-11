#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use zeroize as _;

use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Schema};

fn make_col(name: &str) -> ColumnDef {
    ColumnDef::try_new(name.to_string(), ColumnKind::Text).unwrap()
}

#[test]
fn test_schema_try_new_valid() {
    let cols = vec![make_col("id"), make_col("name")];
    let result = Schema::try_new(cols);
    assert!(result.is_ok());
    let schema = result.unwrap();
    assert_eq!(schema.len(), 2);
    assert!(!schema.is_empty());
}

#[test]
fn test_schema_try_new_empty() {
    let result = Schema::try_new(vec![]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptySchema));
}

#[test]
fn test_schema_try_new_duplicate_case_insensitive() {
    let cols = vec![make_col("Name"), make_col("name")];
    let result = Schema::try_new(cols);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::DuplicateColumn(_)));
}

#[test]
fn test_schema_try_new_multiple_identity() {
    let cols = vec![
        make_col("id1").with_identity(),
        make_col("id2").with_identity(),
    ];
    let result = Schema::try_new(cols);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::MultipleIdentityColumns { count: 2 }
    ));
}

#[test]
fn test_schema_index_of_and_column() {
    let cols = vec![make_col("ID"), make_col("Name")];
    let schema = Schema::try_new(cols).unwrap();

    assert_eq!(schema.index_of("id"), Some(0));
    assert_eq!(schema.index_of("ID"), Some(0));
    assert_eq!(schema.index_of("missing"), None);

    assert!(schema.column(0).is_some());
    assert_eq!(schema.column(0).unwrap().name(), "ID");
    assert!(schema.column(10).is_none());
}

#[test]
fn test_schema_indices() {
    let cols = vec![
        make_col("id").with_identity(),
        make_col("name").with_searchable(),
        make_col("email").with_searchable(),
    ];
    let schema = Schema::try_new(cols).unwrap();

    assert_eq!(schema.identity_index(), Some(0));
    assert_eq!(schema.identity_indices(), vec![0]);
    assert_eq!(schema.searchable_indices(), vec![1, 2]);
}
