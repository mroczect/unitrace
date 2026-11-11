use alloc::sync::Arc;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use unitrace_db_handler::traits::Checksum;
use unitrace_db_handler::{ColumnDef, ColumnKind, Error, Metadata, Schema};

use crate::checksum::Sha256Checksum;

#[derive(Serialize, Deserialize)]
struct MetadataJson {
    schema_version: u32,
    data_version: String,
    row_count: u64,
    checksum_sha256: String,
    built_at: String,
    schema: SchemaJson,
}

#[derive(Serialize, Deserialize)]
struct SchemaJson {
    columns: Vec<ColumnJson>,
}

#[derive(Serialize, Deserialize)]
struct ColumnJson {
    name: String,
    kind: ColumnKindJson,
    searchable: bool,
    normalize: bool,
    identity: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ColumnKindJson {
    Text,
    Integer,
    Decimal,
    Date,
    Boolean,
}

impl From<ColumnKind> for ColumnKindJson {
    fn from(kind: ColumnKind) -> Self {
        match kind {
            ColumnKind::Text => Self::Text,
            ColumnKind::Integer => Self::Integer,
            ColumnKind::Decimal => Self::Decimal,
            ColumnKind::Date => Self::Date,
            ColumnKind::Boolean => Self::Boolean,
        }
    }
}

impl From<ColumnKindJson> for ColumnKind {
    fn from(kind: ColumnKindJson) -> Self {
        match kind {
            ColumnKindJson::Text => Self::Text,
            ColumnKindJson::Integer => Self::Integer,
            ColumnKindJson::Decimal => Self::Decimal,
            ColumnKindJson::Date => Self::Date,
            ColumnKindJson::Boolean => Self::Boolean,
        }
    }
}

impl From<&Metadata> for MetadataJson {
    fn from(meta: &Metadata) -> Self {
        let columns = meta
            .schema()
            .columns()
            .iter()
            .map(|c| ColumnJson {
                name: c.name().to_string(),
                kind: c.kind().into(),
                searchable: c.is_searchable(),
                normalize: c.is_normalized(),
                identity: c.is_identity(),
            })
            .collect();
        Self {
            schema_version: meta.schema_version(),
            data_version: meta.data_version().to_string(),
            row_count: meta.row_count(),
            checksum_sha256: meta.checksum_sha256().to_string(),
            built_at: meta.built_at().to_string(),
            schema: SchemaJson { columns },
        }
    }
}

impl TryFrom<MetadataJson> for Metadata {
    type Error = Error;

    fn try_from(json: MetadataJson) -> Result<Self, Error> {
        let mut columns = Vec::with_capacity(json.schema.columns.len());
        for c in json.schema.columns {
            let mut def = ColumnDef::try_new(c.name, c.kind.into())?;
            if c.searchable {
                def = def.with_searchable();
            }
            if c.normalize {
                def = def.with_normalize();
            }
            if c.identity {
                def = def.with_identity();
            }
            columns.push(def);
        }
        let schema = Arc::new(Schema::try_new(columns)?);
        Self::try_new(
            json.schema_version,
            json.data_version,
            json.row_count,
            json.checksum_sha256,
            json.built_at,
            schema,
        )
    }
}

pub fn write_metadata(path: &Path, metadata: &Metadata) -> Result<(), Error> {
    let json = MetadataJson::from(metadata);
    let contents = serde_json::to_string_pretty(&json).map_err(|e| Error::Json(e.to_string()))?;
    let tmp_path = tmp_path_for(path);
    fs::write(&tmp_path, contents.as_bytes()).map_err(|e| Error::Io(e.to_string()))?;
    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(Error::Io(e.to_string()));
    }
    Ok(())
}

pub fn read_metadata(path: &Path) -> Result<Metadata, Error> {
    let contents = fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::MetadataNotFound(path.display().to_string())
        } else {
            Error::Io(e.to_string())
        }
    })?;
    let json: MetadataJson =
        serde_json::from_str(&contents).map_err(|e| Error::Json(e.to_string()))?;
    Metadata::try_from(json)
}

pub fn verify_checksum(db_path: &Path, metadata: &Metadata) -> Result<(), Error> {
    let data = fs::read(db_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::DatabaseNotFound(db_path.display().to_string())
        } else {
            Error::Io(e.to_string())
        }
    })?;
    let actual = hex::encode(Sha256Checksum::new().checksum(&data));
    let expected = metadata.checksum_sha256();
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(Error::ChecksumMismatch {
            expected: expected.to_string(),
            found: actual,
        });
    }
    Ok(())
}

fn tmp_path_for(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let ext = tmp.extension().map_or_else(
        || OsString::from("tmp"),
        |e| {
            let mut new_ext = e.to_os_string();
            new_ext.push(".tmp");
            new_ext
        },
    );
    let _ = tmp.set_extension(ext);
    tmp
}
