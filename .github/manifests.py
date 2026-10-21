#!/usr/bin/env python3
"""Writes winget and Scoop manifests for one release, with real hashes.

Usage: manifests.py <tag> <owner/repo> <setup.exe> <portable.zip> <out dir>

winget: three YAML files under <out>/winget/<Identifier>/<version>/ ready to
be copied into a microsoft/winget-pkgs PR. Scoop: <out>/scoop/aidc.json for a
bucket repo; its `autoupdate` block lets `checkver` bump later versions.
"""
import hashlib
import pathlib
import sys

tag, repo, setup, zipped, out = sys.argv[1:6]
version = tag.lstrip("v")
setup = pathlib.Path(setup)
zipped = pathlib.Path(zipped)
out = pathlib.Path(out)
owner, name = repo.split("/")
ident = "nethero23.AIDiskCleaner"


def sha256(p: pathlib.Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest().upper()


def asset_url(p: pathlib.Path) -> str:
    # GitHub replaces spaces in asset names with dots.
    return f"https://github.com/{repo}/releases/download/{tag}/{p.name.replace(' ', '.')}"


setup_sha = sha256(setup)
zip_sha = sha256(zipped)

wdir = out / "winget" / ident / version
wdir.mkdir(parents=True, exist_ok=True)
(wdir / f"{ident}.yaml").write_text(
    f"""# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json
PackageIdentifier: {ident}
PackageVersion: {version}
DefaultLocale: zh-CN
ManifestType: version
ManifestVersion: 1.6.0
""",
    encoding="utf-8",
)
(wdir / f"{ident}.installer.yaml").write_text(
    f"""# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json
PackageIdentifier: {ident}
PackageVersion: {version}
InstallerType: nullsoft
Scope: user
InstallModes:
- interactive
- silent
UpgradeBehavior: install
Installers:
- Architecture: x64
  InstallerUrl: {asset_url(setup)}
  InstallerSha256: {setup_sha}
ManifestType: installer
ManifestVersion: 1.6.0
""",
    encoding="utf-8",
)
(wdir / f"{ident}.locale.zh-CN.yaml").write_text(
    f"""# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json
PackageIdentifier: {ident}
PackageVersion: {version}
PackageLocale: zh-CN
Publisher: {owner}
PublisherUrl: https://github.com/{owner}
PublisherSupportUrl: https://github.com/{repo}/issues
PackageName: AI Disk Cleaner
PackageUrl: https://github.com/{repo}
License: MIT
LicenseUrl: https://github.com/{repo}/blob/main/LICENSE
ShortDescription: 最懂 AI 的 Windows C 盘瘦身工具：找出 AI 工具留下的模型、缓存和环境，分级后安全清理。
Tags:
- ai
- cleaner
- disk
- ollama
- huggingface
ReleaseNotesUrl: https://github.com/{repo}/releases/tag/{tag}
ManifestType: defaultLocale
ManifestVersion: 1.6.0
""",
    encoding="utf-8",
)

sdir = out / "scoop"
sdir.mkdir(parents=True, exist_ok=True)
(sdir / "aidc.json").write_text(
    f"""{{
    "version": "{version}",
    "description": "Windows disk cleaner that understands AI tools: models, caches and environments, rated and cleaned safely.",
    "homepage": "https://github.com/{repo}",
    "license": "MIT",
    "url": "{asset_url(zipped)}",
    "hash": "{zip_sha.lower()}",
    "bin": "AI Disk Cleaner.exe",
    "shortcuts": [["AI Disk Cleaner.exe", "AI Disk Cleaner"]],
    "persist": "data",
    "checkver": "github",
    "autoupdate": {{
        "url": "https://github.com/{repo}/releases/download/v$version/AI.Disk.Cleaner_$version_x64-portable.zip"
    }}
}}
""",
    encoding="utf-8",
)
print(f"wrote winget manifests to {wdir} and scoop manifest to {sdir}")
