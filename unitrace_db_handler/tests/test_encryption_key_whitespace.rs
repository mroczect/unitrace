#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use zeroize as _;

use unitrace_db_handler::{EncryptionKey, Error};

#[test]
fn rejects_padded_short_key() {
    let padded = "   a   b   c  ".to_string();
    let result = EncryptionKey::try_new(padded);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::KeyTooShort { min: 12, got: 9 }
    ));
}

#[test]
fn accepts_trimmed_long_key() {
    let padded = "   supersecretkey123   ".to_string();
    let result = EncryptionKey::try_new(padded);
    assert!(result.is_ok());
    let key = result.unwrap();
    assert_eq!(key.as_str(), "   supersecretkey123   ");
}

#[test]
fn rejects_all_whitespace_long_key() {
    let ws = " ".repeat(64);
    let result = EncryptionKey::try_new(ws);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptyKey));
}
