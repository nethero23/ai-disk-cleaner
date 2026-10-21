#!/usr/bin/env python3
"""Writes the Tauri updater manifest (latest.json) for a GitHub release.

Usage: latest_json.py <tag> <owner/repo> <nsis bundle dir> <out file>

Looks for the signed NSIS installer (`*-setup.exe` + `.sig`) that
`tauri build` produces when `bundle.createUpdaterArtifacts` is on and a
signing key is present. Without a `.sig` nothing is written, so an unsigned
release simply is not offered as an update.
"""
import datetime
import json
import pathlib
import sys

tag, repo, bundle_dir, out = sys.argv[1:5]
sigs = sorted(pathlib.Path(bundle_dir).glob("*-setup.exe.sig"))
if not sigs:
    print("no .sig found; skipping latest.json")
    sys.exit(0)
sig = sigs[0]
exe = sig.with_suffix("")  # strip .sig
# GitHub replaces spaces in asset names with dots.
asset = exe.name.replace(" ", ".")
manifest = {
    "version": tag.lstrip("v"),
    "notes": f"See https://github.com/{repo}/releases/tag/{tag}",
    "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "platforms": {
        "windows-x86_64": {
            "signature": sig.read_text().strip(),
            "url": f"https://github.com/{repo}/releases/download/{tag}/{asset}",
        }
    },
}
pathlib.Path(out).write_text(json.dumps(manifest, indent=2))
print(f"wrote {out} for {asset}")
