#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

extern crate alloc;

use alloc::sync::Arc;

use std::path::PathBuf;

use zeroize as _;

use unitrace_db_handler::{
    BuildConfig, ColumnDef, ColumnKind, EncryptionKey, Error, OpenConfig, Schema,
};

fn make_valid_schema() -> Arc<Schema> {
    let col = ColumnDef::try_new("id".to_string(), ColumnKind::Text).unwrap();
    Arc::new(Schema::try_new(vec![col]).unwrap())
}

fn make_valid_key() -> EncryptionKey {
    EncryptionKey::try_new("supersecretkey123".to_string()).unwrap()
}

#[test]
fn test_build_config_try_new_valid() {
    let result = BuildConfig::try_new(
        make_valid_schema(),
        PathBuf::from("/tmp/test.db"),
        PathBuf::from("/tmp/test.meta.json"),
        PathBuf::from("/tmp/test.csv"),
        make_valid_key(),
        "v1.0".to_string(),
    );
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.db_path().to_str().unwrap(), "/tmp/test.db");
    assert_eq!(config.data_version(), "v1.0");
}

#[test]
fn test_build_config_try_new_empty_db_path() {
    let result = BuildConfig::try_new(
        make_valid_schema(),
        PathBuf::from(""),
        PathBuf::from("/tmp/test.meta.json"),
        PathBuf::from("/tmp/test.csv"),
        make_valid_key(),
        "v1.0".to_string(),
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidConfig(_)));
}

#[test]
fn test_build_config_try_new_empty_data_version() {
    let result = BuildConfig::try_new(
        make_valid_schema(),
        PathBuf::from("/tmp/test.db"),
        PathBuf::from("/tmp/test.meta.json"),
        PathBuf::from("/tmp/test.csv"),
        make_valid_key(),
        String::new(),
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptyDataVersion));
}

#[test]
fn test_open_config_try_new_valid() {
    let result = OpenConfig::try_new(
        PathBuf::from("/tmp/test.db"),
        PathBuf::from("/tmp/test.meta.json"),
        make_valid_key(),
    );
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(
        config.metadata_path().to_str().unwrap(),
        "/tmp/test.meta.json"
    );
}

#[test]
fn test_open_config_try_new_empty_metadata_path() {
    let result = OpenConfig::try_new(
        PathBuf::from("/tmp/test.db"),
        PathBuf::from(""),
        make_valid_key(),
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidConfig(_)));
}
