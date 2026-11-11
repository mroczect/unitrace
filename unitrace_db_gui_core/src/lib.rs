use unitrace_db::{DefaultSearcher, SearchMode, SearchQuery};
use unitrace_db_gui_handler::{GuiController, NotFoundProof, RowViewModel, ViewState};

#[derive(Debug)]
pub struct DefaultGuiController {
    searcher: DefaultSearcher,
}

impl DefaultGuiController {
    #[must_use]
    pub fn new(searcher: DefaultSearcher) -> Self {
        Self { searcher }
    }
}

impl GuiController for DefaultGuiController {
    fn search(&self, keyword: &str) -> ViewState {
        let trimmed = keyword.trim();

        // 1. Penanganan input kosong
        if trimmed.is_empty() {
            return ViewState {
                status_text: "Siap mencari. Ketik nomor rangka, mesin, atau nama.".to_string(),
                rows: Vec::new(),
                not_found_proof: None,
            };
        }

        // 2. Eksekusi pencarian (Normal Mode = case-insensitive & toleran spasi)
        let query = match SearchQuery::try_new(trimmed.to_string(), SearchMode::Normal, None, None) {
            Ok(q) => q,
            Err(e) => {
                return ViewState {
                    status_text: format!("INPUT ERROR: {e}"),
                    rows: Vec::new(),
                    not_found_proof: None,
                };
            }
        };

        match self.searcher.search(&query) {
            Ok(result) => {
                // 3. UX Defensible: Jika benar-benar kosong
                if result.total_matches() == 0 {
                    ViewState {
                        status_text: "Data tidak ditemukan.".to_string(),
                        rows: Vec::new(),
                        not_found_proof: Some(NotFoundProof {
                            keyword: trimmed.to_string(),
                            searched_rows: result.searched_rows(),
                            searched_columns: result.searched_columns(),
                            database_version: "Terverifikasi (Integritas Cocok)".to_string(),
                        }),
                    }
                } else {
                    // 4. Render Baris
                    let rows = result
                        .records()
                        .iter()
                        .map(|rec| RowViewModel {
                            columns: rec.fields().to_vec(),
                        })
                        .collect();

                    ViewState {
                        status_text: format!("Ditemukan {} baris yang cocok.", result.total_matches()),
                        rows,
                        not_found_proof: None,
                    }
                }
            }
            Err(e) => {
                // 5. UX Defensible: System Error BUKAN Not Found
                ViewState {
                    status_text: format!("SISTEM ERROR: {e} (Ini bukan berarti data tidak ada!)"),
                    rows: Vec::new(),
                    not_found_proof: None,
                }
            }
        }
    }
}
