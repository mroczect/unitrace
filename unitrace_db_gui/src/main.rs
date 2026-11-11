use std::path::PathBuf;
use std::rc::Rc;

use unitrace_db::{DefaultSearcher, EncryptionKey, OpenConfig};
use unitrace_db_gui_core::DefaultGuiController;
use unitrace_db_gui_handler::GuiController;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. SETUP DATABASE ---
    // Sesuai permintaan: murni database only.
    // GUI ini hanya membaca data.db yang sudah dibuild.
    let db_path = PathBuf::from("data.db");
    let meta_path = PathBuf::from("data.meta.json");

    // Di tahap produksi, key ini didapat dari proses aktivasi.
    let key = EncryptionKey::try_new("supersecretkey123".to_string())?;
    let config = OpenConfig::try_new(db_path, meta_path, key)?;

    // Buka Searcher
    let searcher = DefaultSearcher::open(&config)?;
    let controller = Rc::new(DefaultGuiController::new(searcher));

    // --- 2. SETUP UI SLINT ---
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();
    let controller_clone = Rc::clone(&controller);

    // --- 3. HUBUNGKAN SINYAL ---
    ui.on_search(move |keyword| {
        let state = controller_clone.search(&keyword);

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text(state.status_text.as_str().into());

            // Update tabel
            let results_model: Vec<RecordView> = state.rows.iter().map(|row| {
                RecordView {
                    columns: row.columns.iter().map(|s| s.as_str().into()).collect(),
                }
            }).collect();

            ui.set_results(slint::ModelRc::new(slint::VecModel::from(results_model)));

            // Update panel Not Found
            if let Some(proof) = state.not_found_proof {
                ui.set_not_found_keyword(proof.keyword.as_str().into());
                ui.set_not_found_rows(proof.searched_rows as i32);
                ui.set_not_found_cols(proof.searched_columns as i32);
                ui.set_not_found_version(proof.database_version.as_str().into());
            } else {
                ui.set_not_found_keyword("".into());
            }
        }
    });

    // --- 4. JALANKAN ---
    ui.set_status_text("Silakan ketik untuk mencari.".into());
    ui.run()?;

    Ok(())
}
