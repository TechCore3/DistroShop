use dioxus::html::a::download;
use dioxus::logger::tracing::dispatcher::get_default;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;

use crate::list_handler::distro_list;

mod list_handler;

#[component]
fn app() -> Element {
    distro_list()
}
fn main() {
    dioxus::launch(app);
}
