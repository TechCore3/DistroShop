use crate::list_handler::{self, get_config_dir};
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use crate::distro_testing::vm_handler;

use std::fs;

use std::path::PathBuf;
use std::time::Duration;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::flashing::flasher_unix;
#[cfg(target_os = "windows")]
use crate::flashing::flasher_win;
static CSS: &str = include_str!("../../assets/main.css");

#[component]
fn form_handler(distro: list_handler::distro, show_form: Signal<bool>, mut status: Signal<String>, mut show_button: Signal<bool>) -> Element {
    show_button.set(false);
    let mut blockdev = use_signal(String::new);
    let mut selected = use_signal(|| "safe".to_string());

    let device_label = {
        #[cfg(target_os = "macos")]
        {
            "Target Block Device (e.g. /dev/diskb)"
        }
        #[cfg(target_os = "linux")]
        {
            "Target Block Device (e.g. /dev/sdb)"
        }
        #[cfg(target_os = "windows")]
        {
            "Target Drive (e.g. D )"
        }
    };

    let on_submit = move |_evt: Event<FormData>| {
        if !blockdev.read().is_empty() {
            show_form.set(false);
            download_and_flash_handler(&distro, &blockdev.read().clone(), selected(), status).unwrap();
        } else {
            status.set("Please enter a valid device!".to_string());
        }
    };
    rsx! {

        form { class: "modal-form", onsubmit: on_submit,
            div { class: "form-field",
                label { class: "form-label", "{device_label}" }
                input {
                    class: "form-input",
                    value: "{blockdev}",
                    oninput: move |evt| blockdev.set(evt.value()),
                }
            }
            div { class: "form-field",
                label { class: "form-label", "Flash mode" }
                select {
                    class: "form-select",
                    value: "{selected}",
                    onchange: move |evt| selected.set(evt.value()),
                    option {value: "fast", "Fast"}
                    option {value: "safe", "Safe"}
                }
            }
            div { class: "form-actions",
                button { class: "primary-button", r#type: "submit", "Confirm" }
            }
        }
    }
}

 fn download_and_flash_handler(
    distro: &list_handler::distro,
    blockdev: &String,
    flashmode: String,
    mut status: Signal<String>,
) -> Result<(), Box<dyn std::error::Error>> 
{
    let distro_to_download = distro.clone();
    let blockdev_for_flash = blockdev.to_string();
    let mut iso_filename = get_config_dir().to_string_lossy().into_owned();
    iso_filename.push_str(&distro.filename);
    let flashmode_for_flash = flashmode.clone();

    
        spawn_forever(async move {

            status.set("Downloading iso image... (will take a while)".to_string());
            match download_distro(distro_to_download).await {
             Ok(_) => {
                status.set("Flashing to block device (ui might freeze and that's normal)".to_string()); //can't be unintended behavior if bugs are intended
                tokio::time::sleep(Duration::from_millis(100)).await;
                #[cfg(any(target_os = "linux", target_os = "macos"))] 
                match flasher_unix::flasher(&blockdev_for_flash, &iso_filename, &flashmode_for_flash) {
                    Ok(()) => {
                        status.set("Successfully flashed iso image!".to_string());
                    }
                    Err(e) => status.set(format!("Flashing failed: {e}")),
                }
                #[cfg(target_os = "windows")]
                match flasher_win::flasher(&blockdev_for_flash, &iso_filename, &flashmode_for_flash) {
                Ok(()) => {
                    status.set("Successfully flashed iso image!".to_string());
                }
                Err(e) => status.set(format!("Flashing failed: {e}")),
                }
            }
        Err(e) => status.set(format!("Download failed: {e}")),
            }
        });

    Ok(())
}

pub async fn download_distro(distro: list_handler::distro) -> Result<(), Box<dyn std::error::Error>> {
    let distro= distro.clone();
    info!{"requesting {}", distro.downloadlink};
    let response = reqwest::get(distro.downloadlink).await?.error_for_status()?;
    info!("got headers: {} len={:?}", response.status(), response.content_length());

    let mut file_path = PathBuf::from(get_config_dir());
    fs::create_dir_all(&file_path)?; // already creating if list isnt found but may be an edge case; flash_handler/ln86
    file_path.push(distro.filename);

    let contents = response.bytes().await?;
    info!("got body: {} bytes", contents.len());
    fs::write(&file_path, contents)?;
    info!("wrote {:?}", file_path);

    Ok(())
}


#[component]
pub fn show_more(distro: list_handler::distro, is_showing: Signal<bool>) -> Element {
    let mut show_form = use_signal(|| false);
    let mut show_button = use_signal(|| true);
    let status = use_signal(|| "".to_string());

    let distro_for_vm = distro.clone();

    let back_button_handler = move |_: MouseEvent| {
        if show_form(){
            show_form.set(false);
            show_button.set(true);
        } else {
            is_showing.set(false);
        }
    };
    rsx! {
        document::Style { "{CSS}" }
        div { class: "modal-overlay",
            div { class: "modal-panel",
                div { class: "modal-header",
                    h2 { class: "modal-title", "{distro.name}" }
                    button { class: "secondary-button", onclick: back_button_handler, "Back" }
                }
                div { class: "modal-body",
                    img {
                        class: "center distro-image",
                        src: "{distro.image}",
                    }
                    p { class: "modal-description center", "{distro.descriptionfull}" }
                    h3 { class: "modal-status center", "{status}" }
                    if show_form()  {
                        form_handler { distro: distro.clone(), show_form: show_form,status: status, show_button: show_button, }
                    } else if show_button() {
                        div { class: "form-actions",
                            button { class: "primary-button center", onclick: move |_| {
                                let distro_for_vm_run = distro_for_vm.clone();
                                spawn_forever(async move {
                                    vm_handler::run_vm(&distro_for_vm_run).await;
                                });
                            }, "Test in a VM" }
                            button { class: "primary-button center",onclick: move |_| show_form.set(true), "Download and flash" }
                        }
                    }
                }
            }
        }
    }
}
