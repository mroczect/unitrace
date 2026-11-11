use core::fmt;

use zeroize::Zeroize;

use crate::error::Error;

pub const MIN_KEY_LENGTH: usize = 12;

pub struct EncryptionKey(String);

impl EncryptionKey {
    pub fn try_new(value: String) -> Result<Self, Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(Error::EmptyKey);
        }
        let effective_len = trimmed.chars().count();
        if effective_len < MIN_KEY_LENGTH {
            return Err(Error::KeyTooShort {
                min: MIN_KEY_LENGTH,
                got: effective_len,
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EncryptionKey(***)")
    }
}

impl Drop for EncryptionKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
