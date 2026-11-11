use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::checksum::Sha256Checksum;
use unitrace_db_handler::Error;
use unitrace_db_handler::traits::Checksum;

pub const SIGNATURE_LEN: usize = 64;
pub const PUBLIC_KEY_HEX_LEN: usize = 64;

pub const EMBEDDED_PUBLIC_KEY_HEX: &str = include_str!("../public_key.hex");

fn parse_hex32(s: &str) -> Option<[u8; 32]> {
    let trimmed = s.trim();
    if trimmed.len() != 64 {
        return None;
    }
    let bytes = hex::decode(trimmed).ok()?;
    bytes.try_into().ok()
}

#[must_use]
pub fn resolve_public_key() -> Option<VerifyingKey> {
    if let Ok(env_key) = std::env::var("UNITRACE_PUBLIC_KEY")
        && let Some(arr) = parse_hex32(&env_key)
    {
        return VerifyingKey::from_bytes(&arr).ok();
    }
    let arr = parse_hex32(EMBEDDED_PUBLIC_KEY_HEX)?;
    if arr == [0u8; 32] {
        return None;
    }
    VerifyingKey::from_bytes(&arr).ok()
}

#[must_use]
pub fn combine_hash(db: &[u8], meta: &[u8]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(16usize.saturating_add(db.len()).saturating_add(meta.len()));
    buf.extend_from_slice(&(db.len() as u64).to_le_bytes());
    buf.extend_from_slice(&(meta.len() as u64).to_le_bytes());
    buf.extend_from_slice(db);
    buf.extend_from_slice(meta);
    Sha256Checksum::new().checksum(&buf)
}

#[must_use]
pub fn sign_bytes(sk: &SigningKey, db: &[u8], meta: &[u8]) -> [u8; SIGNATURE_LEN] {
    sk.sign(&combine_hash(db, meta)).to_bytes()
}

pub fn verify_bytes(
    pk: &VerifyingKey,
    db: &[u8],
    meta: &[u8],
    sig: &[u8; SIGNATURE_LEN],
) -> Result<(), Error> {
    let signature = Signature::from_bytes(sig);
    pk.verify(&combine_hash(db, meta), &signature).map_err(|_| {
        Error::PackageInvalid(
            "signature verification failed: package bukan dari developer resmi, atau sudah diubah"
                .into(),
        )
    })
}

pub fn sign_files(
    sk: &SigningKey,
    db_path: &Path,
    meta_path: &Path,
) -> Result<[u8; SIGNATURE_LEN], Error> {
    let db = std::fs::read(db_path).map_err(|e| Error::Io(e.to_string()))?;
    let meta = std::fs::read(meta_path).map_err(|e| Error::Io(e.to_string()))?;
    Ok(sign_bytes(sk, &db, &meta))
}

pub fn verify_files(
    pk: &VerifyingKey,
    db_path: &Path,
    meta_path: &Path,
    sig_path: &Path,
) -> Result<(), Error> {
    let db = std::fs::read(db_path).map_err(|e| Error::Io(e.to_string()))?;
    let meta = std::fs::read(meta_path).map_err(|e| Error::Io(e.to_string()))?;
    let sig = std::fs::read(sig_path).map_err(|e| Error::Io(e.to_string()))?;
    let arr: [u8; SIGNATURE_LEN] = sig
        .try_into()
        .map_err(|_| Error::PackageInvalid("signature file must be 64 bytes".into()))?;
    verify_bytes(pk, &db, &meta, &arr)
}
