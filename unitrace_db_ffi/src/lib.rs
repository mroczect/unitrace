use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::time::{Duration, Instant};

use serde::Serialize;
use tempfile::TempDir;

use unitrace_db::{
    DatabaseReader, DefaultGuiController, DefaultSearcher, EncryptionKey, GuiController,
    OpenConfig, Package, ViewState, resolve_public_key, verify_bytes,
};

thread_local! {
    static STATE: RefCell<Option<Loaded>> = const { RefCell::new(None) };
    static FAILURES: RefCell<Vec<Instant>> = const { RefCell::new(Vec::new()) };
}

const MAX_FAILED_ATTEMPTS: usize = 3;
const WINDOW: Duration = Duration::from_secs(60);
const FAIL_DELAY: Duration = Duration::from_millis(1500);

struct Loaded {
    controller: DefaultGuiController,
    _tempdir: TempDir,
}

#[derive(Serialize)]
struct InitResponse {
    status: String,
    message: String,
    columns: Vec<String>,
    signature_verified: bool,
}

#[derive(Serialize)]
struct FfiNotFound {
    keyword: String,
    searched_rows: u64,
    searched_columns: u64,
    database_version: String,
}

#[derive(Serialize)]
struct FfiViewState {
    status_text: String,
    rows: Vec<Vec<String>>,
    not_found_proof: Option<FfiNotFound>,
}

impl From<ViewState> for FfiViewState {
    fn from(s: ViewState) -> Self {
        Self {
            status_text: s.status_text,
            rows: s.rows.into_iter().map(|r| r.columns).collect(),
            not_found_proof: s.not_found_proof.map(|p| FfiNotFound {
                keyword: p.keyword,
                searched_rows: p.searched_rows,
                searched_columns: p.searched_columns,
                database_version: p.database_version,
            }),
        }
    }
}

fn to_c(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn fail(msg: &str, sig_verified: bool) -> *mut c_char {
    to_c(
        serde_json::to_string(&InitResponse {
            status: "error".into(),
            message: msg.into(),
            columns: Vec::new(),
            signature_verified: sig_verified,
        })
        .unwrap_or_else(|_| "{}".into()),
    )
}

fn rate_ok() -> Result<(), String> {
    FAILURES.with(|v| {
        let mut v = v.borrow_mut();
        let now = Instant::now();
        v.retain(|t| now.duration_since(*t) < WINDOW);
        if v.len() >= MAX_FAILED_ATTEMPTS {
            let wait = WINDOW.saturating_sub(now.duration_since(v[0]));
            return Err(format!(
                "terlalu banyak percobaan. Coba lagi dalam {} detik.",
                wait.as_secs().max(1)
            ));
        }
        v.push(now);
        Ok(())
    })
}

#[cfg(unix)]
fn check_perms(path: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let md = std::fs::metadata(path).map_err(|e| format!("stat {path}: {e}"))?;
    let mode = md.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(format!(
            "{path} permission {mode:o} terlalu longgar (butuh 600). Jalankan: chmod 600 {path}"
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_perms(_path: &str) -> Result<(), String> {
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unitrace_init(
    package_path: *const c_char,
    key: *const c_char,
) -> *mut c_char {
    if let Err(msg) = rate_ok() {
        return fail(&msg, false);
    }
    if package_path.is_null() || key.is_null() {
        return fail("null pointer passed to unitrace_init", false);
    }
    let pkg_path = unsafe { CStr::from_ptr(package_path) }
        .to_str()
        .unwrap_or("");
    let pass = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");

    if let Err(e) = check_perms(pkg_path) {
        return fail(&e, false);
    }

    let pkg = match Package::read(std::path::Path::new(pkg_path)) {
        Ok(p) => p,
        Err(e) => return fail(&format!("package read error: {e}"), false),
    };

    let pk = match resolve_public_key() {
        Some(k) => k,
        None => {
            return fail(
                "public key belum dikonfigurasi di binary (still placeholder). Developer harus update public_key.hex dan rebuild.",
                false,
            );
        }
    };
    if let Err(e) = verify_bytes(&pk, &pkg.db, &pkg.meta, &pkg.signature) {
        std::thread::sleep(FAIL_DELAY);
        return fail(&format!("signature invalid: {e}"), false);
    }

    let tmp = match TempDir::new() {
        Ok(t) => t,
        Err(e) => return fail(&format!("tempdir error: {e}"), true),
    };
    let db_path = tmp.path().join("data.db");
    let meta_path = tmp.path().join("data.meta.json");
    if let Err(e) = std::fs::write(&db_path, &pkg.db) {
        return fail(&format!("write db: {e}"), true);
    }
    if let Err(e) = std::fs::write(&meta_path, &pkg.meta) {
        return fail(&format!("write meta: {e}"), true);
    }

    let config = match OpenConfig::try_new(
        db_path,
        meta_path,
        match EncryptionKey::try_new(pass.to_string()) {
            Ok(k) => k,
            Err(e) => {
                std::thread::sleep(FAIL_DELAY);
                return fail(&format!("invalid key: {e}"), true);
            }
        },
    ) {
        Ok(c) => c,
        Err(e) => return fail(&format!("config error: {e}"), true),
    };

    let searcher = match DefaultSearcher::open(&config) {
        Ok(s) => s,
        Err(e) => {
            std::thread::sleep(FAIL_DELAY);
            return fail(&format!("db open error: {e}"), true);
        }
    };

    let columns: Vec<String> = searcher
        .schema()
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    let controller = DefaultGuiController::new(searcher);
    STATE.with(|s| {
        *s.borrow_mut() = Some(Loaded {
            controller,
            _tempdir: tmp,
        });
    });

    FAILURES.with(|v| v.borrow_mut().clear());

    to_c(
        serde_json::to_string(&InitResponse {
            status: "ok".into(),
            message: "package opened (signature verified)".into(),
            columns,
            signature_verified: true,
        })
        .unwrap_or_else(|_| "{}".into()),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unitrace_search(keyword: *const c_char) -> *mut c_char {
    if keyword.is_null() {
        return std::ptr::null_mut();
    }
    let kw = unsafe { CStr::from_ptr(keyword) }.to_str().unwrap_or("");
    STATE.with(|s| {
        let guard = s.borrow();
        let Some(loaded) = guard.as_ref() else {
            return std::ptr::null_mut();
        };
        let state = loaded.controller.search(kw);
        to_c(serde_json::to_string(&FfiViewState::from(state)).unwrap_or_else(|_| "{}".into()))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unitrace_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn unitrace_close() {
    STATE.with(|s| {
        *s.borrow_mut() = None;
    });
}
