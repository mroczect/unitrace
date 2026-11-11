use alloc::sync::Arc;

use crate::error::Error;
use crate::schema::{SCHEMA_VERSION, Schema};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    schema_version: u32,
    data_version: String,
    row_count: u64,
    checksum_sha256: String,
    built_at: String,
    schema: Arc<Schema>,
}

impl Metadata {
    pub fn try_new(
        schema_version: u32,
        data_version: String,
        row_count: u64,
        checksum_sha256: String,
        built_at: String,
        schema: Arc<Schema>,
    ) -> Result<Self, Error> {
        if schema_version != SCHEMA_VERSION {
            return Err(Error::SchemaVersionMismatch {
                expected: SCHEMA_VERSION,
                found: schema_version,
            });
        }
        if data_version.is_empty() {
            return Err(Error::EmptyDataVersion);
        }
        if !is_valid_sha256_hex(&checksum_sha256) {
            return Err(Error::InvalidChecksumHex);
        }
        if !is_valid_built_at(&built_at) {
            return Err(Error::InvalidBuiltAt);
        }
        Ok(Self {
            schema_version,
            data_version,
            row_count,
            checksum_sha256,
            built_at,
            schema,
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    #[must_use]
    pub const fn data_version(&self) -> &str {
        self.data_version.as_str()
    }

    #[must_use]
    pub const fn row_count(&self) -> u64 {
        self.row_count
    }

    #[must_use]
    pub const fn checksum_sha256(&self) -> &str {
        self.checksum_sha256.as_str()
    }

    #[must_use]
    pub const fn built_at(&self) -> &str {
        self.built_at.as_str()
    }

    #[must_use]
    pub const fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }
}

fn is_valid_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_valid_built_at(s: &str) -> bool {
    !s.is_empty() && s.bytes().any(|b| b.is_ascii_digit()) && s.bytes().any(|b| b == b'-')
}
