use crate::flashing::flash_handler;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
const DISTROLISTGITHUB: &str =
    "https://raw.githubusercontent.com/TechCore3/DistroShop/refs/heads/main/assets/distros.json";
static CSS: &str = include_str!("../assets/main.css");

#[derive(Deserialize, Clone, PartialEq)]
#[allow(non_camel_case_types)] // cant be bothered to change case at this point in development lol
pub struct distro {
    pub id: u8, //might expand later who knows
    pub name: String,
    pub description: String,
    pub descriptionfull: String,
    pub image: String,
    pub downloadlink: String,
    pub filename: String,
}



// // // // // // // // // // // // // // // //

pub fn get_config_dir() -> PathBuf {
    let mut home_dir = dirs::home_dir().expect("unable to find home dir!");
    home_dir.push(".config/distroshop/"); // %LOCALAPPDATA% be damned
    home_dir
}

fn get_local_distro_list() -> PathBuf {
    let mut path = get_config_dir();
    path.push("distros.json");
    path
}

fn read_from_disk() -> Result<Vec<distro>, Box<dyn std::error::Error>> {
    let file_path = get_local_distro_list();
    info!("reading from cached JSON file");
    let file_content = fs::read_to_string(file_path)?;
    let items: Vec<distro> = serde_json::from_str(&file_content)?;

    Ok(items)
}

async fn download_and_save_list() -> Result<Vec<distro>, Box<dyn std::error::Error>> {
    let response = reqwest::get(DISTROLISTGITHUB).await?;
    let json_text = response.text().await?;
    let distros: Vec<distro> = serde_json::from_str(&json_text)?;

    let file_path = get_local_distro_list();
    fs::write(&file_path, json_text)?;

    Ok(distros)
}
#[component]
pub fn distro_list() -> Element {
    let mut items_signal = use_signal(Vec::<distro>::new);
    let mut status_signal = use_signal(|| String::from("Initializing..."));
    let mut is_loading = use_signal(|| false);
    let mut active_distro_id = use_signal(|| None::<u8>);
    let mut is_showing_more = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let file_path = get_local_distro_list();

            if file_path.exists() {
                info!("list found locally. loading from disk.");
                match read_from_disk() {
                    Ok(items) => {
                        items_signal.set(items);
                        status_signal.set("".to_string());
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
                let configdir = get_config_dir();
                fs::create_dir(configdir).expect("unable to create config directory! (~/.config/distroshop/)");
                match download_and_save_list().await {
                    Ok(items) => {
                        items_signal.set(items);
                        status_signal.set("".to_string());
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
                    status_signal.set("".to_string());
                }
                Err(e) => {
                    error!("Manual refresh failed: {e}");
                    status_signal.set(format!("Sync error: {e}"));
                    status_signal.set("".to_string());
                }
            }
            is_loading.set(false);
        });
    };
    let distros = items_signal.read();
    rsx! {
        document::Style { "{CSS}" }
        div { class: "app-shell",
            div { class: "toolbar",
                div { class: "toolbar-copy",
                    h1 { class: "app-title", "DistroShop" }
                }
                div { class: "toolbar-actions",
                    button { class: "primary-button", onclick: handle_manual_sync, disabled: is_loading(),
                        if is_loading() {
                            "Syncing..."
                        } else {
                            "Refresh list"
                        }
                    }
                }
            }
            p { class: "status-line", "{status_signal}" }
            
            div {
                id: "distrolist",
                for distro in distros.iter().cloned().collect::<Vec<_>>() {
                    
                    li { class: "distro-card",
                        div { class: "distro-image-frame",
                            img {
                                class: "distro-image",
                                src: "{distro.image}",
                            }
                        }
                        div { class: "distro-details",
                            h3 { class: "distro-title", "{distro.name}" }
                            p { class: "distro-description", "{distro.description}" }
                        }
                        div { class: "distro-actions",
                            button {
                                class: "distro-button",
                                onclick: move |_| {
                                    active_distro_id.set(Some(distro.id));
                                    is_showing_more.set(true);
                                },
                                "Show more"
                            }
                        }
                    }
                    if active_distro_id() == Some(distro.id) && is_showing_more() {
                        flash_handler::show_more { distro: distro.clone(), is_showing: is_showing_more }
                    }
                }
            }
        }
    }
}
