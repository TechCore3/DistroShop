use crate::list_handler::{self, get_config_dir};
use std::{io, process::Command};
use dioxus::prelude::*;
use crate::flashing::flash_handler;

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

pub async fn run_vm(distro: &list_handler::distro) {
    

    let qemu_path = find_qemu().expect("qemu-system not found. install it with your package manager");
    let ram_mb = 4096;
    let cpus = 4;
    let mut iso_path = get_config_dir();
    iso_path.push(&distro.filename);
    if !iso_path.exists() {
        let distro_for_download = distro.clone();
        download_handler(distro_for_download).unwrap();

    }
    let iso_arg = iso_path.display().to_string().replace(',', ",,");

    let status = Command::new(qemu_path)
    .args(["-enable-kvm", "-cpu", "host"]) // linux only for now
    .args(["-m", &ram_mb.to_string()])
    .args(["-smp", &cpus.to_string()])
    .arg("-drive")
    .arg(format!("file={iso_arg},media=cdrom,format=raw"))
    .args(["-boot", "d"])
    .args(["-nic", "user,model=virtio-net-pci"])
    .args(["-vga", "virtio"])
    .status()
    .expect("can't start up qemu");

    if !status.success() {
    eprintln!("qemu exited with: {status}");
    }
}