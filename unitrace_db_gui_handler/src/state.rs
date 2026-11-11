#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowViewModel {
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotFoundProof {
    pub keyword: String,
    pub searched_rows: u64,
    pub searched_columns: u64,
    pub database_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewState {
    pub status_text: String,
    pub rows: Vec<RowViewModel>,
    pub not_found_proof: Option<NotFoundProof>,
}
