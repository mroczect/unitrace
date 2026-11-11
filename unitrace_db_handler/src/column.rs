use crate::error::Error;

pub const MAX_COLUMN_NAME_LEN: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnKind {
    Text,
    Integer,
    Decimal,
    Date,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    name: String,
    kind: ColumnKind,
    searchable: bool,
    normalize: bool,
    identity: bool,
}

impl ColumnKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Date => "date",
            Self::Boolean => "boolean",
        }
    }
}

impl ColumnDef {
    pub fn try_new(name: String, kind: ColumnKind) -> Result<Self, Error> {
        validate_column_name(&name)?;
        Ok(Self {
            name,
            kind,
            searchable: false,
            normalize: false,
            identity: false,
        })
    }

    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub const fn kind(&self) -> ColumnKind {
        self.kind
    }

    #[must_use]
    pub const fn is_searchable(&self) -> bool {
        self.searchable
    }

    #[must_use]
    pub const fn is_normalized(&self) -> bool {
        self.normalize
    }

    #[must_use]
    pub const fn is_identity(&self) -> bool {
        self.identity
    }

    #[must_use]
    pub const fn with_searchable(mut self) -> Self {
        self.searchable = true;
        self
    }

    #[must_use]
    pub const fn with_normalize(mut self) -> Self {
        self.normalize = true;
        self
    }

    #[must_use]
    pub const fn with_identity(mut self) -> Self {
        self.identity = true;
        self
    }
}

fn validate_column_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(Error::EmptyColumnName);
    }
    if name.len() > MAX_COLUMN_NAME_LEN {
        return Err(Error::ColumnNameTooLong {
            max: MAX_COLUMN_NAME_LEN,
            got: name.len(),
        });
    }
    let mut chars = name.chars();
    let first = chars.next().ok_or(Error::EmptyColumnName)?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(Error::InvalidColumnName(name.to_string()));
    }
    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return Err(Error::InvalidColumnName(name.to_string()));
        }
    }
    Ok(())
}
