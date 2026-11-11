use std::collections::HashSet;

use crate::column::{ColumnDef, ColumnKind};
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

    pub fn validate_field_types(&self, fields: &[String]) -> Result<(), Error> {
        if fields.len() != self.columns.len() {
            return Err(Error::RecordLengthMismatch {
                expected: self.columns.len(),
                found: fields.len(),
            });
        }
        for (col, value) in self.columns.iter().zip(fields) {
            validate_field_value(col.name(), col.kind(), value)?;
        }
        Ok(())
    }
}

fn validate_field_value(column: &str, kind: ColumnKind, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Ok(());
    }
    let ok = match kind {
        ColumnKind::Text => true,
        ColumnKind::Integer => value.parse::<i64>().is_ok(),
        ColumnKind::Decimal => value.parse::<f64>().is_ok(),
        ColumnKind::Date => is_valid_date(value),
        ColumnKind::Boolean => is_valid_boolean(value),
    };
    if ok {
        Ok(())
    } else {
        Err(Error::FieldTypeMismatch {
            column: column.to_string(),
            expected: kind.as_str(),
            actual: value.to_string(),
        })
    }
}

fn is_valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    let mut digits = 0usize;
    let mut sep_count = 0usize;
    let mut sep_char = 0u8;
    for &b in bytes {
        if b.is_ascii_digit() {
            digits = digits.saturating_add(1);
        } else if b == b'-' || b == b'/' {
            sep_count = sep_count.saturating_add(1);
            if sep_count == 1 {
                sep_char = b;
            } else if b != sep_char {
                return false;
            }
        } else {
            return false;
        }
    }
    digits == 8 && sep_count == 2
}

fn is_valid_boolean(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("false")
        || value == "1"
        || value == "0"
}
