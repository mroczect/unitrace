pub use unitrace_db_handler::traits::{
    Checksum, DatabaseBuilder, DatabaseReader, Integrity, Normalizer,
};
pub use unitrace_db_handler::{
    BuildConfig, ColumnDef, ColumnKind, EncryptionKey, Error, Metadata, OpenConfig, Record, Schema,
    SearchMode, SearchQuery, SearchResult,
};

pub use unitrace_db_core::builder::DefaultBuilder;
pub use unitrace_db_core::checksum::Sha256Checksum;
pub use unitrace_db_core::integrity::DefaultIntegrity;
pub use unitrace_db_core::normalize::DefaultNormalizer;
pub use unitrace_db_core::searcher::DefaultSearcher;

pub use unitrace_db_core::package::Package;
pub use unitrace_db_core::signature::{
    EMBEDDED_PUBLIC_KEY_HEX, PUBLIC_KEY_HEX_LEN, SIGNATURE_LEN, combine_hash, resolve_public_key,
    sign_bytes, sign_files, verify_bytes, verify_files,
};

pub use unitrace_db_gui_core::DefaultGuiController;
pub use unitrace_db_gui_handler::{GuiController, NotFoundProof, RowViewModel, ViewState};

#[cfg(test)]
use ed25519_dalek as _;
