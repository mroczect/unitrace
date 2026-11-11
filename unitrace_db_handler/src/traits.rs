use std::path::Path;

use crate::config::BuildConfig;
use crate::error::Error;
use crate::metadata::Metadata;
use crate::query::{SearchQuery, SearchResult};
use crate::record::Record;
use crate::schema::Schema;

pub trait DatabaseBuilder {
    fn build(&self, config: &BuildConfig) -> Result<Metadata, Error>;
}

pub trait DatabaseReader {
    fn search(&self, query: &SearchQuery) -> Result<SearchResult, Error>;
    fn get_by_id(&self, id: i64) -> Result<Option<Record>, Error>;
    fn row_count(&self) -> Result<u64, Error>;
    fn schema(&self) -> &Schema;
}

pub trait Normalizer {
    fn normalize(&self, input: &str) -> String;
}

pub trait Checksum {
    fn checksum(&self, data: &[u8]) -> [u8; 32];
}

pub trait Integrity {
    fn verify(&self, db_path: &Path, metadata: &Metadata) -> Result<(), Error>;
}
