use std::path::Path;

use rusqlite::{Connection, ToSql};

use unitrace_db_handler::traits::Normalizer;
use unitrace_db_handler::{Error, Schema};

pub const TABLE_NAME: &str = "nasabah";
pub const NORM_ALL_COLUMN: &str = "_norm_all";

pub fn create_table(conn: &Connection, schema: &Schema) -> Result<(), Error> {
    let mut cols = String::with_capacity(schema.len().saturating_mul(32).saturating_add(64));
    for col in schema.columns() {
        cols.push_str(col.name());
        cols.push_str(" TEXT NOT NULL DEFAULT '', ");
    }
    cols.push_str(NORM_ALL_COLUMN);
    cols.push_str(" TEXT NOT NULL DEFAULT ''");
    let ddl = format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} ({cols});");
    conn.execute_batch(&ddl)
        .map_err(|e| Error::Sqlite(e.to_string()))?;
    let idx_name = format!("idx_{TABLE_NAME}_{NORM_ALL_COLUMN}");
    let idx_ddl =
        format!("CREATE INDEX IF NOT EXISTS {idx_name} ON {TABLE_NAME}({NORM_ALL_COLUMN});");
    conn.execute_batch(&idx_ddl)
        .map_err(|e| Error::Sqlite(e.to_string()))?;
    Ok(())
}

pub fn import_csv(
    conn: &Connection,
    csv_path: &Path,
    schema: &Schema,
    normalizer: &dyn Normalizer,
) -> Result<u64, Error> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(csv_path)
        .map_err(|e| Error::Csv(e.to_string()))?;

    let headers = reader
        .headers()
        .map_err(|e| Error::Csv(e.to_string()))?
        .clone();
    let indices = resolve_columns(&headers, schema)?;
    let insert_sql = build_insert_sql(schema);

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| Error::Sqlite(e.to_string()))?;

    let mut count = 0u64;
    for (row_idx, record_result) in reader.records().enumerate() {
        let line = row_idx.saturating_add(2);
        let record = record_result.map_err(|e| Error::Csv(format!("line {line}: {e}")))?;
        let fields = extract_fields(&record, &indices);
        schema.validate_field_types(&fields)?;
        let norm_all = build_norm_all(&fields, schema, normalizer);
        let mut params: Vec<&dyn ToSql> = Vec::with_capacity(fields.len().saturating_add(1));
        for f in &fields {
            params.push(f);
        }
        params.push(&norm_all);
        let _rows = tx
            .execute(&insert_sql, params.as_slice())
            .map_err(|e| Error::Sqlite(format!("line {line}: {e}")))?;
        count = count.saturating_add(1);
    }

    tx.commit().map_err(|e| Error::Sqlite(e.to_string()))?;
    Ok(count)
}

fn resolve_columns(headers: &csv::StringRecord, schema: &Schema) -> Result<Vec<usize>, Error> {
    let mut indices = Vec::with_capacity(schema.len());
    for col in schema.columns() {
        let target = col.name();
        let mut found = None;
        for (i, header) in headers.iter().enumerate() {
            if header.trim().eq_ignore_ascii_case(target) {
                found = Some(i);
                break;
            }
        }
        match found {
            Some(i) => indices.push(i),
            None => return Err(Error::MissingCsvColumn(target.to_string())),
        }
    }
    Ok(indices)
}

fn extract_fields(record: &csv::StringRecord, indices: &[usize]) -> Vec<String> {
    indices
        .iter()
        .map(|&i| record.get(i).unwrap_or("").to_string())
        .collect()
}

fn build_norm_all(fields: &[String], schema: &Schema, normalizer: &dyn Normalizer) -> String {
    let mut out = String::new();
    for (i, col) in schema.columns().iter().enumerate() {
        if col.is_normalized()
            && let Some(v) = fields.get(i)
        {
            out.push_str(&normalizer.normalize(v));
        }
    }
    out
}

fn build_insert_sql(schema: &Schema) -> String {
    let mut cols: Vec<&str> = Vec::with_capacity(schema.len().saturating_add(1));
    for col in schema.columns() {
        cols.push(col.name());
    }
    cols.push(NORM_ALL_COLUMN);
    let placeholders = vec!["?"; cols.len()].join(", ");
    format!(
        "INSERT INTO {TABLE_NAME} ({}) VALUES ({placeholders})",
        cols.join(", ")
    )
}
