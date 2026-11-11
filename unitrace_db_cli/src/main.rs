#![allow(
    clippy::print_stdout,
    reason = "CLI: user-facing output is the whole point"
)]
use alloc::sync::Arc;
use core::error::Error as CoreError;
use std::path::{Path, PathBuf};
use unitrace_db::{
    BuildConfig, ColumnDef, ColumnKind, DatabaseBuilder, DefaultBuilder, EMBEDDED_PUBLIC_KEY_HEX,
    EncryptionKey, Package, SIGNATURE_LEN, Schema, resolve_public_key, sign_bytes, verify_bytes,
};

fn main() -> Result<(), Box<dyn CoreError>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map_or("help", String::as_str);
    let rest = args.get(1..).unwrap_or(&[]);
    match cmd {
        "keygen" => cmd_keygen(rest),
        "build" => cmd_build(rest),
        "verify" => cmd_verify(rest),
        "inspect" => cmd_inspect(rest),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => {
            eprintln!("unknown command: {other}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!("unitrace-db-cli — developer toolchain untuk Unitrace database");
    println!();
    println!("USAGE:");
    println!("  unitrace-db-cli keygen <signing.key>");
    println!("      Generate Ed25519 keypair. Tulis private key ke <signing.key>");
    println!("      (chmod 600), print public key hex ke stdout.");
    println!("      Public key harus di-embed ke unitrace_db_core/public_key.hex,");
    println!("      lalu rebuild binary app.");
    println!();
    println!("  unitrace-db-cli build <csv> <signing.key> <out.upkg>");
    println!("      CSV → SQLCipher encrypt (env UNITRACE_DB_KEY) → sign → package.");
    println!("      Hasil: 1 file .upkg siap dikirim ke user.");
    println!();
    println!("  unitrace-db-cli verify <in.upkg>");
    println!("      Verify signature (terhadap embedded public key).");
    println!();
    println!("  unitrace-db-cli inspect <in.upkg>");
    println!("      Tampilkan metadata package (signature, ukuran, dsb).");
}

fn cmd_keygen(args: &[String]) -> Result<(), Box<dyn CoreError>> {
    use rand::rngs::OsRng;
    let path = args.first().ok_or("usage: keygen <signing.key>")?;
    if Path::new(path).exists() {
        return Err(format!(
            "{path} sudah ada. Hapus dulu kalau mau regenerate (akan invalidate semua package lama)."
        )
        .into());
    }
    let sk = ed25519_dalek::SigningKey::generate(&mut OsRng);
    std::fs::write(path, sk.to_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(path)?.permissions();
        p.set_mode(0o600);
        std::fs::set_permissions(path, p)?;
    }
    let pub_hex = hex::encode(sk.verifying_key().to_bytes());
    println!("Signing key: {path}  (chmod 600, simpan offline!)");
    println!();
    println!("Public key (hex 64 chars):");
    println!("  {pub_hex}");
    println!();
    println!("Langkah selanjutnya:");
    println!("  1. Timpa unitrace_db_core/public_key.hex dengan:");
    println!("     {pub_hex}");
    println!("  2. Rebuild binary app:");
    println!("     cargo build --release -p unitrace_db_ffi");
    println!("  3. Build package:");
    println!("     unitrace-db-cli build data.csv {path} out.upkg");
    println!();
    println!("PENTING: simpan {path} di tempat aman. Kalau hilang, semua package");
    println!("harus di-rebuild dengan key baru.");
    Ok(())
}

fn cmd_build(args: &[String]) -> Result<(), Box<dyn CoreError>> {
    if args.len() < 3 {
        return Err("usage: build <csv> <signing.key> <out.upkg>".into());
    }
    let csv_path = args.get(0).ok_or("missing csv")?;
    let key_path = args.get(1).ok_or("missing key")?;
    let out_path = args.get(2).ok_or("missing out")?;

    let password = std::env::var("UNITRACE_DB_KEY").map_err(|_| {
        "env UNITRACE_DB_KEY wajib diisi. Generate dulu:
\
head -c 32 /dev/urandom | xxd -p -c 64"
    })?;
    if !Path::new(csv_path).is_file() {
        return Err(format!("CSV tidak ditemukan: {csv_path}").into());
    }
    let key_bytes = std::fs::read(key_path).map_err(|e| format!("read {key_path}: {e}"))?;
    let key_arr: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| format!("{key_path} harus 32 bytes"))?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_arr);
    let tmp = tempfile::TempDir::new()?;
    let db_path = tmp.path().join("data.db");
    let meta_path = tmp.path().join("data.meta.json");
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
        db_path.clone(),
        meta_path.clone(),
        PathBuf::from(csv_path),
        EncryptionKey::try_new(password)?,
        "v1.0.0".into(),
    )?;
    eprintln!("[1/3] build SQLCipher db dari CSV...");
    let metadata = DefaultBuilder::new().build(&config)?;
    eprintln!("[2/3] sign db+meta dengan Ed25519...");
    let db_bytes = std::fs::read(&db_path)?;
    let meta_bytes = std::fs::read(&meta_path)?;
    let sig = sign_bytes(&signing_key, &db_bytes, &meta_bytes);
    eprintln!("[3/3] package ke {out_path}...");
    let pkg = Package::new(db_bytes, meta_bytes, sig);
    pkg.write(Path::new(out_path))?;
    let total = std::fs::metadata(out_path)?.len();
    println!("Package dibuat:");
    println!("  out     : {out_path}");
    println!("  size    : {total} bytes");
    println!("  rows    : {}", metadata.row_count());
    println!("  build   : {}", metadata.built_at());
    println!("  db_sha  : {}", metadata.checksum_sha256());
    println!();
    println!("Verifikasi embedded public key sebelum ship:");
    let embedded = parse_embedded_pub_hex();
    let signing_pub = hex::encode(signing_key.verifying_key().to_bytes());
    if embedded == signing_pub {
        println!("  OK — public key di binary sudah match signing key ini.");
    } else {
        println!("  WARNING — public key di binary TIDAK match dengan signing key ini!");
        println!("  Embedded : {embedded}");
        println!("  Signing  : {signing_pub}");
        println!("  Update unitrace_db_core/public_key.hex dan rebuild binary.");
    }
    Ok(())
}

fn parse_embedded_pub_hex() -> String {
    let trimmed = EMBEDDED_PUBLIC_KEY_HEX.trim();
    if trimmed.len() == 64 {
        trimmed.to_string()
    } else {
        String::from("<placeholder>")
    }
}

fn cmd_verify(args: &[String]) -> Result<(), Box<dyn CoreError>> {
    let path = args.first().ok_or("usage: verify <in.upkg>")?;
    let pkg = Package::read(Path::new(path))?;
    let pk = resolve_public_key().ok_or(
        "embedded public key belum dikonfigurasi (still placeholder). Update public_key.hex lalu rebuild.",
    )?;
    verify_bytes(&pk, &pkg.db, &pkg.meta, &pkg.signature)?;
    println!("OK — signature valid. Package asli dari developer.");
    Ok(())
}

fn cmd_inspect(args: &[String]) -> Result<(), Box<dyn CoreError>> {
    let path = args.first().ok_or("usage: inspect <in.upkg>")?;
    let pkg = Package::read(Path::new(path))?;
    let db_size = pkg.db.len();
    let meta_size = pkg.meta.len();
    let sig_ok = resolve_public_key()
        .is_some_and(|pk| verify_bytes(&pk, &pkg.db, &pkg.meta, &pkg.signature).is_ok());
    println!("Package: {path}");
    println!("  db size         : {db_size} bytes");
    println!("  meta size       : {meta_size} bytes");
    println!("  signature       : {}", hex::encode(&pkg.signature[..8]));
    println!("  signature valid : {sig_ok}");
    if let Ok(text) = core::str::from_utf8(&pkg.meta)
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(text)
    {
        println!();
        println!("Metadata:");
        if let Some(v) = json.get("data_version") {
            println!("  data_version    : {v}");
        }
        if let Some(v) = json.get("row_count") {
            println!("  row_count       : {v}");
        }
        if let Some(v) = json.get("built_at") {
            println!("  built_at        : {v}");
        }
        if let Some(v) = json.get("checksum_sha256") {
            println!("  checksum_sha256 : {v}");
        }
    }
    println!();
    println!("Signature constant: {SIGNATURE_LEN} bytes (Ed25519).");
    Ok(())
}
