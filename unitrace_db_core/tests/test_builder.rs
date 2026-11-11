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

use unitrace_db_core::builder::DefaultBuilder;
use unitrace_db_core::checksum::checksum_hex;
use unitrace_db_core::csv_import::TABLE_NAME;
use unitrace_db_core::metadata_io::{read_metadata, verify_checksum};
use unitrace_db_core::sqlcipher::open_encrypted;
use unitrace_db_handler::traits::DatabaseBuilder;
use unitrace_db_handler::{BuildConfig, ColumnDef, ColumnKind, EncryptionKey, Error, Schema};

const KEY: &str = "supersecretkey123";

fn sample_schema() -> Arc<Schema> {
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

fn write_csv(dir: &std::path::Path, content: &str) {
    std::fs::write(dir.join("data.csv"), content).unwrap();
}

fn make_config(dir: &std::path::Path) -> BuildConfig {
    BuildConfig::try_new(
        sample_schema(),
        dir.join("data.db"),
        dir.join("data.meta.json"),
        dir.join("data.csv"),
        key(),
        "v1.0".to_string(),
    )
    .unwrap()
}

const CSV_ONE: &str = "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
                       OD-001,Alice Santoso,JM0000555156,24\n";

const CSV_TWO: &str = "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
                       OD-001,Alice Santoso,JM0000555156,24\n\
                       OD-002,Budi Foo,JM02E9999999,36\n";

#[test]
fn build_creates_db_and_metadata() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let builder = DefaultBuilder::new();
    let metadata = builder.build(&config).unwrap();

    assert!(config.db_path().exists());
    assert!(config.metadata_path().exists());
    assert_eq!(metadata.row_count(), 1);
    assert_eq!(metadata.data_version(), "v1.0");
    assert_eq!(metadata.schema_version(), 1);
    assert_eq!(metadata.checksum_sha256().len(), 64);
    assert!(metadata.built_at().contains('-'));
}

#[test]
fn build_output_db_is_encrypted() {
    let dir = tempdir().unwrap();
    write_csv(
        dir.path(),
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,MAGICPLAINTEXT,JM0000555156,24\n",
    );
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config).unwrap();

    let bytes = std::fs::read(config.db_path()).unwrap();
    let needle = b"MAGICPLAINTEXT";
    assert!(
        !bytes.windows(needle.len()).any(|w| w == needle),
        "plaintext leaked into encrypted file"
    );
}

#[test]
fn build_can_be_reopened_with_same_key() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config).unwrap();

    let conn = open_encrypted(config.db_path(), &key()).unwrap();
    let count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {TABLE_NAME}"), [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn build_rejects_wrong_key_on_reopen() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config).unwrap();

    let wrong = EncryptionKey::try_new("differentpass5678".to_string()).unwrap();
    let result = open_encrypted(config.db_path(), &wrong);
    assert!(matches!(result.unwrap_err(), Error::InvalidKey));
}

#[test]
fn build_overwrites_existing_files() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config).unwrap();
    assert_eq!(
        read_metadata(config.metadata_path()).unwrap().row_count(),
        1
    );

    write_csv(dir.path(), CSV_TWO);
    let _ = DefaultBuilder::new().build(&config).unwrap();
    assert_eq!(
        read_metadata(config.metadata_path()).unwrap().row_count(),
        2
    );
}

#[test]
fn build_removes_stale_metadata_on_start() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    std::fs::write(config.metadata_path(), b"stale garbage").unwrap();

    let _ = DefaultBuilder::new().build(&config).unwrap();

    let metadata = read_metadata(config.metadata_path()).unwrap();
    assert_eq!(metadata.row_count(), 1);
}

#[test]
fn build_with_missing_csv_returns_error() {
    let dir = tempdir().unwrap();
    let config = make_config(dir.path());

    let result = DefaultBuilder::new().build(&config);
    assert!(result.is_err());
}

#[test]
fn build_with_missing_csv_cleans_up_partial_state() {
    let dir = tempdir().unwrap();
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config);

    assert!(!config.db_path().exists(), "orphan db should be removed");
    assert!(
        !config.metadata_path().exists(),
        "orphan metadata should be removed"
    );
}

#[test]
fn build_with_bad_csv_returns_error() {
    let dir = tempdir().unwrap();
    write_csv(
        dir.path(),
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,Alice,JM0000555156,not_a_number\n",
    );
    let config = make_config(dir.path());

    let result = DefaultBuilder::new().build(&config);
    assert!(matches!(
        result.unwrap_err(),
        Error::FieldTypeMismatch { .. }
    ));
}

#[test]
fn built_metadata_checksum_matches_db_file() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config).unwrap();

    let metadata = read_metadata(config.metadata_path()).unwrap();
    verify_checksum(config.db_path(), &metadata).unwrap();
}

#[test]
fn built_metadata_checksum_is_sha256_of_file_bytes() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config).unwrap();

    let bytes = std::fs::read(config.db_path()).unwrap();
    let expected = checksum_hex(&bytes);
    let metadata = read_metadata(config.metadata_path()).unwrap();
    assert_eq!(metadata.checksum_sha256(), expected);
}

#[test]
fn rebuild_produces_different_checksum_same_row_count() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let m1 = DefaultBuilder::new().build(&config).unwrap();
    let m2 = DefaultBuilder::new().build(&config).unwrap();

    assert_eq!(m1.row_count(), m2.row_count());
    assert_ne!(
        m1.checksum_sha256(),
        m2.checksum_sha256(),
        "SQLCipher should use a fresh random salt per build"
    );
}

#[test]
fn build_with_custom_normalizer() {
    use unitrace_db_core::normalize::DefaultNormalizer;

    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let builder = DefaultBuilder::with_normalizer(Box::new(DefaultNormalizer::new()));
    let metadata = builder.build(&config).unwrap();
    assert_eq!(metadata.row_count(), 1);
}

#[test]
fn built_db_norm_all_is_populated() {
    let dir = tempdir().unwrap();
    write_csv(dir.path(), CSV_ONE);
    let config = make_config(dir.path());

    let _ = DefaultBuilder::new().build(&config).unwrap();

    let conn = open_encrypted(config.db_path(), &key()).unwrap();
    let norm: String = conn
        .query_row(&format!("SELECT _norm_all FROM {TABLE_NAME}"), [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(norm.contains("ALICESANTOSO"));
    assert!(norm.contains("JM0000555156"));
}
