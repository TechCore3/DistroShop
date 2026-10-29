use dioxus::prelude::*;
use serde_json;
use serde::{self, Deserialize};

#[derive(Deserialize, Clone, PartialEq)]
pub struct Settings{
    check_hash: bool,
    use_udisks2: bool,
}

static CSS: &str = include_str!("../../assets/main.css");

#[component]
pub fn options_menu() -> Element {
    rsx! {}
}