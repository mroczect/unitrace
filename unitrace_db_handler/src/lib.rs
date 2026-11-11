extern crate alloc;

pub mod column;
pub mod config;
pub mod error;
pub mod metadata;
pub mod newtype;
pub mod query;
pub mod record;
pub mod schema;
pub mod traits;

pub use column::{ColumnDef, ColumnKind, MAX_COLUMN_NAME_LEN};
pub use config::{BuildConfig, OpenConfig};
pub use error::Error;
pub use metadata::Metadata;
pub use newtype::EncryptionKey;
pub use query::{SearchMode, SearchQuery, SearchResult};
pub use record::Record;
pub use schema::Schema;
