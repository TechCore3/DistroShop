use dioxus::prelude::*;
use serde_json;
use reqwest;
use std::io::Write;
use std::fs::{self, File};
use crate::options::Settings;
use crate::main_page::get_config_dir;
const DEFAULTCONFIGGITHUB: &str =
    "https://raw.githubusercontent.com/TechCore3/DistroShop/refs/heads/main/assets/config.json";



//static CSS: &str = include_str!("../../assets/main.css");

pub async fn download_default_config() -> std::result::Result<(), Box<dyn std::error::Error>> {
    if !get_config_dir().exists() {
        fs::create_dir(get_config_dir())?;
    }
    let response = reqwest::get(DEFAULTCONFIGGITHUB).await?;
    let json_text = response.text().await?;
    let mut file = File::create(get_config_dir().join("config.json"))?;
    file.write_all(json_text.as_bytes())?;
    Ok(())
}

pub fn parse_config() -> Result<Settings, Box<dyn std::error::Error>> {
    let json_text = fs::read_to_string(get_config_dir().join("config.json"))?;
    let settings: Settings = serde_json::from_str(&json_text)?;
    Ok(settings)
}

#[component]
pub fn options_menu() -> Element {
    rsx! {
        h1 {"check hash: settings.check_hash"}
        h1 {"use udisks2: settings.use_udisks2"}
    }
}