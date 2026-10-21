#!/usr/bin/env python3
"""Pin the newest release of each distro in distros.json.

Usage:
  python3 update_distro.py <name | all> <distros.json> [--dry-run]
"""
import hashlib
import json
import re
import sys
import urllib.error
import urllib.request

UA = {"User-Agent": "distroshop-updater"}

# One config per distro, keyed by the "name" field in distros.json.
# Discovery is one of:
#   index:  page listing version folders (version, dir, subdir, iso)
#   probe:  candidate ISO URLs tried newest first (url, majors, minors)
#   latest: a "latest" URL that redirects to the current ISO
#   manual: no discovery; update the link by hand
# checksum: vendor checksum filename, with {iso} as the ISO filename, or None
# Nothing here is verified against the live sites yet.
DISTROS = {
    "arch linux": {
        "index": "https://archive.archlinux.org/iso/",
        "version": r"(\d{4}\.\d{2}\.\d{2})",
        "dir": "https://archive.archlinux.org/iso/{v}/",
        "iso": r"archlinux-{v}-x86_64\.iso",
        "checksum": "sha256sums.txt",
    },
    "tiny core linux": {
        "probe": {
            "url": "https://ftp.nl.freebsd.org/os/Linux/distr/tinycorelinux/{maj}.x/x86/release/TinyCore-{v}.iso",
            "majors": range(20, 16, -1),
            "minors": range(9, -1, -1),
        },
        "checksum": None,
    },
    "linux mint": {
        "index": "https://mirrors.kernel.org/linuxmint/stable/",
        "version": r"(\d+\.\d+)",
        "dir": "https://mirrors.kernel.org/linuxmint/stable/{v}/",
        "iso": r"linuxmint-{v}-cinnamon-64bit\.iso",
        "checksum": "sha256sum.txt",
    },
    "ubuntu": {
        "index": "https://releases.ubuntu.com/",
        "version": r"(\d+\.04(?:\.\d+)?)",  # LTS releases only
        "dir": "https://releases.ubuntu.com/{v}/",
        "iso": r"ubuntu-{v}-desktop-amd64\.iso",
        "checksum": "SHA256SUMS",
    },
    "debian": {
        "index": "https://cdimage.debian.org/debian-cd/",
        "version": r"(\d+\.\d+\.\d+)",
        "dir": "https://cdimage.debian.org/debian-cd/{v}/amd64/iso-cd/",
        "iso": r"debian-{v}-amd64-netinst\.iso",
        "checksum": "SHA256SUMS",
    },
    "fedora workstation": {
        "index": "https://download.fedoraproject.org/pub/fedora/linux/releases/",
        "version": r"(\d+)",
        "dir": "https://download.fedoraproject.org/pub/fedora/linux/releases/{v}/Workstation/x86_64/iso/",
        "iso": r"Fedora-Workstation-Live-{v}-[\d.]+\.x86_64\.iso",
        "checksum": None,
    },
    "manjaro": {
        "manual": True,  # ISO names include a build date; update by hand
    },
    "opensuse leap": {
        "index": "https://download.opensuse.org/distribution/leap/",
        "version": r"(\d+\.\d+)",
        "dir": "https://download.opensuse.org/distribution/leap/{v}/offline/",
        "iso": r"Leap-{v}-offline-installer-x86_64\.install\.iso",
        "checksum": "{iso}.sha256",
    },
    "kali linux": {
        "index": "https://cdimage.kali.org/",
        "version": r"kali-(\d{4}\.\d+)",
        "dir": "https://cdimage.kali.org/kali-{v}/",
        "iso": r"kali-linux-{v}-installer-amd64\.iso",
        "checksum": "SHA256SUMS",
    },
    "alpine linux": {
        "index": "https://dl-cdn.alpinelinux.org/alpine/",
        "version": r"v(\d+\.\d+)",
        "dir": "https://dl-cdn.alpinelinux.org/alpine/v{v}/releases/x86_64/",
        "iso": r"alpine-standard-\d+\.\d+\.\d+-x86_64\.iso",
        "checksum": "{iso}.sha256",
    },
    "nixos": {
        "latest": "https://channels.nixos.org/nixos-26.05/latest-nixos-graphical-x86_64-linux.iso",
        "checksum": None,
    },
}


def fetch(url: str) -> str:
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=60) as resp:
        return resp.read().decode("utf-8", "replace")


def listing(url: str) -> list[str]:
    """Names of the entries on an index page."""
    names = []
    for href in re.findall(r'href="([^"]+)"', fetch(url)):
        name = href.split("?")[0].split("#")[0].rstrip("/").split("/")[-1]
        if name and name not in (".", ".."):
            names.append(name)
    return names


def natural_key(s: str) -> list:
    """Sort so that 26.4 < 26.10 and 2026.10.01 > 2026.9.30."""
    return [int(p) if p.isdigit() else p for p in re.split(r"(\d+)", s)]


def url_exists(url: str) -> bool:
    req = urllib.request.Request(url, method="HEAD", headers=UA)
    try:
        with urllib.request.urlopen(req, timeout=30):
            return True
    except urllib.error.HTTPError:
        return False  # 404 and similar; network errors still raise


def find_by_probe(p: dict) -> tuple[str, str, str]:
    """Try candidate ISO URLs newest first and return the first that exists."""
    for maj in p["majors"]:
        for mnr in p["minors"]:
            v = f"{maj}.{mnr}"
            url = p["url"].format(maj=maj, v=v)
            if url_exists(url):
                base, iso = url.rsplit("/", 1)
                return v, base + "/", iso
    raise RuntimeError("probe found no existing ISO")


def find_by_redirect(latest_url: str) -> tuple[str, str, str]:
    """Follow a 'latest' URL and read the real ISO location from the final URL."""
    req = urllib.request.Request(latest_url, method="HEAD", headers=UA)
    with urllib.request.urlopen(req, timeout=30) as resp:
        final = resp.geturl()
    base, iso = final.rsplit("/", 1)
    m = re.search(r"nixos-(\d+\.\d+)", latest_url)
    v = m.group(1) if m else iso
    return v, base + "/", iso


def find_by_index(cfg: dict) -> tuple[str, str, str]:
    """Find the newest version folder that contains a matching ISO."""
    try:
        names = listing(cfg["index"])
    except urllib.error.URLError as e:
        raise RuntimeError(f"index {cfg['index']} failed: {e}") from e

    versions = set()
    for name in names:
        m = re.fullmatch(cfg["version"], name)
        if m:
            versions.add(m.group(1))
    if not versions:
        raise RuntimeError(
            f"no folder matched {cfg['version']!r} at {cfg['index']}; "
            f"first entries were {names[:15]}"
        )

    for v in sorted(versions, key=natural_key, reverse=True):
        try:
            base = cfg["dir"].format(v=v)
            if cfg.get("subdir"):
                subs = [n for n in listing(base) if re.fullmatch(cfg["subdir"], n)]
                if not subs:
                    raise RuntimeError("no nested release folder")
                base += max(subs, key=natural_key) + "/"

            pattern = cfg["iso"].replace("{v}", re.escape(v))
            isos = [n for n in listing(base) if re.fullmatch(pattern, n)]
            if isos:
                return v, base, max(isos, key=natural_key)
            print(f"  skipping {v}: no matching ISO yet")
        except (urllib.error.URLError, RuntimeError) as e:
            print(f"  skipping {v}: {e}")
    raise RuntimeError("no release with a matching ISO")


def find_release(cfg: dict) -> tuple[str, str, str]:
    if "probe" in cfg:
        return find_by_probe(cfg["probe"])
    if "latest" in cfg:
        return find_by_redirect(cfg["latest"])
    return find_by_index(cfg)


def checksum_for(cfg: dict, base: str, iso: str) -> str | None:
    """Read the ISO's hash from the vendor checksum file, if there is one."""
    if not cfg.get("checksum"):
        return None
    try:
        text = fetch(base + cfg["checksum"].format(iso=iso))
    except urllib.error.URLError:
        return None
    for line in text.splitlines():
        m = re.fullmatch(r"([0-9a-fA-F]{64})\s+\*?(.+)", line.strip())
        if m and m.group(2).split("/")[-1] == iso:
            return m.group(1).lower()
    return None


def sha256_of_url(url: str) -> str:
    h = hashlib.sha256()
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=120) as resp:
        while chunk := resp.read(1 << 20):
            h.update(chunk)
    return h.hexdigest()


def pin(cfg: dict) -> dict:
    v, base, iso = find_release(cfg)
    print(f"  version {v}, ISO {iso}")
    sha = checksum_for(cfg, base, iso)
    if sha:
        print("  hash from vendor checksum file (signature not verified)")
    else:
        print("  no vendor checksum; downloading ISO to hash it")
        sha = sha256_of_url(base + iso)
    return {"downloadlink": base + iso, "filename": iso, "sha256": sha}


def main() -> None:
    dry_run = "--dry-run" in sys.argv
    args = [a for a in sys.argv[1:] if a != "--dry-run"]
    if len(args) != 2:
        sys.exit(__doc__)
    target, path = args[0].lower(), args[1]

    with open(path, encoding="utf-8") as f:
        distros = json.load(f)
    entries = {d["name"]: d for d in distros}
    names = list(DISTROS) if target == "all" else [target]

    changed, failed = False, []
    for name in names:
        print(f"{name}:")
        cfg = DISTROS.get(name)
        entry = entries.get(name)
        if cfg is None or entry is None:
            print(f"::error title=Missing::{name!r} needs a DISTROS config and a JSON entry")
            failed.append(name)
            continue
        if cfg.get("manual"):
            print("  manual update, skipped")
            continue

        try:
            new = pin(cfg)
        except Exception as e:
            print(f"::error title=Update failed::{name}: {e}")
            failed.append(name)
            continue

        if new["downloadlink"] == entry.get("downloadlink"):
            if new["sha256"] != entry.get("sha256"):
                # Same URL, different bytes: suspicious, so flag it for review
                print(f"::warning title=Hash changed::{name}: same URL, new hash")
        else:
            print(f"  link changed to {new['downloadlink']}")

        if any(entry.get(k) != v for k, v in new.items()):
            entry.update(new)
            changed = True
        else:
            print("  up to date")

    if changed and not dry_run:
        with open(path, "w", encoding="utf-8") as f:
            json.dump(distros, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print("Wrote changes to", path)
    elif dry_run:
        print("Dry run: no file written")

    if failed:
        print("Failed:", ", ".join(failed))
        sys.exit(1)


if __name__ == "__main__":
    main()