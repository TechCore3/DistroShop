use dioxus::prelude::*;
use serde_json;
use std::io::Write;
use std::fs::{self, File};
use crate::options::Settings;
use crate::main_page::get_config_dir;
use dioxus_primitives::checkbox::CheckboxState;
use crate::components::checkbox::Checkbox;

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

fn state_assesser(entry: &str) -> CheckboxState {
    match entry {
        "udisks2" => {
            if parse_config().unwrap_or_default().use_udisks2 {
                return CheckboxState::Checked;
            } else {
                return CheckboxState::Unchecked;
            }
        },
        "sha256" => {
            if parse_config().unwrap_or_default().check_hash {
                return CheckboxState::Checked;
            } else {
                return CheckboxState::Unchecked;
            }
        },
        _ => {
            error!("failed to parse struct for checkbox handling");
            return CheckboxState::Indeterminate;
        }
    }
}

#[component]
pub fn options_menu() -> Element {
    let mut settings = use_signal(||parse_config().unwrap_or_default());
    let mut status = use_signal(||"".to_string());
    rsx! {
        document::Style { "{CSS}" }
        div{ class: "modal-form",
        div{ class: "toolbar-actions",
        Checkbox{
            default_checked: state_assesser("sha256"),
            on_checked_change: move |_| {
                let mut settings = settings.write();
                settings.check_hash = !settings.check_hash;
            }
        }
        h2 {"Check sha256 hash"}
        }
        div{ class: "toolbar-actions",
        Checkbox{
            default_checked: state_assesser("udisks2"),
            on_checked_change: move |_| {
                let mut settings = settings.write();
                settings.use_udisks2 = !settings.use_udisks2;
            }
        }
        h2 {"Use udisks2 for flashing (must be checked on flatpak)"}
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
            h4 {"{status}"}
    }
}