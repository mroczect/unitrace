use alloc::string::String;

use sha2::{Digest, Sha256};
use unitrace_db_handler::traits::Checksum;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Checksum;

impl Sha256Checksum {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Checksum for Sha256Checksum {
    fn checksum(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

#[must_use]
pub fn checksum_hex(data: &[u8]) -> String {
    hex::encode(Sha256Checksum::new().checksum(data))
}
