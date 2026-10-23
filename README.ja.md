<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Clearwisp アイコン">

# Clearwisp：AI ツールを理解するディスククリーナー

**AI 開発者のための無料・オープンソースの Windows ディスククリーナー。C ドライブも D ドライブも、すべてのドライブを対象に、Ollama / Hugging Face / LM Studio のモデル、Cursor / Claude Code / VS Code のキャッシュ、pip / conda / npm のキャッシュ、WSL / Docker の仮想ディスクが使っている容量を見つけて解放します。まずスキャン、選ぶのはあなた、削除はすべて元に戻せます。**

[![最新版](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=download&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![ダウンロード数](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ ダウンロード](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [機能](#features) · [他ツールとの比較](#comparison) · [よくある質問](#faq)

</div>

> この翻訳は [英語版](README.en.md) より更新が遅れることがあります。UI 12言語対応の [0.9.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1) は公開済みのプレリリース候補で、安定版ではありません。[NSIS](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-setup.exe) または[ポータブル ZIP](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-portable.zip) を手動で取得できます。両公開パッケージはダウンロードして SHA-256 を検証しましたが、実行していません。Latest は安定版 `v0.7.1` のまま。候補版は自動更新せず、MSI、`latest.json`、winget/Scoop マニフェストを含みません。
>
> クリーンアップ／移動履歴の JSONL は、存在しないか空行のみの場合は空と扱います。読み取り・文字コード・レコード破損はエラーと再試行を表示し、読み込み中やエラー時は復元／移動を戻す操作を無効にします。ごみ箱の一覧取得失敗を「完全に消失」と誤表示しません。
>
> 履歴の検証・オープンに失敗すると削除せず、途中の書き込み／同期失敗では後続項目を停止し、実際の成功分と警告を前面／バックグラウンド結果に残します。バッチ ID や末尾改行のない有効な旧記録も対応。クラッシュ安全な WAL ではなく、移動ログ書き込みのトランザクション化は未完了です。エラー時は再試行し、元ログを保持して権限と空き容量を確認してください。
>
> ブラウザー67項目（選択25＋履歴30＋復元12）、ローカル自動検査、未署名 NSIS ビルドが合格。ローカル構築の隔離ポータブル `0.9.0-rc.1` で、ログ占有（OS error 32）と両履歴の破損を実 IPC で表示し、既存行の保持と解除／元バイト復元後の再試行を確認。読み取り専用スキャン、選択・保護10項目、再起動後の履歴2行も合格（2項目は無効状態の確認のみでクリックなし）。元の `0.8.0-rc.1` を再起動し、設定・履歴・プログラム・試料の基準ハッシュは不変。実 UI の削除／復元、インストール更新、クラウド公開パッケージの実行は未検証です。[受け入れチェックリスト](docs/RELEASE_CHECKLIST.md) を参照してください。

## なぜ必要か

AI の作業に使う PC では、ディスクを埋めるのは従来のシステムゴミではありません。**ローカル LLM の重み、Python 環境、AI コードエディタのキャッシュ、WSL / Docker の仮想ディスク**で、数百 GB になることもよくあります。CCleaner などはそこを見ません。WizTree はどのフォルダが大きいかは教えてくれますが、それが何なのか、消してよいのかは教えてくれません。

しかも C ドライブだけではありません。モデルライブラリ、conda 環境、Docker のディスク、プロジェクトの `node_modules` は D や E ドライブにもたまります。Clearwisp は選んだすべてのドライブをスキャンします。

Clearwisp は 1 GB ごとに説明します。**どのモデルか、どの conda 環境か、どのツールのキャッシュか、最後に使ったのはいつか、消すとどうなるか**。各項目に安全レベルが付き、チェックを入れるまで何も削除されず、削除はごみ箱へ移動し、直前のクリーンアップはワンクリックで元に戻せます。

<a id="features"></a>
## 機能

### 🧠 AI ツールのゴミを理解（136 ルール、90 以上のツール）

| 分類 | 対応 |
|---|---|
| ローカル LLM | Ollama、Hugging Face（hub / datasets）、LM Studio、ModelScope、GPT4All、Jan、Msty、llama.cpp、vLLM、Xinference、Open WebUI、AnythingLLM、Cherry Studio、Chatbox |
| 画像 / 音声 | ComfyUI、Stable Diffusion WebUI、Stability Matrix、Pinokio、Whisper、OpenAI CLIP |
| AI コーディングツール | Cursor、Windsurf、Claude Code、Codex、Gemini CLI、Qwen Code、GitHub Copilot、Cline、Roo Code、Kilo Code、Continue、Aider、Trae、Kiro、Zed、OpenCode、Antigravity、Goose |
| Python / 機械学習 | pip、uv、conda、Poetry、PyTorch、Keras、Triton、PaddlePaddle、Jupyter、Kaggle、Gradio |
| 一般的な開発 | npm、pnpm、Yarn、Bun、Cargo、Go、Maven、Gradle、NuGet、Composer、Playwright、Puppeteer、Electron、JetBrains、VS Code |
| コンテナ | Docker Desktop、WSL（ext4.vhdx の圧縮） |
| Windows | Windows Update キャッシュ、Windows.old、一時ファイル、NVIDIA / AMD / Intel ドライバキャッシュ、Edge / Chrome / Firefox キャッシュ、Office、Teams、OneDrive |

Ollama のマニフェスト、Hugging Face のキャッシュ構造、conda 環境、LM Studio のモデルフォルダ、Cursor のワークスペースを読み取るので、フォルダサイズだけでなく**実際のモデル名や環境名**が分かります。

### 🛡️ 安全第一

- **読み取り専用のスキャン**：スキャン中はファイルを一切書き込みません。
- **3 段階の安全レベル**：🟢 安全（純粋なキャッシュ、自動で再生成）／🟡 再ダウンロード可／🔴 要確認（ユーザーデータ、または使用中の可能性）。初期状態でチェックされるのは「安全」だけです。
- **チャット履歴を保護**：Cursor の `state.vscdb` と Claude Code の `~/.claude/projects` には会話が保存されています。「要確認」と表示され、削除前にエクスポートできます。
- **実行中のプロセスを検出**：ツールが動いているときは警告し、使用中のファイルには触れません。
- **元に戻せる**：削除はすべてごみ箱へ移動し、ログを残します。直前のクリーンアップをワンクリックで取り消すか、履歴ページから項目ごとに復元できます。
- **プレビュー**：削除前に中身を確認できます。

### 💾 消さずに空ける

- **別ドライブへ移動**：Ollama のモデル、Hugging Face のキャッシュ、pip / uv / conda のキャッシュなどを D ドライブへ移動。コピーして検証してから `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR` などを設定するか、ディレクトリジャンクションを残すので、各ツールはそのまま動きます。いつでも戻せます。
- **ツール間のモデル重複排除**：HF キャッシュ、LM Studio、ComfyUI にある同じ重みをハードリンクで 1 つにまとめます。どのツールも動き続け、容量は 1 つ分だけになります。
- **WSL / Docker の仮想ディスクを圧縮**：`ext4.vhdx` は大きくなる一方です。実際のサイズまで縮めます。

### ⚡ 便利

- **すべてのドライブ**：C、D、外付けディスクなど、チェックしたドライブをスキャン。フォルダの特徴から、別ドライブに移したモデルやキャッシュも見つけます
- ドライブごとの「容量はどこへ？」概要
- 増加の監視：どのフォルダが増えているかを表示し、1 週間の増加がしきい値を超えると通知
- 使っていないモデルの提案：長期間使っていないモデルを見つけます
- システムの詳細クリーンアップ：WinSxS コンポーネントストア、復元ポイント、休止状態ファイル、配信の最適化キャッシュ、Windows Update キャッシュ
- 未知の大きなフォルダ：ルールがまだ知らない大きなフォルダも表示し、ワンクリックでルールを提案
- 大きなファイルのスキャン、ビルド成果物（`node_modules`、`target`、`.venv` など）
- ツール別グループ、検索、並べ替え。進捗表示とキャンセル付きの並列スキャン
- タスクトレイ常駐、定期的なバックグラウンドスキャン、しきい値超過の通知、「安全」項目の自動クリーンアップ（初期状態はオフ）
- 初回起動ガイドと、スキャン後のひとこと要約
- アプリ内自動更新、ルールのオンライン更新、TOML で独自ルールも作成可能

### 🚫 やらないこと

レジストリのクリーンアップ、しつこいポップアップ、バンドルソフト、ファイル一覧のアップロードはしません。スキャンとクリーンアップはローカルで行われます。アプリとルールの更新では GitHub に接続します。「AI ツール」ページの MCP バージョン確認は既定で有効ですが、設定で無効にできます。npm / PyPI に送信するのはパッケージ名のみで、設定内容は送信しません。

## インストール

1. [最新リリース](https://github.com/nethero23/ai-disk-cleaner/releases/latest)を開く
2. `*_x64-setup.exe`（推奨、アプリ内更新に対応）または `.msi` をダウンロード
3. 起動して **スキャン** をクリック

**ポータブル版**：`*_x64-portable.zip` をダウンロードして展開し、`AI Disk Cleaner.exe` を実行します。設定とログは隣の `data` フォルダに保存されます。ポータブル版は自動更新しません。

動作環境：Windows 10 / 11 x64、WebView2（Windows 11 は標準搭載、Windows 10 はインストーラーが対応）。

> インストーラーはまだコード署名されていないため、SmartScreen に「Windows によって PC が保護されました」と表示されることがあります。「詳細情報 → 実行」をクリックしてください。コードはすべて公開されています。

**近日対応**：`winget install nethero23.AIDiskCleaner` と `scoop install aidc`。

<a id="comparison"></a>
## 他ツールとの比較

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| Ollama / HF / LM Studio のモデル | ✅ モデル名まで | ❌ | ❌ | ❌ | ❌ サイズのみ |
| Cursor / Claude Code / VS Code のキャッシュ | ✅ | ❌ | ✅（6.0 以降） | ❌ | ❌ |
| pip / conda / npm / uv のキャッシュ | ✅ | ❌ | 一部 | ❌ | ❌ |
| キャッシュとチャット履歴を区別 | ✅ 要確認 + エクスポート | ❌ | ❌ | ❌ | ❌ |
| 別ドライブへ移動（環境変数を設定） | ✅ | ❌ | ❌ | ❌ | ❌ |
| ツール間のモデル重複排除 | ✅ ハードリンク | ❌ | ❌ | ❌ | ❌ |
| WSL / Docker vhdx の圧縮 | ✅ | ❌ | ❌ | ❌ | ❌ |
| 削除の取り消し | ✅ ごみ箱 + 取り消し | ❌ | ❌ | ❌ | ❌ |
| Windows のシステムゴミ | ✅ 主な項目 | ✅ 豊富 | ✅ | ✅ 豊富 | ❌ |
| レジストリのクリーンアップ | ❌ あえてしない | ✅ | ✅ | ✅ | ❌ |
| 無料・オープンソース | ✅ MIT | ❌ 無料版は制限あり | ✅ GPL | 無料、開発終了 | 個人は無料 |

まとめ：従来のシステムゴミならどれでも構いません。AI や開発ツールのゴミには、そのために作られたこのツールを。併用しても問題ありません。

<a id="faq"></a>
## よくある質問

### C ドライブ（または D ドライブ）がいっぱいです。何が容量を使っているか確認するには？
「容量はどこへ？」を開いてドライブを選ぶと、フォルダごとの内訳が表示されます（`Users\<ユーザー>\AppData\Local` や `Downloads` などは個別に表示）。続けてスキャンすると、ツールごと・モデルごとの使用量が分かります。

### Ollama のモデルはどこに保存されますか？別のドライブへ移すには？
既定では `C:\Users\<ユーザー>\.ollama\models` です。スキャン結果で Ollama の項目を選び、「別ドライブへ移動」で D ドライブのフォルダを選びます。コピー、検証のあと `OLLAMA_MODELS` を設定します。Ollama を再起動すると新しい場所から読み込みます。

### Hugging Face のキャッシュ（`.cache\huggingface`）は削除できますか？
できます。「再ダウンロード可」に分類され、必要になったときに再びダウンロードされます。再ダウンロードしたくない場合は移動（`HF_HUB_CACHE` を自動設定）するか、モデル重複排除で LM Studio / ComfyUI との重複をまとめてください。

### Cursor が数十 GB を使っています。`state.vscdb` は消せますか？
Cursor のキャッシュ、ログ、拡張機能のダウンロードキャッシュは安全に削除できます。ただし `state.vscdb` には **AI とのチャット履歴がすべて**入っています（77 GB に達した報告もあります）。「要確認」に分類され、初期状態ではチェックされず、先にエクスポートできます。

### Claude Code の `.claude` フォルダは削除できますか？
`~/.claude/projects` にはセッション記録（jsonl）があり、「要確認」に分類されます。残したい場合は先にエクスポートしてください。キャッシュ類は安全に削除できます。

### pip / conda / npm のキャッシュを消すには？
Python / 開発キャッシュのグループに「安全」として表示され、次のインストール時に再生成されます。conda 環境は名前と最終使用日付きで表示されるので、使っていない環境も削除できます。

### WSL や Docker の `ext4.vhdx` を小さくするには？
仮想ディスクは自動では縮みません。vhdx を選んで「圧縮」をクリックすると、`wsl --shutdown` を実行してから diskpart の `compact vdisk` で圧縮します（管理者権限が必要、Windows Home でも動作）。中のデータには触れません。

### C ドライブだけですか？D や E ドライブもスキャンできますか？
どのドライブもスキャンできます。「スキャン範囲」でドライブにチェックを入れてください（設定画面とスキャン結果の上部で変更できます）。他のドライブにあるモデルライブラリ、キャッシュ、仮想ディスク、ビルド成果物、大きなファイルも見つかります。C ドライブが足りないときは、大きなモデルを別ドライブへ移すこともできます。

### 間違えて削除しました。戻せますか？
戻せます。削除はすべてごみ箱へ移動します。「直前のクリーンアップを取り消す」を使うか、履歴ページから項目ごとに復元してください。

### Windows が壊れることはありませんか？
スキャンは読み取り専用です。削除するのはルールに載っているフォルダだけで、それぞれ削除の影響が書かれています。初期状態では「安全」だけがチェックされ、レジストリには触れず、実行中のツールは警告されます。

### CCleaner の代わりになりますか？
CCleaner が扱わない AI モデル、Python 環境、AI エディタのキャッシュを扱い、一般的な Windows のゴミも掃除します。併用しても問題ありません。

### 無料ですか？広告やテレメトリは？
無料、MIT ライセンス、広告なし、バンドルなし。スキャンとクリーンアップはローカルで行われます。アプリとルールの更新では GitHub に接続します。「AI ツール」ページの MCP バージョン確認は既定で有効ですが、設定で無効にできます。npm / PyPI に送信するのはパッケージ名のみで、設定内容は送信しません。

### macOS / Linux には対応していますか？
今のところ Windows 10 / 11 のみです。スキャンエンジンはクロスプラットフォームの Rust なので、今後対応する可能性があります。

## 貢献

使っている AI ツールが対応していませんか？[Issue を作成](https://github.com/nethero23/ai-disk-cleaner/issues)するか、ルールを追加してください（数行の TOML です）。ビルド方法とルール形式は [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) を参照してください。

役に立ったら ⭐ をお願いします。ディスクがいっぱいで困っている人が見つけやすくなります。

## ライセンス

[MIT](LICENSE)
