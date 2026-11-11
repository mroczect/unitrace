# `unitrace_db_handler` API Reference

Contract-only crate for the Unitrace database workspace. Contains domain types, validators, error taxonomy, and traits. No I/O, no SQL, no serde, no trait implementations. Every runtime concern — build, search, checksum verification — belongs to downstream crates that `impl` the traits in `traits`.

Audience: developers writing `DatabaseBuilder` / `DatabaseReader` implementations, or code consuming these types from other crates.

---

## 1. Crate metadata

```toml
[package]
name = "unitrace_db_handler"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"

[dependencies]
zeroize = { version = "1.9.1", default-features = false, features = ["alloc"] }
```

- Not `no_std`. `schema.rs` uses `std::collections::HashSet`; `config.rs` uses `std::path::{Path, PathBuf}`. `extern crate alloc` in `lib.rs` exists only to allow explicit imports of `alloc::sync::Arc`. Do not port to `no_std` targets.
- `EncryptionKey` intentionally omits `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`. One owner per key.
- Non-test code must not use `unwrap`, `expect`, or `panic` — workspace lints deny these. Test modules apply `#![allow(clippy::unwrap_used, clippy::expect_used)]` locally.
- All paths in `BuildConfig` / `OpenConfig` are caller-owned `PathBuf`. The crate performs no resolution, canonicalization, or existence checks.

---

## 2. Module map

```
lib.rs
├── column    → ColumnDef, ColumnKind, MAX_COLUMN_NAME_LEN
├── schema    → Schema, SCHEMA_VERSION, DB_FILE_NAME, METADATA_FILE_NAME, MIN_KEYWORD_LEN
├── metadata  → Metadata
├── record    → Record
├── config    → BuildConfig, OpenConfig
├── query     → SearchMode, SearchQuery, SearchResult
├── newtype   → EncryptionKey, MIN_KEY_LENGTH
├── error     → Error
└── traits    → DatabaseBuilder, DatabaseReader, Normalizer, Checksum, Integrity
```

## 3. Root re-exports

```rust
pub use column::{ColumnDef, ColumnKind, MAX_COLUMN_NAME_LEN};
pub use config::{BuildConfig, OpenConfig};
pub use error::Error;
pub use metadata::Metadata;
pub use newtype::EncryptionKey;
pub use query::{SearchMode, SearchQuery, SearchResult};
pub use record::Record;
pub use schema::Schema;
```

Constants other than `MAX_COLUMN_NAME_LEN` are not re-exported at the root. Access them through their modules: `schema::SCHEMA_VERSION`, `newtype::MIN_KEY_LENGTH`, `schema::MIN_KEYWORD_LEN`, `schema::DB_FILE_NAME`, `schema::METADATA_FILE_NAME`.

---

## 4. `column`

```rust
pub const MAX_COLUMN_NAME_LEN: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnKind {
    Text,
    Integer,
    Decimal,
    Date,
    Boolean,
}

impl ColumnKind {
    pub const fn as_str(self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef { /* private */ }

impl ColumnDef {
    pub fn try_new(name: String, kind: ColumnKind) -> Result<Self, Error>;

    pub const fn name(&self) -> &str;
    pub const fn kind(&self) -> ColumnKind;
    pub const fn is_searchable(&self) -> bool;
    pub const fn is_normalized(&self) -> bool;
    pub const fn is_identity(&self) -> bool;

    pub const fn with_searchable(mut self) -> Self;
    pub const fn with_normalize(mut self) -> Self;
    pub const fn with_identity(mut self) -> Self;
}
```

`ColumnKind::as_str` returns `"text"`, `"integer"`, `"decimal"`, `"date"`, or `"boolean"`.

### Name validation

Checks run in this order inside `try_new`:

| Step | Condition                                              | Error                                 |
| ---- | ------------------------------------------------------ | ------------------------------------- |
| 1    | `name.is_empty()`                                      | `EmptyColumnName`                     |
| 2    | `name.len() > 128` (bytes, not chars)                  | `ColumnNameTooLong { max: 128, got }` |
| 3    | First char not ASCII alphabetic and not `_`            | `InvalidColumnName`                   |
| 4    | Any subsequent char not ASCII alphanumeric and not `_` | `InvalidColumnName`                   |

Consequences:

- Non-ASCII input is rejected. `naïve`, `名称`, emoji — all fail.
- Space, hyphen, dot, colon, and every punctuation mark fail.
- Leading `_`, trailing `_`, and `__foo` all pass.
- Because the character class is restricted to ASCII, `len()` equals character count. No multibyte edge case.

### Builder flags

`with_searchable`, `with_normalize`, `with_identity` are `const fn` that consume `self`. Typical construction:

```rust
ColumnDef::try_new("id".into(), ColumnKind::Integer)?
    .with_identity()
    .with_searchable()
```

The flags are pure metadata. `ColumnDef` does not store any normalization logic. Interpretation of `normalize` and `searchable` is the responsibility of `DatabaseReader` / `Normalizer` downstream.

---

## 5. `schema`

```rust
pub const SCHEMA_VERSION: u32 = 1;
pub const DB_FILE_NAME: &str = "data.db";
pub const METADATA_FILE_NAME: &str = "data.meta.json";
pub const MIN_KEYWORD_LEN: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema { /* private */ }

impl Schema {
    pub fn try_new(columns: Vec<ColumnDef>) -> Result<Self, Error>;

    pub const fn columns(&self) -> &[ColumnDef];
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;

    pub fn index_of(&self, name: &str) -> Option<usize>;
    pub fn column(&self, index: usize) -> Option<&ColumnDef>;

    pub fn identity_index(&self) -> Option<usize>;
    pub fn searchable_indices(&self) -> Vec<usize>;
    pub fn identity_indices(&self) -> Vec<usize>;

    pub fn validate_field_types(&self, fields: &[String]) -> Result<(), Error>;
}
```

### `try_new` invariants

1. `columns` non-empty. Otherwise `EmptySchema`.
2. Column names unique, case-insensitive. Duplicates trigger `DuplicateColumn(name_as_original_case)`. `"Name"` and `"name"` collide. `"Nama"` and `"NAMA"` collide.
3. At most one column with `identity` flag. Otherwise `MultipleIdentityColumns { count }`. Count is accumulated with `saturating_add`.

### Lookup behavior

- `index_of(name)` performs `eq_ignore_ascii_case` linear scan. Complexity O(n). No hash index stored inside `Schema`. For per-row lookups at scale, build a `HashMap<&str, usize>` at the reader layer.
- `column(i)` is `Vec::get`. Returns `None` when out of range.
- `identity_index()` returns the position of the first identity column, or `None`.
- `identity_indices()` returns `Vec<usize>` of length 0 or 1. Kept as `Vec` for forward compatibility if the invariant relaxes.
- `searchable_indices()` preserves column order.

### `validate_field_types`

Order of operations:

```rust
if fields.len() != self.columns.len() {
    return Err(Error::RecordLengthMismatch { expected: self.columns.len(), found: fields.len() });
}
```

Then per column:

```rust
if value.trim().is_empty() {
    return Ok(());   // short-circuit
}
```

Empty strings and whitespace-only strings pass for every `ColumnKind`. This is a deliberate contract — tested in `empty_value_is_accepted_for_all_kinds` and `whitespace_only_value_is_accepted_for_all_kinds`. Enforce NOT NULL at the reader layer.

Per-kind rules after the trim short-circuit:

| Kind      | Rule                                                                                                           | Notes                                                                                                                                                               |
| --------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Text`    | Always `true`                                                                                                  | Unicode, emoji, any content accepted                                                                                                                                |
| `Integer` | `value.parse::<i64>().is_ok()`                                                                                 | `"0"`, `"42"`, `"-1"`, `"+5"` pass. `"1.5"`, `"1,5"`, `"12a"` fail                                                                                                  |
| `Decimal` | `value.parse::<f64>().is_ok()`                                                                                 | `"1.5"`, `"-3.14"` pass. `"inf"`, `"NaN"`, `"1e5"` also pass — `f64::from_str` is permissive                                                                        |
| `Date`    | Length 10 bytes, exactly 8 ASCII digits, exactly 2 separators, both separators identical and either `-` or `/` | No month or day range validation. `"2025-99-99"` passes. `"11-11-2025"` passes. `"1-2345678-9"` also passes — the check is positional only in the sense of counting |
| `Boolean` | `eq_ignore_ascii_case("true")`, `eq_ignore_ascii_case("false")`, `"1"`, `"0"`                                  | `"TRUE"`, `"True"`, `"tRuE"` pass. `"yes"`, `"2"`, `"t"`, `"tru"` fail                                                                                              |

Error returned: `Error::FieldTypeMismatch { column, expected: kind.as_str(), actual: value.to_string() }`. The `actual` field holds the original value, not the trimmed value. Validation stops at the first failing column.

---

## 6. `metadata`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata { /* private */ }

impl Metadata {
    pub fn try_new(
        schema_version: u32,
        data_version: String,
        row_count: u64,
        checksum_sha256: String,
        built_at: String,
        schema: Arc<Schema>,
    ) -> Result<Self, Error>;

    pub const fn schema_version(&self) -> u32;
    pub const fn data_version(&self) -> &str;
    pub const fn row_count(&self) -> u64;
    pub const fn checksum_sha256(&self) -> &str;
    pub const fn built_at(&self) -> &str;
    pub const fn schema(&self) -> &Arc<Schema>;
}
```

### Validation order in `try_new`

Stops at first failure:

1. `schema_version != SCHEMA_VERSION` → `SchemaVersionMismatch { expected: 1, found }`
2. `data_version.is_empty()` → `EmptyDataVersion`. Value is not trimmed. `" "` passes.
3. Checksum fails hex validation → `InvalidChecksumHex`
4. `built_at` fails format validation → `InvalidBuiltAt`

### Checksum rule

Length equal to 64 **and** every byte satisfies `is_ascii_hexdigit()`. Uppercase accepted (`"A".repeat(64)` is valid). `"g".repeat(64)` rejected. 65 characters rejected at the length check.

### `built_at` rule

```rust
!s.is_empty()
    && s.bytes().any(|b| b.is_ascii_digit())
    && s.bytes().any(|b| b == b'-')
```

This is not an ISO 8601 parser. The contract requires only: non-empty, at least one digit, at least one hyphen. Passing examples: `"2023-10-27T10:00:00Z"`, `"2023-10-27"`, `"x1-y"`, `"1-"`. Failing examples: `"no-digits-here"`, `"20231027"`, `""`.

If a real timestamp is required, parse it in the builder layer and emit `Error::InvalidBuiltAt` yourself.

The `schema` field is stored as `Arc<Schema>` and shared across `Record` and `SearchResult`. No `Arc::make_mut` usage.

---

## 7. `record`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record { /* private */ }

impl Record {
    pub fn try_new(id: i64, fields: Vec<String>, schema: Arc<Schema>) -> Result<Self, Error>;

    pub const fn id(&self) -> i64;
    pub const fn fields(&self) -> &[String];
    pub fn at(&self, index: usize) -> Option<&str>;
    pub fn field(&self, name: &str) -> Option<&str>;
    pub const fn schema(&self) -> &Arc<Schema>;
}
```

### Validation order

1. `schema.validate_field_types(&fields)?` — surfaces `RecordLengthMismatch` and `FieldTypeMismatch` before identity checking.
2. If `schema` has an identity column at index `i`:
   - Fetch `fields[i]` with `ok_or(RecordLengthMismatch)`. Unreachable after step 1; defensive.
   - Compare with strict string equality against `id.to_string()`.
   - On mismatch, return `IdentityMismatch { id, column_value }`.

### Identity comparison semantics

The check is `fields[i] == id.to_string()`, not numeric parsing:

| `id` | `fields[i]` | Outcome            |
| ---- | ----------- | ------------------ |
| `42` | `"42"`      | Pass               |
| `-1` | `"-1"`      | Pass               |
| `7`  | `"007"`     | `IdentityMismatch` |
| `0`  | `""`        | `IdentityMismatch` |
| `42` | `"+42"`     | `IdentityMismatch` |

This is consistent with the trim short-circuit in `validate_field_types`: empty values pass type validation but fail identity comparison. The only valid representation is the canonical decimal form without padding, sign, or whitespace.

### Field access

- `at(i)` returns `Option<&str>`.
- `field(name)` calls `schema.index_of(name)` then `at(i)`. Case-insensitive, O(n).
- `fields()` returns the full slice in schema column order.

`Record` implements `PartialEq`, comparing `Arc<Schema>` via `Schema: PartialEq`. Two records pointing to distinct `Arc` allocations but with identical `Schema` contents are `==`.

---

## 8. `config`

```rust
#[derive(Debug)]
pub struct BuildConfig { /* private */ }

impl BuildConfig {
    pub fn try_new(
        schema: Arc<Schema>,
        db_path: PathBuf,
        metadata_path: PathBuf,
        csv_path: PathBuf,
        key: EncryptionKey,
        data_version: String,
    ) -> Result<Self, Error>;

    pub const fn schema(&self) -> &Arc<Schema>;
    pub fn db_path(&self) -> &Path;
    pub fn metadata_path(&self) -> &Path;
    pub fn csv_path(&self) -> &Path;
    pub const fn key(&self) -> &EncryptionKey;
    pub const fn data_version(&self) -> &str;
}

#[derive(Debug)]
pub struct OpenConfig { /* private */ }

impl OpenConfig {
    pub fn try_new(
        db_path: PathBuf,
        metadata_path: PathBuf,
        key: EncryptionKey,
    ) -> Result<Self, Error>;

    pub fn db_path(&self) -> &Path;
    pub fn metadata_path(&self) -> &Path;
    pub const fn key(&self) -> &EncryptionKey;
}
```

### Validation

Both structs reject empty paths via `path.as_os_str().is_empty()`, producing `Error::InvalidConfig("<field> must not be empty")`.

`BuildConfig` additionally rejects empty `data_version` with `Error::EmptyDataVersion` — not `InvalidConfig`.

Validation order in `BuildConfig::try_new`: `db_path` → `metadata_path` → `csv_path` → `data_version`.

Not validated:

- File or directory existence.
- Absolute vs relative path.
- File extension.
- Schema consistency against CSV contents.
- `data_version` format.

Neither struct is `Clone` or `PartialEq` because they hold `EncryptionKey`, which is `!Clone`. To share across builders, wrap in `Arc<BuildConfig>` or construct the key twice.

---

## 9. `query`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Normal,
    Raw,
    Exact,
}

#[derive(Debug, Clone)]
pub struct SearchQuery { /* private */ }

impl SearchQuery {
    pub fn try_new(
        keyword: String,
        mode: SearchMode,
        columns: Option<Vec<String>>,
        limit: Option<usize>,
    ) -> Result<Self, Error>;

    pub const fn keyword(&self) -> &str;
    pub const fn mode(&self) -> SearchMode;
    pub fn columns(&self) -> Option<&[String]>;
    pub const fn limit(&self) -> Option<usize>;
}

#[derive(Debug, Clone)]
pub struct SearchResult { /* private */ }

impl SearchResult {
    pub const fn new(
        schema: Arc<Schema>,
        records: Vec<Record>,
        total_matches: u64,
        searched_columns: u64,
        searched_rows: u64,
        elapsed_ms: u64,
    ) -> Self;

    pub const fn schema(&self) -> &Arc<Schema>;
    pub const fn records(&self) -> &[Record];
    pub const fn total_matches(&self) -> u64;
    pub const fn searched_columns(&self) -> u64;
    pub const fn searched_rows(&self) -> u64;
    pub const fn elapsed_ms(&self) -> u64;
}
```

### `SearchQuery::try_new` validation order

Stops at first failure. Order matters — tested in `query_empty_keyword_and_bad_columns_reports_keyword_first`:

1. `keyword.is_empty()` → `EmptyKeyword`
2. `columns == Some(vec![])` → `InvalidConfig("columns list must not be empty when provided")`
3. `limit == Some(0)` → `InvalidConfig("limit must be greater than zero when provided")`

Behavior notes:

- `keyword` is not trimmed. `"   "` passes. To enforce a minimum length, use `MIN_KEYWORD_LEN` and throw `Error::KeywordTooShort { min, got }` in the reader. This crate ships the variant but does not enforce it.
- `columns` is not validated against a `Schema`. Non-existent column names surface as `Error::ColumnNotFound` from `search()`.
- `limit` has no upper bound. `usize::MAX` is valid.

`SearchMode` is a pure tag. No behavior in this crate. Reasonable semantics for implementors:

| Mode     | Suggested behavior                                           |
| -------- | ------------------------------------------------------------ |
| `Normal` | Case-insensitive, honors the `normalize` flag on `ColumnDef` |
| `Raw`    | Byte-exact match, no normalization                           |
| `Exact`  | Whole-value match, not substring                             |

No `Default` for `SearchMode`. Callers must choose.

### `SearchResult`

`SearchResult::new` is a `const fn` with no validation. Container only. All statistics are supplied by the `DatabaseReader::search` implementor. There is no enforced relationship between `records().len()` and `total_matches()` — the convention is that `total_matches` reflects the count before `limit` truncation, but nothing in the type enforces it.

`elapsed_ms` is `u64` milliseconds. `Duration` is not used to keep the crate dependency-free.

---

## 10. `newtype`

```rust
pub const MIN_KEY_LENGTH: usize = 12;

pub struct EncryptionKey(String); // private field

impl EncryptionKey {
    pub fn try_new(value: String) -> Result<Self, Error>;
    pub const fn as_str(&self) -> &str;
}

impl fmt::Debug for EncryptionKey;   // prints "EncryptionKey(***)"
impl Drop for EncryptionKey;         // calls self.0.zeroize()
```

### Validation

```rust
let trimmed = value.trim();
if trimmed.is_empty() { return Err(Error::EmptyKey); }
let effective_len = trimmed.chars().count();
if effective_len < MIN_KEY_LENGTH {
    return Err(Error::KeyTooShort { min: 12, got: effective_len });
}
```

- Length is measured after trim, in **characters**, not bytes. `"éééééééééééé"` (12 chars, 24 bytes) passes.
- The stored value is `value`, not `trimmed`. `EncryptionKey::try_new("   supersecretkey123   ".into())?.as_str()` returns the string with surrounding spaces. Tested in `accepts_trimmed_long_key`.
- Consequence: the SQLCipher implementor decides whether to trim again before applying the key. The crate contract is: key equals the exact bytes returned by `as_str()`.
- `"   a   b   c  "` → trimmed to `"a   b   c"` (9 chars) → `KeyTooShort { min: 12, got: 9 }`. Internal spaces count.
- All-whitespace (`" ".repeat(64)`) → `EmptyKey`, not `KeyTooShort`.

### Debug and Drop

- `Debug` is implemented manually, always `EncryptionKey(***)`. Do not add `derive(Debug)`.
- `Drop` calls `Zeroize` on the inner `String` via `zeroize::Zeroize` with the `alloc` feature. This clears heap contents. It does not guarantee that stack or register copies are erased. Do not rely on this for threat models stronger than "key not left in heap after drop".
- Since the type is `!Clone`, keys can only be moved. `BuildConfig::key()` and `OpenConfig::key()` return `&EncryptionKey`.

---

## 11. `error`

```rust
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
    FieldTypeMismatch { column: String, expected: &'static str, actual: String },
    IdentityMismatch { id: i64, column_value: String },
    ArithmeticOverflow,
    InvalidConfig(String),
}

impl fmt::Display for Error;
impl core::error::Error for Error;
```

30 variants. Marked `#[non_exhaustive]`. Downstream `match` must include a `_` arm.

`impl core::error::Error`, not `std::error::Error`. Because the crate itself uses `std`, both `Box<dyn core::error::Error>` and `Box<dyn std::error::Error>` are usable downstream.

### Display strings

| Variant                   | Payload                        | Display                                                                                                                      |
| ------------------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| `EmptyKey`                | —                              | `encryption key must not be empty`                                                                                           |
| `KeyTooShort`             | `min`, `got`                   | `encryption key too short: minimum {min} characters, got {got}`                                                              |
| `EmptySchema`             | —                              | `schema must contain at least one column`                                                                                    |
| `EmptyColumnName`         | —                              | `column name must not be empty`                                                                                              |
| `InvalidColumnName`       | `String`                       | `invalid column name: {n:?} (must start with ASCII letter or underscore, followed by ASCII letters, digits, or underscores)` |
| `ColumnNameTooLong`       | `max`, `got`                   | `column name too long: max {max} bytes, got {got}`                                                                           |
| `DuplicateColumn`         | `String`                       | `duplicate column: {n}`                                                                                                      |
| `MultipleIdentityColumns` | `count`                        | `schema must have at most one identity column, found {count}`                                                                |
| `ColumnNotFound`          | `String`                       | `column not found: {n}`                                                                                                      |
| `EmptyKeyword`            | —                              | `search keyword must not be empty`                                                                                           |
| `KeywordTooShort`         | `min`, `got`                   | `keyword too short: minimum {min}, got {got}`                                                                                |
| `EmptyDataVersion`        | —                              | `data version must not be empty`                                                                                             |
| `InvalidBuiltAt`          | —                              | `built_at must be a non-empty ISO-like timestamp string`                                                                     |
| `InvalidChecksumHex`      | —                              | `checksum must be exactly 64 hexadecimal characters`                                                                         |
| `DatabaseNotFound`        | `String`                       | `database not found: {p}`                                                                                                    |
| `MetadataNotFound`        | `String`                       | `metadata not found: {p}`                                                                                                    |
| `Sqlite`                  | `String`                       | `sqlite: {e}`                                                                                                                |
| `Io`                      | `String`                       | `io: {e}`                                                                                                                    |
| `Json`                    | `String`                       | `json: {e}`                                                                                                                  |
| `Csv`                     | `String`                       | `csv: {e}`                                                                                                                   |
| `SchemaVersionMismatch`   | `expected`, `found`            | `schema version mismatch: expected {expected}, found {found}`                                                                |
| `ChecksumMismatch`        | `expected`, `found`            | `checksum mismatch: expected {expected}, found {found}`                                                                      |
| `CipherUnavailable`       | —                              | `SQLCipher is not available`                                                                                                 |
| `InvalidKey`              | —                              | `invalid key or corrupted database`                                                                                          |
| `MissingCsvColumn`        | `String`                       | `missing CSV column: {c}`                                                                                                    |
| `RecordLengthMismatch`    | `expected`, `found`            | `record length mismatch: expected {expected}, found {found}`                                                                 |
| `FieldTypeMismatch`       | `column`, `expected`, `actual` | `field type mismatch in column {column:?}: expected {expected}, got {actual:?}`                                              |
| `IdentityMismatch`        | `id`, `column_value`           | `record id {id} does not match identity column value {column_value:?}`                                                       |
| `ArithmeticOverflow`      | —                              | `arithmetic overflow`                                                                                                        |
| `InvalidConfig`           | `String`                       | `invalid configuration: {c}`                                                                                                 |

### Variants not produced by this crate

This crate is contract-only. The following variants are declared but never returned by any `try_new` or constructor in this crate. They exist for downstream implementors:

`ColumnNotFound`, `KeywordTooShort`, `DatabaseNotFound`, `MetadataNotFound`, `Sqlite`, `Io`, `Json`, `Csv`, `ChecksumMismatch`, `CipherUnavailable`, `InvalidKey`, `MissingCsvColumn`, `ArithmeticOverflow`.

Do not write tests that expect this crate to produce those variants.

---

## 12. `traits`

```rust
use std::path::Path;
use crate::config::BuildConfig;
use crate::error::Error;
use crate::metadata::Metadata;
use crate::query::{SearchQuery, SearchResult};
use crate::record::Record;
use crate::schema::Schema;

pub trait DatabaseBuilder {
    fn build(&self, config: &BuildConfig) -> Result<Metadata, Error>;
}

pub trait DatabaseReader {
    fn search(&self, query: &SearchQuery) -> Result<SearchResult, Error>;
    fn get_by_id(&self, id: i64) -> Result<Option<Record>, Error>;
    fn row_count(&self) -> Result<u64, Error>;
    fn schema(&self) -> &Schema;
}

pub trait Normalizer {
    fn normalize(&self, input: &str) -> String;
}

pub trait Checksum {
    fn checksum(&self, data: &[u8]) -> [u8; 32];
}

pub trait Integrity {
    fn verify(&self, db_path: &Path, metadata: &Metadata) -> Result<(), Error>;
}
```

Contract notes:

- `DatabaseBuilder::build` takes `&self`. The implementor owns `Sync`/`Send` requirements. No bounds are added by this crate.
- `DatabaseBuilder::build` returns `Metadata`. The implementor must populate `checksum_sha256` with 64 lowercase hex characters and `built_at` conforming to `is_valid_built_at`. Otherwise `Metadata::try_new` rejects the result.
- `DatabaseReader::schema` returns `&Schema`, not `&Arc<Schema>`. `search`, `get_by_id`, and `row_count` return `Result` since they touch I/O.
- `Checksum::checksum` returns raw `[u8; 32]`. Conversion to 64-char hex for `Metadata` is the caller's responsibility. No hex helper is provided in this crate.
- `Integrity::verify` receives `db_path` and `&Metadata`. This is where `ChecksumMismatch`, `DatabaseNotFound`, `MetadataNotFound`, and `InvalidKey` should be produced.
- No blanket implementations. No default methods.

---

## 13. Invariants guaranteed by construction

Holds for every value that constructs successfully.

**`ColumnDef`**

- `name()` matches `^[A-Za-z_][A-Za-z0-9_]*$`, byte length 1–128 inclusive.
- Flags are independent booleans. No forbidden combinations.

**`Schema`**

- `!is_empty()`.
- Column names unique, case-insensitive.
- `identity_indices().len() <= 1`.

**`Metadata`**

- `schema_version() == 1`.
- `data_version()` non-empty (not necessarily trimmed).
- `checksum_sha256()` is exactly 64 ASCII hex characters.
- `built_at()` non-empty, contains at least one digit and at least one hyphen.

**`Record`**

- `fields().len() == schema().len()`.
- Each field passes its column kind's type validation, unless it is empty or whitespace-only (short-circuit).
- If the schema has an identity column at index `i`, then `fields()[i] == id().to_string()` exactly.

**`EncryptionKey`**

- `as_str().trim()` is non-empty.
- `as_str().trim().chars().count() >= 12`.

**`BuildConfig` / `OpenConfig`**

- All paths non-empty.
- `BuildConfig::data_version()` non-empty.

**`SearchQuery`**

- `keyword()` non-empty. Whitespace-only allowed.
- `columns()` when `Some`, has length ≥ 1.
- `limit()` when `Some`, value ≥ 1.

---

## 14. Integration gotchas

1. Empty or whitespace fields always pass `validate_field_types`. Enforce NOT NULL at the reader, not in `Record::try_new`.
2. Identity comparison uses string equality. `7` vs `"007"` fails. `0` vs `""` fails. Never zero-pad the `id` via `format!`.
3. `Decimal` accepts `"inf"`, `"-inf"`, `"NaN"`, `"1e5"`. `f64::from_str` is permissive. Filter at the reader if strict decimal is needed.
4. `Date` does not validate month or day ranges. `"9999-99-99"` passes. `"11-11-2025"` (day-first) passes. Do not assume Y-M-D ordering in downstream code.
5. `Date` validation checks length, digit count, and separator count with identical separator characters. It does not check positional layout. `"1-2345678-9"` passes.
6. `EncryptionKey::as_str()` returns the original untrimmed string. Trim is only applied for the length measurement. Downstream must decide whether to trim before applying the key.
7. `index_of` is O(n). For hot paths, cache a map at the reader layer.
8. `SearchQuery` does not validate column names against a schema. Errors surface inside `search()`.
9. `MIN_KEYWORD_LEN = 3` is not enforced by any constructor. `SearchQuery::try_new("x", ...)` succeeds.
10. `Error` is `#[non_exhaustive]`. Always include a `_` arm in `match`.
11. `Metadata::try_new` rejects `schema_version != 1`. Bumping `SCHEMA_VERSION` downstream requires forking this crate. The constant is not a runtime parameter.
12. `Record`, `Metadata`, `SearchResult` hold `Arc<Schema>`. Replacing a schema means replacing the `Arc`, not mutating the schema. `Schema` has no interior mutability.
13. This crate is not `no_std`, despite `extern crate alloc` in `lib.rs`. Do not attempt to compile against targets without `std`.

---

## 15. Example composition

```rust
use std::path::PathBuf;
use std::sync::Arc;

use unitrace_db_handler::{
    BuildConfig, ColumnDef, ColumnKind, EncryptionKey, Metadata, OpenConfig,
    Record, Schema, SearchMode, SearchQuery, schema,
    traits::DatabaseBuilder,
};

fn make_schema() -> Result<Arc<Schema>, unitrace_db_handler::Error> {
    let cols = vec![
        ColumnDef::try_new("id".into(), ColumnKind::Integer)?.with_identity(),
        ColumnDef::try_new("name".into(), ColumnKind::Text)?.with_searchable(),
        ColumnDef::try_new("born".into(), ColumnKind::Date)?,
    ];
    Ok(Arc::new(Schema::try_new(cols)?))
}

fn make_build_config() -> Result<BuildConfig, unitrace_db_handler::Error> {
    BuildConfig::try_new(
        make_schema()?,
        PathBuf::from("./out/data.db"),
        PathBuf::from("./out/data.meta.json"),
        PathBuf::from("./out/data.csv"),
        EncryptionKey::try_new("supersecretkey123".into())?,
        "v1.0.0".into(),
    )
}

fn make_open_config() -> Result<OpenConfig, unitrace_db_handler::Error> {
    OpenConfig::try_new(
        PathBuf::from("./out/data.db"),
        PathBuf::from("./out/data.meta.json"),
        EncryptionKey::try_new("supersecretkey123".into())?,
    )
}

fn make_record(schema: Arc<Schema>) -> Result<Record, unitrace_db_handler::Error> {
    Record::try_new(42, vec!["42".into(), "Alice".into(), "2025-11-11".into()], schema)
}

fn make_query() -> Result<SearchQuery, unitrace_db_handler::Error> {
    SearchQuery::try_new(
        "alice".into(),
        SearchMode::Normal,
        Some(vec!["name".into()]),
        Some(20),
    )
}

struct MyBuilder;

impl DatabaseBuilder for MyBuilder {
    fn build(&self, cfg: &BuildConfig) -> Result<Metadata, unitrace_db_handler::Error> {
        let _ = (
            cfg.db_path(),
            cfg.metadata_path(),
            cfg.csv_path(),
            cfg.key().as_str(),
            cfg.data_version(),
        );
        let _version = schema::SCHEMA_VERSION;
        todo!()
    }
}
```

---

## 16. Implementor checklist

- `build()` populates `checksum_sha256` with 64 lowercase hex characters, and `built_at` with at least one digit and at least one hyphen.
- `build()` maps I/O failures to `Error::Io(String)`, SQLite failures to `Error::Sqlite(String)`, CSV failures to `Error::Csv(String)` or `Error::MissingCsvColumn(String)`.
- `search()` validates `SearchQuery::columns()` names against `Schema` and returns `ColumnNotFound` for missing names.
- `search()` enforces `MIN_KEYWORD_LEN` itself via `Error::KeywordTooShort` — the crate does not.
- `get_by_id()` returns `Ok(None)` for missing ids, not `Err`.
- `Integrity::verify` produces `ChecksumMismatch { expected: metadata.checksum_sha256().to_string(), found }` on mismatch.
- Callers of `match` on `Error` include a `_` arm because the enum is `#[non_exhaustive]`.
- Non-test code contains no `unwrap`, `expect`, or `panic` calls.
- `EncryptionKey` is never cloned. Wrap `BuildConfig` / `OpenConfig` in `Arc` if sharing is needed.
