#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use core::mem::size_of;

use csv as _;
use rusqlite as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use ed25519_dalek as _;
use ed25519_dalek as _;
use unitrace_db_core::checksum::{Sha256Checksum, checksum_hex};
use unitrace_db_handler::traits::Checksum;

const SHA256_EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const SHA256_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[test]
fn empty_input_matches_known_hash() {
    assert_eq!(checksum_hex(b""), SHA256_EMPTY);
}

#[test]
fn abc_matches_known_hash() {
    assert_eq!(checksum_hex(b"abc"), SHA256_ABC);
}

#[test]
fn returns_32_raw_bytes() {
    let bytes = Sha256Checksum::new().checksum(b"data");
    assert_eq!(bytes.len(), 32);
}

#[test]
fn different_inputs_produce_different_hashes() {
    let h1 = checksum_hex(b"abc");
    let h2 = checksum_hex(b"abd");
    assert_ne!(h1, h2);
}

#[test]
fn single_byte_mutation_changes_hash() {
    let h1 = checksum_hex(b"password");
    let h2 = checksum_hex(b"passw0rd");
    assert_ne!(h1, h2);
}

#[test]
fn hex_is_lowercase_only() {
    let h = checksum_hex(b"data");
    assert!(h.chars().all(|c| !c.is_ascii_uppercase()));
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hex_has_64_characters() {
    assert_eq!(checksum_hex(b"data").len(), 64);
    assert_eq!(checksum_hex(b"").len(), 64);
}

#[test]
fn hex_has_64_characters_for_all_byte_lengths() {
    for len in [0, 1, 31, 32, 33, 63, 64, 65, 127, 128, 255, 256, 1024] {
        let input = vec![0u8; len];
        assert_eq!(checksum_hex(&input).len(), 64);
    }
}

#[test]
fn raw_and_hex_agree() {
    let raw = Sha256Checksum::new().checksum(b"data");
    let expected_hex = hex::encode(raw);
    assert_eq!(checksum_hex(b"data"), expected_hex);
}

#[test]
fn deterministic() {
    let input = b"JM0000555156";
    let h1 = checksum_hex(input);
    let h2 = checksum_hex(input);
    let h3 = checksum_hex(input);
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

#[test]
fn handles_all_byte_values() {
    let input: Vec<u8> = (0..=255).collect();
    let h = checksum_hex(&input);
    assert_eq!(h.len(), 64);
}

#[test]
fn large_input_does_not_overflow() {
    let input = vec![0xABu8; 1024 * 1024];
    let h = checksum_hex(&input);
    assert_eq!(h.len(), 64);
}

#[test]
fn checksum_is_zero_sized() {
    assert_eq!(size_of::<Sha256Checksum>(), 0);
}
