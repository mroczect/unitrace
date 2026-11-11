use alloc::sync::Arc;
use core::fmt;
use core::fmt::Write as _;
use std::time::Instant;

use rusqlite::{Connection, Row};

use unitrace_db_handler::traits::{DatabaseReader, Integrity, Normalizer};
use unitrace_db_handler::{
    ColumnDef, Error, OpenConfig, Record, Schema, SearchMode, SearchQuery, SearchResult,
};

use crate::csv_import::{NORM_ALL_COLUMN, TABLE_NAME};
use crate::integrity::DefaultIntegrity;
use crate::metadata_io::read_metadata;
use crate::normalize::DefaultNormalizer;
use crate::sqlcipher::open_encrypted;

pub struct DefaultSearcher {
    conn: Connection,
    schema: Arc<Schema>,
    normalizer: Box<dyn Normalizer>,
}

impl DefaultSearcher {
    pub fn open(config: &OpenConfig) -> Result<Self, Error> {
        let metadata = read_metadata(config.metadata_path())?;

        let integrity = DefaultIntegrity::new();
        integrity.verify(config.db_path(), &metadata)?;

        let conn = open_encrypted(config.db_path(), config.key())?;

        Ok(Self {
            conn,
            schema: Arc::clone(metadata.schema()),
            normalizer: Box::new(DefaultNormalizer::new()),
        })
    }

    #[must_use]
    pub fn with_normalizer(mut self, normalizer: Box<dyn Normalizer>) -> Self {
        self.normalizer = normalizer;
        self
    }

    fn build_where_clause(
        &self,
        query: &SearchQuery,
        keyword_norm: &str,
        selected_indices: &[usize],
    ) -> (String, Vec<String>) {
        match query.mode() {
            SearchMode::Normal => {
                let pattern = format!("%{keyword_norm}%");
                (format!("WHERE {NORM_ALL_COLUMN} LIKE ?1"), vec![pattern])
            }
            SearchMode::Raw => {
                let pattern = query.keyword().to_string();
                let conds: Vec<String> = selected_indices
                    .iter()
                    .map(|&i| format!("instr({}, ?1) > 0", self.column_name(i)))
                    .collect();
                (format!("WHERE {}", conds.join(" OR ")), vec![pattern])
            }
            SearchMode::Exact => {
                let conds: Vec<String> = selected_indices
                    .iter()
                    .map(|&i| format!("{} = ?1", self.column_name(i)))
                    .collect();
                (
                    format!("WHERE {}", conds.join(" OR ")),
                    vec![query.keyword().to_string()],
                )
            }
        }
    }

    fn column_name(&self, index: usize) -> &str {
        self.schema.column(index).map_or("", ColumnDef::name)
    }

    fn resolve_columns(&self, query: &SearchQuery) -> Result<Vec<usize>, Error> {
        match query.columns() {
            Some(cols) => {
                let mut indices = Vec::with_capacity(cols.len());
                for name in cols {
                    let idx = self
                        .schema
                        .index_of(name)
                        .ok_or_else(|| Error::ColumnNotFound(name.clone()))?;
                    indices.push(idx);
                }
                Ok(indices)
            }
            None => Ok((0..self.schema.len()).collect()),
        }
    }
}

impl fmt::Debug for DefaultSearcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultSearcher")
            .field("schema_len", &self.schema.len())
            .finish_non_exhaustive()
    }
}

impl DatabaseReader for DefaultSearcher {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn row_count(&self) -> Result<u64, Error> {
        let count: i64 = self
            .conn
            .query_row(&format!("SELECT COUNT(*) FROM {TABLE_NAME}"), [], |r| {
                r.get(0)
            })
            .map_err(|e| Error::Sqlite(e.to_string()))?;
        u64::try_from(count).map_err(|_| Error::ArithmeticOverflow)
    }

    fn get_by_id(&self, id: i64) -> Result<Option<Record>, Error> {
        let sql = format!("SELECT rowid, * FROM {TABLE_NAME} WHERE rowid = ?1");
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Sqlite(e.to_string()))?;
        let mut rows = stmt.query([id]).map_err(|e| Error::Sqlite(e.to_string()))?;

        match rows.next().map_err(|e| Error::Sqlite(e.to_string()))? {
            Some(row) => Ok(Some(row_to_record(row, &self.schema)?)),
            None => Ok(None),
        }
    }

    fn search(&self, query: &SearchQuery) -> Result<SearchResult, Error> {
        let start = Instant::now();
        let selected_indices = self.resolve_columns(query)?;
        let searched_rows = self.row_count()?;
        let searched_columns = u64::try_from(self.schema.len()).unwrap_or(u64::MAX);

        let keyword_norm = self.normalizer.normalize(query.keyword());

        if query.mode() == SearchMode::Normal && keyword_norm.is_empty() {
            return Ok(SearchResult::new(
                Arc::clone(&self.schema),
                Vec::new(),
                0,
                searched_columns,
                searched_rows,
                elapsed_ms(start),
            ));
        }

        let (where_clause, params) =
            self.build_where_clause(query, &keyword_norm, &selected_indices);

        let count_sql = format!("SELECT COUNT(*) FROM {TABLE_NAME} {where_clause}");
        let mut stmt_count = self
            .conn
            .prepare(&count_sql)
            .map_err(|e| Error::Sqlite(e.to_string()))?;
        let total_matches: i64 = stmt_count
            .query_row(rusqlite::params_from_iter(params.iter()), |r| r.get(0))
            .map_err(|e| Error::Sqlite(e.to_string()))?;
        let total_matches = u64::try_from(total_matches).unwrap_or(u64::MAX);

        let mut select_sql = format!("SELECT rowid, * FROM {TABLE_NAME} {where_clause}");
        if let Some(limit) = query.limit() {
            let _ = write!(select_sql, " LIMIT {limit}");
        }

        let mut stmt_select = self
            .conn
            .prepare(&select_sql)
            .map_err(|e| Error::Sqlite(e.to_string()))?;
        let mut rows = stmt_select
            .query(rusqlite::params_from_iter(params.iter()))
            .map_err(|e| Error::Sqlite(e.to_string()))?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(|e| Error::Sqlite(e.to_string()))? {
            records.push(row_to_record(row, &self.schema)?);
        }

        Ok(SearchResult::new(
            Arc::clone(&self.schema),
            records,
            total_matches,
            searched_columns,
            searched_rows,
            elapsed_ms(start),
        ))
    }
}

fn row_to_record(row: &Row<'_>, schema: &Arc<Schema>) -> Result<Record, Error> {
    let id: i64 = row.get(0).map_err(|e| Error::Sqlite(e.to_string()))?;
    let mut fields = Vec::with_capacity(schema.len());
    for i in 0..schema.len() {
        let val: String = row
            .get(i.saturating_add(1))
            .map_err(|e| Error::Sqlite(e.to_string()))?;
        fields.push(val);
    }
    Record::try_new(id, fields, Arc::clone(schema))
}

fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}
