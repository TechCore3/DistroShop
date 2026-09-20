# DistroShop

> **Work in progress.** Things will break, change, or disappear without notice.

A minimal desktop app (built with [Dioxus](https://dioxuslabs.com/)) for browsing Linux distros and flashing them to a USB drive.

## What it does

- Fetches a list of distros from a [remote JSON source](https://github.com/TechCore3/DistroShop)
- Caches the list locally (`$TMPDIR/distros.json`) so it works offline after the first sync
- Lets you manually refresh the cached list
- Displays each distro's name, description, and image

## Planned

- [ ] "Download and Flash" — write an image to a USB drive using `dd` (or maybe a native file writing function??)
- [ ] Download progress / status feedback
- [ ] Error handling polish
- [ ] Distro search / filtering

## Running

```bash
cargo run
```

Requires the [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started) if you want hot-reload / bundling:

```bash
dx serve
```

## Project structure

```
src/
├── main.rs          # app entry point
└── list_handler.rs  # distro list fetching, caching, and UI
```

## License

GPL v3.0
