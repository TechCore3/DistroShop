use std::io;
use std::io::{Read, Write};
use nix::unistd::Uid;
use std::env;
use std::process::Command;
use std::fs::File;
use dioxus::prelude::*;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(target_os = "macos")]
fn run_mac_command(exe_path: &PathBuf, args: &[&str]) -> std::io::Result<std::process::ExitStatus> {
    let joined_args = args.join(" ");
    let inner_cmd = format!("{} {}", exe_path.display(), joined_args).replace('\'', "'\\''");

    let script = format!("do shell script \"{}\" with administrator privileges", inner_cmd);

    Ok(Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()?)
}

pub fn flasher(blockdev: &str, isoimg: &str, flashmode: &str) -> io::Result<()> {

    let args = vec![blockdev, isoimg, flashmode]; //the original arguments get shadowed later 

    if Uid::current().is_root(){
        info!("looking for blockdev and iso");
    let mut blockdev = File::options().write(true).open(&blockdev)?;
        info!("found");
    let mut isoimg = File::open(&isoimg)?;
        info!("found");
    match flashmode {
        "safe" => {
            info!("using safe mode");
            io::copy(&mut isoimg, &mut blockdev)?;
            blockdev.sync_all()?;
        }
        "fast" => {
            info!("using fast mode");
            let mut buffer = Vec::new();
            isoimg.read_to_end(&mut buffer)?;
            blockdev.write_all(&buffer)?;
            blockdev.sync_all()?;
        }
        _ => {
            error!("invalid flash mode");
            std::process::exit(1);
        }
    }
 } else {
    info!("looking for current exe");
    let exe_path = env::current_exe()?;
    info!("found");

    // it breaks here
    #[cfg(target_os = "linux")]
    let status = Command::new("pkexec") 
        .arg(&exe_path)
        .arg("-f")
        .args(&args)
        .status()?;
        
    #[cfg(target_os = "macos")]
    let status = run_mac_command(&exe_path, &args).unwrap_or_default();

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("pkexec exited with {status}")
        ));
    }

 }
    Ok(())
}