# 开发文档 / Development

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

应用内自动更新走 Tauri updater 插件，更新源是 GitHub Releases 里的 `latest.json`（CI 由 `.github/latest_json.py` 生成）。更新包必须签名：仓库 Secrets 里需要 `TAURI_SIGNING_PRIVATE_KEY`（`tauri signer generate` 生成的私钥文件内容）和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（没有密码就留空）；公钥在 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`。没配 Secrets 时发布照常，只是不生成签名和 `latest.json`，应用不会提示这个版本。

## 设置

应用内「设置」页可以：

- 排除目录（扫描结果里每一项也有「排除此目录」）
- 关闭某条规则
- 指定自定义规则目录：放入你自己的 `.toml`（格式同 `crates/cleaner-core/rules/`），同 id 覆盖内置规则
- 在线更新规则：从 `rules_update_url`（默认指向本仓库 main 分支）下载规则文件到应用数据目录。先全部解析成功才写盘。

后台运行：可选「关闭窗口缩到托盘」「开机自启（缩在托盘）」「定时后台扫描」「定时扫描后自动清理安全项」（默认关，只动安全级且工具未运行的项）。后台扫描完成后，垃圾超过阈值就弹系统通知，打开窗口直接看到结果；托盘菜单里也有「立即扫描」。

加载顺序：内置 → 在线更新 → 自定义目录 → 去掉关闭的规则。设置保存在应用数据目录的 `settings.json`。

## 仓库 Topics 建议

在 GitHub 仓库首页右侧 About → ⚙ 里填（最多 20 个）：

```
disk-cleaner windows c-drive-cleaner junk-cleaner storage-cleanup
ai-tools ollama huggingface lm-studio llm cursor claude-code
pip-cache conda docker wsl tauri rust cleaner ccleaner-alternative
```

About 描述建议：`最懂 AI 的 Windows C 盘瘦身工具 · Free disk cleaner for AI developers: clean Ollama / Hugging Face / LM Studio models, Cursor / Claude Code / pip / conda caches, WSL & Docker vhdx. Scan first, undo anytime.`

Website 栏可填 Releases 页链接。
