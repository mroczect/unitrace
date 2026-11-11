use std::collections::HashSet;

use crate::column::ColumnDef;
use crate::error::Error;

pub const SCHEMA_VERSION: u32 = 1;
pub const DB_FILE_NAME: &str = "data.db";
pub const METADATA_FILE_NAME: &str = "data.meta.json";
pub const MIN_KEYWORD_LEN: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    columns: Vec<ColumnDef>,
}

impl Schema {
    pub fn try_new(columns: Vec<ColumnDef>) -> Result<Self, Error> {
        if columns.is_empty() {
            return Err(Error::EmptySchema);
        }
        let mut seen: HashSet<String> = HashSet::with_capacity(columns.len());
        let mut identity_count = 0usize;
        for col in &columns {
            let lower = col.name().to_ascii_lowercase();
            if !seen.insert(lower) {
                return Err(Error::DuplicateColumn(col.name().to_string()));
            }
            if col.is_identity() {
                identity_count = identity_count.saturating_add(1);
            }
        }
        if identity_count > 1 {
            return Err(Error::MultipleIdentityColumns {
                count: identity_count,
            });
        }
        Ok(Self { columns })
    }

    #[must_use]
    pub const fn columns(&self) -> &[ColumnDef] {
        self.columns.as_slice()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.columns.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    #[must_use]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|c| c.name().eq_ignore_ascii_case(name))
    }

    #[must_use]
    pub fn column(&self, index: usize) -> Option<&ColumnDef> {
        self.columns.get(index)
    }

    #[must_use]
    pub fn identity_index(&self) -> Option<usize> {
        self.columns.iter().position(ColumnDef::is_identity)
    }

    #[must_use]
    pub fn searchable_indices(&self) -> Vec<usize> {
        self.columns
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.is_searchable().then_some(i))
            .collect()
    }

    #[must_use]
    pub fn identity_indices(&self) -> Vec<usize> {
        self.columns
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.is_identity().then_some(i))
            .collect()
    }
}
