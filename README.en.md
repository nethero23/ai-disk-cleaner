<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Clearwisp icon">

# Clearwisp: the disk cleaner that understands AI tools

**Free, open-source Windows disk cleaner for AI developers, for C:, D: and every other drive. Finds and frees the space taken by Ollama / Hugging Face / LM Studio models, Cursor / Claude Code / VS Code caches, pip / conda / npm caches and WSL / Docker virtual disks. Scan first, you choose, every delete can be undone.**

<sub>Formerly AI Disk Cleaner</sub>

[![Latest release](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=download&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/nethero23/ai-disk-cleaner/ci.yml?branch=main)](https://github.com/nethero23/ai-disk-cleaner/actions)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
![Rust](https://img.shields.io/badge/Rust-Tauri%202-orange?logo=rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ Download](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [Features](#features) · [vs CCleaner / BleachBit / Dism++](#comparison) · [FAQ](#faq)

</div>

<!-- Screenshot placeholder: uncomment once docs/images/hero.gif exists
<p align="center"><img src="docs/images/hero.gif" width="900" alt="Clearwisp scanning drives for AI model caches and cleaning them"></p>
-->

## Why

On a PC used for AI work, the C: drive rarely fills up with classic system junk. It fills up with **local LLM weights, Python environments, AI code editor caches and WSL/Docker virtual disks**, often hundreds of GB. CCleaner and similar tools don't look there; WizTree shows which folder is big but not what it is or whether it is safe to delete.

And it is not only C:. Model libraries, conda environments, Docker disks and project `node_modules` end up on D: and E: too. Clearwisp scans every drive you select.

Clearwisp explains every gigabyte: **which model, which conda environment, which tool's cache, when it was last used, and what happens if you delete it**. Each item gets a safety level, nothing is deleted until you tick it, deletes go to the Recycle Bin, and the last clean can be undone in one click.

## Features

### 🧠 Knows AI-tool junk (126 rules, 90+ tools)

| Category | Covered |
|---|---|
| Local LLMs | Ollama, Hugging Face (hub / datasets), LM Studio, ModelScope, GPT4All, Jan, Msty, llama.cpp, vLLM, Xinference, Open WebUI, AnythingLLM, Cherry Studio, Chatbox |
| Image / speech | ComfyUI, Stable Diffusion WebUI, Stability Matrix, Pinokio, Whisper, OpenAI CLIP |
| AI coding tools | Cursor, Windsurf, Claude Code, Codex, Gemini CLI, Qwen Code, GitHub Copilot, Cline, Roo Code, Kilo Code, Continue, Aider, Trae, Kiro, Zed, OpenCode, Antigravity, Goose |
| Python / ML | pip, uv, conda, Poetry, PyTorch, Keras, Triton, PaddlePaddle, Jupyter, Kaggle, Gradio |
| General dev | npm, pnpm, Yarn, Bun, Cargo, Go, Maven, Gradle, NuGet, Composer, Playwright, Puppeteer, Electron, JetBrains, VS Code |
| Containers | Docker Desktop, WSL (ext4.vhdx compaction) |
| Windows | Windows Update cache, Windows.old, temp files, NVIDIA / AMD / Intel driver caches, Edge / Chrome / Firefox caches, Office, Teams, OneDrive |

It reads Ollama manifests, the Hugging Face cache layout, conda environments, LM Studio model folders and Cursor workspaces, so you see **the actual model and environment names**, not just folder sizes.

### 🛡️ Safe by design

- **Read-only scan**: scanning never writes a file.
- **Three safety levels**: 🟢 Safe (pure cache, rebuilt automatically) / 🟡 Re-downloadable / 🔴 Review (user data or possibly in use). Only "Safe" is ticked by default.
- **Protects chat history**: Cursor's `state.vscdb` and Claude Code's `~/.claude/projects` hold your conversations. They are marked "Review" and can be exported before cleaning.
- **Running-process check**: warns you when a tool is running instead of pulling files out from under it.
- **Undo**: everything goes to the Recycle Bin with a cleaning log. Undo the last clean in one click, or restore single items from the history page.
- **Preview**: look inside any item before you delete it.

### 💾 Free space without deleting

- **Move to another drive**: move Ollama models, the Hugging Face cache, pip/uv/conda caches and more to D:. The app copies, verifies, then sets `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR` etc. or leaves a directory junction, so every tool keeps working. Move it back any time.
- **Cross-tool model dedup**: the same weights in the HF cache, LM Studio and ComfyUI are merged with hard links. Every tool still works; the space is used once.
- **Compact WSL / Docker virtual disks**: `ext4.vhdx` only grows. Shrink it back to its real size.

### ⚡ Convenient

- **All drives**: scan C:, D:, external disks, whichever you tick; finds models and caches that were moved to other drives by their folder signature
- "Where did my space go?" overview for each drive
- Growth monitor: see which folders are growing, with a notification when weekly growth passes a threshold
- Idle-model hints: find models you have not used in a long time
- Deep system clean: WinSxS component store, restore points, hibernation file, Delivery Optimization cache, Windows Update cache
- Unrecognized big folders: large folders no rule knows yet are listed too, with a one-click rule suggestion
- Big-file scan, project build output (`node_modules`, `target`, `.venv`…)
- Group by tool, search, sort; parallel scan with progress and cancel
- Tray icon, scheduled background scans, notification when junk passes a threshold, optional auto-clean of Safe items (off by default)
- First-run guide and a one-line summary after each scan
- In-app auto-update; online rule updates; write your own rules in TOML
- Chinese and English UI

### 🚫 Never

No registry cleaning, no nag pop-ups, no bundled software, no uploading of your file list. Works offline; it only goes online to check for updates.

## Install

1. Open the [latest release](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
2. Download `*_x64-setup.exe` (recommended, supports in-app updates) or the `.msi`
3. Run it and click **Scan**

**Portable**: download `*_x64-portable.zip`, unzip and run `AI Disk Cleaner.exe`. Settings and logs live in the `data` folder next to it. The portable build does not auto-update.

Requires Windows 10 / 11 x64 with WebView2 (built into Windows 11; the installer handles Windows 10).

> The installer is not code-signed yet, so SmartScreen may show "Windows protected your PC". Click "More info → Run anyway". All code is open source; you can review it or build from source.

**Coming soon**: `winget install nethero23.AIDiskCleaner` and `scoop install aidc` (manifests are generated with every release and are being submitted).

CLI for developers: `cargo run -p cleaner-core --bin aidc -- scan` (also `scan --json`, `dupes`, `move`, `compact`).

## Comparison

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| Ollama / HF / LM Studio models | ✅ by model name | ❌ | ❌ | ❌ | ❌ size only |
| Cursor / Claude Code / VS Code caches | ✅ | ❌ | ✅ (since 6.0) | ❌ | ❌ |
| pip / conda / npm / uv caches | ✅ | ❌ | partial | ❌ | ❌ |
| Tells cache apart from chat history | ✅ "Review" + export | ❌ | ❌ | ❌ | ❌ |
| Move to another drive (sets env vars) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Cross-tool model dedup | ✅ hard links | ❌ | ❌ | ❌ | ❌ |
| WSL / Docker vhdx compaction | ✅ | ❌ | ❌ | ❌ | ❌ |
| Undoable delete | ✅ Recycle Bin + undo | ❌ | ❌ | ❌ | ❌ |
| Windows system junk | ✅ common items | ✅ extensive | ✅ | ✅ extensive | ❌ |
| Registry cleaning | ❌ on purpose | ✅ | ✅ | ✅ | ❌ |
| Free and open source | ✅ MIT | ❌ limited free tier | ✅ GPL | free, unmaintained | free for personal use |

In short: for classic system junk, any of them will do. For AI and developer-tool junk, this is the tool built for it. They work fine side by side.

## FAQ

### My C: (or D:) drive is full. How do I see what is using the space?
Open "Where did my space go?", pick a drive, and get a per-folder breakdown (with `Users\<you>\AppData\Local`, `Downloads` etc. listed separately), then run a scan to see usage per tool and per model.

### Where does Ollama store models, and how do I move them to another drive?
By default in `C:\Users\<you>\.ollama\models`. Find the Ollama item in the scan results, choose "Move to another drive" and pick a folder on D:. The app copies, verifies and sets `OLLAMA_MODELS`. Restart Ollama and it reads from the new place.

### Can I delete the Hugging Face cache (`.cache\huggingface`)?
Yes. It is marked "Re-downloadable": models are downloaded again when needed. If you would rather not re-download, move it (the app sets `HF_HUB_CACHE`), or merge duplicates with LM Studio / ComfyUI using model dedup.

### Cursor is using tens of GB. Can I delete `state.vscdb`?
Cursor's caches, logs and extension download cache are safe to delete. `state.vscdb`, however, holds **all your AI chat history** (reports of it reaching 77 GB exist). It is marked "Review", is not ticked by default, and can be exported first.

### Can I delete Claude Code's `.claude` folder?
`~/.claude/projects` holds session transcripts (jsonl) and is marked "Review"; export it first if you want to keep them. Cache-type files are safe to clean.

### How do I clear the pip / conda / npm cache?
They appear under the Python / dev cache group as "Safe": they are rebuilt on the next install. Conda environments are listed by name with a last-used date, so unused environments can be removed too.

### How do I shrink a WSL or Docker `ext4.vhdx`?
Virtual disks never shrink on their own. Select the vhdx and click "Compact": the app runs `wsl --shutdown` and then diskpart's `compact vdisk` (admin rights needed; works on Windows Home). Data inside is not touched.

### Does it only clean C:? Can it scan D: and E:?
It scans any drive. Tick the drives in "Scan range" (in Settings and at the top of the results). Model libraries, caches, virtual disks, build output and big files on other drives are all found. When C: is tight, you can also move big models to another drive.

### I deleted something by mistake. Can I get it back?
Yes. Everything goes to the Recycle Bin. Use "Undo last clean", or restore single items from the history page.

### Can it break Windows?
The scan is read-only. Only folders listed in the rule set are cleaned, each with its consequence written out. Only "Safe" items are ticked by default, the registry is never touched, and running tools are flagged.

### Is it a CCleaner alternative?
It covers what CCleaner does not: AI models, Python environments, AI editor caches. It also cleans common Windows junk. Using both is fine.

### Is it free? Any ads or telemetry?
Free, MIT-licensed, no ads, no bundles. All scanning and cleaning happens locally; the app only contacts GitHub to check for updates and (optionally) to update rules.

### macOS / Linux?
Windows 10 / 11 only for now. The scan engine is cross-platform Rust, so other platforms may follow.

## Roadmap

- [x] v0.1–v0.3 Scan, safety levels, Recycle Bin delete, running-process check, move to drive / move back, model dedup, vhdx compaction, chat export, online rules
- [x] v0.4–v0.5 Big-file scan, tray + scheduled scans, parallel scan, preview, one-click Safe clean, 126 rules
- [x] v0.6 Search / group / sort, multi-drive scan, auto-update, drive overview
- [x] v0.7 Undo, first-run guide, growth monitoring, idle-model hints, deep system clean (WinSxS etc.), unknown-big-folder reports, portable build
- [ ] winget / Scoop listing, installer code signing
- [ ] v0.8 Material Design 3 UI, AI-tool config management (skills / MCP / plugins)

## Contributing

Missing an AI tool you use? [Open an issue](https://github.com/nethero23/ai-disk-cleaner/issues) or add a rule (a few lines of TOML). Build and rule format: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

If it saved you some gigabytes, a ⭐ helps others with a full disk find it.

## License

[MIT](LICENSE)
