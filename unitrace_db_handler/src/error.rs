use core::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    EmptyKey,
    KeyTooShort { min: usize, got: usize },
    EmptySchema,
    EmptyColumnName,
    InvalidColumnName(String),
    ColumnNameTooLong { max: usize, got: usize },
    DuplicateColumn(String),
    MultipleIdentityColumns { count: usize },
    ColumnNotFound(String),
    EmptyKeyword,
    KeywordTooShort { min: usize, got: usize },
    EmptyDataVersion,
    InvalidBuiltAt,
    InvalidChecksumHex,
    DatabaseNotFound(String),
    MetadataNotFound(String),
    Sqlite(String),
    Io(String),
    Json(String),
    Csv(String),
    SchemaVersionMismatch { expected: u32, found: u32 },
    ChecksumMismatch { expected: String, found: String },
    CipherUnavailable,
    InvalidKey,
    MissingCsvColumn(String),
    RecordLengthMismatch { expected: usize, found: usize },
    IdentityMismatch { id: i64, column_value: String },
    ArithmeticOverflow,
    InvalidConfig(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => f.write_str("encryption key must not be empty"),
            Self::KeyTooShort { min, got } => {
                write!(
                    f,
                    "encryption key too short: minimum {min} characters, got {got}"
                )
            }
            Self::EmptySchema => f.write_str("schema must contain at least one column"),
            Self::EmptyColumnName => f.write_str("column name must not be empty"),
            Self::InvalidColumnName(n) => write!(
                f,
                "invalid column name: {n:?} (must start with ASCII letter or underscore, followed by ASCII letters, digits, or underscores)"
            ),
            Self::ColumnNameTooLong { max, got } => {
                write!(f, "column name too long: max {max} bytes, got {got}")
            }
            Self::DuplicateColumn(n) => write!(f, "duplicate column: {n}"),
            Self::MultipleIdentityColumns { count } => {
                write!(
                    f,
                    "schema must have at most one identity column, found {count}"
                )
            }
            Self::ColumnNotFound(n) => write!(f, "column not found: {n}"),
            Self::EmptyKeyword => f.write_str("search keyword must not be empty"),
            Self::KeywordTooShort { min, got } => {
                write!(f, "keyword too short: minimum {min}, got {got}")
            }
            Self::EmptyDataVersion => f.write_str("data version must not be empty"),
            Self::InvalidBuiltAt => {
                f.write_str("built_at must be a non-empty ISO-like timestamp string")
            }
            Self::InvalidChecksumHex => {
                f.write_str("checksum must be exactly 64 hexadecimal characters")
            }
            Self::DatabaseNotFound(p) => write!(f, "database not found: {p}"),
            Self::MetadataNotFound(p) => write!(f, "metadata not found: {p}"),
            Self::Sqlite(e) => write!(f, "sqlite: {e}"),
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Json(e) => write!(f, "json: {e}"),
            Self::Csv(e) => write!(f, "csv: {e}"),
            Self::SchemaVersionMismatch { expected, found } => {
                write!(
                    f,
                    "schema version mismatch: expected {expected}, found {found}"
                )
            }
            Self::ChecksumMismatch { expected, found } => {
                write!(f, "checksum mismatch: expected {expected}, found {found}")
            }
            Self::CipherUnavailable => f.write_str("SQLCipher is not available"),
            Self::InvalidKey => f.write_str("invalid key or corrupted database"),
            Self::MissingCsvColumn(c) => write!(f, "missing CSV column: {c}"),
            Self::RecordLengthMismatch { expected, found } => {
                write!(
                    f,
                    "record length mismatch: expected {expected}, found {found}"
                )
            }
            Self::IdentityMismatch { id, column_value } => {
                write!(
                    f,
                    "record id {id} does not match identity column value {column_value:?}"
                )
            }
            Self::ArithmeticOverflow => f.write_str("arithmetic overflow"),
            Self::InvalidConfig(c) => write!(f, "invalid configuration: {c}"),
        }
    }
}

impl core::error::Error for Error {}
