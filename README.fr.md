<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Icône Clearwisp">

# Clearwisp : le nettoyeur de disque qui comprend les outils d’IA

**Nettoyeur de disque Windows gratuit et open source pour les développeurs IA, pour C:, D: et tous les autres lecteurs. Il trouve et libère l’espace occupé par les modèles Ollama / Hugging Face / LM Studio, les caches de Cursor / Claude Code / VS Code, les caches pip / conda / npm et les disques virtuels WSL / Docker. D’abord l’analyse, ensuite c’est vous qui choisissez, et chaque suppression peut être annulée.**

[![Dernière version](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=download&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![Téléchargements](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ Télécharger](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [Fonctionnalités](#features) · [Comparaison](#comparison) · [FAQ](#faq)

</div>

> Cette traduction peut être en retard sur la [version anglaise](README.en.md). `0.8.0-rc.2` est une préversion candidate publiée avec 12 langues d’interface, pas une version stable. Téléchargez manuellement l’installateur NSIS ou le ZIP portable dans la [liste des versions](https://github.com/nethero23/ai-disk-cleaner/releases). Latest reste sur la version stable `v0.7.1` ; les candidates ne sont pas distribuées par mise à jour automatique et ne fournissent ni MSI, ni `latest.json`, ni manifestes winget/Scoop.
>
> L’installateur GitHub publié `rc.1` a réussi la réinstallation sur place, le redémarrage complet et 10 vérifications de sélection. Réglages, historique et raccourcis ont conservé les mêmes empreintes ; aucun nettoyage. La protection des éléments utilisés reste active.
>
> `rc.2` ajoute l’état de chargement, conserve les lignes et permet de réessayer après un échec IPC, ignore les réponses anciennes et garde les longs chemins/erreurs lisibles. La restauration et le retour à l’emplacement initial sont désactivés pendant le chargement ou une erreur. 53 vérifications navigateur (28 historique + 25 existantes) et un test bureau isolé ont réussi sans remplacer l’application installée. Tous les échecs de lecture disque ne sont pas couverts. Étapes restantes : [liste de validation](docs/RELEASE_CHECKLIST.md).

## Pourquoi

Sur un PC utilisé pour l’IA, ce ne sont pas les fichiers inutiles classiques qui remplissent le disque. Ce sont **les poids des LLM locaux, les environnements Python, les caches des éditeurs de code IA et les disques virtuels WSL / Docker**, souvent des centaines de Go. CCleaner et les outils similaires n’y regardent pas ; WizTree montre quel dossier est gros, mais pas ce qu’il contient ni s’il peut être supprimé.

Et ce n’est pas seulement C:. Les bibliothèques de modèles, les environnements conda, les disques Docker et les `node_modules` des projets finissent aussi sur D: et E:. Clearwisp analyse tous les lecteurs que vous sélectionnez.

Clearwisp explique chaque gigaoctet : **quel modèle, quel environnement conda, quel cache d’outil, quand il a été utilisé pour la dernière fois et ce qui se passe si vous le supprimez**. Chaque élément a un niveau de sécurité, rien n’est supprimé avant que vous le cochiez, les suppressions vont dans la Corbeille, et le dernier nettoyage s’annule en un clic.

<a id="features"></a>
## Fonctionnalités

### 🧠 Connaît les déchets des outils d’IA (136 règles, plus de 90 outils)

| Catégorie | Couverture |
|---|---|
| LLM locaux | Ollama, Hugging Face (hub / datasets), LM Studio, ModelScope, GPT4All, Jan, Msty, llama.cpp, vLLM, Xinference, Open WebUI, AnythingLLM, Cherry Studio, Chatbox |
| Image / voix | ComfyUI, Stable Diffusion WebUI, Stability Matrix, Pinokio, Whisper, OpenAI CLIP |
| Outils de code IA | Cursor, Windsurf, Claude Code, Codex, Gemini CLI, Qwen Code, GitHub Copilot, Cline, Roo Code, Kilo Code, Continue, Aider, Trae, Kiro, Zed, OpenCode, Antigravity, Goose |
| Python / ML | pip, uv, conda, Poetry, PyTorch, Keras, Triton, PaddlePaddle, Jupyter, Kaggle, Gradio |
| Développement général | npm, pnpm, Yarn, Bun, Cargo, Go, Maven, Gradle, NuGet, Composer, Playwright, Puppeteer, Electron, JetBrains, VS Code |
| Conteneurs | Docker Desktop, WSL (compactage de ext4.vhdx) |
| Windows | cache de Windows Update, Windows.old, fichiers temporaires, caches des pilotes NVIDIA / AMD / Intel, caches d’Edge / Chrome / Firefox, Office, Teams, OneDrive |

Il lit les manifestes Ollama, la structure du cache Hugging Face, les environnements conda, les dossiers de modèles LM Studio et les espaces de travail Cursor : vous voyez **les vrais noms des modèles et des environnements**, pas seulement la taille des dossiers.

### 🛡️ La sécurité d’abord

- **Analyse en lecture seule** : l’analyse n’écrit aucun fichier.
- **Trois niveaux de sécurité** : 🟢 Sûr (cache pur, reconstruit automatiquement) / 🟡 Retéléchargeable / 🔴 À vérifier (données utilisateur ou peut-être en cours d’utilisation). Seul « Sûr » est coché par défaut.
- **Protège l’historique des conversations** : le `state.vscdb` de Cursor et le `~/.claude/projects` de Claude Code contiennent vos conversations. Ils sont marqués « À vérifier » et peuvent être exportés avant le nettoyage.
- **Détection des processus en cours** : vous avertit quand un outil tourne, au lieu de lui retirer ses fichiers.
- **Annulation** : tout va dans la Corbeille avec un journal. Annulez le dernier nettoyage en un clic, ou restaurez des éléments un par un depuis l’historique.
- **Aperçu** : regardez le contenu d’un élément avant de le supprimer.

### 💾 Libérer de l’espace sans supprimer

- **Déplacer vers un autre lecteur** : déplacez les modèles Ollama, le cache Hugging Face, les caches pip / uv / conda et d’autres vers D:. L’application copie, vérifie, puis définit `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR`, etc., ou laisse une jonction de répertoire : chaque outil continue de fonctionner. Retour possible à tout moment.
- **Dédoublonnage des modèles entre outils** : les mêmes poids présents dans le cache HF, LM Studio et ComfyUI sont fusionnés par liens physiques. Chaque outil fonctionne toujours, l’espace n’est utilisé qu’une fois.
- **Compactage des disques virtuels WSL / Docker** : `ext4.vhdx` ne fait que grossir. Ramenez-le à sa taille réelle.

### ⚡ Pratique

- **Tous les lecteurs** : analysez C:, D:, les disques externes, ceux que vous cochez ; retrouve les modèles et caches déplacés vers d’autres lecteurs grâce à la structure de leurs dossiers
- Vue « Où est passé mon espace ? » pour chaque lecteur
- Suivi de la croissance : voyez quels dossiers grossissent, avec une notification quand la hausse hebdomadaire dépasse un seuil
- Modèles inutilisés : repère les modèles que vous n’avez pas utilisés depuis longtemps
- Nettoyage système approfondi : magasin de composants WinSxS, points de restauration, fichier de mise en veille prolongée, cache de l’optimisation de la distribution, cache de Windows Update
- Gros dossiers non reconnus : les gros dossiers qu’aucune règle ne connaît encore sont aussi listés, avec une suggestion de règle en un clic
- Recherche des gros fichiers, sorties de compilation (`node_modules`, `target`, `.venv`…)
- Regroupement par outil, recherche, tri ; analyse parallèle avec progression et annulation
- Icône dans la zone de notification, analyses planifiées en arrière-plan, notification au-delà d’un seuil, nettoyage automatique facultatif des éléments sûrs (désactivé par défaut)
- Guide au premier lancement et résumé en une phrase après chaque analyse
- Mise à jour automatique intégrée, mise à jour des règles en ligne, règles personnelles en TOML

### 🚫 Jamais

Pas de nettoyage du registre, pas de fenêtres intempestives, pas de logiciels groupés, pas d’envoi de votre liste de fichiers. L’analyse et le nettoyage se font en local ; les mises à jour de l’application et des règles utilisent GitHub. La vérification des versions MCP sur la page « Outils d’IA » est activée par défaut et peut être désactivée dans les paramètres. Elle envoie uniquement les noms des paquets à npm / PyPI, jamais le contenu des configurations.

## Installation

1. Ouvrez la [dernière version](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
2. Téléchargez `*_x64-setup.exe` (recommandé, prend en charge les mises à jour intégrées) ou le `.msi`
3. Lancez-le et cliquez sur **Analyser**

**Version portable** : téléchargez `*_x64-portable.zip`, décompressez et lancez `AI Disk Cleaner.exe`. Les réglages et journaux se trouvent dans le dossier `data` à côté. La version portable ne se met pas à jour automatiquement.

Configuration requise : Windows 10 / 11 x64 avec WebView2 (inclus dans Windows 11 ; l’installateur s’en charge sous Windows 10).

> L’installateur n’est pas encore signé, SmartScreen peut donc afficher « Windows a protégé votre ordinateur ». Cliquez sur « Informations complémentaires → Exécuter quand même ». Tout le code est ouvert.

**Bientôt** : `winget install nethero23.AIDiskCleaner` et `scoop install aidc`.

<a id="comparison"></a>
## Comparaison

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| Modèles Ollama / HF / LM Studio | ✅ par nom de modèle | ❌ | ❌ | ❌ | ❌ taille seulement |
| Caches Cursor / Claude Code / VS Code | ✅ | ❌ | ✅ (depuis 6.0) | ❌ | ❌ |
| Caches pip / conda / npm / uv | ✅ | ❌ | en partie | ❌ | ❌ |
| Distingue cache et historique de conversation | ✅ « À vérifier » + export | ❌ | ❌ | ❌ | ❌ |
| Déplacement vers un autre lecteur (variables d’environnement) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Dédoublonnage des modèles entre outils | ✅ liens physiques | ❌ | ❌ | ❌ | ❌ |
| Compactage vhdx WSL / Docker | ✅ | ❌ | ❌ | ❌ | ❌ |
| Suppression annulable | ✅ Corbeille + annulation | ❌ | ❌ | ❌ | ❌ |
| Fichiers inutiles de Windows | ✅ éléments courants | ✅ très complet | ✅ | ✅ très complet | ❌ |
| Nettoyage du registre | ❌ volontairement | ✅ | ✅ | ✅ | ❌ |
| Gratuit et open source | ✅ MIT | ❌ version gratuite limitée | ✅ GPL | gratuit, plus maintenu | gratuit pour un usage personnel |

En bref : pour les fichiers inutiles classiques, n’importe lequel fera l’affaire. Pour les déchets des outils d’IA et de développement, c’est l’outil conçu pour ça. Les deux s’utilisent très bien côte à côte.

<a id="faq"></a>
## FAQ

### Mon lecteur C: (ou D:) est plein. Comment voir ce qui occupe l’espace ?
Ouvrez « Où est passé mon espace ? », choisissez un lecteur et obtenez la répartition par dossier (`Users\<vous>\AppData\Local`, `Downloads`, etc. sont listés à part). Lancez ensuite une analyse pour voir l’occupation par outil et par modèle.

### Où Ollama stocke-t-il ses modèles, et comment les déplacer vers un autre lecteur ?
Par défaut dans `C:\Users\<vous>\.ollama\models`. Dans les résultats, choisissez l’élément Ollama, puis « Déplacer vers un autre lecteur » et un dossier sur D:. L’application copie, vérifie et définit `OLLAMA_MODELS`. Redémarrez Ollama et il lira depuis le nouvel emplacement.

### Puis-je supprimer le cache Hugging Face (`.cache\huggingface`) ?
Oui. Il est marqué « Retéléchargeable » : les modèles seront retéléchargés au besoin. Pour éviter cela, déplacez-le (l’application définit `HF_HUB_CACHE`) ou fusionnez les doublons avec LM Studio / ComfyUI grâce au dédoublonnage.

### Cursor occupe des dizaines de Go. Puis-je supprimer `state.vscdb` ?
Les caches, journaux et le cache de téléchargement des extensions de Cursor peuvent être supprimés sans risque. Mais `state.vscdb` contient **tout votre historique de conversations IA** (des cas à 77 Go ont été signalés). Il est marqué « À vérifier », n’est pas coché par défaut et peut être exporté d’abord.

### Puis-je supprimer le dossier `.claude` de Claude Code ?
`~/.claude/projects` contient les transcriptions de sessions (jsonl) et est marqué « À vérifier » ; exportez-les d’abord si vous voulez les garder. Les fichiers de type cache peuvent être nettoyés sans risque.

### Comment vider le cache pip / conda / npm ?
Ils apparaissent dans le groupe Python / caches de développement comme « Sûr » : ils sont reconstruits à la prochaine installation. Les environnements conda sont listés par nom avec leur date de dernière utilisation, vous pouvez donc aussi supprimer ceux qui ne servent plus.

### Comment réduire un `ext4.vhdx` de WSL ou Docker ?
Les disques virtuels ne rétrécissent jamais seuls. Sélectionnez le vhdx et cliquez sur « Compacter » : l’application exécute `wsl --shutdown` puis `compact vdisk` de diskpart (droits administrateur requis ; fonctionne sous Windows Famille). Les données à l’intérieur ne sont pas touchées.

### Ne nettoie-t-il que C: ? Peut-il analyser D: et E: ?
Il analyse n’importe quel lecteur. Cochez les lecteurs dans « Étendue de l’analyse » (dans les réglages et en haut des résultats). Bibliothèques de modèles, caches, disques virtuels, sorties de compilation et gros fichiers sur les autres lecteurs sont tous trouvés. Quand C: manque de place, vous pouvez aussi déplacer les gros modèles vers un autre lecteur.

### J’ai supprimé quelque chose par erreur. Puis-je le récupérer ?
Oui. Tout va dans la Corbeille. Utilisez « Annuler le dernier nettoyage » ou restaurez des éléments un par un depuis l’historique.

### Peut-il casser Windows ?
L’analyse est en lecture seule. Seuls les dossiers listés dans les règles sont nettoyés, chacun avec ses conséquences décrites. Seul « Sûr » est coché par défaut, le registre n’est jamais touché et les outils en cours d’exécution sont signalés.

### Est-ce une alternative à CCleaner ?
Il couvre ce que CCleaner ignore : modèles d’IA, environnements Python, caches des éditeurs IA. Il nettoie aussi les fichiers inutiles courants de Windows. Utiliser les deux ne pose aucun problème.

### Est-ce gratuit ? Publicité ou télémétrie ?
Gratuit, sous licence MIT, sans publicité ni logiciel groupé. L’analyse et le nettoyage se font en local ; les mises à jour de l’application et des règles utilisent GitHub. La vérification des versions MCP sur la page « Outils d’IA » est activée par défaut et peut être désactivée dans les paramètres. Elle envoie uniquement les noms des paquets à npm / PyPI, jamais le contenu des configurations.

### macOS / Linux ?
Uniquement Windows 10 / 11 pour l’instant. Le moteur d’analyse est en Rust multiplateforme, d’autres plateformes pourraient suivre.

## Contribuer

Il manque un outil d’IA que vous utilisez ? [Ouvrez une issue](https://github.com/nethero23/ai-disk-cleaner/issues) ou ajoutez une règle (quelques lignes de TOML). Compilation et format des règles : [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

S’il vous a fait gagner quelques gigaoctets, une ⭐ aide d’autres personnes au disque plein à le trouver.

## Licence

[MIT](LICENSE)
