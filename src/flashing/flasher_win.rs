use dioxus::prelude::*;
use serde::Deserialize;
use std::{io::Write, process::{Command, Stdio}};
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use wmi::{COMLibrary, WMIConnection};
use std::io;
use std::env;
use std::fs::File;
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Partition { // originally held 2 values
    disk_index: u32,
}


fn clean_disk(pdrive: String) -> std::io::Result<()> {
    let prefix = r"\\.\PhysicalDrive";
    let index = pdrive
        .get(..prefix.len())
        .filter(|value| value.eq_ignore_ascii_case(prefix))
        .and_then(|_| pdrive.get(prefix.len()..))
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .ok_or_else(|| std::io::Error::other(format!("invalid physical drive path: {pdrive}")))?;
    let mut part = Command::new("diskpart")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = part
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("diskpart stdin was unavailable"))?;
    write!(stdin, "select disk {index}\r\nclean\r\nexit\r\n")?;
    drop(stdin);

    let output = part.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        let details = String::from_utf8_lossy(&output.stdout);
        let errors = String::from_utf8_lossy(&output.stderr);
        Err(std::io::Error::other(format!(
            "diskpart failed with {}: {}{}",
            output.status,
            details.trim(),
            errors.trim()
        )))
    }
}// modern problems require modern solutions

fn physical_drive_for(letter: String) -> Result<String, Box<dyn std::error::Error>> {
    let letter = letter.trim().trim_end_matches(':').to_ascii_uppercase();
    if letter.len() != 1 || !letter.as_bytes()[0].is_ascii_alphabetic() {
        return Err(format!("invalid drive letter: {letter:?}").into());
    }
    let result = std::thread::spawn(move || -> Result<String, String> {
        let com = COMLibrary::new().map_err(|e| e.to_string())?;
        let wmi = WMIConnection::new(com).map_err(|e| e.to_string())?;

        let query = format!(
            "ASSOCIATORS OF {{Win32_LogicalDisk.DeviceID='{letter}:'}} \
             WHERE AssocClass = Win32_LogicalDiskToPartition"
        );
        let partitions: Vec<Partition> = wmi.raw_query(&query).map_err(|e| e.to_string())?;

        partitions
            .first()
            .map(|p| format!(r"\\.\PhysicalDrive{}", p.disk_index))
            .ok_or_else(|| format!("no physical drive found for {letter}:"))
    })
    .join()
    .map_err(|_| "WMI thread panicked".to_string())?;

    Ok(result?)
}

pub fn flasher(driveletter: &str, isoimg: &String, flashmode: &str, physdrive: &str) -> std::io::Result <()> { 
    if !is_elevated::is_elevated() {
        let drive = physical_drive_for(driveletter.to_string().to_lowercase()).unwrap();
        let exe = env::current_exe().expect("no current exe");
        let args = vec![drive.clone(), isoimg.to_owned(), flashmode.to_string(), drive.clone()];
        let status = runas::Command::new(exe).args(&args).status()?;
             if !status.success() {
                return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("flashing process exited with {status}")
                 ));
             }
     } else {
    clean_disk(physdrive.to_string())?;
    let mut drive = File::options().write(true).open(&physdrive)?;
    let mut isoimg = File::open(&isoimg)?;
    match flashmode {
        "safe" => {
            info!("using safe mode");
            io::copy(&mut isoimg, &mut drive)?;
            drive.sync_all()?;
        }
        /*"fast" => {
            info!("using fast mode");
            let mut buffer = Vec::new();
            isoimg.read_to_end(&mut buffer)?;
            drive.write_all(&buffer)?;
            drive.sync_all()?;
        }*/
        _ => {
            error!("invalid flash mode");
            std::process::exit(1);
        }
    }
     }
     Ok(())
}