use dioxus::prelude::*;

use crate::list_handler::distro_list;

pub mod flash_handler;
pub mod list_handler;
static CSS: Asset = asset!("/assets/main.css");

#[component]
fn app() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        div { distro_list {} }
    }
}

fn main() {
    dioxus::launch(app);
}
