use std::path::Path;

use rusqlite::{Connection, ErrorCode};

use unitrace_db_handler::EncryptionKey;
use unitrace_db_handler::Error;

pub fn open_encrypted(path: &Path, key: &EncryptionKey) -> Result<Connection, Error> {
    let conn = Connection::open(path).map_err(|e| classify(&e))?;
    apply_key(&conn, key)?;
    verify_cipher(&conn)?;
    probe_connection(&conn)?;
    Ok(conn)
}

pub fn apply_key(conn: &Connection, key: &EncryptionKey) -> Result<(), Error> {
    conn.pragma_update(None, "key", key.as_str())
        .map_err(|e| classify(&e))?;
    Ok(())
}

pub fn verify_cipher(conn: &Connection) -> Result<(), Error> {
    match conn.query_row("PRAGMA cipher_version;", [], |row| row.get::<_, String>(0)) {
        Ok(version) if !version.trim().is_empty() => Ok(()),
        Ok(_) => Err(Error::CipherUnavailable),
        Err(_) => Err(Error::CipherUnavailable),
    }
}

pub fn probe_connection(conn: &Connection) -> Result<(), Error> {
    let _table_count: i64 = conn
        .query_row("SELECT count(*) FROM sqlite_master;", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| classify(&e))?;
    Ok(())
}

fn classify(err: &rusqlite::Error) -> Error {
    if let rusqlite::Error::SqliteFailure(code, _) = err
        && code.code == ErrorCode::NotADatabase
    {
        return Error::InvalidKey;
    }
    Error::Sqlite(err.to_string())
}
