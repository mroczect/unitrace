#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use zeroize as _;

use unitrace_db_handler::{
    ColumnDef, ColumnKind, Error, Schema, SearchMode, SearchQuery, SearchResult,
};

fn make_valid_schema() -> Arc<Schema> {
    let col = ColumnDef::try_new("id".to_string(), ColumnKind::Text).unwrap();
    Arc::new(Schema::try_new(vec![col]).unwrap())
}

#[test]
fn test_search_query_try_new_valid() {
    let result = SearchQuery::try_new(
        "keyword".to_string(),
        SearchMode::Normal,
        Some(vec!["col1".to_string()]),
        Some(10),
    );
    assert!(result.is_ok());
    let query = result.unwrap();
    assert_eq!(query.keyword(), "keyword");
    assert_eq!(query.mode(), SearchMode::Normal);
    assert_eq!(query.limit(), Some(10));
}

#[test]
fn test_search_query_try_new_empty_keyword() {
    let result = SearchQuery::try_new(String::new(), SearchMode::Normal, None, None);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptyKeyword));
}

#[test]
fn test_search_query_try_new_empty_columns_list() {
    let result = SearchQuery::try_new(
        "keyword".to_string(),
        SearchMode::Normal,
        Some(vec![]),
        None,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidConfig(_)));
}

#[test]
fn test_search_query_try_new_zero_limit() {
    let result = SearchQuery::try_new("keyword".to_string(), SearchMode::Normal, None, Some(0));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidConfig(_)));
}

#[test]
fn test_search_result_getters() {
    let schema = make_valid_schema();
    let result = SearchResult::new(schema.clone(), vec![], 5, 2, 100, 15);
    assert_eq!(result.schema(), &schema);
    assert_eq!(result.records().len(), 0);
    assert_eq!(result.total_matches(), 5);
    assert_eq!(result.searched_columns(), 2);
    assert_eq!(result.searched_rows(), 100);
    assert_eq!(result.elapsed_ms(), 15);
}
