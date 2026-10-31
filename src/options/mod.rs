use serde::{Deserialize, Serialize};
use std::path::Path;
pub mod options_menu;
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Settings{
    pub check_hash: bool,
    pub use_udisks2: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            check_hash: true,
            use_udisks2: Path::new("/.flatpak-info").exists(),
        }
    }
}