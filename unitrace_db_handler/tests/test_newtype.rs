#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use zeroize as _;

use unitrace_db_handler::{EncryptionKey, Error, newtype};

#[test]
fn test_encryption_key_try_new_valid() {
    let key_str = "a".repeat(newtype::MIN_KEY_LENGTH);
    let result = EncryptionKey::try_new(key_str.clone());
    assert!(result.is_ok());
    let key = result.unwrap();
    assert_eq!(key.as_str(), key_str);
}

#[test]
fn test_encryption_key_try_new_empty() {
    let result = EncryptionKey::try_new("   ".to_string());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EmptyKey));
}

#[test]
fn test_encryption_key_try_new_too_short() {
    let short_len = newtype::MIN_KEY_LENGTH.saturating_sub(1);
    let short_key = "a".repeat(short_len);
    let result = EncryptionKey::try_new(short_key);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), Error::KeyTooShort { min, got } if min == newtype::MIN_KEY_LENGTH && got == short_len)
    );
}

#[test]
fn test_encryption_key_debug_masking() {
    let key = EncryptionKey::try_new("supersecretkey123".to_string()).unwrap();
    let debug_str = format!("{:?}", key);
    assert_eq!(debug_str, "EncryptionKey(***)");
}
