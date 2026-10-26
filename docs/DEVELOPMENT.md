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
npm run tauri build -- --bundles nsis # 当前候选仅构建 NSIS；签名/本地覆盖见验收清单
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

## 安装包种类

- **安装包**：Releases 里的 `*-setup.exe`（NSIS，推荐，支持应用内自动更新）或 `*.msi`。
- **便携版**：`*_x64-portable.zip`，解压后双击 `AI Disk Cleaner.exe`，设置和日志放在旁边的 `data` 文件夹；不自动更新。
- **winget / Scoop**：稳定版发布会附带 `*_manifests.zip`，里面是 `nethero23.AIDiskCleaner` 的 winget 清单（三份 YAML）和 Scoop 的 `aidc.json`，哈希已填好；候选版不生成。提交到 [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) 和自己的 Scoop bucket 后，就能 `winget install nethero23.AIDiskCleaner` / `scoop install aidc`。

## 发布

获得发布授权后，推送 `vX.Y.Z` tag、推送 `release/vX.Y.Z` 分支，或在 Actions 里手动运行 CI 并填 tag，都会在 Windows 上构建安装包并发布到 GitHub Releases。tag 必须匹配 `package.json`、Tauri 配置及 Rust workspace 的版本；可先运行 `node tools/release-policy.mjs v0.9.0-rc.1` 只读校验。

带预发布后缀的候选（如已发布的 [v0.9.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1)）仅发布 NSIS 和便携 ZIP，标为 prerelease，不成为 Latest，不生成 MSI、`latest.json` 或 winget/Scoop 清单。候选需从 Releases 列表手动下载；稳定版仍构建 NSIS/MSI，并按签名情况生成更新与清单。当前验收边界见 [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)。

应用内自动更新走 Tauri updater 插件，更新源是 GitHub Releases 里的 `latest.json`（CI 由 `.github/latest_json.py` 生成）。更新包必须签名：仓库 Secrets 里需要 `TAURI_SIGNING_PRIVATE_KEY`（`tauri signer generate` 生成的私钥文件内容）和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（没有密码就留空）；公钥在 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`。没配 Secrets 时发布照常，只是不生成签名和 `latest.json`，应用不会提示这个版本。

## 界面回归验收

先运行 `npm run dev`，再传入已安装的 gstack browse 可执行文件路径：

```sh
node tools/ui-smoke-acceptance.mjs <path-to-gstack-browse>
node tools/ui-history-acceptance.mjs <path-to-gstack-browse>
node tools/ui-recovery-acceptance.mjs <path-to-gstack-browse>
```

三条离线 mock-IPC 流程分别覆盖 25 项通用/勾选、30 项历史与 12 项恢复检查，共 67 项。历史包含加载、错误/重试、旧响应、12 语言窄窗口与加载失败时的操作保护；恢复流程注入 `log_error`，检查前台/后台提示和部分成功后的剩余统计。它们不调用真实清理/还原，也不能证明真实 IPC、回收站或安装流程可用。私有结果写入忽略的 `target/`；详情见 [验收清单](RELEASE_CHECKLIST.md)。

**安装前隔离检查点**：`0.9.0-rc.1` 的本地便携版在 `http://tauri.localhost` 完成真实 IPC／只读检查：锁定测试 `clean-log` 触发 OS error 32，清理日志第 3 行／搬迁日志第 1 行损坏分别显错，保留原两行；解除占用或恢复原字节后重试成功。4 项 137 B 样本扫描、10 项选择／保护检查（含两项只断言禁用、未点击）及正常重启历史检查通过。该阶段重开原安装版后，12 项原装与样本基线哈希不变，未做安装升级或清理／恢复。

**后续真实安装检查点**：已发布 GitHub NSIS 经 368 文件及两个快捷方式的备份／校验和单独安装批准，以「请勿卸载」原位升级 `0.8.0-rc.1 → 0.9.0-rc.1`。真实安装版十项鼠标／Space／分组／占用禁用／默认安全项检查及完整退出重启通过；95 条历史（81 成功）、设置、14 个非扫描 Roaming 配置／规则文件和两个快捷方式哈希不变。49 项／69.4 GB／3.6 秒仅为单机只读扫描观察。增长、概览及 WebView 运行数据可变，不宣称全配置目录字节不变。未执行任何清理／恢复；公开便携程序未运行，MSI、全新 Windows/WebView2、签名和稳定全矩阵仍未覆盖。见[安装复测记录](RELEASE_CHECKLIST.md#github-v09-in-place-upgrade-and-retest-2026-10-24)；私有备份与收据保留在忽略的 `target/github-v09-install-acceptance-20261024/`。

## 恢复日志与失败边界

- `crates/cleaner-core/src/journal.rs` 严格读取清理和搬迁 JSONL：文件不存在或只有空行才为空，读取、UTF-8 或记录解析失败必须返回错误。错误只报告路径/行号，不回显记录内容；有效旧记录无需迁移。
- `clean_log`、`relocations`、`restore_status` 和 `restore_items` 的失败通过真实 Tauri IPC 拒绝返回；不要改回空数组或成功状态来隐藏错误。历史加载/出错时，前端保留已有行但禁止恢复与搬回。
- 清理必须先校验并打开日志；追加或同步失败后停止后续项目。`CleanSummary.log_error` 和 `background-clean.log_error` 必须传到界面，已实际成功的项目不能重新算作失败。
- 回收站枚举失败不等于 `Gone`；恢复只接受成功清理记录，且不可覆盖原路径已有文件。
- 本轮不是 crash-safe WAL；回收站移动与日志写入仍非原子操作，搬迁日志写入事务性也未完成。不要把离线故障注入或只读桌面验证写成已通过真实清理/恢复。

相关引擎回归可运行 `cargo test -p cleaner-core journal::tests`、`cargo test -p cleaner-core clean::tests`、`cargo test -p cleaner-core restore::tests`；完整引擎与桌面验收仍按验收清单执行。

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
disk-cleaner windows c-drive-cleaner disk-space junk-cleaner storage-cleanup
ai-tools ollama huggingface lm-studio llm cursor claude-code
pip-cache conda docker wsl tauri rust ccleaner-alternative
```

About 描述建议：`最懂 AI 的硬盘清理工具 · Free Windows disk cleaner for AI developers, all drives: clean Ollama / Hugging Face / LM Studio models, Cursor / Claude Code / pip / conda caches, WSL & Docker vhdx. Scan first, undo anytime.`

Website 栏可填 Releases 页链接。

## README 多语言

`README.md`（简体中文）和 `README.en.md`（英文）是主版本。`README.ja/ko/de/fr/es/pt-BR/ru/ar/hi.md` 是从英文版翻译的，不含路线图和开发说明。改功能描述时先改中英文两份，再同步其他语言；每份顶部都写了「可能落后于英文版」。
