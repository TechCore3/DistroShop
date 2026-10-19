use std::{collections::HashMap, error::Error, fs::{self, File}, io, os::fd::OwnedFd as StdOwnedFd, path::PathBuf};
use zbus::{Connection, OwnedFd as ZBusOwnedFd, Proxy, Variant};

const UD: &str = "org.freedesktop.UDisks2";
const ROOT: &str = "/org/freedesktop/UDisks2/block_devices";

pub async fn flasher(iso: PathBuf, dev: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = dev.file_name().unwrap().to_str().unwrap().to_owned();
    let sys = fs::canonicalize(format!("/sys/class/block/{name}"))?;
    let removable = fs::read_to_string(sys.join("removable")).map_or(false, |s| s.trim() == "1");
    if !(removable || sys.to_string_lossy().contains("/usb")) {
        return Err("refusing: not a USB or removable device".into());
    }

    let c = Connection::system().await?;
    let parts: Vec<String> = fs::read_dir(&sys)?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(&name))
        .collect();
    for p in parts.iter().chain([&name]) {
        let fsp = Proxy::new(&c, UD, format!("{ROOT}/{p}"), "org.freedesktop.UDisks2.Filesystem").await?;
        let _: zbus::Result<()> = fsp.call("Unmount", &(HashMap::<&str, Variant>::new(),)).await;
    }

    let blk = Proxy::new(&c, UD, format!("{ROOT}/{name}"), "org.freedesktop.UDisks2.Block").await?;
    let opts = HashMap::from([("flags", Variant::from(libc::O_EXCL))]);
    let fd: ZBusOwnedFd = blk.call("OpenDevice", &("w", opts)).await?;
    let mut out = File::from(StdOwnedFd::from(fd));

    tokio::task::spawn_blocking(move || -> io::Result<()> {
        io::copy(&mut File::open(iso)?, &mut out)?;
        out.sync_all()
    })
    .await??;
    Ok(())
}