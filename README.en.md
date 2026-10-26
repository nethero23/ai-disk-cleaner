<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Clearwisp icon">

# Clearwisp: the disk cleaner that understands AI tools

**Free, open-source Windows disk cleaner for AI developers, for C:, D: and every other drive. Finds and frees the space taken by Ollama / Hugging Face / LM Studio models, Cursor / Claude Code / VS Code caches, pip / conda / npm caches and WSL / Docker virtual disks. Scan first, you choose, standard cleanup can be undone.**

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

Clearwisp explains every gigabyte: **which model, which conda environment, which tool's cache, when it was last used, and what happens if you delete it**. Each item gets a safety level and nothing is deleted until you tick it. Standard cleanup uses the Recycle Bin and supports undo; Windows.old requires separate permanent system cleanup without undo.

## v0.9.0-rc.2 rule-language candidate (release preparation)

[v0.9.0-rc.2](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.2) is planned, pending exact-source Windows CI and NSIS/portable ZIP packaging. **The currently published candidate remains `v0.9.0-rc.1` below.** Stable Latest remains `v0.7.1`; candidates are manual downloads, with no MSI, updater or package-manager manifests.

- The original 126 scan rules gain names and deletion consequences in ten more languages, with complete-field, dynamic-placeholder and special-character validation. Ten newer rules and some dynamic status messages still use English/Chinese fallback; not every scan string is translated.
- Fast language changes cannot publish an obsolete rule response. Manual/background scans capture their language at startup; changing language clears the old report and selection, requiring a new scan.
- Changed rule source text uses its current warning rather than an outdated translation. This candidate also includes the earlier Windows.old source repair: separate administrator confirmation, permanent removal, no Recycle Bin or undo.

Local Linux checks passed: 129 core tests, 37 Node tests, eight Python audit tests, fmt, Clippy, locale parity and frontend build. **Exact-source Windows native build, candidate packaging and new manual desktop language-switch/restart acceptance remain pending.** Historical installation receipts do not certify this candidate. See the [acceptance checklist](docs/RELEASE_CHECKLIST.md).

## v0.9.0-rc.1 recovery reliability prerelease candidate

[v0.9.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1) is published as a **prerelease candidate, not a stable release**. Manually download its [NSIS installer](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-setup.exe) or [portable ZIP](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-portable.zip). Latest remains on stable `v0.7.1`; candidates are not offered through automatic updates and provide no MSI, `latest.json`, or winget/Scoop manifests.

- **Real history failures are visible**: missing or blank cleanup/relocation JSONL logs are empty; unreadable files, invalid encoding, or damaged records report errors with Retry instead of silently skipping records. Failed Recycle Bin enumeration no longer reports items as permanently gone.
- **Unavailable history stops cleanup**: validate and open the cleanup log before touching files. A failed preflight leaves files untouched; an append/sync failure stops subsequent items, retains actual successful cleanup totals, and surfaces a warning in foreground and background results.
- **Legacy records remain supported**: valid older logs need no migration, including records without batch IDs or a final newline.
- **Read-only AI inventory**: inspect MCP entries, skills, plugins, and extensions, with duplicate/conflict hints and sensitive-value redaction. AI configurations are not automatically merged, synchronized, or edited.
- **Older versions need review**: candidates are marked Review, not selected by default, and excluded from automatic Safe cleanup.
- **Live post-clean totals**: rebuild remaining items, total size, and safety-level amounts from backend-confirmed successes; retain failed items.
- **Bounded inventory concurrency**: at most four tool readers run concurrently, without a persistent cache.
- **Manual selection repaired**: click the checkbox center or press Space to select/clear a row. Groups select/clear eligible rows, while all-in-use groups stay disabled and running-file protections remain intact.
- **History no longer appears empty while loading**: pending reads show a loading state; rejected IPC requests retain loaded rows and offer Retry. Older replies cannot replace newer results, and restore/move-back controls stay disabled during loading or errors.
- **Readable history in narrow windows**: long paths truncate with full-path tooltips, errors wrap, and action columns stay inside the window. New status text covers all 12 UI languages.

This round passed 67 offline browser checks (25 selection, 30 history, 12 recovery), local automated checks, and an unsigned NSIS build. **Pre-installation isolated checkpoint:** a locally built `0.9.0-rc.1` portable copy passed real IPC/read-only checks: a locked log (OS error 32) and damaged cleanup/relocation logs showed errors while retaining rows; Retry succeeded after unlocking or restoring original bytes. A four-item, 137 B fixture scan, 10 selection/protection checks and two history rows after restart passed; two checks only asserted disabled states without clicking. That candidate was exited and the then-installed `0.8.0-rc.1` reopened; 12 original/fixture baseline hashes stayed unchanged. No cleanup/restore or installation ran in that lane.

**Publication checkpoint:** both public assets were downloaded and their SHA-256 hashes matched GitHub digests. The portable executable reports `0.9.0-rc.1`; both Windows executables are unsigned. At that point packages were inspected, not executed, and the installed app was unchanged. Asset sizes and hashes are in the acceptance checklist.

**Subsequent GitHub NSIS upgrade accepted:** on 2026-10-24, after a verified backup of 368 program/profile/WebView files plus two shortcuts and separate installation approval, the actual published installer upgraded the existing `0.8.0-rc.1` to `0.9.0-rc.1` with **Do not uninstall** selected. Extraction and successful completion were observed. The 18,881,024-byte installed executable differs from the public portable payload only in Tauri's three-byte NSIS marker. All 10 installed checks passed: mouse-center/Space select and clear, eight-item group select/clear, actual clicks on disabled busy group/item controls, and default Safe clear/reselect. Selection returned to 15 items / 2.8 GB. The read-only scan found 49 items / 69.4 GB in 3.6 seconds: one host observation, not a cold-start benchmark.

After full exit and relaunch through the original app entry, all 95 history records (81 successful) loaded normally. Settings, history, 14 non-scan Roaming configuration/rule files and both shortcuts retained their hashes. Growth, overview and WebView runtime data may change; whole-profile byte identity is not claimed. **No cleanup, restore, uninstall or reset ran; the public portable executable remains unexecuted.** This closes one NSIS upgrade case, not every stable-release gate. See the [installed retest record](docs/RELEASE_CHECKLIST.md#github-v09-in-place-upgrade-and-retest-2026-10-24).

This is **not a crash-safe write-ahead journal (WAL)**: a log write may fail after a file reaches the Recycle Bin; inspect the Windows Recycle Bin in that case. Transactional relocation-log writes remain deferred. If history fails to load, try Retry, then preserve the original log and check permissions and free disk space; do not empty it or delete damaged lines to conceal the error. MSI, pristine Windows/WebView2, code signing, and signed updates are outside verified coverage. See the [acceptance checklist](docs/RELEASE_CHECKLIST.md).

## Windows.old source fix (unreleased)

Selecting Windows.old now opens a separate system-cleanup confirmation explaining permanent removal, no history restore and loss of Windows rollback. Administrator elevation is requested only after confirmation; other selected scan items are left untouched. Ordinary recycle failures show the item, path and actual reason, retain failed selections and bring feedback into view.

Offline tests and mock-IPC UI checks passed at the local R2 checkpoint, including repeated separators from real scanner paths. **That local `0.9.0` R2 build passed in-place installation and complete app exit/restart acceptance**, preserving settings and all 95 history records. Installed Windows.old confirmation/cancel passed; refusal feedback used isolated fixtures and byte-identical application code. Later dot/parent-segment rejection and success-feedback scrolling are limited to fresh offline tests/build checks, not a new installation; R2 desktop acceptance does not cover those subsequent source bytes.

This records the October 24 source/docs-only checkpoint: no package, tag, updater or stable promotion ran then, and main retained `0.9.0-rc.1`. Those source changes are now included in the planned `0.9.0-rc.2` candidate above; local uncommitted stable-version preparation and the installed R2 program are outside this publication operation. No real Windows.old cleanup ran; actual system removal is not certified. See the [focused acceptance record](docs/WINDOWS_OLD_CLEANUP_ACCEPTANCE.md).

## Features

### 🧠 Knows AI-tool junk (136 rules, 90+ tools)

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
- **Undo standard cleanup**: ordinary cleanup uses the Recycle Bin and a cleaning log. Undo the last clean in one click, or restore single items from history. Windows.old uses separate permanent system cleanup with no Recycle Bin or undo.
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
- 12 UI languages in the development candidate, including Simplified and Traditional Chinese

### 🚫 Never

No registry cleaning, no nag pop-ups, no bundled software, no uploading of your file list. Scanning and cleaning stay local; app and rule updates contact GitHub. MCP version checks on the AI tools page are enabled by default and can be disabled in Settings; they send only package names to npm / PyPI, never configuration contents.

## Install

1. Open the [latest release](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
2. Download `*_x64-setup.exe` (recommended, supports in-app updates) or the `.msi`
3. Run it and click **Scan**

**Portable**: download `*_x64-portable.zip`, unzip and run `AI Disk Cleaner.exe`. Settings and logs live in the `data` folder next to it. The portable build does not auto-update.

Requires Windows 10 / 11 x64 with WebView2 (built into Windows 11; the installer handles Windows 10).

> The installer is not code-signed yet, so SmartScreen may show "Windows protected your PC". Click "More info → Run anyway". All code is open source; you can review it or build from source.

**Coming soon**: `winget install nethero23.AIDiskCleaner` and `scoop install aidc` (manifests are generated for stable releases and are being submitted; candidates generate none).

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
Free, MIT-licensed, no ads, no bundles. All scanning and cleaning happens locally; app and rule updates contact GitHub. MCP version checks on the AI tools page are enabled by default and can be disabled in Settings; they send only package names to npm / PyPI, never configuration contents.

### macOS / Linux?
Windows 10 / 11 only for now. The scan engine is cross-platform Rust, so other platforms may follow.

## Roadmap

- [x] v0.1–v0.3 Scan, safety levels, Recycle Bin delete, running-process check, move to drive / move back, model dedup, vhdx compaction, chat export, online rules
- [x] v0.4–v0.5 Big-file scan, tray + scheduled scans, parallel scan, preview, one-click Safe clean, 126 rules
- [x] v0.6 Search / group / sort, multi-drive scan, auto-update, drive overview
- [x] v0.7 Undo, first-run guide, growth monitoring, idle-model hints, deep system clean (WinSxS etc.), unknown-big-folder reports, portable build
- [ ] winget / Scoop listing, installer code signing
- [ ] v0.8 candidate acceptance: Material Design 3 UI, read-only AI-tool inventory (skills / MCP / plugins), duplicate/conflict hints, and optional version checks
- [ ] v0.9 recovery reliability candidate published as prerelease: isolated read-only desktop, public hashes and one GitHub NSIS upgrade/selection/restart accepted; full stable matrix and UI cleanup/restore remain incomplete

The published candidate is [0.9.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1), available by manual download; candidates do not replace stable releases. Automatic configuration merging, synchronization, and editing are out of scope. See [docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md).

## Contributing

Missing an AI tool you use? [Open an issue](https://github.com/nethero23/ai-disk-cleaner/issues) or add a rule (a few lines of TOML). Build and rule format: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md). See [CHANGELOG.md](CHANGELOG.md) for changes and [PROJECT_CONTEXT.md](PROJECT_CONTEXT.md) for milestone status.

If it saved you some gigabytes, a ⭐ helps others with a full disk find it.

## License

[MIT](LICENSE)
