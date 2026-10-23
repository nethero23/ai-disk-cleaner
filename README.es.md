<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Icono de Clearwisp">

# Clearwisp: el limpiador de disco que entiende las herramientas de IA

**Limpiador de disco para Windows, gratuito y de código abierto, pensado para desarrolladores de IA, para C:, D: y cualquier otra unidad. Encuentra y libera el espacio que ocupan los modelos de Ollama / Hugging Face / LM Studio, las cachés de Cursor / Claude Code / VS Code, las cachés de pip / conda / npm y los discos virtuales de WSL / Docker. Primero analiza, después decides tú, y cada borrado se puede deshacer.**

[![Última versión](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=download&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![Descargas](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ Descargar](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [Funciones](#features) · [Comparativa](#comparison) · [Preguntas frecuentes](#faq)

</div>

> Esta traducción puede ir por detrás de la [versión en inglés](README.en.md). [0.9.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1), con 12 idiomas de interfaz, está publicada como candidata preliminar, no como versión estable. Descarga manualmente [NSIS](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-setup.exe) o el [ZIP portable](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-portable.zip). Ambos paquetes públicos se descargaron y verificaron por SHA-256, pero no se ejecutaron. Latest sigue en la estable `v0.7.1`; las candidatas no incluyen actualización automática, MSI, `latest.json` ni manifiestos winget/Scoop.
>
> Los registros JSONL de limpieza/traslado inexistentes se consideran vacíos; los fallos de lectura, codificación o datos se muestran con Reintentar. Restaurar y devolver al origen siguen bloqueados durante la carga o los errores. Un fallo al enumerar la Papelera ya no se presenta como pérdida definitiva.
>
> Si falla validar o abrir el registro, no se limpian archivos. Un fallo de escritura/sincronización detiene los elementos posteriores, conserva los éxitos reales y muestra una advertencia en resultados de primer y segundo plano. Los registros antiguos válidos sin ID de lote ni salto de línea final siguen siendo compatibles. No es un WAL seguro ante fallos; queda pendiente la escritura transaccional de traslados. Reintenta y conserva el registro original mientras compruebas permisos y espacio libre.
>
> Pasaron 67 comprobaciones de navegador (25 selección + 30 historial + 12 recuperación), las pruebas automáticas locales y una compilación NSIS sin firma. Una copia portable `0.9.0-rc.1` local y aislada mostró mediante IPC real el bloqueo del registro (OS error 32) y daños en ambos historiales, conservó las filas y permitió reintentar tras desbloquear o restaurar los bytes originales. Pasaron el análisis de solo lectura, 10 comprobaciones de selección/protección y dos filas de historial tras reiniciar; dos comprobaciones solo verificaron estados desactivados, sin pulsarlos. Se reabrió la original `0.8.0-rc.1`, con hashes de ajustes, historial, programa y muestras intactos. No se probaron limpieza/restauración real en la interfaz, instalación/actualización ni ejecución de un paquete publicado en la nube. Véase la [lista de aceptación](docs/RELEASE_CHECKLIST.md).

## Por qué

En un PC que se usa para IA, lo que llena el disco rara vez es la basura clásica del sistema. Son **los pesos de LLM locales, los entornos de Python, las cachés de los editores de código con IA y los discos virtuales de WSL / Docker**, a menudo cientos de GB. CCleaner y herramientas parecidas no miran ahí; WizTree te dice qué carpeta es grande, pero no qué es ni si se puede borrar.

Y no es solo C:. Las bibliotecas de modelos, los entornos de conda, los discos de Docker y los `node_modules` de los proyectos también acaban en D: y E:. Clearwisp analiza todas las unidades que elijas.

Clearwisp explica cada gigabyte: **qué modelo, qué entorno de conda, qué caché de qué herramienta, cuándo se usó por última vez y qué pasa si lo borras**. Cada elemento tiene un nivel de seguridad, no se borra nada hasta que lo marcas, lo borrado va a la Papelera de reciclaje y la última limpieza se deshace con un clic.

<a id="features"></a>
## Funciones

### 🧠 Conoce la basura de las herramientas de IA (136 reglas, más de 90 herramientas)

| Categoría | Cubre |
|---|---|
| LLM locales | Ollama, Hugging Face (hub / datasets), LM Studio, ModelScope, GPT4All, Jan, Msty, llama.cpp, vLLM, Xinference, Open WebUI, AnythingLLM, Cherry Studio, Chatbox |
| Imagen / voz | ComfyUI, Stable Diffusion WebUI, Stability Matrix, Pinokio, Whisper, OpenAI CLIP |
| Herramientas de programación con IA | Cursor, Windsurf, Claude Code, Codex, Gemini CLI, Qwen Code, GitHub Copilot, Cline, Roo Code, Kilo Code, Continue, Aider, Trae, Kiro, Zed, OpenCode, Antigravity, Goose |
| Python / ML | pip, uv, conda, Poetry, PyTorch, Keras, Triton, PaddlePaddle, Jupyter, Kaggle, Gradio |
| Desarrollo general | npm, pnpm, Yarn, Bun, Cargo, Go, Maven, Gradle, NuGet, Composer, Playwright, Puppeteer, Electron, JetBrains, VS Code |
| Contenedores | Docker Desktop, WSL (compactación de ext4.vhdx) |
| Windows | caché de Windows Update, Windows.old, archivos temporales, cachés de controladores NVIDIA / AMD / Intel, cachés de Edge / Chrome / Firefox, Office, Teams, OneDrive |

Lee los manifiestos de Ollama, la estructura de la caché de Hugging Face, los entornos de conda, las carpetas de modelos de LM Studio y los espacios de trabajo de Cursor, así que ves **los nombres reales de modelos y entornos**, no solo el tamaño de las carpetas.

### 🛡️ La seguridad primero

- **Análisis de solo lectura**: el análisis no escribe ningún archivo.
- **Tres niveles de seguridad**: 🟢 Seguro (caché pura, se regenera sola) / 🟡 Se puede volver a descargar / 🔴 Revisar (datos del usuario o posiblemente en uso). Por defecto solo se marca «Seguro».
- **Protege el historial de chats**: el `state.vscdb` de Cursor y el `~/.claude/projects` de Claude Code guardan tus conversaciones. Se marcan como «Revisar» y se pueden exportar antes de limpiar.
- **Detección de procesos en ejecución**: te avisa cuando una herramienta está abierta en lugar de quitarle los archivos que usa.
- **Deshacer**: todo va a la Papelera con un registro de limpieza. Deshaz la última limpieza con un clic o restaura elementos sueltos desde el historial.
- **Vista previa**: mira dentro de cualquier elemento antes de borrarlo.

### 💾 Libera espacio sin borrar

- **Mover a otra unidad**: lleva los modelos de Ollama, la caché de Hugging Face, las cachés de pip / uv / conda y más a D:. La aplicación copia, verifica y luego define `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR`, etc., o deja una unión de directorio, así que cada herramienta sigue funcionando. Puedes devolverlo cuando quieras.
- **Deduplicación de modelos entre herramientas**: los mismos pesos en la caché de HF, LM Studio y ComfyUI se unen con enlaces físicos. Todas las herramientas siguen funcionando y el espacio se usa una sola vez.
- **Compactar discos virtuales de WSL / Docker**: `ext4.vhdx` solo crece. Redúcelo a su tamaño real.

### ⚡ Cómodo

- **Todas las unidades**: analiza C:, D:, discos externos, las que marques; encuentra modelos y cachés movidos a otras unidades por la estructura de sus carpetas
- Vista «¿Adónde se fue mi espacio?» para cada unidad
- Monitor de crecimiento: muestra qué carpetas crecen, con aviso cuando el crecimiento semanal supera un umbral
- Modelos sin uso: encuentra modelos que llevas mucho tiempo sin usar
- Limpieza profunda del sistema: almacén de componentes WinSxS, puntos de restauración, archivo de hibernación, caché de Optimización de distribución, caché de Windows Update
- Carpetas grandes no reconocidas: también se listan las carpetas grandes que ninguna regla conoce todavía, con sugerencia de regla en un clic
- Búsqueda de archivos grandes, resultados de compilación (`node_modules`, `target`, `.venv`…)
- Agrupar por herramienta, buscar, ordenar; análisis en paralelo con progreso y cancelación
- Icono en la bandeja, análisis programados en segundo plano, aviso al superar un umbral, limpieza automática opcional de los elementos seguros (desactivada por defecto)
- Guía en el primer inicio y resumen de una frase tras cada análisis
- Actualización automática integrada, reglas actualizables en línea, reglas propias en TOML

### 🚫 Nunca

Sin limpieza del registro, sin ventanas molestas, sin software incluido, sin subir tu lista de archivos. El análisis y la limpieza se hacen en local; las actualizaciones de la aplicación y de las reglas acceden a GitHub. La comprobación de versiones MCP en la página «Herramientas de IA» está activada de forma predeterminada y se puede desactivar en Ajustes. Solo envía nombres de paquetes a npm / PyPI, nunca el contenido de la configuración.

## Instalación

1. Abre la [última versión](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
2. Descarga `*_x64-setup.exe` (recomendado, admite actualizaciones integradas) o el `.msi`
3. Ábrelo y pulsa **Analizar**

**Portable**: descarga `*_x64-portable.zip`, descomprímelo y ejecuta `AI Disk Cleaner.exe`. Los ajustes y registros se guardan en la carpeta `data` de al lado. La versión portable no se actualiza sola.

Requisitos: Windows 10 / 11 x64 con WebView2 (incluido en Windows 11; en Windows 10 lo gestiona el instalador).

> El instalador aún no tiene firma de código, así que SmartScreen puede mostrar «Windows protegió su PC». Pulsa «Más información → Ejecutar de todas formas». Todo el código es abierto.

**Próximamente**: `winget install nethero23.AIDiskCleaner` y `scoop install aidc`.

<a id="comparison"></a>
## Comparativa

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| Modelos de Ollama / HF / LM Studio | ✅ por nombre de modelo | ❌ | ❌ | ❌ | ❌ solo tamaño |
| Cachés de Cursor / Claude Code / VS Code | ✅ | ❌ | ✅ (desde 6.0) | ❌ | ❌ |
| Cachés de pip / conda / npm / uv | ✅ | ❌ | parcial | ❌ | ❌ |
| Distingue caché e historial de chats | ✅ «Revisar» + exportar | ❌ | ❌ | ❌ | ❌ |
| Mover a otra unidad (define variables de entorno) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Deduplicación de modelos entre herramientas | ✅ enlaces físicos | ❌ | ❌ | ❌ | ❌ |
| Compactar vhdx de WSL / Docker | ✅ | ❌ | ❌ | ❌ | ❌ |
| Borrado que se puede deshacer | ✅ Papelera + deshacer | ❌ | ❌ | ❌ | ❌ |
| Basura del sistema Windows | ✅ lo habitual | ✅ muy completo | ✅ | ✅ muy completo | ❌ |
| Limpieza del registro | ❌ a propósito | ✅ | ✅ | ✅ | ❌ |
| Gratuito y de código abierto | ✅ MIT | ❌ versión gratuita limitada | ✅ GPL | gratis, sin mantenimiento | gratis para uso personal |

En resumen: para la basura clásica del sistema sirve cualquiera. Para la basura de las herramientas de IA y de desarrollo, esta es la herramienta hecha para eso. Puedes usarlas juntas sin problema.

<a id="faq"></a>
## Preguntas frecuentes

### Mi disco C: (o D:) está lleno. ¿Cómo veo qué ocupa el espacio?
Abre «¿Adónde se fue mi espacio?», elige una unidad y verás el reparto por carpetas (`Users\<tú>\AppData\Local`, `Downloads`, etc. aparecen por separado). Después lanza un análisis para ver el uso por herramienta y por modelo.

### ¿Dónde guarda Ollama los modelos y cómo los muevo a otra unidad?
Por defecto en `C:\Users\<tú>\.ollama\models`. En los resultados busca el elemento de Ollama, elige «Mover a otra unidad» y una carpeta en D:. La aplicación copia, verifica y define `OLLAMA_MODELS`. Reinicia Ollama y leerá desde la nueva ubicación.

### ¿Puedo borrar la caché de Hugging Face (`.cache\huggingface`)?
Sí. Está marcada como «Se puede volver a descargar»: los modelos se descargan de nuevo cuando hagan falta. Si no quieres volver a descargarlos, muévela (la aplicación define `HF_HUB_CACHE`) o une los duplicados con LM Studio / ComfyUI con la deduplicación de modelos.

### Cursor ocupa decenas de GB. ¿Puedo borrar `state.vscdb`?
Las cachés, los registros y la caché de descargas de extensiones de Cursor se pueden borrar sin riesgo. Pero `state.vscdb` guarda **todo tu historial de chats con la IA** (hay casos de 77 GB). Está marcado como «Revisar», no viene marcado por defecto y se puede exportar antes.

### ¿Puedo borrar la carpeta `.claude` de Claude Code?
`~/.claude/projects` contiene las transcripciones de sesiones (jsonl) y está marcada como «Revisar»; expórtalas antes si quieres conservarlas. Los archivos de tipo caché se pueden limpiar sin riesgo.

### ¿Cómo limpio la caché de pip / conda / npm?
Aparecen en el grupo de Python / cachés de desarrollo como «Seguro»: se regeneran en la siguiente instalación. Los entornos de conda se listan con su nombre y la fecha de último uso, así que también puedes eliminar los que ya no usas.

### ¿Cómo reduzco un `ext4.vhdx` de WSL o Docker?
Los discos virtuales nunca se encogen solos. Selecciona el vhdx y pulsa «Compactar»: la aplicación ejecuta `wsl --shutdown` y luego `compact vdisk` de diskpart (requiere permisos de administrador; funciona en Windows Home). Los datos de dentro no se tocan.

### ¿Solo limpia C:? ¿Puede analizar D: y E:?
Analiza cualquier unidad. Marca las unidades en «Alcance del análisis» (en Ajustes y en la parte superior de los resultados). Encuentra bibliotecas de modelos, cachés, discos virtuales, resultados de compilación y archivos grandes en otras unidades. Si C: va justo, también puedes mover los modelos grandes a otra unidad.

### Borré algo por error. ¿Puedo recuperarlo?
Sí. Todo va a la Papelera de reciclaje. Usa «Deshacer la última limpieza» o restaura elementos sueltos desde el historial.

### ¿Puede estropear Windows?
El análisis es de solo lectura. Solo se limpian las carpetas incluidas en las reglas, cada una con sus consecuencias explicadas. Por defecto solo se marca «Seguro», el registro nunca se toca y se avisa de las herramientas en ejecución.

### ¿Es una alternativa a CCleaner?
Cubre lo que CCleaner no: modelos de IA, entornos de Python, cachés de editores con IA. También limpia la basura habitual de Windows. Usar los dos no es problema.

### ¿Es gratis? ¿Tiene anuncios o telemetría?
Gratis, con licencia MIT, sin anuncios ni software incluido. El análisis y la limpieza se hacen en local; las actualizaciones de la aplicación y de las reglas acceden a GitHub. La comprobación de versiones MCP en la página «Herramientas de IA» está activada de forma predeterminada y se puede desactivar en Ajustes. Solo envía nombres de paquetes a npm / PyPI, nunca el contenido de la configuración.

### ¿macOS / Linux?
Por ahora solo Windows 10 / 11. El motor de análisis está escrito en Rust multiplataforma, así que podrían llegar otras plataformas.

## Contribuir

¿Falta una herramienta de IA que usas? [Abre una issue](https://github.com/nethero23/ai-disk-cleaner/issues) o añade una regla (unas pocas líneas de TOML). Compilación y formato de reglas: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

Si te ha ahorrado unos gigas, una ⭐ ayuda a que otras personas con el disco lleno lo encuentren.

## Licencia

[MIT](LICENSE)
