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
use unitrace_db_core::csv_import::{NORM_ALL_COLUMN, TABLE_NAME, create_table, import_csv};
use unitrace_db_core::normalize::DefaultNormalizer;
use unitrace_db_core::sqlcipher::open_encrypted;
use unitrace_db_handler::{ColumnDef, ColumnKind, EncryptionKey, Error, Schema};

fn key() -> EncryptionKey {
    EncryptionKey::try_new("supersecretkey123".to_string()).unwrap()
}

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
        ColumnDef::try_new("tenor".to_string(), ColumnKind::Integer)
            .unwrap()
            .with_normalize(),
    ];
    Arc::new(Schema::try_new(cols).unwrap())
}

#[test]
fn imports_small_csv() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,Alice Santoso,JM0000555156,24\n\
         OD-002,Budi Foo,JM02E9999999,36\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
    assert_eq!(count, 2);

    let rows: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {TABLE_NAME}"), [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(rows, 2);
}

#[test]
fn norm_all_contains_normalized_concat() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,Alice Santoso,JM0000555156,24\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
    assert_eq!(count, 1);

    let norm: String = conn
        .query_row(
            &format!("SELECT {NORM_ALL_COLUMN} FROM {TABLE_NAME}"),
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(norm.contains("ALICESANTOSO"));
    assert!(norm.contains("JM0000555156"));
    assert!(norm.contains("24"));
    assert!(
        !norm.contains("OD001"),
        "no_perjanjian should not be included"
    );
}

#[test]
fn imports_with_shuffled_column_order() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "tenor,no_mesin,no_perjanjian,nama_nasabah\n\
         24,JM0000555156,OD-001,Alice\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
    assert_eq!(count, 1);

    let name: String = conn
        .query_row(&format!("SELECT nama_nasabah FROM {TABLE_NAME}"), [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(name, "Alice");
}

#[test]
fn imports_with_extra_columns_ignored() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,extra_col,nama_nasabah,no_mesin,tenor,another_extra\n\
         OD-001,ignore_me,Alice,JM0000555156,24,also_ignore\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn missing_column_returns_error() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,tenor\n\
         OD-001,Alice,24\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let result = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new());
    assert!(matches!(result.unwrap_err(), Error::MissingCsvColumn(_)));
}

#[test]
fn empty_csv_returns_zero() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(&csv_path, "no_perjanjian,nama_nasabah,no_mesin,tenor\n").unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn invalid_integer_field_returns_error() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,Alice,JM0000555156,not_a_number\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let result = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new());
    assert!(matches!(
        result.unwrap_err(),
        Error::FieldTypeMismatch { .. }
    ));
}

#[test]
fn invalid_type_rolls_back_entire_import() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,Alice,JM0000555156,24\n\
         OD-002,Bob,JM02E9999999,not_a_number\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let result = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new());
    assert!(result.is_err());

    let rows: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {TABLE_NAME}"), [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(rows, 0, "transaction should have rolled back all rows");
}

#[test]
fn unicode_names_are_preserved() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,José Müller,JM0000555156,24\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
    assert_eq!(count, 1);

    let name: String = conn
        .query_row(&format!("SELECT nama_nasabah FROM {TABLE_NAME}"), [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(name, "José Müller");

    let norm: String = conn
        .query_row(
            &format!("SELECT {NORM_ALL_COLUMN} FROM {TABLE_NAME}"),
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        norm.contains("JOSMLLER"),
        "unicode letters stripped in norm, got: {norm}"
    );
}

#[test]
fn create_table_is_idempotent() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    create_table(&conn, &schema).unwrap();
}

#[test]
fn imports_and_persists_across_reopen() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,Alice,JM0000555156,24\n",
    )
    .unwrap();

    {
        let conn = open_encrypted(&db_path, &key()).unwrap();
        let schema = sample_schema();
        create_table(&conn, &schema).unwrap();
        let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
        assert_eq!(count, 1);
    }

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {TABLE_NAME}"), [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn empty_fields_are_stored_as_empty_strings() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let csv_path = dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "no_perjanjian,nama_nasabah,no_mesin,tenor\n\
         OD-001,,JM0000555156,\n",
    )
    .unwrap();

    let conn = open_encrypted(&db_path, &key()).unwrap();
    let schema = sample_schema();
    create_table(&conn, &schema).unwrap();
    let count = import_csv(&conn, &csv_path, &schema, &DefaultNormalizer::new()).unwrap();
    assert_eq!(count, 1);

    let (name, tenor): (String, String) = conn
        .query_row(
            &format!("SELECT nama_nasabah, tenor FROM {TABLE_NAME}"),
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(name, "");
    assert_eq!(tenor, "");
}
