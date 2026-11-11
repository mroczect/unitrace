#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use csv as _;
use hex as _;
use rusqlite as _;
use serde as _;
use serde_json as _;
use sha2 as _;

use tempfile::tempdir;

use unitrace_db_core::signature::{
    SIGNATURE_LEN, combine_hash, resolve_public_key, sign_bytes, sign_files, verify_bytes,
    verify_files,
};
use unitrace_db_handler::Error;

const SEED_A: [u8; 32] = [0x11; 32];
const SEED_B: [u8; 32] = [0x22; 32];

fn signing_key(seed: [u8; 32]) -> ed25519_dalek::SigningKey {
    ed25519_dalek::SigningKey::from_bytes(&seed)
}

#[test]
fn signature_len_is_64() {
    assert_eq!(SIGNATURE_LEN, 64);
}

#[test]
fn sign_verify_roundtrip() {
    let sk = signing_key(SEED_A);
    let pk = sk.verifying_key();
    let db = b"hello db bytes";
    let meta = b"{}";
    let sig = sign_bytes(&sk, db, meta);
    assert_eq!(sig.len(), SIGNATURE_LEN);
    verify_bytes(&pk, db, meta, &sig).expect("valid signature should verify");
}

#[test]
fn different_signing_keys_produce_different_signatures() {
    let db = b"data";
    let meta = b"{}";
    let sig_a = sign_bytes(&signing_key(SEED_A), db, meta);
    let sig_b = sign_bytes(&signing_key(SEED_B), db, meta);
    assert_ne!(sig_a, sig_b);
}

#[test]
fn verify_with_wrong_public_key_fails() {
    let db = b"data";
    let meta = b"{}";
    let sig = sign_bytes(&signing_key(SEED_A), db, meta);
    let wrong_pk = signing_key(SEED_B).verifying_key();
    let result = verify_bytes(&wrong_pk, db, meta, &sig);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn tampered_db_fails_verification() {
    let sk = signing_key(SEED_A);
    let pk = sk.verifying_key();
    let db = b"original db";
    let meta = b"{}";
    let sig = sign_bytes(&sk, db, meta);
    let tampered = b"tampered db";
    let result = verify_bytes(&pk, tampered, meta, &sig);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn tampered_meta_fails_verification() {
    let sk = signing_key(SEED_A);
    let pk = sk.verifying_key();
    let db = b"db";
    let meta = b"{\"v\":1}";
    let sig = sign_bytes(&sk, db, meta);
    let tampered = b"{\"v\":2}";
    let result = verify_bytes(&pk, db, tampered, &sig);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn tampered_signature_fails_verification() {
    let sk = signing_key(SEED_A);
    let pk = sk.verifying_key();
    let db = b"db";
    let meta = b"{}";
    let mut sig = sign_bytes(&sk, db, meta);
    if let Some(b) = sig.get_mut(0) {
        *b ^= 0xFF;
    }
    let result = verify_bytes(&pk, db, meta, &sig);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn empty_db_and_meta_sign_verify() {
    let sk = signing_key(SEED_A);
    let pk = sk.verifying_key();
    let sig = sign_bytes(&sk, b"", b"");
    verify_bytes(&pk, b"", b"", &sig).unwrap();
}

#[test]
fn combine_hash_is_deterministic() {
    let h1 = combine_hash(b"abc", b"def");
    let h2 = combine_hash(b"abc", b"def");
    assert_eq!(h1, h2);
}

#[test]
fn combine_hash_db_and_meta_order_matters() {
    let h1 = combine_hash(b"ab", b"c");
    let h2 = combine_hash(b"a", b"bc");
    assert_ne!(h1, h2);
}

#[test]
fn combine_hash_different_inputs_differ() {
    let h1 = combine_hash(b"db1", b"meta");
    let h2 = combine_hash(b"db2", b"meta");
    assert_ne!(h1, h2);
}

#[test]
fn sign_files_and_verify_files_roundtrip() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let meta_path = dir.path().join("data.meta.json");
    let sig_path = dir.path().join("data.sig");
    std::fs::write(&db_path, b"db content").unwrap();
    std::fs::write(&meta_path, b"{\"v\":1}").unwrap();

    let sk = signing_key(SEED_A);
    let pk = sk.verifying_key();
    let sig = sign_files(&sk, &db_path, &meta_path).unwrap();
    std::fs::write(&sig_path, sig).unwrap();

    verify_files(&pk, &db_path, &meta_path, &sig_path).unwrap();
}

#[test]
fn verify_files_fails_when_sig_wrong_size() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("data.db");
    let meta_path = dir.path().join("data.meta.json");
    let sig_path = dir.path().join("data.sig");
    std::fs::write(&db_path, b"db").unwrap();
    std::fs::write(&meta_path, b"{}").unwrap();
    std::fs::write(&sig_path, b"short").unwrap();

    let pk = signing_key(SEED_A).verifying_key();
    let result = verify_files(&pk, &db_path, &meta_path, &sig_path);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn verify_files_fails_when_db_missing() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("does-not-exist.db");
    let meta_path = dir.path().join("data.meta.json");
    let sig_path = dir.path().join("data.sig");
    std::fs::write(&meta_path, b"{}").unwrap();
    std::fs::write(&sig_path, [0u8; 64]).unwrap();

    let pk = signing_key(SEED_A).verifying_key();
    let result = verify_files(&pk, &db_path, &meta_path, &sig_path);
    assert!(matches!(result, Err(Error::Io(_))));
}

#[test]
#[ignore = "requires placeholder public_key.hex; enable after embedding real key"]
fn resolve_public_key_returns_none_for_placeholder() {
    let result = resolve_public_key();
    assert!(
        result.is_none(),
        "embedded public key is expected to be all-zero placeholder"
    );
}

#[test]
fn combine_hash_distinguishes_empty_prefix() {
    let h1 = combine_hash(b"", b"abc");
    let h2 = combine_hash(b"abc", b"");
    assert_ne!(h1, h2);
}
