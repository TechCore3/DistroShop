# DistroShop

A small desktop app for browsing distro images, testing them in a VM, and flashing them to a USB stick.


## What it does

- Shows a list of Linux distros from a remote JSON catalog
- Caches the list locally in `~/.config/distroshop/distros.json`
- Lets you refresh the catalog manually
- Displays each distro's image, summary, and full description
- Downloads the selected ISO
- Boots up a Virtual Machine running the ISO
- Flashes the ISO to a chosen block device
- Offers a `safe` and `fast` flash mode (streaming the file little by little or loading the entire iso into memory and then writing to disk)

## Current status

Supported on MacOS, Windows, and Linux.
Windows is only supported on x86_64. All others are supported on both x86_64 and arm64.


## Requirements

- Rust and Cargo (if compiling from source)
- qemu-system
- A target USB device 
- A graphical desktop environment

## Running the app

I recommend using the prebuilt binary, but you can also run it  with cargo:

```bash
cargo run
```

If you want the Dioxus dev workflow instead of the plain desktop app:

```bash
dx serve
```

## Flashing safely

Before flashing, double-check the target device path.

Useful commands:

```bash
lsblk
sudo fdisk -l
```

The app writes directly to the selected block device. Make sure you are targeting the USB stick, not your system drive.

## Project layout

```text
src/
├── main.rs            # app entry point and launch logic
├── list_handler.rs    # distro catalog loading, caching, and UI
├── distro_testing/    # VM-based testing of distro images
│   ├── mod.rs
│   └── vm_handler.rs  # VM setup and test runs
└── flashing/          # download + flashing logic
    ├── mod.rs
    ├── flash_handler.rs
    ├── flasher_unix.rs
    └── flasher_win.rs
assets/
├── distros.json       # distro catalog used by the app
|── logo.png           # app favicon (works on everything except wayland)
└── main.css           # styling
```

## License

GPL-3.0
