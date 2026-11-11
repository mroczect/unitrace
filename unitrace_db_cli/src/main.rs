#![allow(
    clippy::print_stdout,
    reason = "CLI: user-facing output is the whole point"
)]

use std::path::PathBuf;
use std::sync::Arc;

use unitrace_db::{
    BuildConfig, ColumnDef, ColumnKind, DatabaseBuilder, DefaultBuilder, EncryptionKey, Schema,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let csv_path = args.next().unwrap_or_else(|| "data.csv".to_string());
    let db_path = args.next().unwrap_or_else(|| "data.db".to_string());
    let meta_path = args.next().unwrap_or_else(|| "data.meta.json".to_string());

    let password = "supersecretkey123".to_string();

    let cols = vec![
        ColumnDef::try_new("no_perjanjian".into(), ColumnKind::Text)?
            .with_searchable()
            .with_normalize(),
        ColumnDef::try_new("nama_nasabah".into(), ColumnKind::Text)?
            .with_searchable()
            .with_normalize(),
        ColumnDef::try_new("no_polisi".into(), ColumnKind::Text)?
            .with_searchable()
            .with_normalize(),
        ColumnDef::try_new("status_nasabah".into(), ColumnKind::Text)?,
        ColumnDef::try_new("pic".into(), ColumnKind::Text)?,
        ColumnDef::try_new("kategori".into(), ColumnKind::Text)?,
        ColumnDef::try_new("jatuh_tempo".into(), ColumnKind::Date)?,
        ColumnDef::try_new("no_rangka".into(), ColumnKind::Text)?
            .with_searchable()
            .with_normalize(),
        ColumnDef::try_new("no_mesin".into(), ColumnKind::Text)?
            .with_searchable()
            .with_normalize(),
        ColumnDef::try_new("merek".into(), ColumnKind::Text)?,
        ColumnDef::try_new("tipe".into(), ColumnKind::Text)?,
        ColumnDef::try_new("warna".into(), ColumnKind::Text)?,
        ColumnDef::try_new("status_unit".into(), ColumnKind::Text)?,
        ColumnDef::try_new("tgl_penyelesaian".into(), ColumnKind::Date)?,
        ColumnDef::try_new("angsuran_ke".into(), ColumnKind::Integer)?,
        ColumnDef::try_new("tenor".into(), ColumnKind::Integer)?,
    ];
    let schema = Arc::new(Schema::try_new(cols)?);

    let config = BuildConfig::try_new(
        schema,
        PathBuf::from(&db_path),
        PathBuf::from(&meta_path),
        PathBuf::from(&csv_path),
        EncryptionKey::try_new(password)?,
        "v1.0.0".into(),
    )?;

    let metadata = DefaultBuilder::new().build(&config)?;

    println!("Database berhasil dibuat:");
    println!("  csv    : {csv_path}");
    println!("  db     : {db_path}");
    println!("  meta   : {meta_path}");
    println!("  rows   : {}", metadata.row_count());
    println!("  build  : {}", metadata.built_at());
    println!("  sha256 : {}", metadata.checksum_sha256());
    Ok(())
}
