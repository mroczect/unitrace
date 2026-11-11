#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: unwrap/expect express test assertions idiomatically"
)]
use unitrace_db_core as _;
use unitrace_db_gui_core as _;
use unitrace_db_gui_handler as _;
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

#[test]
fn test_package_and_signature_exports_are_reachable() {
    use unitrace_db::{
        EMBEDDED_PUBLIC_KEY_HEX, PUBLIC_KEY_HEX_LEN, Package, SIGNATURE_LEN, combine_hash,
        resolve_public_key, sign_bytes, sign_files, verify_bytes, verify_files,
    };
    assert_eq!(SIGNATURE_LEN, 64);
    assert_eq!(PUBLIC_KEY_HEX_LEN, 64);
    assert!(!EMBEDDED_PUBLIC_KEY_HEX.is_empty());
    let pkg = Package::new(b"db".to_vec(), b"meta".to_vec(), [0u8; 64]);
    assert_eq!(pkg.db, b"db");
    assert_eq!(pkg.meta, b"meta");
    let bytes = pkg.to_bytes();
    let restored = Package::from_bytes(&bytes).unwrap();
    assert_eq!(restored.db, pkg.db);
    let _f1: fn() -> Option<ed25519_dalek::VerifyingKey> = resolve_public_key;
    let _f2: fn(&[u8], &[u8]) -> [u8; 32] = combine_hash;
    let _f3: fn(&ed25519_dalek::SigningKey, &[u8], &[u8]) -> [u8; 64] = sign_bytes;
    type VerifyFn = fn(
        &ed25519_dalek::VerifyingKey,
        &[u8],
        &[u8],
        &[u8; 64],
    ) -> Result<(), Error>;
    let _f4: VerifyFn = verify_bytes;
    type SignFilesFn = fn(
        &ed25519_dalek::SigningKey,
        &std::path::Path,
        &std::path::Path,
    ) -> Result<[u8; 64], Error>;
    let _f5: SignFilesFn = sign_files;
    type VerifyFilesFn = fn(
        &ed25519_dalek::VerifyingKey,
        &std::path::Path,
        &std::path::Path,
        &std::path::Path,
    ) -> Result<(), Error>;
    let _f6: VerifyFilesFn = verify_files;
}

#[test]
fn test_gui_exports_are_reachable() {
    use unitrace_db::{
        DefaultGuiController, GuiController, NotFoundProof, RowViewModel, ViewState,
    };
    let _view = ViewState {
        status_text: String::new(),
        rows: Vec::new(),
        not_found_proof: None,
    };
    let _nf = NotFoundProof {
        keyword: "x".to_string(),
        searched_rows: 0,
        searched_columns: 0,
        database_version: "v".to_string(),
    };
    let _row = RowViewModel {
        columns: vec!["a".to_string()],
    };
    fn assert_controller<C: GuiController>() {}
    assert_controller::<DefaultGuiController>();
}
