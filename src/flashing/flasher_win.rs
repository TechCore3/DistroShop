use dioxus::prelude::*;
use serde::Deserialize;
use runas::Command;
use wmi::{COMLibrary, WMIConnection};
use std::io;
use std::env;
use std::io::{Read, Write};
use std::fs::File;
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Partition { // originally held 2 values but i changed the function and im afraid to touch the struct
    disk_index: u32,
}

fn physical_drive_for(letter: String) -> Result<String, Box<dyn std::error::Error>> {
    let com = COMLibrary::new()?;
    let wmi = WMIConnection::new(com)?;
    let query = format!(
        "ASSOCIATORS OF {{Win32_LogicalDisk.DeviceID='{}:'}} \
         WHERE AssocClass = Win32_LogicalDiskToPartition",
        letter.to_ascii_uppercase()
    );
    let partitions: Vec<Partition> = wmi.raw_query(&query)?;

    partitions
    .first()
    .map(|p| format!(r"\\.\PhysicalDrive{}", p.disk_index))
    .ok_or_else(|| format!("no physical drive found for {}:", letter).into())
}

pub fn flasher(driveletter: &str, isoimg: &str, flashmode: &str) -> std::io::Result <()> { 
    let driveletter = physical_drive_for(driveletter.to_string()).expect("");
    if !is_elevated::is_elevated() {
        let exe = env::current_exe().expect("no current exe");
        let args = vec![drive, isoimg.to_string(), flashmode.to_string()];

        let status = Command::new(exe).arg("-f").args(&args).status()?;
             if !status.success() {
                return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("flashing process exited with {status}")
                 ));
             }
     } else {
    let mut drive = File::options().write(true).open(&driveletter)?;
    let mut isoimg = File::open(&isoimg)?;
    match flashmode {
        "safe" => {
            info!("using safe mode");
            io::copy(&mut isoimg, &mut drive)?;
            drive.sync_all()?;
        }
        "fast" => {
            info!("using fast mode");
            let mut buffer = Vec::new();
            isoimg.read_to_end(&mut buffer)?;
            drive.write_all(&buffer)?;
            drive.sync_all()?;
        }
        _ => {
            error!("invalid flash mode");
            std::process::exit(1);
        }
    }
     }
     Ok(())
}