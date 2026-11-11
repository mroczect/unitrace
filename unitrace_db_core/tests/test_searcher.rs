#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use csv as _;
use hex as _;
use rusqlite as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile::tempdir;

use ed25519_dalek as _;
use ed25519_dalek as _;
use ed25519_dalek as _;
use ed25519_dalek as _;
use unitrace_db_core::builder::DefaultBuilder;
use unitrace_db_core::searcher::DefaultSearcher;
use unitrace_db_handler::traits::{DatabaseBuilder, DatabaseReader};
use unitrace_db_handler::{
    BuildConfig, ColumnDef, ColumnKind, EncryptionKey, Error, OpenConfig, Schema, SearchMode,
    SearchQuery,
};

const KEY: &str = "supersecretkey123";

const CSV: &str = "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
                   OD-001,Alice Santoso,JM02E1618594,24\n\
                   OD-002,Budi Hartono,JM02E9999999,36\n\
                   OD-003,Charlie Wijaya,AB12CD34567,12\n";

fn schema() -> Arc<Schema> {
    let cols = vec![
        ColumnDef::try_new("no_perjanjian".to_string(), ColumnKind::Text).unwrap(),
        ColumnDef::try_new("nama_nasabah".to_string(), ColumnKind::Text)
            .unwrap()
            .with_normalize()
            .with_searchable(),
        ColumnDef::try_new("no_mesin".to_string(), ColumnKind::Text)
            .unwrap()
            .with_normalize()
            .with_searchable(),
        ColumnDef::try_new("tenor".to_string(), ColumnKind::Integer).unwrap(),
    ];
    Arc::new(Schema::try_new(cols).unwrap())
}

fn key() -> EncryptionKey {
    EncryptionKey::try_new(KEY.to_string()).unwrap()
}

fn build_fixture(dir: &std::path::Path) -> OpenConfig {
    std::fs::write(dir.join("data.csv"), CSV).unwrap();
    let build = BuildConfig::try_new(
        schema(),
        dir.join("data.db"),
        dir.join("data.meta.json"),
        dir.join("data.csv"),
        key(),
        "v1.0".to_string(),
    )
    .unwrap();
    let _ = DefaultBuilder::new().build(&build).unwrap();

    OpenConfig::try_new(dir.join("data.db"), dir.join("data.meta.json"), key()).unwrap()
}

fn open_searcher(config: &OpenConfig) -> DefaultSearcher {
    DefaultSearcher::open(config).unwrap()
}

fn normal_query(kw: &str) -> SearchQuery {
    SearchQuery::try_new(kw.to_string(), SearchMode::Normal, None, None).unwrap()
}

fn raw_query(kw: &str) -> SearchQuery {
    SearchQuery::try_new(kw.to_string(), SearchMode::Raw, None, None).unwrap()
}

fn exact_query(kw: &str) -> SearchQuery {
    SearchQuery::try_new(kw.to_string(), SearchMode::Exact, None, None).unwrap()
}

#[test]
fn open_succeeds_with_valid_files() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);
    assert_eq!(searcher.row_count().unwrap(), 3);
}

#[test]
fn open_fails_with_missing_db() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    std::fs::remove_file(config.db_path()).unwrap();
    let result = DefaultSearcher::open(&config);
    assert!(matches!(result.unwrap_err(), Error::DatabaseNotFound(_)));
}

#[test]
fn open_fails_with_missing_metadata() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    std::fs::remove_file(config.metadata_path()).unwrap();
    let result = DefaultSearcher::open(&config);
    assert!(matches!(result.unwrap_err(), Error::MetadataNotFound(_)));
}

#[test]
fn open_fails_with_wrong_key() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let wrong = OpenConfig::try_new(
        config.db_path().to_path_buf(),
        config.metadata_path().to_path_buf(),
        EncryptionKey::try_new("differentpass5678".to_string()).unwrap(),
    )
    .unwrap();
    let result = DefaultSearcher::open(&wrong);
    assert!(matches!(result.unwrap_err(), Error::InvalidKey));
}

#[test]
fn open_fails_on_tampered_db() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());

    let bytes = std::fs::read(config.db_path()).unwrap();
    let mut tampered = bytes;
    if let Some(byte) = tampered.get_mut(100) {
        *byte ^= 0xFF;
    }
    std::fs::write(config.db_path(), tampered).unwrap();

    let result = DefaultSearcher::open(&config);
    assert!(matches!(
        result.unwrap_err(),
        Error::ChecksumMismatch { .. }
    ));
}

#[test]
fn schema_is_exposed() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);
    assert_eq!(searcher.schema().len(), 4);
}

#[test]
fn row_count_returns_total_rows() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);
    assert_eq!(searcher.row_count().unwrap(), 3);
}

#[test]
fn normal_search_finds_substring_in_no_mesin() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let result = searcher.search(&normal_query("1618594")).unwrap();
    assert_eq!(result.total_matches(), 1);
    assert_eq!(result.records().len(), 1);
    let record = result.records().first().unwrap();
    assert_eq!(record.field("nama_nasabah"), Some("Alice Santoso"));
}

#[test]
fn normal_search_is_case_and_space_insensitive() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let variants = ["alice", "ALICE", " alice ", "Ali ce", "j m 0 2 e"];
    for kw in variants {
        let result = searcher.search(&normal_query(kw)).unwrap();
        assert!(
            result.total_matches() >= 1,
            "keyword {kw:?} did not match, got {}",
            result.total_matches()
        );
    }
}

#[test]
fn normal_search_finds_partial_without_letters() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let result = searcher.search(&normal_query("34567")).unwrap();
    assert_eq!(result.total_matches(), 1);
}

#[test]
fn normal_search_with_empty_after_normalize_returns_zero() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let result = searcher.search(&normal_query("   ")).unwrap();
    assert_eq!(result.total_matches(), 0);
    assert_eq!(result.records().len(), 0);
}

#[test]
fn raw_search_is_case_sensitive() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let upper = searcher.search(&raw_query("JM02E1618594")).unwrap();
    assert_eq!(upper.total_matches(), 1);

    let lower = searcher.search(&raw_query("jm02e1618594")).unwrap();
    assert_eq!(lower.total_matches(), 0);
}

#[test]
fn exact_search_requires_full_value() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let exact = searcher.search(&exact_query("Alice Santoso")).unwrap();
    assert_eq!(exact.total_matches(), 1);

    let partial = searcher.search(&exact_query("Alice")).unwrap();
    assert_eq!(partial.total_matches(), 0);
}

#[test]
fn search_respects_limit() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let query = SearchQuery::try_new("a".to_string(), SearchMode::Normal, None, Some(1)).unwrap();
    let result = searcher.search(&query).unwrap();
    assert_eq!(result.records().len(), 1);
    assert!(result.total_matches() > 1);
}

#[test]
fn search_with_column_restriction() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let query = SearchQuery::try_new(
        "24".to_string(),
        SearchMode::Raw,
        Some(vec!["tenor".to_string()]),
        None,
    )
    .unwrap();
    let result = searcher.search(&query).unwrap();
    assert_eq!(result.total_matches(), 1);
}

#[test]
fn search_with_unknown_column_returns_error() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let query = SearchQuery::try_new(
        "x".to_string(),
        SearchMode::Normal,
        Some(vec!["nonexistent".to_string()]),
        None,
    )
    .unwrap();
    let result = searcher.search(&query);
    assert!(matches!(result.unwrap_err(), Error::ColumnNotFound(_)));
}

#[test]
fn get_by_id_returns_matching_record() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let record = searcher.get_by_id(1).unwrap().unwrap();
    assert_eq!(record.id(), 1);
    assert_eq!(record.field("nama_nasabah"), Some("Alice Santoso"));
}

#[test]
fn get_by_id_returns_none_for_missing() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let result = searcher.get_by_id(9999).unwrap();
    assert!(result.is_none());
}

#[test]
fn search_result_includes_searched_stats() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let result = searcher.search(&normal_query("1618594")).unwrap();
    assert_eq!(result.searched_rows(), 3);
    assert_eq!(result.searched_columns(), 4);
}

#[test]
fn search_result_carries_schema() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let result = searcher.search(&normal_query("1618594")).unwrap();
    assert_eq!(result.schema().len(), 4);
}

#[test]
fn search_returns_zero_for_no_match() {
    let dir = tempdir().unwrap();
    let config = build_fixture(dir.path());
    let searcher = open_searcher(&config);

    let result = searcher.search(&normal_query("zzz999zzz")).unwrap();
    assert_eq!(result.total_matches(), 0);
    assert!(result.records().is_empty());
}
