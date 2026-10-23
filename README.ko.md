<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Clearwisp 아이콘">

# Clearwisp: AI 도구를 이해하는 디스크 정리 프로그램

**AI 개발자를 위한 무료 오픈소스 Windows 디스크 정리 도구. C 드라이브, D 드라이브 등 모든 드라이브에서 Ollama / Hugging Face / LM Studio 모델, Cursor / Claude Code / VS Code 캐시, pip / conda / npm 캐시, WSL / Docker 가상 디스크가 차지한 공간을 찾아 비워 줍니다. 먼저 스캔하고, 선택은 사용자가 하며, 모든 삭제는 되돌릴 수 있습니다.**

[![최신 버전](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=download&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![다운로드](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ 다운로드](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [기능](#features) · [다른 도구와 비교](#comparison) · [자주 묻는 질문](#faq)

</div>

> 이 번역은 [영어판](README.en.md)보다 업데이트가 늦을 수 있습니다. UI 언어 12개를 제공하는 `0.9.0-rc.1`은 출시 전 검증 중이며 아직 공개되지 않은 비안정 후보입니다. 이전 공개 후보는 `0.8.0-rc.2`입니다. 검증 후 [릴리스 목록](https://github.com/nethero23/ai-disk-cleaner/releases)에서 NSIS와 포터블 ZIP만 수동 배포할 예정입니다. Latest는 안정판 `v0.7.1`을 유지하며, 후보판은 자동 업데이트되지 않고 MSI, `latest.json`, winget/Scoop 매니페스트를 제공하지 않습니다.
>
> 정리·이동 JSONL 기록은 파일이 없거나 빈 줄뿐이면 빈 기록으로 처리합니다. 읽기·인코딩·레코드 손상은 오류와 재시도를 표시하며, 로딩·오류 중에는 복원·원위치 이동을 막습니다. 휴지통 목록 조회 실패를 영구 삭제로 잘못 표시하지 않습니다.
>
> 로그 검증·열기에 실패하면 파일을 삭제하지 않습니다. 도중 쓰기·동기화 실패 시 후속 항목을 중지하고 실제 성공 통계와 경고를 전경·백그라운드 결과에 유지합니다. 배치 ID나 마지막 줄바꿈 없는 유효한 이전 로그도 지원합니다. 충돌 안전 WAL은 아니며 이동 로그 쓰기의 트랜잭션 처리는 미완료입니다. 오류 시 재시도하고 원본 로그를 보존하며 권한·여유 공간을 확인하세요.
>
> 브라우저 67개 검사(선택 25+기록 30+복구 12), 로컬 자동 검사, 미서명 NSIS 빌드를 통과했습니다. `0.9` 실제 데스크톱 검증은 미완료이며 설치된 앱은 교체하지 않았습니다. 남은 항목: [인수 점검표](docs/RELEASE_CHECKLIST.md).

## 왜 필요한가

AI 작업용 PC에서 디스크를 채우는 것은 흔한 시스템 쓰레기가 아닙니다. **로컬 LLM 가중치, Python 환경, AI 코드 에디터 캐시, WSL / Docker 가상 디스크**가 수백 GB씩 차지하는 경우가 많습니다. CCleaner 같은 도구는 이런 곳을 보지 않고, WizTree는 어떤 폴더가 큰지는 알려 주지만 그게 무엇인지, 지워도 되는지는 알려 주지 않습니다.

게다가 C 드라이브만의 문제가 아닙니다. 모델 라이브러리, conda 환경, Docker 디스크, 프로젝트의 `node_modules`는 D, E 드라이브에도 쌓입니다. Clearwisp는 선택한 모든 드라이브를 스캔합니다.

Clearwisp는 1 GB마다 설명해 줍니다. **어떤 모델인지, 어떤 conda 환경인지, 어떤 도구의 캐시인지, 마지막으로 언제 사용했는지, 지우면 어떻게 되는지.** 항목마다 안전 등급이 표시되고, 체크하기 전에는 아무것도 삭제되지 않으며, 삭제는 휴지통으로 이동하고, 마지막 정리는 클릭 한 번으로 되돌릴 수 있습니다.

<a id="features"></a>
## 기능

### 🧠 AI 도구 쓰레기를 이해합니다 (규칙 136개, 도구 90개 이상)

| 분류 | 지원 |
|---|---|
| 로컬 LLM | Ollama, Hugging Face (hub / datasets), LM Studio, ModelScope, GPT4All, Jan, Msty, llama.cpp, vLLM, Xinference, Open WebUI, AnythingLLM, Cherry Studio, Chatbox |
| 이미지 / 음성 | ComfyUI, Stable Diffusion WebUI, Stability Matrix, Pinokio, Whisper, OpenAI CLIP |
| AI 코딩 도구 | Cursor, Windsurf, Claude Code, Codex, Gemini CLI, Qwen Code, GitHub Copilot, Cline, Roo Code, Kilo Code, Continue, Aider, Trae, Kiro, Zed, OpenCode, Antigravity, Goose |
| Python / 머신러닝 | pip, uv, conda, Poetry, PyTorch, Keras, Triton, PaddlePaddle, Jupyter, Kaggle, Gradio |
| 일반 개발 | npm, pnpm, Yarn, Bun, Cargo, Go, Maven, Gradle, NuGet, Composer, Playwright, Puppeteer, Electron, JetBrains, VS Code |
| 컨테이너 | Docker Desktop, WSL (ext4.vhdx 압축) |
| Windows | Windows 업데이트 캐시, Windows.old, 임시 파일, NVIDIA / AMD / Intel 드라이버 캐시, Edge / Chrome / Firefox 캐시, Office, Teams, OneDrive |

Ollama 매니페스트, Hugging Face 캐시 구조, conda 환경, LM Studio 모델 폴더, Cursor 작업 공간을 읽기 때문에 폴더 크기만이 아니라 **실제 모델 이름과 환경 이름**을 보여 줍니다.

### 🛡️ 안전 우선

- **읽기 전용 스캔**: 스캔 중에는 파일을 하나도 쓰지 않습니다.
- **3단계 안전 등급**: 🟢 안전(순수 캐시, 자동으로 다시 생성) / 🟡 재다운로드 가능 / 🔴 확인 필요(사용자 데이터이거나 사용 중일 수 있음). 기본으로는 "안전"만 체크됩니다.
- **채팅 기록 보호**: Cursor의 `state.vscdb`와 Claude Code의 `~/.claude/projects`에는 대화가 저장되어 있습니다. "확인 필요"로 표시되며 정리 전에 내보낼 수 있습니다.
- **실행 중인 프로세스 감지**: 도구가 실행 중이면 경고하고, 사용 중인 파일은 건드리지 않습니다.
- **되돌리기**: 모든 삭제는 휴지통으로 이동하고 로그를 남깁니다. 마지막 정리를 한 번에 되돌리거나, 기록 페이지에서 항목별로 복원할 수 있습니다.
- **미리 보기**: 삭제 전에 안에 무엇이 있는지 볼 수 있습니다.

### 💾 지우지 않고 공간 확보

- **다른 드라이브로 이동**: Ollama 모델, Hugging Face 캐시, pip / uv / conda 캐시 등을 D 드라이브로 옮깁니다. 복사하고 검증한 뒤 `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR` 등을 설정하거나 디렉터리 정션을 남기므로 모든 도구가 그대로 작동합니다. 언제든 다시 옮겨 올 수 있습니다.
- **도구 간 모델 중복 제거**: HF 캐시, LM Studio, ComfyUI에 있는 같은 가중치를 하드 링크로 합칩니다. 모든 도구가 그대로 작동하고 공간은 한 번만 사용합니다.
- **WSL / Docker 가상 디스크 압축**: `ext4.vhdx`는 커지기만 합니다. 실제 크기로 줄여 줍니다.

### ⚡ 편리함

- **모든 드라이브**: C, D, 외장 디스크 등 체크한 드라이브를 스캔하고, 폴더 특징으로 다른 드라이브에 옮겨 둔 모델과 캐시도 찾습니다
- 드라이브별 "공간은 어디로 갔나?" 개요
- 증가 모니터링: 어떤 폴더가 커지고 있는지 보여 주고, 일주일 증가량이 기준을 넘으면 알림
- 사용하지 않는 모델 제안: 오랫동안 쓰지 않은 모델을 찾아 줍니다
- 시스템 심층 정리: WinSxS 구성 요소 저장소, 복원 지점, 최대 절전 모드 파일, 전송 최적화 캐시, Windows 업데이트 캐시
- 알 수 없는 큰 폴더: 규칙이 아직 모르는 큰 폴더도 보여 주고, 클릭 한 번으로 규칙을 제안
- 큰 파일 스캔, 빌드 산출물(`node_modules`, `target`, `.venv` 등)
- 도구별 그룹, 검색, 정렬. 진행률 표시와 취소가 가능한 병렬 스캔
- 트레이 아이콘, 예약 백그라운드 스캔, 기준 초과 알림, "안전" 항목 자동 정리(기본값 꺼짐)
- 첫 실행 안내와 스캔 후 한 줄 요약
- 앱 내 자동 업데이트, 규칙 온라인 업데이트, TOML로 직접 규칙 작성

### 🚫 하지 않는 것

레지스트리 정리, 귀찮은 팝업, 끼워 팔기 소프트웨어, 파일 목록 업로드는 하지 않습니다. 스캔과 정리는 로컬에서 이루어지며, 앱 및 규칙 업데이트는 GitHub에 연결합니다. AI 도구 페이지의 MCP 버전 확인은 기본으로 켜져 있으며 설정에서 끌 수 있습니다. npm / PyPI에는 패키지 이름만 전송하며 구성 내용은 전송하지 않습니다.

## 설치

1. [최신 릴리스](https://github.com/nethero23/ai-disk-cleaner/releases/latest)를 엽니다
2. `*_x64-setup.exe`(권장, 앱 내 업데이트 지원) 또는 `.msi`를 다운로드합니다
3. 실행한 뒤 **스캔**을 클릭합니다

**포터블 버전**: `*_x64-portable.zip`을 다운로드해 압축을 풀고 `AI Disk Cleaner.exe`를 실행합니다. 설정과 로그는 옆의 `data` 폴더에 저장됩니다. 포터블 버전은 자동 업데이트되지 않습니다.

요구 사항: Windows 10 / 11 x64, WebView2(Windows 11에는 기본 포함, Windows 10은 설치 프로그램이 처리).

> 설치 프로그램은 아직 코드 서명이 되어 있지 않아 SmartScreen이 "Windows의 PC 보호" 메시지를 띄울 수 있습니다. "추가 정보 → 실행"을 클릭하세요. 모든 코드는 공개되어 있습니다.

**곧 지원**: `winget install nethero23.AIDiskCleaner`, `scoop install aidc`.

<a id="comparison"></a>
## 다른 도구와 비교

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| Ollama / HF / LM Studio 모델 | ✅ 모델 이름까지 | ❌ | ❌ | ❌ | ❌ 크기만 |
| Cursor / Claude Code / VS Code 캐시 | ✅ | ❌ | ✅ (6.0부터) | ❌ | ❌ |
| pip / conda / npm / uv 캐시 | ✅ | ❌ | 일부 | ❌ | ❌ |
| 캐시와 채팅 기록 구분 | ✅ 확인 필요 + 내보내기 | ❌ | ❌ | ❌ | ❌ |
| 다른 드라이브로 이동(환경 변수 설정) | ✅ | ❌ | ❌ | ❌ | ❌ |
| 도구 간 모델 중복 제거 | ✅ 하드 링크 | ❌ | ❌ | ❌ | ❌ |
| WSL / Docker vhdx 압축 | ✅ | ❌ | ❌ | ❌ | ❌ |
| 삭제 되돌리기 | ✅ 휴지통 + 되돌리기 | ❌ | ❌ | ❌ | ❌ |
| Windows 시스템 쓰레기 | ✅ 주요 항목 | ✅ 매우 많음 | ✅ | ✅ 매우 많음 | ❌ |
| 레지스트리 정리 | ❌ 의도적으로 안 함 | ✅ | ✅ | ✅ | ❌ |
| 무료 오픈소스 | ✅ MIT | ❌ 무료판 제한 | ✅ GPL | 무료, 개발 중단 | 개인 무료 |

요약: 일반 시스템 쓰레기는 어떤 도구든 괜찮습니다. AI와 개발 도구의 쓰레기는 이를 위해 만든 이 도구가 맞습니다. 함께 사용해도 됩니다.

<a id="faq"></a>
## 자주 묻는 질문

### C 드라이브(또는 D 드라이브)가 가득 찼어요. 무엇이 공간을 차지하는지 어떻게 보나요?
"공간은 어디로 갔나?"를 열고 드라이브를 선택하면 폴더별 사용량이 표시됩니다(`Users\<사용자>\AppData\Local`, `Downloads` 등은 따로 표시). 이어서 스캔하면 도구별, 모델별 사용량을 볼 수 있습니다.

### Ollama 모델은 어디에 저장되나요? 다른 드라이브로 옮기려면?
기본 위치는 `C:\Users\<사용자>\.ollama\models`입니다. 스캔 결과에서 Ollama 항목을 찾아 "다른 드라이브로 이동"을 누르고 D 드라이브의 폴더를 고르세요. 복사와 검증 후 `OLLAMA_MODELS`를 설정합니다. Ollama를 다시 시작하면 새 위치에서 읽습니다.

### Hugging Face 캐시(`.cache\huggingface`)를 지워도 되나요?
됩니다. "재다운로드 가능"으로 분류되며 필요할 때 다시 다운로드됩니다. 다시 받고 싶지 않다면 이동하거나(`HF_HUB_CACHE` 자동 설정), 모델 중복 제거로 LM Studio / ComfyUI와 겹치는 파일을 합치세요.

### Cursor가 수십 GB를 차지해요. `state.vscdb`를 지워도 되나요?
Cursor의 캐시, 로그, 확장 다운로드 캐시는 안전하게 지울 수 있습니다. 하지만 `state.vscdb`에는 **AI 채팅 기록 전체**가 들어 있습니다(77 GB까지 커졌다는 보고도 있습니다). "확인 필요"로 분류되고, 기본으로 체크되지 않으며, 먼저 내보낼 수 있습니다.

### Claude Code의 `.claude` 폴더를 지워도 되나요?
`~/.claude/projects`에는 세션 기록(jsonl)이 있어 "확인 필요"로 분류됩니다. 보관하려면 먼저 내보내세요. 캐시 종류의 파일은 안전하게 지울 수 있습니다.

### pip / conda / npm 캐시는 어떻게 지우나요?
Python / 개발 캐시 그룹에 "안전"으로 표시되며 다음 설치 때 다시 생성됩니다. conda 환경은 이름과 마지막 사용 날짜가 표시되므로 쓰지 않는 환경도 지울 수 있습니다.

### WSL이나 Docker의 `ext4.vhdx` 크기를 줄이려면?
가상 디스크는 저절로 줄어들지 않습니다. vhdx를 선택하고 "압축"을 누르면 `wsl --shutdown`을 실행한 뒤 diskpart의 `compact vdisk`로 압축합니다(관리자 권한 필요, Windows Home에서도 작동). 안의 데이터는 건드리지 않습니다.

### C 드라이브만 정리하나요? D, E 드라이브도 스캔할 수 있나요?
모든 드라이브를 스캔할 수 있습니다. "스캔 범위"에서 드라이브를 체크하세요(설정 페이지와 스캔 결과 상단에서 변경 가능). 다른 드라이브의 모델 라이브러리, 캐시, 가상 디스크, 빌드 산출물, 큰 파일도 모두 찾습니다. C 드라이브가 부족하면 큰 모델을 다른 드라이브로 옮길 수도 있습니다.

### 실수로 지웠어요. 되살릴 수 있나요?
네. 모든 삭제는 휴지통으로 이동합니다. "마지막 정리 되돌리기"를 쓰거나 기록 페이지에서 항목별로 복원하세요.

### Windows가 망가지지 않나요?
스캔은 읽기 전용입니다. 규칙에 등록된 폴더만 정리하며 각 항목에 삭제 결과가 적혀 있습니다. 기본으로는 "안전" 항목만 체크되고, 레지스트리는 건드리지 않으며, 실행 중인 도구는 경고합니다.

### CCleaner를 대체할 수 있나요?
CCleaner가 다루지 않는 AI 모델, Python 환경, AI 에디터 캐시를 다루고, 일반적인 Windows 쓰레기도 정리합니다. 함께 써도 괜찮습니다.

### 무료인가요? 광고나 원격 측정은요?
무료, MIT 라이선스, 광고 없음, 끼워 팔기 없음. 스캔과 정리는 로컬에서 이루어지며, 앱 및 규칙 업데이트는 GitHub에 연결합니다. AI 도구 페이지의 MCP 버전 확인은 기본으로 켜져 있으며 설정에서 끌 수 있습니다. npm / PyPI에는 패키지 이름만 전송하며 구성 내용은 전송하지 않습니다.

### macOS / Linux도 지원하나요?
현재는 Windows 10 / 11만 지원합니다. 스캔 엔진은 크로스 플랫폼 Rust라서 앞으로 확장될 수 있습니다.

## 기여

쓰는 AI 도구가 빠져 있나요? [이슈를 등록](https://github.com/nethero23/ai-disk-cleaner/issues)하거나 규칙을 추가해 주세요(TOML 몇 줄이면 됩니다). 빌드 방법과 규칙 형식은 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)를 보세요.

도움이 됐다면 ⭐를 눌러 주세요. 디스크가 가득 찬 다른 사람들이 찾기 쉬워집니다.

## 라이선스

[MIT](LICENSE)
