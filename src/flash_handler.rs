use crate::list_handler;
use dioxus::prelude::*;
use std::fs::File;
use std::io;
use std::io::Read;

static CSS: Asset = asset!("/assets/main.css");
#[component]
pub fn flasher(blockdev: String, isoimg: String, flashmode: String) -> io::Result<()> {
    let mut blockdev = File::options().write(true).open(&blockdev)?;
    let mut isoimg = File::open(&isoimg)?;
    match flashmode.as_str() {
        "tiny buffer (safe)" => {
            io::copy(&mut isoimg, &mut blockdev)?;
            blockdev.sync_all()?;
        }
        "whole file (fast)" => {
            let mut buffer = Vec::new();
            isoimg.read_to_end(&mut buffer)?;
        }
        _ => error!("somehow reached this case (must be cosmic ray"),
    }
    Ok(())
}
#[component]
pub fn show_more(distro: list_handler::distro, is_showing: Signal<bool>) -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        div { style: "position: absolute; width: 100vw; height: 100vh; overflow: hidden;",

            div { style: "
                    position: fixed;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    pointer-events: none; 
                    z-index: 10;
                    background: radial-gradient(ellipse at center, rgba(0, 0, 0, 0) 40%, rgba(0, 0, 0, 0.85) 100%);
                " }

            div { style: "
                    display: flex;
                    flex-direction: column;
                    align-items: flex-start;
                    gap: 8px;
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    z-index: 50; 
                    padding: 2rem;
                    background: black;
                    border-radius: 8px;
                    box-shadow: 0 10px 25px rgba(0,0,0,0.5);
                ",
                button { onclick: move |_| is_showing.set(false), "back" }
                img {
                    class: "center",
                    style: "max-width:100px; max-height:100px; width: auto; height:auto; ",
                    src: "{distro.image}",
                }
                h1 { "{distro.name}" }
                p { class: "center", "{distro.descriptionfull}" }
                button { "download and flash" }
            }
        }
    }
}
