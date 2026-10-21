# AI Disk Cleaner

最懂 AI 的 Windows C 盘瘦身工具。先扫描，把每一 GB 翻译成人能看懂的东西（哪个模型、哪个环境、多久没用），标出安全等级，你勾选后再移入回收站，可还原。

A Windows disk cleaner that understands AI tools: Ollama / Hugging Face / ModelScope / LM Studio models, pip / uv / conda caches, Cursor / Windsurf / Claude Code / Codex leftovers, plus ordinary Windows junk. Scan first, decide second, delete reversibly.

## 为什么

CCleaner、360、电脑管家不扫模型和 Python 环境；WizTree 只告诉你哪个文件夹大，不告诉你是什么、能不能删。一台做 AI 的电脑里这些动辄几百 GB。

## 原则

- **只读扫描**：扫描过程不写任何文件。
- **三级安全标签**：安全（纯缓存，自动重建）／可重下（删了要重新下载）／需确认（含用户数据或可能在用）。默认只勾「安全」。
- **可撤销**：删除一律进回收站，并写清理日志。
- **不做**：注册表清理、后台常驻、弹窗、捆绑、上传文件列表。

## 结构

```
crates/cleaner-core/   扫描引擎（纯 Rust，可在任何平台测试）
  rules/*.toml         规则库：每条规则 = 路径模板 + 安全等级 + 删除后果
  src/inspect/         按工具解析内容：Ollama manifest、HF hub、conda env、Cursor workspace…
  src/bin/aidc.rs      命令行：aidc scan [--json] [--en] / dupes / move / compact
src-tauri/             Tauri 2 应用壳（Windows 需要 WebView2）
src/                   React 界面
```

## 开发

```bash
# 引擎测试（任何平台）
cargo test -p cleaner-core
cargo run -p cleaner-core --bin aidc -- scan

# 桌面应用（Windows；需要 Rust、Node 22、WebView2）
npm install
npm run tauri dev
npm run tauri build   # 生成 NSIS / MSI 安装包
```

## 添加规则

在 `crates/cleaner-core/rules/` 里加一段：

```toml
[[rule]]
id = "tool.what"
tool = "Tool"
category = "package_cache"
name_zh = "……"
name_en = "…"
paths = ["${LOCALAPPDATA}/Tool/Cache"]   # 第一个存在的路径生效；all_paths = true 则全部
safety = "safe"                           # safe | redownload | review
consequence_zh = "……"
consequence_en = "…"
```

`cargo test` 会校验所有规则文件。

## 发布

推送 `vX.Y.Z` tag、推送 `release/vX.Y.Z` 分支，或在 Actions 里手动运行 CI 并填 tag，都会在 Windows 上构建 NSIS/MSI 安装包并发布到 GitHub Releases。

## 设置

应用内「设置」页可以：

- 排除目录（扫描结果里每一项也有「排除此目录」）
- 关闭某条规则
- 指定自定义规则目录：放入你自己的 `.toml`（格式同 `crates/cleaner-core/rules/`），同 id 覆盖内置规则
- 在线更新规则：从 `rules_update_url`（默认指向本仓库 main 分支）下载规则文件到应用数据目录。先全部解析成功才写盘。

后台运行：可选「关闭窗口缩到托盘」「开机自启（缩在托盘）」「定时后台扫描」「定时扫描后自动清理安全项」（默认关，只动安全级且工具未运行的项）。后台扫描完成后，垃圾超过阈值就弹系统通知，打开窗口直接看到结果；托盘菜单里也有「立即扫描」。

加载顺序：内置 → 在线更新 → 自定义目录 → 去掉关闭的规则。设置保存在应用数据目录的 `settings.json`。

## 路线

- v0.1 扫描 + 勾选 + 回收站删除
- v0.2 运行中检测、项目构建产物扫描、清理记录、模型搬家到其他盘、跨工具重复模型去重
- v0.3 搬回来、ModelScope、WSL/Docker 虚拟磁盘压缩、清理前导出对话记录、设置页 + 在线规则更新、正式图标
- v0.4 全盘大文件扫描、规则库扩到 110+ 条（ComfyUI / SD WebUI / Msty / vLLM / Xinference / Trae / Kiro / Cline …）、托盘常驻 + 定时扫描、清理记录统计
- v0.5 规则并行扫描 + 进度 + 取消、清理前「看看里面」预览、一键清理安全项 + 定时自动清理、规则库 125+ 条（当前）
- 下一步 安装包签名

License: MIT
