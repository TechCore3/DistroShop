use dioxus::prelude::*;
use serde_json;
use std::io::Write;
use std::fs::{self, File};
use crate::options::Settings;
use crate::main_page::get_config_dir;

static CSS: &str = include_str!("../../assets/main.css");

pub async fn write_default_config() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let default = Settings::default();
    let json_text = serde_json::to_string_pretty(&default).unwrap();
    let mut file = File::create(get_config_dir().join("config.json"))?;
    file.write_all(json_text.as_bytes())?;
    Ok(())
}

pub fn parse_config() -> Result<Settings, Box<dyn std::error::Error>> {
    let json_text = fs::read_to_string(get_config_dir().join("config.json"))?;
    let settings: Settings = serde_json::from_str(&json_text)?;
    Ok(settings)
}

fn save_changes(settings: &Settings) -> Result<()> {
    let json_data = serde_json::to_string_pretty(&settings).unwrap();
    let mut file = File::create(get_config_dir().join("config.json"))?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

#[component]
pub fn options_menu() -> Element {
    let mut settings = use_signal(||parse_config().unwrap_or_default());
    let mut status = use_signal(||"".to_string());
    rsx! {
        document::Style { "{CSS}" }
        div{ class: "modal-form",
        div{
        h1 {"Check sha256 hash: {settings().check_hash}"}
        button{class: "secondary-button", onclick: move |_| {
            let mut settings = settings.write();
            settings.check_hash = !settings.check_hash;
        },"toggle"}
        }
        div{
        h1 {"Use udisks2 for flashing: {settings().use_udisks2}"}
        button{class: "secondary-button", onclick: move |_| {
            let mut settings = settings.write();
            settings.use_udisks2 = !settings.use_udisks2;
        },"toggle"}
        }
    }
        div { class: "toolbar",
                div { class: "toolbar-copy",
                }
            }
            button {
                    class: "primary-button" , onclick: move |_| {
                        match save_changes(&settings.read()){
                            Ok(_) => status.set("successfully saved changes!".to_string()),
                            Err(e) => status.set(format!("writing file failed: {e}").to_string())
                        }} 
                        , "Save Changes"
                }
            h3 {"{status}"}
    }
}