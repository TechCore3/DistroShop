use serde::Deserialize;

pub mod options_menu;

#[derive(Deserialize, Clone, Copy, PartialEq)]
pub struct Settings{
    pub check_hash: bool,
    pub use_udisks2: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            check_hash: true,
            use_udisks2: false,
        }
    }
}