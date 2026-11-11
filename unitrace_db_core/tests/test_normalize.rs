#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use core::mem::size_of;

use csv as _;
use hex as _;
use rusqlite as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use ed25519_dalek as _;
use ed25519_dalek as _;
use unitrace_db_core::normalize::DefaultNormalizer;
use unitrace_db_handler::traits::Normalizer;

fn norm(input: &str) -> String {
    DefaultNormalizer::new().normalize(input)
}

#[test]
fn strips_surrounding_spaces() {
    assert_eq!(norm(" jm02e 1618594 "), "JM02E1618594");
}

#[test]
fn uppercases_lowercase() {
    assert_eq!(norm("jm02e"), "JM02E");
}

#[test]
fn strips_punctuation() {
    assert_eq!(norm("JM02E-1618594"), "JM02E1618594");
}

#[test]
fn strips_unicode_letters() {
    assert_eq!(norm("José"), "JOS");
}

#[test]
fn strips_emoji() {
    assert_eq!(norm("🚗ABC🚗"), "ABC");
}

#[test]
fn strips_cjk() {
    assert_eq!(norm("日本語123"), "123");
}

#[test]
fn strips_non_breaking_space() {
    assert_eq!(norm("ABC\u{00A0}123"), "ABC123");
}

#[test]
fn strips_tab_and_newline() {
    assert_eq!(norm("AB\tCD\nEF"), "ABCDEF");
}

#[test]
fn strips_null_byte() {
    assert_eq!(norm("AB\0CD"), "ABCD");
}

#[test]
fn strips_control_characters() {
    assert_eq!(norm("A\u{0001}B\u{0007}C"), "ABC");
}

#[test]
fn empty_input_stays_empty() {
    assert_eq!(norm(""), "");
}

#[test]
fn whitespace_only_becomes_empty() {
    assert_eq!(norm("   \t\n  "), "");
}

#[test]
fn punctuation_only_becomes_empty() {
    assert_eq!(norm("---...///"), "");
}

#[test]
fn indonesian_plate() {
    assert_eq!(norm("B 1234 XYZ"), "B1234XYZ");
}

#[test]
fn idempotent() {
    let first = norm("  jm02e-1618594  ");
    let second = norm(first.as_str());
    assert_eq!(first, second);
}

#[test]
fn idempotent_for_mixed_input() {
    for input in [
        "",
        " ",
        "abc",
        "ABC",
        "a b c",
        "A-B-C",
        "1234567890",
        "!@#$%",
        "🚗xyz🚗",
        "\0\0\0",
        "B 1234 XYZ",
        "JM02E1618594",
        "JM02E1618594",
    ] {
        let once = norm(input);
        let twice = norm(once.as_str());
        assert_eq!(once, twice, "not idempotent for {input:?}");
    }
}

#[test]
fn preserves_digits_only() {
    assert_eq!(norm("1618594"), "1618594");
}

#[test]
fn mixed_case_and_punctuation() {
    assert_eq!(norm("Od-25.f0001422"), "OD25F0001422");
}

#[test]
fn all_ascii_bytes_never_panic() {
    for byte in 0u8..=127 {
        let s = char::from(byte).to_string();
        let _ = norm(&s);
    }
}

#[test]
fn long_input_does_not_overflow() {
    let long = "a".repeat(100_000);
    let result = norm(&long);
    assert_eq!(result.len(), 100_000);
    assert!(result.chars().all(|c| c == 'A'));
}

#[test]
fn long_input_with_whitespace_and_punctuation() {
    let long = "ab- cd ".repeat(10_000);
    let result = norm(&long);
    assert!(result.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(result.len(), 40_000);
}

#[test]
fn normalizer_is_zero_sized() {
    assert_eq!(size_of::<DefaultNormalizer>(), 0);
}

#[test]
fn normalize_and_search_agree() {
    let database_value = "JM02E1618594";
    let user_inputs = [
        "JM02E1618594",
        "JM02E1618594",
        " 1618594",
        "1618594 ",
        "1618594",
        "jm02e",
        "JM02E",
        "JM-02E-1618594",
        "  JM02E 1618594  ",
    ];
    let normalized_db = norm(database_value);
    for input in user_inputs {
        let normalized_input = norm(input);
        assert!(
            normalized_db.contains(&normalized_input),
            "expected {database_value:?} to contain {input:?} after normalization"
        );
    }
}
