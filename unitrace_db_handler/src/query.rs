use alloc::sync::Arc;

use crate::error::Error;
use crate::record::Record;
use crate::schema::Schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Normal,
    Raw,
    Exact,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    keyword: String,
    mode: SearchMode,
    columns: Option<Vec<String>>,
    limit: Option<usize>,
}

impl SearchQuery {
    pub fn try_new(
        keyword: String,
        mode: SearchMode,
        columns: Option<Vec<String>>,
        limit: Option<usize>,
    ) -> Result<Self, Error> {
        if keyword.is_empty() {
            return Err(Error::EmptyKeyword);
        }
        if let Some(cols) = &columns
            && cols.is_empty()
        {
            return Err(Error::InvalidConfig(
                "columns list must not be empty when provided".to_string(),
            ));
        }
        if limit == Some(0) {
            return Err(Error::InvalidConfig(
                "limit must be greater than zero when provided".to_string(),
            ));
        }
        Ok(Self {
            keyword,
            mode,
            columns,
            limit,
        })
    }

    #[must_use]
    pub const fn keyword(&self) -> &str {
        self.keyword.as_str()
    }

    #[must_use]
    pub const fn mode(&self) -> SearchMode {
        self.mode
    }

    #[must_use]
    pub fn columns(&self) -> Option<&[String]> {
        self.columns.as_deref()
    }

    #[must_use]
    pub const fn limit(&self) -> Option<usize> {
        self.limit
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    schema: Arc<Schema>,
    records: Vec<Record>,
    total_matches: u64,
    searched_columns: u64,
    searched_rows: u64,
    elapsed_ms: u64,
}

impl SearchResult {
    #[must_use]
    pub const fn new(
        schema: Arc<Schema>,
        records: Vec<Record>,
        total_matches: u64,
        searched_columns: u64,
        searched_rows: u64,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            schema,
            records,
            total_matches,
            searched_columns,
            searched_rows,
            elapsed_ms,
        }
    }

    #[must_use]
    pub const fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    #[must_use]
    pub const fn records(&self) -> &[Record] {
        self.records.as_slice()
    }

    #[must_use]
    pub const fn total_matches(&self) -> u64 {
        self.total_matches
    }

    #[must_use]
    pub const fn searched_columns(&self) -> u64 {
        self.searched_columns
    }

    #[must_use]
    pub const fn searched_rows(&self) -> u64 {
        self.searched_rows
    }

    #[must_use]
    pub const fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }
}
