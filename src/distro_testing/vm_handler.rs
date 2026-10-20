use crate::list_handler::{self, get_config_dir};
use std::{io, process::Command};
use dioxus::prelude::*;
use dioxus::core::spawn_forever;
use crate::flashing::flash_handler;
use std::path::PathBuf;

fn find_qemu() -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("qemu-system-x86_64"))
        .find(|p| p.is_file())
}
fn download_handler(distro_for_download: list_handler::distro) -> io::Result<()> {
    spawn(async move {
        flash_handler::download_distro(distro_for_download).await.unwrap();
    });
    Ok(())
}

#[component]
pub fn form_handler(distro: list_handler::distro, show_form_vm: Signal<bool>, mut status: Signal<String>) -> Element {
    let distro = distro.clone();
    let mut vm_ram_mb: Signal<u32> = use_signal(|| 2048);
    let mut vm_cpu_cores: Signal<u8> = use_signal(|| 2);
    let on_submit = move |_evt: Event<FormData>| {
        let distro = distro.clone();
        let ram_mb = vm_ram_mb;
        let cpu_cores = vm_cpu_cores;

        spawn_forever(async move {
            if let Err(err) = run_vm(distro, ram_mb, cpu_cores).await {
                status.set(format!("Failed to start VM: {err}"));
            }
        });
    };
    rsx! {
        form {
            class: "modal-form", onsubmit: on_submit,
            div { class: "form-field",
            label { class: "form-label", "Enter RAM amount to allocate to the VM (in Megabytes)."}
            input{
                class: "form-input",
                value: "{vm_ram_mb}",
                oninput: move |evt| {
                    vm_ram_mb.set(evt.value().parse::<u32>().unwrap_or(0))}
            }
        }
            div { class: "form-field",
            label { class: "form-label", "Enter the number of CPU cores the VM can use. "}
            input{
                class: "form-input",
                value: "{vm_cpu_cores}",
                oninput: move |evt| {
                    vm_cpu_cores.set(evt.value().parse::<u8>().unwrap_or(0))}
            }
        }
        div { class: "form-actions",
                button { class: "primary-button", r#type: "submit", "Confirm" }
            }

        }
    }
}

pub async fn run_vm(distro: list_handler::distro, ram_mb: Signal<u32>, cpu_cores: Signal<u8>) -> Result<(), Box<dyn std::error::Error>> {
    

    let mut qemu_path = find_qemu().unwrap_or_default();
    if qemu_path.as_os_str().is_empty() {
        if PathBuf::from("/.flatpak-info").exists() {
            qemu_path = PathBuf::from("qemu-system-x86_64");
        } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("qemu-system is not present"),
        )
        .into()); 
    }
    } 
    let mut iso_path = get_config_dir();
    iso_path.push(&distro.filename);
    if !iso_path.exists() {
        let distro_for_download = distro.clone();
        download_handler(distro_for_download).unwrap();

    }
    let iso_arg = iso_path.display().to_string().replace(',', ",,");
    let acceleration = {
        #[cfg(target_os = "linux")]
        {"kvm"}
        #[cfg(target_os = "macos")]
        {"hvf"}
        #[cfg(target_os = "windows")]
        {"whpx"}
    };
    let status = Command::new(qemu_path)
    .args(["-accel",acceleration, "-cpu", "host"])
    .args(["-m", &ram_mb.to_string()])
    .args(["-smp", &cpu_cores.to_string()])
    .arg("-drive")
    .arg(format!("file={iso_arg},media=cdrom,format=raw"))
    .args(["-boot", "d"])
    .args(["-nic", "user,model=virtio-net-pci"])
    .args(["-vga", "virtio"])
    .status()
    .expect("can't start up qemu");

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("QEMU exited with status: {status}"),
        )
        .into());
    }
    Ok(())
}