#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use zeroize as _;

use unitrace_db_handler::{Error, SearchMode, SearchQuery};

#[test]
fn query_keyword_whitespace_only_is_accepted_by_handler() {
    let result = SearchQuery::try_new("   ".to_string(), SearchMode::Normal, None, None);
    assert!(result.is_ok());
    let q = result.unwrap();
    assert_eq!(q.keyword(), "   ");
}

#[test]
fn query_limit_one_is_valid() {
    let result = SearchQuery::try_new("x".to_string(), SearchMode::Raw, None, Some(1));
    assert!(result.is_ok());
}

#[test]
fn query_all_modes_construct() {
    for mode in [SearchMode::Normal, SearchMode::Raw, SearchMode::Exact] {
        let q = SearchQuery::try_new("x".to_string(), mode, None, None).unwrap();
        assert_eq!(q.mode(), mode);
    }
}

#[test]
fn query_columns_with_single_entry_is_valid() {
    let result = SearchQuery::try_new(
        "x".to_string(),
        SearchMode::Normal,
        Some(vec!["col".to_string()]),
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn query_empty_keyword_and_bad_columns_reports_keyword_first() {
    let result = SearchQuery::try_new(String::new(), SearchMode::Normal, Some(vec![]), Some(0));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptyKeyword));
}
