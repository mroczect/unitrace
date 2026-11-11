use std::path::Path;

use unitrace_db_handler::Error;

use crate::signature::SIGNATURE_LEN;

const MAGIC: &[u8; 8] = b"UPTRACE\0";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 8 + 2 + 2 + 8 + 8 + 8;

#[derive(Debug, Clone)]
pub struct Package {
    pub db: Vec<u8>,
    pub meta: Vec<u8>,
    pub signature: [u8; SIGNATURE_LEN],
}

impl Package {
    #[must_use]
    pub const fn new(db: Vec<u8>, meta: Vec<u8>, signature: [u8; SIGNATURE_LEN]) -> Self {
        Self {
            db,
            meta,
            signature,
        }
    }

    pub fn write(&self, path: &Path) -> Result<(), Error> {
        let bytes = self.to_bytes();
        std::fs::write(path, &bytes).map_err(|e| Error::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)
                .map_err(|e| Error::Io(e.to_string()))?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(path, perms).map_err(|e| Error::Io(e.to_string()))?;
        }
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::PackageInvalid(format!("package not found: {}", path.display()))
            } else {
                Error::Io(e.to_string())
            }
        })?;
        Self::from_bytes(&bytes)
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(
            HEADER_LEN
                .saturating_add(self.db.len())
                .saturating_add(self.meta.len())
                .saturating_add(SIGNATURE_LEN),
        );
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(self.db.len() as u64).to_le_bytes());
        out.extend_from_slice(&(self.meta.len() as u64).to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
        out.extend_from_slice(&self.db);
        out.extend_from_slice(&self.meta);
        out.extend_from_slice(&self.signature);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_LEN.saturating_add(SIGNATURE_LEN) {
            return Err(Error::PackageInvalid("file too short".into()));
        }

        let magic = bytes
            .get(..8)
            .ok_or_else(|| Error::PackageInvalid("magic slice out of range".into()))?;
        if magic != MAGIC {
            return Err(Error::PackageInvalid(
                "magic mismatch (not a .upkg file)".into(),
            ));
        }

        let version_bytes = bytes
            .get(8..10)
            .ok_or_else(|| Error::PackageInvalid("version slice out of range".into()))?;
        let version_arr: [u8; 2] = version_bytes
            .try_into()
            .map_err(|_| Error::PackageInvalid("version parse failed".into()))?;
        let version = u16::from_le_bytes(version_arr);
        if version != VERSION {
            return Err(Error::PackageInvalid(format!(
                "package version {version}, expected {VERSION}"
            )));
        }

        let db_len_bytes = bytes
            .get(12..20)
            .ok_or_else(|| Error::PackageInvalid("db_len slice out of range".into()))?;
        let db_len_arr: [u8; 8] = db_len_bytes
            .try_into()
            .map_err(|_| Error::PackageInvalid("db_len parse failed".into()))?;
        let db_len = usize::try_from(u64::from_le_bytes(db_len_arr))
            .map_err(|_| Error::PackageInvalid("db_len exceeds platform usize".into()))?;

        let meta_len_bytes = bytes
            .get(20..28)
            .ok_or_else(|| Error::PackageInvalid("meta_len slice out of range".into()))?;
        let meta_len_arr: [u8; 8] = meta_len_bytes
            .try_into()
            .map_err(|_| Error::PackageInvalid("meta_len parse failed".into()))?;
        let meta_len = usize::try_from(u64::from_le_bytes(meta_len_arr))
            .map_err(|_| Error::PackageInvalid("meta_len exceeds platform usize".into()))?;

        let expected_total = HEADER_LEN
            .checked_add(db_len)
            .and_then(|x| x.checked_add(meta_len))
            .and_then(|x| x.checked_add(SIGNATURE_LEN))
            .ok_or_else(|| Error::PackageInvalid("size arithmetic overflow".into()))?;
        if bytes.len() != expected_total {
            return Err(Error::PackageInvalid(format!(
                "size mismatch: header says {expected_total} bytes, file is {} bytes",
                bytes.len()
            )));
        }

        let db_start = HEADER_LEN;
        let db_end = db_start
            .checked_add(db_len)
            .ok_or_else(|| Error::PackageInvalid("db offset overflow".into()))?;
        let meta_end = db_end
            .checked_add(meta_len)
            .ok_or_else(|| Error::PackageInvalid("meta offset overflow".into()))?;
        let sig_end = meta_end
            .checked_add(SIGNATURE_LEN)
            .ok_or_else(|| Error::PackageInvalid("sig offset overflow".into()))?;

        let db = bytes
            .get(db_start..db_end)
            .ok_or_else(|| Error::PackageInvalid("db slice out of range".into()))?
            .to_vec();
        let meta = bytes
            .get(db_end..meta_end)
            .ok_or_else(|| Error::PackageInvalid("meta slice out of range".into()))?
            .to_vec();
        let signature: [u8; SIGNATURE_LEN] = bytes
            .get(meta_end..sig_end)
            .ok_or_else(|| Error::PackageInvalid("signature slice out of range".into()))?
            .try_into()
            .map_err(|_| Error::PackageInvalid("signature slice wrong size".into()))?;

        Ok(Self {
            db,
            meta,
            signature,
        })
    }
}
