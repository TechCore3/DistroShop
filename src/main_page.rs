use crate::flashing::flash_handler;
use crate::options::options_menu::*;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
const DISTROLISTGITHUB: &str =
    "https://raw.githubusercontent.com/TechCore3/DistroShop/refs/heads/main/assets/distros.json";
static CSS: &str = include_str!("../assets/main.css");

#[derive(Deserialize, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub struct distro {
    pub id: u8,
    pub name: String,
    pub description: String,
    pub descriptionfull: String,
    pub image: String,
    pub downloadlink: String,
    pub filename: String,
    pub sha256: String,
}



// // // // // // // // // // // // // // // //


pub fn get_cache_dir() -> PathBuf {
    let mut cache_dir = dirs::cache_dir().expect("unable to find cache directory!");
    cache_dir.push("distroshop"); 
    cache_dir
}

pub fn get_config_dir() -> PathBuf {
    let mut config_dir = dirs::config_dir().expect("unable to find cache directory!");
    config_dir.push("distroshop"); 
    config_dir
}




async fn load_list() -> Result<Vec<distro>, Box<dyn std::error::Error>> {
    let response = reqwest::get(DISTROLISTGITHUB).await?;
    let json_text = response.text().await?;
    let distros: Vec<distro> = serde_json::from_str(&json_text)?;

    Ok(distros)
}
#[component]
pub fn distro_list() -> Element {
    let mut items_signal = use_signal(Vec::<distro>::new);
    let mut status_signal = use_signal(|| String::from("Initializing..."));
    let mut is_loading = use_signal(|| false);
    let mut active_distro_id = use_signal(|| None::<u8>);
    let mut is_showing_more = use_signal(|| false);
    let mut is_showing_options = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
                info!("Downloading list");
                status_signal.set("Downloading...".to_string());
                is_loading.set(true);
                let cachedir = get_cache_dir();
                if !cachedir.exists(){
                fs::create_dir(cachedir).expect("unable to create cache directory! (~/.cache/distroshop/)");
                }
                let configdir = get_config_dir();
                if !configdir.exists(){
                fs::create_dir(configdir).expect("unable to create config directory! (~/.config/distroshop/)");
                }
                if !get_config_dir().join("config.json").exists() {
                    match download_default_config().await {
                    Ok(()) => {
                        status_signal.set("".to_string());
                    }
                    Err(e) => {
                        error!("download failed: {e}");
                        status_signal.set(format!("Download failed! {e}"));
                    }
                }
                }
                match load_list().await {
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
            
        });
    });
    let handle_manual_sync = move |_| {
        to_owned![items_signal, status_signal, is_loading];
        spawn(async move {
            is_loading.set(true);
            status_signal.set("Downloading data from GitHub...".to_string());

            match load_list().await {
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
    //let settings = parse_config().unwrap_or_default();
    rsx! {
        document::Style { "{CSS}" }
        div { class: "app-shell",
            //
            
            div { class: "toolbar",
                div { class: "toolbar-copy",
                    h1 { class: "app-title", "DistroShop" }
                }
                if is_showing_options(){
                    div {
                        class: "primary-button", onclick: move |_| {is_showing_options.set(false)}, "Back"
                    }
                }
            if !is_showing_options() {
                div { class: "toolbar-actions",
                    button {
                        class: "primary-button", onclick: move |_| is_showing_options.set(true) ,"Options"
                    }
                    button { class: "primary-button", onclick: handle_manual_sync, disabled: is_loading(),
                        if is_loading() {
                            "Syncing..."
                        } else {
                            "Refresh list"
                        }
                    }
                }
            }
            }
            if !is_showing_options() {
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
            } else {
                options_menu {  }
            }
        }
    }
}
