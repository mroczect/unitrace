use alloc::sync::Arc;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::newtype::EncryptionKey;
use crate::schema::Schema;

#[derive(Debug)]
pub struct BuildConfig {
    schema: Arc<Schema>,
    db_path: PathBuf,
    metadata_path: PathBuf,
    csv_path: PathBuf,
    key: EncryptionKey,
    data_version: String,
}

impl BuildConfig {
    pub fn try_new(
        schema: Arc<Schema>,
        db_path: PathBuf,
        metadata_path: PathBuf,
        csv_path: PathBuf,
        key: EncryptionKey,
        data_version: String,
    ) -> Result<Self, Error> {
        if db_path.as_os_str().is_empty() {
            return Err(Error::InvalidConfig(
                "db_path must not be empty".to_string(),
            ));
        }
        if metadata_path.as_os_str().is_empty() {
            return Err(Error::InvalidConfig(
                "metadata_path must not be empty".to_string(),
            ));
        }
        if csv_path.as_os_str().is_empty() {
            return Err(Error::InvalidConfig(
                "csv_path must not be empty".to_string(),
            ));
        }
        if data_version.is_empty() {
            return Err(Error::EmptyDataVersion);
        }
        Ok(Self {
            schema,
            db_path,
            metadata_path,
            csv_path,
            key,
            data_version,
        })
    }

    #[must_use]
    pub const fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    #[must_use]
    pub fn db_path(&self) -> &Path {
        self.db_path.as_path()
    }

    #[must_use]
    pub fn metadata_path(&self) -> &Path {
        self.metadata_path.as_path()
    }

    #[must_use]
    pub fn csv_path(&self) -> &Path {
        self.csv_path.as_path()
    }

    #[must_use]
    pub const fn key(&self) -> &EncryptionKey {
        &self.key
    }

    #[must_use]
    pub const fn data_version(&self) -> &str {
        self.data_version.as_str()
    }
}

#[derive(Debug)]
pub struct OpenConfig {
    db_path: PathBuf,
    metadata_path: PathBuf,
    key: EncryptionKey,
}

impl OpenConfig {
    pub fn try_new(
        db_path: PathBuf,
        metadata_path: PathBuf,
        key: EncryptionKey,
    ) -> Result<Self, Error> {
        if db_path.as_os_str().is_empty() {
            return Err(Error::InvalidConfig(
                "db_path must not be empty".to_string(),
            ));
        }
        if metadata_path.as_os_str().is_empty() {
            return Err(Error::InvalidConfig(
                "metadata_path must not be empty".to_string(),
            ));
        }
        Ok(Self {
            db_path,
            metadata_path,
            key,
        })
    }

    #[must_use]
    pub fn db_path(&self) -> &Path {
        self.db_path.as_path()
    }

    #[must_use]
    pub fn metadata_path(&self) -> &Path {
        self.metadata_path.as_path()
    }

    #[must_use]
    pub const fn key(&self) -> &EncryptionKey {
        &self.key
    }
}
