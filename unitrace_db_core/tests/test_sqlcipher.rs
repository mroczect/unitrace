#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use std::path::Path;

use csv as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile::tempdir;

use ed25519_dalek as _;
use ed25519_dalek as _;
use unitrace_db_core::sqlcipher::{apply_key, open_encrypted, probe_connection, verify_cipher};
use unitrace_db_handler::{EncryptionKey, Error};

fn key(s: &str) -> EncryptionKey {
    EncryptionKey::try_new(s.to_string()).unwrap()
}

fn open_ok(path: &Path, passphrase: &str) -> rusqlite::Connection {
    open_encrypted(path, &key(passphrase)).unwrap()
}

#[test]
fn creates_new_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");
    assert!(!path.exists());
    let conn = open_ok(&path, "correcthorse1234");
    drop(conn);
    assert!(path.exists());
}

#[test]
fn writes_are_persisted_across_reopen() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");

    {
        let conn = open_ok(&path, "correcthorse1234");
        conn.execute_batch("CREATE TABLE t (v TEXT); INSERT INTO t VALUES ('hello');")
            .unwrap();
    }

    {
        let conn = open_ok(&path, "correcthorse1234");
        let value: String = conn
            .query_row("SELECT v FROM t;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "hello");
    }
}

#[test]
fn wrong_key_is_rejected() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");

    {
        let conn = open_ok(&path, "correcthorse1234");
        conn.execute_batch("CREATE TABLE t (v TEXT);").unwrap();
    }

    let result = open_encrypted(&path, &key("differentpass5678"));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), Error::InvalidKey),
        "expected InvalidKey, got something else"
    );
}

#[test]
fn cipher_is_active() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");
    let conn = open_ok(&path, "correcthorse1234");
    let version: String = conn
        .query_row("PRAGMA cipher_version;", [], |row| row.get(0))
        .unwrap();
    assert!(!version.trim().is_empty(), "cipher_version is empty");
    assert!(
        version.chars().any(|c| c.is_ascii_digit()),
        "cipher_version should contain a version number, got: {version:?}"
    );
}

#[test]
fn verify_cipher_succeeds_on_valid_connection() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");
    let conn = open_ok(&path, "correcthorse1234");
    assert!(verify_cipher(&conn).is_ok());
}

#[test]
fn plaintext_file_is_rejected() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");

    {
        let plain = rusqlite::Connection::open(&path).unwrap();
        plain
            .execute_batch("CREATE TABLE plain (v TEXT); INSERT INTO plain VALUES ('x');")
            .unwrap();
    }

    let result = open_encrypted(&path, &key("correcthorse1234"));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidKey));
}

#[test]
fn apply_key_is_safe_with_single_quote() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");
    let passphrase = "quote'inside\"passphrase1234";

    {
        let conn = open_ok(&path, passphrase);
        conn.execute_batch("CREATE TABLE t (v TEXT);").unwrap();
    }

    let reopened = open_encrypted(&path, &key(passphrase));
    assert!(reopened.is_ok(), "re-open with quote-containing key failed");
}

#[test]
fn apply_key_twice_is_idempotent() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");
    let conn = open_ok(&path, "correcthorse1234");
    assert!(apply_key(&conn, &key("correcthorse1234")).is_ok());
}

#[test]
fn probe_succeeds_on_fresh_encrypted_db() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");
    let conn = open_ok(&path, "correcthorse1234");
    assert!(probe_connection(&conn).is_ok());
}

#[test]
fn encrypted_file_bytes_are_not_plaintext() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.db");

    {
        let conn = open_ok(&path, "correcthorse1234");
        conn.execute_batch(
            "CREATE TABLE secrets (v TEXT); INSERT INTO secrets VALUES ('MAGICPLAINTEXTSTRING');",
        )
        .unwrap();
    }

    let bytes = std::fs::read(&path).unwrap();
    let needle = b"MAGICPLAINTEXTSTRING";
    let found = bytes.windows(needle.len()).any(|w| w == needle);
    assert!(
        !found,
        "plaintext leaked into encrypted file — SQLCipher is not encrypting"
    );
}
