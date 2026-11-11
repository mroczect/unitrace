#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]

use unitrace_db_core as _;
use unitrace_db_handler as _;

use unitrace_db::{
    BuildConfig, Checksum, ColumnDef, ColumnKind, DatabaseBuilder, DatabaseReader, DefaultBuilder,
    DefaultIntegrity, DefaultNormalizer, DefaultSearcher, EncryptionKey, Error, Integrity,
    Metadata, Normalizer, OpenConfig, Record, Schema, SearchMode, SearchQuery, SearchResult,
    Sha256Checksum,
};

const fn assert_traits<B, R, C, N, I>()
where
    B: DatabaseBuilder,
    R: DatabaseReader,
    C: Checksum,
    N: Normalizer,
    I: Integrity,
{
}

#[test]
fn test_all_exports_are_reachable() {
    let _: Option<BuildConfig> = None;
    let _: Option<OpenConfig> = None;
    let _: Option<EncryptionKey> = None;
    let _: Option<Schema> = None;
    let _: Option<ColumnDef> = None;
    let _: Option<ColumnKind> = None;
    let _: Option<SearchQuery> = None;
    let _: Option<SearchMode> = None;
    let _: Option<SearchResult> = None;
    let _: Option<Record> = None;
    let _: Option<Metadata> = None;
    let _: Option<Error> = None;

    assert_traits::<
        DefaultBuilder,
        DefaultSearcher,
        Sha256Checksum,
        DefaultNormalizer,
        DefaultIntegrity,
    >();
}
