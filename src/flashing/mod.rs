pub mod flash_handler;
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod flasher_unix;
#[cfg(target_os = "linux")]
pub mod udisks2_wrapper;

#[cfg(target_os = "windows")]
pub mod flasher_win;
//i dont know why i have to use these cfg gates on multiple files but the compiler is mad if i dont