#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use csv as _;
use ed25519_dalek::SigningKey;
use hex as _;
use rusqlite as _;
use serde as _;
use serde_json as _;
use sha2 as _;

use tempfile::tempdir;

use unitrace_db_core::package::Package;
use unitrace_db_core::signature::{sign_bytes, verify_bytes};
use unitrace_db_handler::Error;

const HEADER_LEN: usize = 8 + 2 + 2 + 8 + 8 + 8;

fn sample_sig(seed: u8) -> [u8; 64] {
    let mut s = [0u8; 64];
    let mut i = 0u8;
    for b in &mut s {
        *b = seed.wrapping_add(i);
        i = i.wrapping_add(1);
    }
    s
}

#[test]
fn roundtrip_bytes_preserves_all_fields() {
    let db = b"db content here".to_vec();
    let meta = b"{\"v\":1}".to_vec();
    let sig = sample_sig(0x42);

    let pkg = Package::new(db.clone(), meta.clone(), sig);
    let bytes = pkg.to_bytes();
    let restored = Package::from_bytes(&bytes).unwrap();

    assert_eq!(restored.db, db);
    assert_eq!(restored.meta, meta);
    assert_eq!(restored.signature, sig);
}

#[test]
fn write_read_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.upkg");
    let db = b"db".to_vec();
    let meta = b"meta".to_vec();
    let sig = sample_sig(0x11);

    let pkg = Package::new(db.clone(), meta.clone(), sig);
    pkg.write(&path).unwrap();

    let read_back = Package::read(&path).unwrap();
    assert_eq!(read_back.db, db);
    assert_eq!(read_back.meta, meta);
    assert_eq!(read_back.signature, sig);
}

#[test]
fn empty_db_and_meta_roundtrip() {
    let pkg = Package::new(Vec::new(), Vec::new(), [0u8; 64]);
    let restored = Package::from_bytes(&pkg.to_bytes()).unwrap();
    assert!(restored.db.is_empty());
    assert!(restored.meta.is_empty());
}

#[test]
fn read_missing_file_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nope.upkg");
    let result = Package::read(&path);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn wrong_magic_rejected() {
    let mut bytes = Package::new(b"x".to_vec(), b"y".to_vec(), [0u8; 64]).to_bytes();
    if let Some(b) = bytes.get_mut(0) {
        *b = b'X';
    }
    let result = Package::from_bytes(&bytes);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn truncated_header_rejected() {
    let result = Package::from_bytes(&[0u8; 10]);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn wrong_version_rejected() {
    let mut bytes = Package::new(b"x".to_vec(), b"y".to_vec(), [0u8; 64]).to_bytes();
    if let Some(b) = bytes.get_mut(8) {
        *b = 99;
    }
    let result = Package::from_bytes(&bytes);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn truncated_body_rejected() {
    let pkg = Package::new(b"long content".to_vec(), b"meta".to_vec(), [0u8; 64]);
    let mut bytes = pkg.to_bytes();
    bytes.truncate(bytes.len().saturating_sub(10));
    let result = Package::from_bytes(&bytes);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn extra_bytes_rejected() {
    let pkg = Package::new(b"db".to_vec(), b"meta".to_vec(), [0u8; 64]);
    let mut bytes = pkg.to_bytes();
    bytes.push(0xFF);
    let result = Package::from_bytes(&bytes);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn package_with_signature_is_verifiable() {
    let sk = SigningKey::from_bytes(&[0x33; 32]);
    let pk = sk.verifying_key();

    let db = b"sqlcipher bytes".to_vec();
    let meta = b"{\"version\":\"v1\"}".to_vec();
    let sig = sign_bytes(&sk, &db, &meta);

    let bytes = Package::new(db, meta, sig).to_bytes();
    let parsed = Package::from_bytes(&bytes).unwrap();

    verify_bytes(&pk, &parsed.db, &parsed.meta, &parsed.signature).unwrap();
}

#[test]
fn tampered_package_fails_signature_verification() {
    let sk = SigningKey::from_bytes(&[0x33; 32]);
    let pk = sk.verifying_key();

    let db = b"sqlcipher bytes".to_vec();
    let meta = b"{\"version\":\"v1\"}".to_vec();
    let sig = sign_bytes(&sk, &db, &meta);

    let mut pkg_bytes = Package::new(db, meta, sig).to_bytes();
    if let Some(b) = pkg_bytes.get_mut(HEADER_LEN) {
        *b ^= 0xFF;
    }

    let parsed = Package::from_bytes(&pkg_bytes).unwrap();
    let result = verify_bytes(&pk, &parsed.db, &parsed.meta, &parsed.signature);
    assert!(matches!(result, Err(Error::PackageInvalid(_))));
}

#[test]
fn multiple_packages_are_independent() {
    let dir = tempdir().unwrap();
    let p1 = dir.path().join("a.upkg");
    let p2 = dir.path().join("b.upkg");

    Package::new(b"aaa".to_vec(), b"a-meta".to_vec(), sample_sig(0x01))
        .write(&p1)
        .unwrap();
    Package::new(b"bbb".to_vec(), b"b-meta".to_vec(), sample_sig(0x02))
        .write(&p2)
        .unwrap();

    let r1 = Package::read(&p1).unwrap();
    let r2 = Package::read(&p2).unwrap();
    assert_eq!(r1.db, b"aaa");
    assert_eq!(r2.db, b"bbb");
}
