use crate::state::ViewState;

pub trait GuiController {
    fn search(&self, keyword: &str) -> ViewState;
}
