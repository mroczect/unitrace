use alloc::sync::Arc;
use core::fmt;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use unitrace_db_handler::traits::{DatabaseBuilder, Normalizer};
use unitrace_db_handler::{BuildConfig, Error, Metadata, schema};

use crate::checksum::checksum_hex;
use crate::csv_import::{create_table, import_csv};
use crate::metadata_io::write_metadata;
use crate::normalize::DefaultNormalizer;
use crate::sqlcipher::open_encrypted;

pub struct DefaultBuilder {
    normalizer: Box<dyn Normalizer>,
}

impl DefaultBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            normalizer: Box::new(DefaultNormalizer::new()),
        }
    }

    #[must_use]
    pub fn with_normalizer(normalizer: Box<dyn Normalizer>) -> Self {
        Self { normalizer }
    }

    fn try_build(&self, config: &BuildConfig) -> Result<Metadata, Error> {
        remove_if_exists(config.db_path())?;
        remove_if_exists(config.metadata_path())?;

        let row_count = build_database(config, self.normalizer.as_ref())?;
        let checksum = compute_file_checksum(config.db_path())?;

        let metadata = Metadata::try_new(
            schema::SCHEMA_VERSION,
            config.data_version().to_string(),
            row_count,
            checksum,
            now_iso8601(),
            Arc::clone(config.schema()),
        )?;

        write_metadata(config.metadata_path(), &metadata)?;
        Ok(metadata)
    }
}

impl Default for DefaultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DefaultBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultBuilder").finish_non_exhaustive()
    }
}

impl DatabaseBuilder for DefaultBuilder {
    fn build(&self, config: &BuildConfig) -> Result<Metadata, Error> {
        match self.try_build(config) {
            Ok(metadata) => Ok(metadata),
            Err(e) => {
                let _ = fs::remove_file(config.db_path());
                let _ = fs::remove_file(config.metadata_path());
                Err(e)
            }
        }
    }
}

fn remove_if_exists(path: &Path) -> Result<(), Error> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io(e.to_string())),
    }
}

fn build_database(config: &BuildConfig, normalizer: &dyn Normalizer) -> Result<u64, Error> {
    let conn = open_encrypted(config.db_path(), config.key())?;
    create_table(&conn, config.schema())?;
    let count = import_csv(&conn, config.csv_path(), config.schema(), normalizer)?;
    close_connection(conn)?;
    Ok(count)
}

fn close_connection(conn: Connection) -> Result<(), Error> {
    conn.close().map_err(|(_, e)| Error::Sqlite(e.to_string()))
}

fn compute_file_checksum(path: &Path) -> Result<String, Error> {
    let bytes = fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
    Ok(checksum_hex(&bytes))
}

fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let secs_i = i64::try_from(secs).unwrap_or(i64::MAX);
    format_unix_timestamp(secs_i)
}

fn format_unix_timestamp(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let hour = secs_of_day.div_euclid(3_600);
    let minute = secs_of_day.rem_euclid(3_600).div_euclid(60);
    let second = secs_of_day.rem_euclid(60);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

const fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days.saturating_add(719_468);
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = doe
        .saturating_sub(doe.div_euclid(1_460))
        .saturating_add(doe.div_euclid(36_524))
        .saturating_sub(doe.div_euclid(146_096))
        .div_euclid(365);
    let year0 = yoe.saturating_add(era.saturating_mul(400));
    let doy = doe.saturating_sub(
        365_i64
            .saturating_mul(yoe)
            .saturating_add(yoe.div_euclid(4))
            .saturating_sub(yoe.div_euclid(100)),
    );
    let mp = 5_i64.saturating_mul(doy).saturating_add(2).div_euclid(153);
    let day = doy
        .saturating_sub(153_i64.saturating_mul(mp).saturating_add(2).div_euclid(5))
        .saturating_add(1);
    let month = if mp < 10 {
        mp.saturating_add(3)
    } else {
        mp.saturating_sub(9)
    };
    let year = if month <= 2 {
        year0.saturating_add(1)
    } else {
        year0
    };
    (year, month, day)
}
