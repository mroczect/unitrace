use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde::Serialize;
use unitrace_db::{DatabaseReader, DefaultSearcher, EncryptionKey, OpenConfig};
use unitrace_db_gui_core::DefaultGuiController;
use unitrace_db_gui_handler::{GuiController, ViewState};

thread_local! {
    static CONTROLLER: RefCell<Option<DefaultGuiController>> = const { RefCell::new(None) };
}

#[derive(Serialize)]
struct InitResponse {
    status: String,
    message: String,
    columns: Vec<String>,
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
    fn from(state: ViewState) -> Self {
        Self {
            status_text: state.status_text,
            rows: state.rows.into_iter().map(|r| r.columns).collect(),
            not_found_proof: state.not_found_proof.map(|p| FfiNotFound {
                keyword: p.keyword,
                searched_rows: p.searched_rows,
                searched_columns: p.searched_columns,
                database_version: p.database_version,
            }),
        }
    }
}

fn to_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unitrace_init(
    db_path: *const c_char,
    meta_path: *const c_char,
    key: *const c_char,
) -> *mut c_char {
    let fail = |msg: &str| {
        to_c_string(
            serde_json::to_string(&InitResponse {
                status: "error".to_string(),
                message: msg.to_string(),
                columns: Vec::new(),
            })
            .unwrap_or_else(|_| "{}".to_string()),
        )
    };

    if db_path.is_null() || meta_path.is_null() || key.is_null() {
        return fail("Null pointer passed to unitrace_init");
    }

    let db = unsafe { CStr::from_ptr(db_path) }.to_str().unwrap_or("");
    let meta = unsafe { CStr::from_ptr(meta_path) }.to_str().unwrap_or("");
    let k = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");

    let config = match OpenConfig::try_new(
        db.into(),
        meta.into(),
        match EncryptionKey::try_new(k.to_string()) {
            Ok(k) => k,
            Err(e) => return fail(&format!("Invalid key: {e}")),
        },
    ) {
        Ok(c) => c,
        Err(e) => return fail(&format!("Invalid config: {e}")),
    };

    let searcher = match DefaultSearcher::open(&config) {
        Ok(s) => s,
        Err(e) => return fail(&format!("Database error: {e}")),
    };

    let columns: Vec<String> = searcher
        .schema()
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    let controller = DefaultGuiController::new(searcher);

    CONTROLLER.with(|c| {
        *c.borrow_mut() = Some(controller);
    });

    to_c_string(
        serde_json::to_string(&InitResponse {
            status: "ok".to_string(),
            message: "Database opened successfully".to_string(),
            columns,
        })
        .unwrap_or_else(|_| "{}".to_string()),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unitrace_search(keyword: *const c_char) -> *mut c_char {
    if keyword.is_null() {
        return std::ptr::null_mut();
    }

    let kw = unsafe { CStr::from_ptr(keyword) }.to_str().unwrap_or("");

    CONTROLLER.with(|c| {
        let guard = c.borrow();
        let controller = match guard.as_ref() {
            Some(ctrl) => ctrl,
            None => return std::ptr::null_mut(),
        };

        let state = controller.search(kw);
        let ffi_state = FfiViewState::from(state);
        let json = serde_json::to_string(&ffi_state).unwrap_or_else(|_| "{}".to_string());

        to_c_string(json)
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
