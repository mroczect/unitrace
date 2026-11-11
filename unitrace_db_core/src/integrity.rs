use std::path::Path;

use unitrace_db_handler::traits::Integrity;
use unitrace_db_handler::{Error, Metadata};

use crate::metadata_io::verify_checksum;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultIntegrity;

impl DefaultIntegrity {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Integrity for DefaultIntegrity {
    fn verify(&self, db_path: &Path, metadata: &Metadata) -> Result<(), Error> {
        verify_checksum(db_path, metadata)
    }
}
