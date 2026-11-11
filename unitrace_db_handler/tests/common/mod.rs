use std::path::PathBuf;
use std::sync::Arc;
use unitrace_db_handler::{BuildConfig, ColumnDef, ColumnKind, EncryptionKey, Schema};

pub fn make_valid_column(name: &str) -> ColumnDef {
    ColumnDef::try_new(name.to_string(), ColumnKind::Text).expect("Valid column name")
}

pub fn make_valid_schema() -> Arc<Schema> {
    let cols = vec![
        make_valid_column("id").with_identity(),
        make_valid_column("name").with_searchable(),
    ];
    Arc::new(Schema::try_new(cols).expect("Valid schema"))
}

pub fn make_valid_encryption_key() -> EncryptionKey {
    EncryptionKey::try_new("supersecretkey123".to_string()).expect("Valid encryption key")
}

pub fn make_valid_build_config() -> BuildConfig {
    let schema = make_valid_schema();
    let key = make_valid_encryption_key();
    BuildConfig::try_new(
        schema,
        PathBuf::from("/tmp/test.db"),
        PathBuf::from("/tmp/test.meta.json"),
        PathBuf::from("/tmp/test.csv"),
        key,
        "v1.0.0".to_string(),
    ).expect("Valid build config")
}
