<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Clearwisp Symbol">

# Clearwisp: der Festplatten-Cleaner, der KI-Tools versteht

**Kostenloser Open-Source-Festplatten-Cleaner für Windows, gemacht für KI-Entwickler, für C:, D: und jedes andere Laufwerk. Findet und schafft den Platz frei, den Ollama-, Hugging-Face- und LM-Studio-Modelle, Caches von Cursor, Claude Code und VS Code, pip-, conda- und npm-Caches sowie virtuelle WSL- und Docker-Festplatten belegen. Erst scannen, dann entscheiden Sie, und jedes Löschen lässt sich rückgängig machen.**

[![Neueste Version](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=download&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ Download](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [Funktionen](#features) · [Vergleich](#comparison) · [FAQ](#faq)

</div>

> Diese Übersetzung kann hinter der [englischen Fassung](README.en.md) zurückliegen. [0.9.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1) mit 12 UI-Sprachen ist als Vorabkandidat veröffentlicht, nicht als stabile Version. [NSIS](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-setup.exe) oder [portable ZIP](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-portable.zip) manuell herunterladen. Beide öffentlichen Pakete wurden heruntergeladen und per SHA-256 geprüft, aber nicht ausgeführt. Latest bleibt bei der stabilen `v0.7.1`; Kandidaten erhalten keine automatischen Updates, MSI, `latest.json` oder winget/Scoop-Manifeste.
>
> Fehlende JSONL-Protokolle für Bereinigung und Verschiebung gelten als leer; Lese-, Kodierungs- und Datensatzfehler werden mit Wiederholen gemeldet. Wiederherstellen und Zurückverschieben sind beim Laden oder bei Fehlern gesperrt. Eine fehlgeschlagene Papierkorb-Abfrage wird nicht als endgültiger Verlust ausgegeben.
>
> Scheitern Prüfung oder Öffnen des Bereinigungsprotokolls, werden keine Dateien gelöscht. Schreib-/Synchronisierungsfehler stoppen weitere Einträge; tatsächliche Erfolge und Warnungen bleiben in Vorder-/Hintergrundergebnissen erhalten. Gültige Altprotokolle ohne Batch-ID oder abschließenden Zeilenumbruch bleiben lesbar. Dies ist kein absturzsicheres WAL; transaktionale Verschiebungsprotokolle stehen noch aus. Erst Wiederholen nutzen, dann Originalprotokoll behalten und Rechte sowie freien Speicher prüfen.
>
> 67 Browserprüfungen (25 Auswahl + 30 Verlauf + 12 Wiederherstellung), lokale automatische Prüfungen und ein unsignierter NSIS-Build bestanden. Ein lokal gebauter, isolierter portabler `0.9.0-rc.1` zeigte per echtem IPC die Logsperre (OS error 32) und Schäden an beiden Verläufen, behielt Zeilen und erlaubte Wiederholen nach Entsperren bzw. Wiederherstellen der Originalbytes. Nur lesender Scan, 10 Auswahl-/Schutzprüfungen und zwei Verlaufszeilen nach Neustart bestanden; zwei Prüfungen bestätigten nur deaktivierte Zustände ohne Klick. Die ursprüngliche `0.8.0-rc.1` wurde wieder geöffnet, Basis-Hashes von Einstellungen, Verlauf, Programm und Testdaten blieben gleich. Echte UI-Bereinigung/Wiederherstellung, Installation/Upgrade und Ausführung eines Cloud-Release-Pakets wurden nicht getestet. Siehe [Abnahme-Checkliste](docs/RELEASE_CHECKLIST.md).

## Warum

Auf einem PC für KI-Arbeit füllt selten klassischer Systemmüll die Festplatte. Es sind **lokale LLM-Gewichte, Python-Umgebungen, Caches von KI-Code-Editoren und virtuelle WSL-/Docker-Festplatten**, oft Hunderte GB. CCleaner und ähnliche Tools schauen dort nicht nach; WizTree zeigt, welcher Ordner groß ist, aber nicht, was er enthält und ob man ihn löschen darf.

Und es betrifft nicht nur C:. Modellbibliotheken, conda-Umgebungen, Docker-Festplatten und `node_modules` aus Projekten landen auch auf D: und E:. Clearwisp scannt jedes Laufwerk, das Sie auswählen.

Clearwisp erklärt jedes Gigabyte: **welches Modell, welche conda-Umgebung, welcher Tool-Cache, wann zuletzt benutzt und was beim Löschen passiert**. Jeder Eintrag hat eine Sicherheitsstufe, nichts wird gelöscht, bevor Sie es anhaken, gelöscht wird in den Papierkorb, und die letzte Bereinigung lässt sich mit einem Klick rückgängig machen.

<a id="features"></a>
## Funktionen

### 🧠 Kennt den Müll von KI-Tools (136 Regeln, über 90 Tools)

| Kategorie | Abgedeckt |
|---|---|
| Lokale LLMs | Ollama, Hugging Face (hub / datasets), LM Studio, ModelScope, GPT4All, Jan, Msty, llama.cpp, vLLM, Xinference, Open WebUI, AnythingLLM, Cherry Studio, Chatbox |
| Bild / Sprache | ComfyUI, Stable Diffusion WebUI, Stability Matrix, Pinokio, Whisper, OpenAI CLIP |
| KI-Coding-Tools | Cursor, Windsurf, Claude Code, Codex, Gemini CLI, Qwen Code, GitHub Copilot, Cline, Roo Code, Kilo Code, Continue, Aider, Trae, Kiro, Zed, OpenCode, Antigravity, Goose |
| Python / ML | pip, uv, conda, Poetry, PyTorch, Keras, Triton, PaddlePaddle, Jupyter, Kaggle, Gradio |
| Allgemeine Entwicklung | npm, pnpm, Yarn, Bun, Cargo, Go, Maven, Gradle, NuGet, Composer, Playwright, Puppeteer, Electron, JetBrains, VS Code |
| Container | Docker Desktop, WSL (Komprimieren von ext4.vhdx) |
| Windows | Windows-Update-Cache, Windows.old, temporäre Dateien, Treiber-Caches von NVIDIA / AMD / Intel, Caches von Edge / Chrome / Firefox, Office, Teams, OneDrive |

Es liest Ollama-Manifeste, die Cache-Struktur von Hugging Face, conda-Umgebungen, LM-Studio-Modellordner und Cursor-Arbeitsbereiche. So sehen Sie **die tatsächlichen Modell- und Umgebungsnamen**, nicht nur Ordnergrößen.

### 🛡️ Sicherheit zuerst

- **Nur lesender Scan**: Beim Scannen wird keine Datei geschrieben.
- **Drei Sicherheitsstufen**: 🟢 Sicher (reiner Cache, wird automatisch neu erzeugt) / 🟡 Neu herunterladbar / 🔴 Prüfen (Benutzerdaten oder möglicherweise in Gebrauch). Standardmäßig ist nur „Sicher“ angehakt.
- **Schützt den Chatverlauf**: Cursors `state.vscdb` und Claude Codes `~/.claude/projects` enthalten Ihre Unterhaltungen. Sie sind als „Prüfen“ markiert und lassen sich vor dem Bereinigen exportieren.
- **Erkennt laufende Prozesse**: Warnt, wenn ein Tool läuft, statt ihm Dateien unter den Füßen wegzuziehen.
- **Rückgängig**: Alles geht mit Protokoll in den Papierkorb. Die letzte Bereinigung lässt sich mit einem Klick zurücknehmen, einzelne Einträge über den Verlauf wiederherstellen.
- **Vorschau**: Sehen Sie vor dem Löschen in jeden Eintrag hinein.

### 💾 Platz schaffen ohne Löschen

- **Auf ein anderes Laufwerk verschieben**: Ollama-Modelle, den Hugging-Face-Cache, pip-/uv-/conda-Caches und mehr nach D: verschieben. Die App kopiert, prüft und setzt dann `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR` usw. oder hinterlässt eine Verzeichnisverbindung (Junction), sodass jedes Tool weiter funktioniert. Jederzeit zurückverschiebbar.
- **Modell-Duplikate über Tools hinweg**: Dieselben Gewichte im HF-Cache, in LM Studio und ComfyUI werden per Hardlink zusammengeführt. Jedes Tool funktioniert weiter, der Platz wird nur einmal belegt.
- **Virtuelle WSL-/Docker-Festplatten komprimieren**: `ext4.vhdx` wächst nur. Hier schrumpft sie auf ihre echte Größe.

### ⚡ Komfort

- **Alle Laufwerke**: C:, D:, externe Festplatten scannen, je nach Auswahl; findet auf andere Laufwerke verschobene Modelle und Caches an ihrer Ordnerstruktur
- Übersicht „Wo ist mein Speicher hin?“ für jedes Laufwerk
- Wachstumsüberwachung: zeigt, welche Ordner wachsen, mit Benachrichtigung, wenn das Wochenwachstum einen Schwellenwert überschreitet
- Hinweise auf ungenutzte Modelle: findet Modelle, die lange nicht verwendet wurden
- Tiefe Systembereinigung: WinSxS-Komponentenspeicher, Wiederherstellungspunkte, Ruhezustandsdatei, Übermittlungsoptimierungs-Cache, Windows-Update-Cache
- Unbekannte große Ordner: große Ordner, die noch keine Regel kennt, werden ebenfalls aufgelistet, mit Regelvorschlag per Klick
- Scan nach großen Dateien, Build-Ausgaben (`node_modules`, `target`, `.venv` …)
- Nach Tool gruppieren, suchen, sortieren; paralleler Scan mit Fortschritt und Abbruch
- Tray-Symbol, geplante Hintergrundscans, Benachrichtigung bei Überschreiten eines Schwellenwerts, optionales automatisches Bereinigen sicherer Einträge (standardmäßig aus)
- Einführung beim ersten Start und eine Zusammenfassung in einem Satz nach jedem Scan
- Automatische Updates in der App, Online-Regelupdates, eigene Regeln in TOML

### 🚫 Niemals

Keine Registry-Bereinigung, keine nervigen Pop-ups, keine gebündelte Software, kein Hochladen Ihrer Dateiliste. Scannen und Bereinigen erfolgen lokal; App- und Regelupdates greifen auf GitHub zu. Die MCP-Versionsprüfung auf der Seite „KI-Tools“ ist standardmäßig aktiviert und lässt sich in den Einstellungen deaktivieren. Sie sendet nur Paketnamen an npm / PyPI, niemals Konfigurationsinhalte.

## Installation

1. Die [neueste Version](https://github.com/nethero23/ai-disk-cleaner/releases/latest) öffnen
2. `*_x64-setup.exe` (empfohlen, unterstützt Updates in der App) oder die `.msi` herunterladen
3. Starten und auf **Scannen** klicken

**Portabel**: `*_x64-portable.zip` herunterladen, entpacken und `AI Disk Cleaner.exe` starten. Einstellungen und Protokolle liegen im Ordner `data` daneben. Die portable Version aktualisiert sich nicht automatisch.

Voraussetzungen: Windows 10 / 11 x64 mit WebView2 (in Windows 11 enthalten; unter Windows 10 kümmert sich der Installer darum).

> Der Installer ist noch nicht codesigniert, daher kann SmartScreen „Der Computer wurde durch Windows geschützt“ anzeigen. Klicken Sie auf „Weitere Informationen → Trotzdem ausführen“. Der gesamte Code ist offen.

**Demnächst**: `winget install nethero23.AIDiskCleaner` und `scoop install aidc`.

<a id="comparison"></a>
## Vergleich

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| Ollama- / HF- / LM-Studio-Modelle | ✅ mit Modellname | ❌ | ❌ | ❌ | ❌ nur Größe |
| Caches von Cursor / Claude Code / VS Code | ✅ | ❌ | ✅ (ab 6.0) | ❌ | ❌ |
| pip- / conda- / npm- / uv-Caches | ✅ | ❌ | teilweise | ❌ | ❌ |
| Unterscheidet Cache und Chatverlauf | ✅ „Prüfen“ + Export | ❌ | ❌ | ❌ | ❌ |
| Auf anderes Laufwerk verschieben (setzt Umgebungsvariablen) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Modell-Duplikate über Tools hinweg | ✅ Hardlinks | ❌ | ❌ | ❌ | ❌ |
| WSL- / Docker-vhdx komprimieren | ✅ | ❌ | ❌ | ❌ | ❌ |
| Löschen rückgängig machen | ✅ Papierkorb + Rückgängig | ❌ | ❌ | ❌ | ❌ |
| Windows-Systemmüll | ✅ gängige Einträge | ✅ umfangreich | ✅ | ✅ umfangreich | ❌ |
| Registry-Bereinigung | ❌ bewusst nicht | ✅ | ✅ | ✅ | ❌ |
| Kostenlos und Open Source | ✅ MIT | ❌ eingeschränkte Gratisversion | ✅ GPL | kostenlos, nicht mehr gepflegt | privat kostenlos |

Kurz: Für klassischen Systemmüll ist jedes dieser Tools brauchbar. Für Müll von KI- und Entwicklerwerkzeugen ist dies das passende Tool. Beide lassen sich problemlos nebeneinander nutzen.

<a id="faq"></a>
## FAQ

### Mein Laufwerk C: (oder D:) ist voll. Wie sehe ich, was den Platz belegt?
Öffnen Sie „Wo ist mein Speicher hin?“, wählen Sie ein Laufwerk und Sie erhalten eine Aufschlüsselung nach Ordnern (`Users\<Sie>\AppData\Local`, `Downloads` usw. werden einzeln aufgeführt). Ein Scan zeigt danach die Belegung pro Tool und pro Modell.

### Wo speichert Ollama die Modelle, und wie verschiebe ich sie auf ein anderes Laufwerk?
Standardmäßig unter `C:\Users\<Sie>\.ollama\models`. Wählen Sie in den Scanergebnissen den Ollama-Eintrag, dann „Auf anderes Laufwerk verschieben“ und einen Ordner auf D:. Die App kopiert, prüft und setzt `OLLAMA_MODELS`. Nach einem Neustart liest Ollama vom neuen Ort.

### Darf ich den Hugging-Face-Cache (`.cache\huggingface`) löschen?
Ja. Er ist als „Neu herunterladbar“ markiert: Modelle werden bei Bedarf erneut geladen. Wer nicht neu herunterladen möchte, verschiebt ihn (die App setzt `HF_HUB_CACHE`) oder führt Duplikate mit LM Studio / ComfyUI per Modell-Deduplizierung zusammen.

### Cursor belegt Dutzende GB. Darf ich `state.vscdb` löschen?
Cursors Caches, Protokolle und der Download-Cache für Erweiterungen sind sicher löschbar. `state.vscdb` enthält jedoch **Ihren gesamten KI-Chatverlauf** (es gibt Berichte über 77 GB). Die Datei ist als „Prüfen“ markiert, standardmäßig nicht angehakt und kann vorher exportiert werden.

### Darf ich den `.claude`-Ordner von Claude Code löschen?
`~/.claude/projects` enthält Sitzungsprotokolle (jsonl) und ist als „Prüfen“ markiert; exportieren Sie sie vorher, wenn Sie sie behalten möchten. Cache-Dateien sind sicher zu bereinigen.

### Wie leere ich den pip- / conda- / npm-Cache?
Sie erscheinen in der Gruppe Python / Entwicklungs-Caches als „Sicher“ und werden bei der nächsten Installation neu aufgebaut. conda-Umgebungen werden mit Namen und letztem Nutzungsdatum angezeigt, sodass sich ungenutzte Umgebungen ebenfalls entfernen lassen.

### Wie verkleinere ich eine `ext4.vhdx` von WSL oder Docker?
Virtuelle Festplatten schrumpfen nie von selbst. Wählen Sie die vhdx und klicken Sie auf „Komprimieren“: Die App führt `wsl --shutdown` und dann `compact vdisk` von diskpart aus (Administratorrechte nötig; funktioniert auch unter Windows Home). Die Daten darin bleiben unangetastet.

### Bereinigt es nur C:? Kann es D: und E: scannen?
Es scannt jedes Laufwerk. Haken Sie die Laufwerke unter „Scanbereich“ an (in den Einstellungen und oben in den Ergebnissen). Modellbibliotheken, Caches, virtuelle Festplatten, Build-Ausgaben und große Dateien auf anderen Laufwerken werden gefunden. Wenn C: knapp wird, können Sie große Modelle auch auf ein anderes Laufwerk verschieben.

### Ich habe versehentlich etwas gelöscht. Kann ich es zurückholen?
Ja. Alles geht in den Papierkorb. Nutzen Sie „Letzte Bereinigung rückgängig machen“ oder stellen Sie einzelne Einträge im Verlauf wieder her.

### Kann es Windows beschädigen?
Der Scan ist nur lesend. Bereinigt werden nur Ordner aus dem Regelwerk, jeweils mit beschriebener Folge. Standardmäßig ist nur „Sicher“ angehakt, die Registry bleibt unberührt, und laufende Tools werden gemeldet.

### Ist es eine CCleaner-Alternative?
Es deckt ab, was CCleaner nicht kann: KI-Modelle, Python-Umgebungen, Caches von KI-Editoren. Gängigen Windows-Müll bereinigt es ebenfalls. Beide zusammen zu nutzen ist kein Problem.

### Ist es kostenlos? Werbung oder Telemetrie?
Kostenlos, MIT-lizenziert, ohne Werbung und ohne gebündelte Software. Scannen und Bereinigen erfolgen lokal; App- und Regelupdates greifen auf GitHub zu. Die MCP-Versionsprüfung auf der Seite „KI-Tools“ ist standardmäßig aktiviert und lässt sich in den Einstellungen deaktivieren. Sie sendet nur Paketnamen an npm / PyPI, niemals Konfigurationsinhalte.

### macOS / Linux?
Derzeit nur Windows 10 / 11. Die Scan-Engine ist plattformübergreifendes Rust, weitere Plattformen könnten folgen.

## Mitwirken

Fehlt ein KI-Tool, das Sie nutzen? [Issue eröffnen](https://github.com/nethero23/ai-disk-cleaner/issues) oder eine Regel hinzufügen (ein paar Zeilen TOML). Build und Regelformat: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

Wenn es Ihnen ein paar Gigabyte gespart hat, hilft ein ⭐ anderen mit voller Festplatte, es zu finden.

## Lizenz

[MIT](LICENSE)
