use dioxus::html::a::download;
use dioxus::logger::tracing::dispatcher::get_default;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;

const DISTROLISTGITHUB: &str =
    "https://raw.githubusercontent.com/TechCore3/DistroShop/refs/heads/main/assets/distros.json";
const CSS: Asset = asset!("/assets/main.css");

#[derive(Deserialize, Clone, PartialEq)]
pub struct Distro {
    id: u8, //might expand later who knows
    name: String,
    description: String,
    image: String,
}
// // // // // // // // // // // // // // // //
fn get_local_distro_list() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("distros.json");
    path
}

fn read_from_disk() -> Result<Vec<Distro>, Box<dyn std::error::Error>> {
    let file_path = get_local_distro_list();
    info!("reading from cached JSON file");
    let file_content = fs::read_to_string(file_path)?;
    let items: Vec<Distro> = serde_json::from_str(&file_content)?;

    Ok(items)
}

async fn download_and_save_list() -> Result<Vec<Distro>, Box<dyn std::error::Error>> {
    let response = reqwest::get(DISTROLISTGITHUB).await?;
    let json_text = response.text().await?;
    let distros: Vec<Distro> = serde_json::from_str(&json_text)?;

    let file_path = get_local_distro_list();
    fs::write(&file_path, json_text)?;

    Ok(distros)
}
#[component]
pub fn distro_list() -> Element {
    let mut items_signal = use_signal(Vec::<Distro>::new);
    let mut status_signal = use_signal(|| String::from("Initializing..."));
    let mut is_loading = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let file_path = get_local_distro_list();

            if file_path.exists() {
                info!("list found locally. loading from disk.");
                match read_from_disk() {
                    Ok(items) => {
                        items_signal.set(items);
                        status_signal.set("Loaded cached list".to_string());
                    }
                    Err(e) => {
                        error!("failed to read cached list: {e}");
                        status_signal.set(format!("Cache read failed: {e}"));
                    }
                }
            } else {
                info!("list not found locally. downloading.");
                status_signal.set("Downloading...".to_string());
                is_loading.set(true);

                match download_and_save_list().await {
                    Ok(items) => {
                        items_signal.set(items);
                        status_signal.set("Download complete!".to_string());
                        status_signal.set("Refresh".to_string());
                    }
                    Err(e) => {
                        error!("download failed: {e}");
                        status_signal.set(format!("Download failed! {e}"));
                    }
                }
                is_loading.set(false);
            }
        });
    });
    let handle_manual_sync = move |_| {
        to_owned![items_signal, status_signal, is_loading];
        spawn(async move {
            is_loading.set(true);
            status_signal.set("Downloading data from GitHub...".to_string());

            match download_and_save_list().await {
                Ok(items) => {
                    items_signal.set(items);
                    info!("successfully updated local list");
                    status_signal.set("Updated local list!".to_string());
                    status_signal.set("Refresh".to_string());
                }
                Err(e) => {
                    error!("Manual refresh failed: {e}");
                    status_signal.set(format!("Sync error: {e}"));
                    status_signal.set("Refresh".to_string());
                }
            }
            is_loading.set(false);
        });
    };
    rsx! {
        div { class: "container",
            button { onclick: handle_manual_sync, disabled: is_loading(),
                if is_loading() {
                    "Syncing..."
                } else {
                    "Refresh"
                }
            }
            p { "{status_signal}" }
        }
        div {
            id: "distrolist",
            style: "display: grid; grid-template-columns: repeat(2,1fr); gap: 12px; list-style-type: none; padding:0;",
            for distro in items_signal.read().iter() {
                li { style: "border: 3px solid #7a7a7a;",
                    img {
                        style: "max-width:100px; max-height:100px; width: auto; height:auto;",
                        src: distro.image.clone(),
                    }
                    h3 { "{distro.name}" }
                    p { "{distro.description}" }
                    button { "Download and Flash" } //will use dd and tokio
                }
            }
        }
    }
}
