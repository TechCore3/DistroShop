use std::env;
use dioxus::prelude::*;
use std::io;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::desktop::tao::window::Icon;
use crate::list_handler::distro_list;

pub mod distro_testing;
pub mod flashing;
pub mod list_handler;
static CSS: &str = include_str!("../assets/main.css");

fn load_icon() -> Icon {
    let img = image::load_from_memory(include_bytes!("../assets/logo.png"))
        .expect("invalid icon")
        .into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).expect("bad icon data")
}

#[component]
fn app() -> Element {
    rsx! {
        document::Style { "{CSS}" }
        div { distro_list {} }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        #[cfg(any(target_os = "linux", target_os = "macos"))] 
        flashing::flasher_unix::flasher(&args[2], &args[3], &args[4])?;
        
        #[cfg(target_os = "windows")]
        flashing::flasher_win::flasher(&args[2], &args[3], &args[4])?;
    } else {
        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new()
                    .with_icon(load_icon()) //works on literally everything BUT wayland
                    .with_window(WindowBuilder::new().with_title("DistroShop")),
            )
            .launch(app); 
    }
    Ok(())
}
