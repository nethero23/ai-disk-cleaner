<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Clearwisp 图标">

# Clearwisp 清灵 · 最懂 AI 的硬盘清理工具

**免费开源的 Windows 硬盘清理工具，C 盘、D 盘、所有盘一起扫，专清 AI 工具留下的垃圾：Ollama / Hugging Face / LM Studio 模型，Cursor / Claude Code / VS Code 缓存，pip / conda / npm 缓存，WSL / Docker 虚拟磁盘。先扫描、你来选、删了能撤销。**

<sub>原名 AI Disk Cleaner</sub>

[![最新版本](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=%E4%B8%8B%E8%BD%BD&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![下载量](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/nethero23/ai-disk-cleaner/ci.yml?branch=main)](https://github.com/nethero23/ai-disk-cleaner/actions)
![平台](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
![Rust](https://img.shields.io/badge/Rust-Tauri%202-orange?logo=rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ 下载最新版](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [功能](#功能) · [和 CCleaner / BleachBit / Dism++ 比](#和其他清理软件比) · [常见问题](#常见问题)

</div>

<!-- 截图占位：把 docs/images/hero.gif 放好后取消下面这行的注释
<p align="center"><img src="docs/images/hero.gif" width="900" alt="Clearwisp 扫描硬盘上 AI 模型缓存并一键清理的演示"></p>
-->

## 一句话

C 盘又红了，D 盘也快满了？在一台跑 AI 的电脑上，真正占地方的往往不是系统垃圾，而是 **几十上百 GB 的大模型、Python 环境、AI 编辑器缓存和 WSL/Docker 虚拟磁盘**。CCleaner、360、电脑管家不扫这些；WizTree 只告诉你哪个文件夹大，不告诉你那是什么、能不能删。

而且这些东西不只在 C 盘：模型库、conda 环境、Docker 虚拟磁盘、项目里的 `node_modules` 经常散在 D 盘、E 盘。Clearwisp 会扫你勾选的所有盘。

Clearwisp 把每一 GB 翻译成人话：**这是哪个模型、哪个 conda 环境、哪个工具的缓存、多久没用了、删了会怎样**，标好安全等级，你勾选后再删，删除走回收站，可以一键撤销。

## v0.8.0-rc.2 候选准备

`v0.8.0-rc.2` 正在准备，**尚未发布，不是稳定版**；已发布的候选为 [v0.8.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.8.0-rc.1)。新候选发布后请从 [Releases 列表](https://github.com/nethero23/ai-disk-cleaner/releases) 手动下载 NSIS 安装包（`*_x64-setup.exe`）或便携 ZIP。上方 Latest 下载链接仍指向稳定版 `v0.7.1`，不会自动推送候选；候选不提供 MSI、`latest.json` 或 winget/Scoop 清单。

- **AI 工具只读盘点**：查看 MCP、skills、插件与扩展，提示重复和配置冲突，敏感值脱敏展示；不自动合并、同步或修改 AI 配置。
- **旧版本需审核**：旧版本候选标为「需确认」（Review），不默认勾选，也不参与自动安全清理。
- **清理后即时统计**：按后端确认成功的项目重算剩余条目、总量及各安全等级占用；失败项保留。
- **有界并发盘点**：最多 4 个工具读取任务并发执行，不使用持久缓存。
- **自由勾选修复**：点击勾选框中心或按空格均可选中/取消；分组可选中和取消所有未占用项目，全部被占用时禁用，不绕过运行中保护。
- **历史加载不再假空**：加载时显示加载状态；IPC 请求失败保留已加载记录并可重试，旧请求不覆盖新结果。加载或出错时禁用还原/搬回操作。
- **窄窗口历史可读**：长路径省略显示并保留悬浮完整路径，错误信息换行，操作列不再被挤出窗口；新增状态文案覆盖 12 种界面语言。

已发布的 GitHub `rc.1` 安装包已原位重装并完整重启，10 项勾选检查通过，原有配置、95 行历史及两个快捷方式哈希不变，未执行清理。`rc.2` 的历史问题先复现再修复；28 项历史与原有 25 项浏览器检查共 53 项通过，隔离的真实桌面验证了历史显示及只读扫描/勾选，未覆盖本机安装版。MSI 实际安装、全新 Windows/WebView2、代码签名及签名更新仍未验收；历史错误提示仅覆盖被拒绝的 IPC，不代表修复了后端所有磁盘读取错误。证据与待办见 [验收清单](docs/RELEASE_CHECKLIST.md)。

## 功能

### 🧠 认得 AI 工具的垃圾（136 条规则，90+ 个工具）

| 类别 | 覆盖 |
|---|---|
| 本地大模型 | Ollama、Hugging Face（hub / datasets）、LM Studio、ModelScope、GPT4All、Jan、Msty、llama.cpp、vLLM、Xinference、Open WebUI、AnythingLLM、Cherry Studio、Chatbox |
| AI 绘画 / 语音 | ComfyUI、Stable Diffusion WebUI、Stability Matrix、Pinokio、Whisper、OpenAI CLIP |
| AI 编程工具 | Cursor、Windsurf、Claude Code、Codex、Gemini CLI、Qwen Code、GitHub Copilot、Cline、Roo Code、Kilo Code、Continue、Aider、Trae、Kiro、Zed、OpenCode、Antigravity、Goose |
| Python / 深度学习 | pip、uv、conda、Poetry、PyTorch、Keras、Triton、PaddlePaddle、Jupyter、Kaggle、Gradio |
| 前端 / 通用开发 | npm、pnpm、Yarn、Bun、Cargo、Go、Maven、Gradle、NuGet、Composer、Playwright、Puppeteer、Electron、JetBrains、VS Code |
| 容器 / 虚拟磁盘 | Docker Desktop、WSL（ext4.vhdx 压缩） |
| Windows 系统 | Windows 更新缓存、Windows.old、临时文件、NVIDIA / AMD / Intel 驱动缓存、Edge / Chrome / Firefox 缓存、Office、Teams、OneDrive |

不只看路径：会解析 Ollama manifest、Hugging Face 缓存结构、conda 环境、LM Studio 模型目录、Cursor 工作区，告诉你**具体是哪个模型、哪个环境**。

### 🛡️ 安全第一

- **只读扫描**：扫描过程不写任何文件。
- **三级安全标签**：🟢 安全（纯缓存，会自动重建）／🟡 可重下（删了要重新下载）／🔴 需确认（含用户数据或可能在用）。默认只勾「安全」。
- **保护对话记录**：Cursor 的 `state.vscdb`、Claude Code 的 `~/.claude/projects` 里存着你的全部对话，标为「需确认」，清理前可先导出备份。
- **运行中检测**：工具正在运行时提醒你，不去动它正在用的文件。
- **删了能反悔**：删除一律进回收站并写清理日志，可一键撤销上次清理，清理记录里也能逐项还原。
- **清理前看看里面**：每一项都能预览里面有什么。

### 💾 不删也能腾空间

- **搬到其他盘**：把 Ollama 模型、Hugging Face 缓存等整体搬到 D 盘，自动设置 `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR` 等环境变量，或在原位置留目录联接，工具照常用。先复制并校验，再删原件；随时能搬回来。
- **跨工具模型去重**：同一份模型权重在 HF 缓存、LM Studio、ComfyUI 里各存一份？用硬链接合并，每个工具都照常用，只占一份空间。
- **压缩 WSL / Docker 虚拟磁盘**：`ext4.vhdx` 只涨不缩，一键压缩回实际大小。

### ⚡ 好用

- **全盘扫描**：C 盘、D 盘、移动硬盘，勾哪个扫哪个；按目录特征找到挪到其他盘的模型和缓存
- 「空间去哪儿了」概览：每个盘被谁占了，一眼看清
- 增长监控：看哪个目录最近在长，一周涨幅超过阈值就通知
- 闲置模型建议：按最后使用时间找出很久没用的模型
- 系统深度清理：WinSxS 组件存储、系统还原点、休眠文件、传递优化缓存、Windows 更新缓存
- 未识别的大目录：规则库还不认识的大文件夹也会列出来，可一键提交规则建议
- 全盘大文件扫描、项目构建产物（`node_modules`、`target`、`.venv`…）扫描
- 结果按工具分组、搜索、排序；并行扫描，带进度可取消
- 托盘常驻、定时后台扫描、垃圾超阈值通知、可选自动清理安全项（默认关）
- 首次运行引导、扫描后一句话总结
- 应用内自动更新；规则库可在线更新，也能自己写规则
- 开发候选提供 12 种界面语言（含简体、繁体中文）

### 🚫 不做

注册表清理、后台弹窗、捆绑软件、上传你的文件列表。扫描和清理均在本地完成；应用和规则更新会访问 GitHub。「AI 工具」页的 MCP 版本检查默认开启，可在设置中关闭；它只向 npm / PyPI 发送包名，不发送配置内容。

## 下载安装

1. 打开 [Releases 最新版](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
2. 下载 `*_x64-setup.exe`（推荐，支持应用内自动更新）或 `*.msi`
3. 安装后打开，点「扫描」

**便携版**：下载 `*_x64-portable.zip`，解压后双击 `AI Disk Cleaner.exe`，设置和日志放在旁边的 `data` 文件夹；便携版不自动更新。

系统要求：Windows 10 / 11 x64，需要 WebView2（Windows 11 自带，Windows 10 安装程序会自动处理）。

> 安装包暂未做代码签名，首次运行 Windows SmartScreen 可能提示「已保护你的电脑」，点「更多信息 → 仍要运行」即可。代码全部开源，可自行审查或从源码构建。

**即将支持**：`winget install nethero23.AIDiskCleaner` 和 `scoop install aidc`（稳定版发布生成清单，正在提交到官方仓库；候选版不生成清单）。

命令行版（开发者）：`cargo run -p cleaner-core --bin aidc -- scan`，支持 `scan --json`、`dupes`、`move`、`compact`。

## 和其他清理软件比

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| 识别 Ollama / HF / LM Studio 模型 | ✅ 具体到模型名 | ❌ | ❌ | ❌ | ❌ 只显示大小 |
| Cursor / Claude Code / VS Code 缓存 | ✅ | ❌ | ✅（6.0 起） | ❌ | ❌ |
| pip / conda / npm / uv 缓存 | ✅ | ❌ | 部分 | ❌ | ❌ |
| 区分缓存和对话记录 | ✅ 标「需确认」+ 可导出 | ❌ | ❌ | ❌ | ❌ |
| 搬到其他盘（自动改环境变量） | ✅ | ❌ | ❌ | ❌ | ❌ |
| 跨工具模型去重 | ✅ 硬链接 | ❌ | ❌ | ❌ | ❌ |
| WSL / Docker vhdx 压缩 | ✅ | ❌ | ❌ | ❌ | ❌ |
| 删除可撤销 | ✅ 回收站 + 一键撤销 | ❌ | ❌ | ❌ | ❌ |
| Windows 系统垃圾 | ✅ 常见项 | ✅ 很全 | ✅ | ✅ 很全 | ❌ |
| 注册表清理 | ❌ 故意不做 | ✅ | ✅ | ✅ | ❌ |
| 免费开源 | ✅ MIT | ❌ 免费版有限制 | ✅ GPL | 免费，已停更 | 个人免费 |
| 中文界面 | ✅ | ✅ | ✅ | ✅ | ✅ |

一句话：**系统垃圾，用哪个都行；AI 和开发工具的垃圾，这是目前专门做这件事的工具。** 两者可以一起用。

## 常见问题

### C 盘（或 D 盘）满了，怎么看是被什么占满的？
打开 Clearwisp 的「空间去哪儿了」，选一个盘，会列出每个顶层文件夹的占用，`Users\<你>` 下的 `AppData\Local`、`Downloads` 等也单独列出。再点「扫描」，结果会具体到每个工具、每个模型。

### Ollama 的模型存在哪里？怎么移到 D 盘？
默认在 `C:\Users\<你>\.ollama\models`。在扫描结果里找到 Ollama 模型，点「搬到其他盘」，选 D 盘目录即可。程序会复制、校验、设置 `OLLAMA_MODELS` 环境变量，之后 Ollama 自动从新位置读取。重启 Ollama 生效。

### Hugging Face 缓存（`.cache\huggingface`）可以删吗？
可以，标为「可重下」：删了之后用到时会重新下载。不想重下的话，用「搬到其他盘」移走（自动设置 `HF_HUB_CACHE`），比删更合适。同一个模型在 LM Studio、ComfyUI 里还有一份的话，用「模型去重」合并。

### Cursor 占了几十 GB 怎么办？`state.vscdb` 能删吗？
Cursor 的缓存、日志、扩展下载缓存可以放心删（标「安全」）。但 `state.vscdb` 里是你**全部的 AI 对话记录**，有人报告涨到 77 GB。我们把它标为「需确认」，默认不勾，清理前可先导出对话记录。

### Claude Code 的 `.claude` 文件夹能删吗？
`~/.claude/projects` 里是会话记录（jsonl），标「需确认」，可以先导出再清。其他缓存类文件可以安全清理。

### pip / conda / npm 缓存怎么清理？
扫描后它们会出现在「Python / 开发缓存」分组里，都是「安全」级：删了下次安装时会自动重新下载。conda 环境会显示环境名和最后使用时间，没用的整个环境也可以清掉。

### WSL 或 Docker 的 `ext4.vhdx` 越来越大怎么缩小？
虚拟磁盘只会变大不会自动缩小。在 Clearwisp 里找到对应的 vhdx，点「压缩」，程序会先 `wsl --shutdown` 再用 diskpart 压缩（需要管理员权限，Windows 家庭版也能用），不会删除里面的数据。

### 只能清 C 盘吗？D 盘、E 盘能扫吗？
能。在「扫描范围」里勾选要扫的盘（设置页和扫描结果页顶部都能改）。其他盘上的模型库、缓存、虚拟磁盘、项目构建产物和大文件都会被找出来。C 盘空间紧张时，也可以把大模型「搬到其他盘」。

### 删错了怎么办？
所有删除都进回收站。清理完成后可以「一键撤销」，也可以去「清理记录」里逐项还原。

### 会不会删掉系统文件、把电脑搞坏？
扫描只读，不写文件；只清理规则库里明确列出的缓存目录，每一项都写了删除后果；默认只勾「安全」级；不碰注册表。正在运行的工具会提示你先关闭。

### 和 CCleaner 比有什么不同？能替代吗？
CCleaner 擅长浏览器和系统垃圾，但不认识 AI 模型、Python 环境和 AI 编辑器缓存。Clearwisp 专门清这些，同时也清常见的系统垃圾。两者可以一起用。

### 免费吗？有广告吗？会上传数据吗？
免费、MIT 开源、无广告、无捆绑。所有扫描和清理都在本地完成；检查应用更新和在线更新规则时会访问 GitHub。「AI 工具」页的 MCP 版本检查默认开启，可在设置中关闭，只向 npm / PyPI 发送包名，不发送配置内容。

### 支持 Mac / Linux 吗？
目前只支持 Windows 10 / 11。扫描引擎是跨平台 Rust，未来可能扩展。

## 路线图

- [x] v0.1–v0.3 扫描、安全等级、回收站删除、运行中检测、搬到其他盘 / 搬回来、模型去重、vhdx 压缩、导出对话记录、在线规则更新
- [x] v0.4–v0.5 全盘大文件、托盘 + 定时扫描、并行扫描、清理前预览、一键清理安全项、规则库 126 条
- [x] v0.6 搜索 / 分组 / 排序、多盘扫描、应用内自动更新、「空间去哪儿了」概览
- [x] v0.7 一键撤销、首次运行引导、增长监控、闲置模型建议、系统深度清理（WinSxS 等）、未识别大目录上报、便携版
- [ ] 上架 winget / Scoop、安装包代码签名
- [ ] v0.8 候选验收：Material Design 3 新界面、AI 工具只读盘点（skills / MCP / 插件）、重复与冲突提示、可关闭的版本检查

`0.8.0-rc.2` 正在准备，尚未发布；已发布候选为 `0.8.0-rc.1`，候选不替代稳定版。发布后从 [Releases 列表](https://github.com/nethero23/ai-disk-cleaner/releases) 手动下载。AI 配置自动合并、同步与修改不在本次范围；验收清单见 [docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md)。

## 参与贡献

缺了你常用的 AI 工具？欢迎 [提 Issue](https://github.com/nethero23/ai-disk-cleaner/issues) 或直接加一条规则（一段 TOML 就行）。开发、构建、规则格式见 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)；版本变化见 [CHANGELOG.md](CHANGELOG.md)，项目进展见 [PROJECT_CONTEXT.md](PROJECT_CONTEXT.md)。

觉得有用的话，点个 ⭐ Star，让更多硬盘告急的人找到它。

## License

[MIT](LICENSE)

<sub>关键词：硬盘清理 · 磁盘空间清理 · C盘清理 · C盘瘦身 · D盘清理 · C盘满了怎么清理 · 磁盘清理工具 · 垃圾清理软件 · 大模型缓存清理 · Ollama 模型迁移 · Hugging Face 缓存清理 · Cursor 缓存清理 · conda 环境清理 · pip 缓存清理 · WSL vhdx 压缩 · Docker 磁盘占用 · CCleaner 替代 · Windows disk cleaner for AI developers</sub>
