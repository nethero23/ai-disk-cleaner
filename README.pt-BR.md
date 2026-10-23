<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Ícone do Clearwisp">

# Clearwisp: o limpador de disco que entende as ferramentas de IA

**Limpador de disco gratuito e de código aberto para Windows, feito para desenvolvedores de IA, para C:, D: e qualquer outra unidade. Encontra e libera o espaço ocupado por modelos do Ollama / Hugging Face / LM Studio, caches do Cursor / Claude Code / VS Code, caches do pip / conda / npm e discos virtuais do WSL / Docker. Primeiro ele analisa, depois você escolhe, e toda exclusão pode ser desfeita.**

[![Última versão](https://img.shields.io/github/v/release/nethero23/ai-disk-cleaner?label=download&color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/nethero23/ai-disk-cleaner/total?color=0b7a75)](https://github.com/nethero23/ai-disk-cleaner/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Русский](README.ru.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

[⬇️ Baixar](https://github.com/nethero23/ai-disk-cleaner/releases/latest) · [Recursos](#features) · [Comparação](#comparison) · [Perguntas frequentes](#faq)

</div>

> Esta tradução pode estar atrás da [versão em inglês](README.en.md). [0.9.0-rc.1](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1), com 12 idiomas de interface, foi publicada como candidata de pré-lançamento, não como versão estável. Baixe manualmente o [NSIS](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-setup.exe) ou o [ZIP portátil](https://github.com/nethero23/ai-disk-cleaner/releases/download/v0.9.0-rc.1/AI.Disk.Cleaner_0.9.0-rc.1_x64-portable.zip). Os dois pacotes públicos foram baixados e verificados por SHA-256, sem execução. Latest continua na estável `v0.7.1`; candidatas não oferecem atualização automática, MSI, `latest.json` ou manifestos winget/Scoop.
>
> Logs JSONL de limpeza/movimentação ausentes são vazios; falhas de leitura, codificação ou registros são exibidas com Tentar novamente. Restaurar e mover de volta ficam bloqueados durante o carregamento ou erro. Falha ao listar a Lixeira não é mais tratada como perda definitiva.
>
> Se validar ou abrir o log falhar, nenhum arquivo é limpo. Falhas de escrita/sincronização param os próximos itens, mantêm os sucessos reais e exibem aviso nos resultados em primeiro e segundo plano. Logs antigos válidos sem ID de lote ou quebra de linha final continuam compatíveis. Não é um WAL seguro contra falhas; a escrita transacional das movimentações permanece pendente. Tente novamente, preserve o log original e verifique permissões e espaço livre.
>
> Passaram 67 verificações no navegador (25 seleção + 30 histórico + 12 recuperação), testes automáticos locais e uma compilação NSIS sem assinatura. Uma cópia portátil `0.9.0-rc.1` local e isolada exibiu por IPC real o bloqueio do log (OS error 32) e danos nos dois históricos, preservou as linhas e permitiu tentar novamente após desbloquear ou restaurar os bytes originais. Passaram a análise somente leitura, 10 verificações de seleção/proteção e duas linhas de histórico após reiniciar; duas verificações apenas confirmaram estados desativados, sem clicar. A original `0.8.0-rc.1` foi reaberta, com hashes de configurações, histórico, programa e amostras inalterados. Limpeza/restauração real pela interface, instalação/atualização e execução de pacote publicado na nuvem não foram testadas. Veja a [lista de aceitação](docs/RELEASE_CHECKLIST.md).

## Por quê

Em um PC usado para IA, o que enche o disco raramente é o lixo clássico do sistema. São **pesos de LLMs locais, ambientes Python, caches de editores de código com IA e discos virtuais do WSL / Docker**, muitas vezes centenas de GB. O CCleaner e ferramentas parecidas não olham ali; o WizTree mostra qual pasta é grande, mas não diz o que ela é nem se pode ser apagada.

E não é só o C:. Bibliotecas de modelos, ambientes conda, discos do Docker e `node_modules` de projetos também vão parar no D: e no E:. O Clearwisp analisa todas as unidades que você selecionar.

O Clearwisp explica cada gigabyte: **qual modelo, qual ambiente conda, qual cache de qual ferramenta, quando foi usado pela última vez e o que acontece se você apagar**. Cada item tem um nível de segurança, nada é apagado antes de você marcar, as exclusões vão para a Lixeira e a última limpeza pode ser desfeita com um clique.

<a id="features"></a>
## Recursos

### 🧠 Conhece o lixo das ferramentas de IA (136 regras, mais de 90 ferramentas)

| Categoria | Cobertura |
|---|---|
| LLMs locais | Ollama, Hugging Face (hub / datasets), LM Studio, ModelScope, GPT4All, Jan, Msty, llama.cpp, vLLM, Xinference, Open WebUI, AnythingLLM, Cherry Studio, Chatbox |
| Imagem / voz | ComfyUI, Stable Diffusion WebUI, Stability Matrix, Pinokio, Whisper, OpenAI CLIP |
| Ferramentas de programação com IA | Cursor, Windsurf, Claude Code, Codex, Gemini CLI, Qwen Code, GitHub Copilot, Cline, Roo Code, Kilo Code, Continue, Aider, Trae, Kiro, Zed, OpenCode, Antigravity, Goose |
| Python / ML | pip, uv, conda, Poetry, PyTorch, Keras, Triton, PaddlePaddle, Jupyter, Kaggle, Gradio |
| Desenvolvimento em geral | npm, pnpm, Yarn, Bun, Cargo, Go, Maven, Gradle, NuGet, Composer, Playwright, Puppeteer, Electron, JetBrains, VS Code |
| Contêineres | Docker Desktop, WSL (compactação do ext4.vhdx) |
| Windows | cache do Windows Update, Windows.old, arquivos temporários, caches de drivers NVIDIA / AMD / Intel, caches do Edge / Chrome / Firefox, Office, Teams, OneDrive |

Ele lê os manifestos do Ollama, a estrutura do cache do Hugging Face, os ambientes conda, as pastas de modelos do LM Studio e os workspaces do Cursor, então você vê **os nomes reais dos modelos e ambientes**, não só o tamanho das pastas.

### 🛡️ Segurança em primeiro lugar

- **Análise somente leitura**: a análise não grava nenhum arquivo.
- **Três níveis de segurança**: 🟢 Seguro (cache puro, recriado automaticamente) / 🟡 Pode baixar de novo / 🔴 Revisar (dados do usuário ou possivelmente em uso). Por padrão, só "Seguro" vem marcado.
- **Protege o histórico de conversas**: o `state.vscdb` do Cursor e o `~/.claude/projects` do Claude Code guardam suas conversas. Eles ficam marcados como "Revisar" e podem ser exportados antes da limpeza.
- **Detecção de processos em execução**: avisa quando uma ferramenta está aberta em vez de tirar os arquivos que ela está usando.
- **Desfazer**: tudo vai para a Lixeira com um registro da limpeza. Desfaça a última limpeza com um clique ou restaure itens individuais pelo histórico.
- **Prévia**: veja o que há dentro de qualquer item antes de apagar.

### 💾 Libere espaço sem apagar

- **Mover para outra unidade**: leve os modelos do Ollama, o cache do Hugging Face, os caches do pip / uv / conda e mais para o D:. O aplicativo copia, verifica e então define `OLLAMA_MODELS` / `HF_HUB_CACHE` / `PIP_CACHE_DIR` etc. ou deixa uma junção de diretório, e cada ferramenta continua funcionando. Dá para trazer de volta quando quiser.
- **Desduplicação de modelos entre ferramentas**: os mesmos pesos no cache do HF, no LM Studio e no ComfyUI são unidos com links físicos. Todas as ferramentas continuam funcionando e o espaço é usado uma vez só.
- **Compactar discos virtuais do WSL / Docker**: o `ext4.vhdx` só cresce. Reduza-o ao tamanho real.

### ⚡ Prático

- **Todas as unidades**: analise C:, D:, discos externos, as que você marcar; encontra modelos e caches movidos para outras unidades pela estrutura das pastas
- Visão "Para onde foi meu espaço?" de cada unidade
- Monitor de crescimento: mostra quais pastas estão crescendo, com aviso quando o crescimento semanal passa de um limite
- Modelos parados: encontra modelos que você não usa há muito tempo
- Limpeza profunda do sistema: repositório de componentes WinSxS, pontos de restauração, arquivo de hibernação, cache da Otimização de Entrega, cache do Windows Update
- Pastas grandes não reconhecidas: pastas grandes que nenhuma regra conhece ainda também aparecem, com sugestão de regra em um clique
- Busca de arquivos grandes, saídas de build (`node_modules`, `target`, `.venv`…)
- Agrupar por ferramenta, buscar, ordenar; análise em paralelo com progresso e cancelamento
- Ícone na bandeja, análises agendadas em segundo plano, aviso ao passar de um limite, limpeza automática opcional dos itens seguros (desligada por padrão)
- Guia na primeira execução e um resumo de uma frase após cada análise
- Atualização automática no app, regras atualizadas on-line, regras próprias em TOML

### 🚫 Nunca

Sem limpeza do registro, sem janelas irritantes, sem programas embutidos, sem envio da sua lista de arquivos. A análise e a limpeza são feitas localmente; as atualizações do aplicativo e das regras acessam o GitHub. A verificação de versões MCP na página Ferramentas de IA vem ativada por padrão e pode ser desativada nas Configurações. Ela envia apenas os nomes dos pacotes ao npm / PyPI, nunca o conteúdo das configurações.

## Instalação

1. Abra a [versão mais recente](https://github.com/nethero23/ai-disk-cleaner/releases/latest)
2. Baixe o `*_x64-setup.exe` (recomendado, com atualização no app) ou o `.msi`
3. Abra e clique em **Analisar**

**Portátil**: baixe o `*_x64-portable.zip`, descompacte e execute o `AI Disk Cleaner.exe`. Configurações e registros ficam na pasta `data` ao lado. A versão portátil não se atualiza sozinha.

Requisitos: Windows 10 / 11 x64 com WebView2 (já vem no Windows 11; no Windows 10 o instalador cuida disso).

> O instalador ainda não tem assinatura de código, então o SmartScreen pode mostrar "O Windows protegeu o computador". Clique em "Mais informações → Executar assim mesmo". Todo o código é aberto.

**Em breve**: `winget install nethero23.AIDiskCleaner` e `scoop install aidc`.

<a id="comparison"></a>
## Comparação

| | Clearwisp | CCleaner | BleachBit | Dism++ | WizTree |
|---|:-:|:-:|:-:|:-:|:-:|
| Modelos do Ollama / HF / LM Studio | ✅ pelo nome do modelo | ❌ | ❌ | ❌ | ❌ só tamanho |
| Caches do Cursor / Claude Code / VS Code | ✅ | ❌ | ✅ (desde a 6.0) | ❌ | ❌ |
| Caches do pip / conda / npm / uv | ✅ | ❌ | parcial | ❌ | ❌ |
| Separa cache de histórico de conversas | ✅ "Revisar" + exportar | ❌ | ❌ | ❌ | ❌ |
| Mover para outra unidade (define variáveis de ambiente) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Desduplicação de modelos entre ferramentas | ✅ links físicos | ❌ | ❌ | ❌ | ❌ |
| Compactação de vhdx do WSL / Docker | ✅ | ❌ | ❌ | ❌ | ❌ |
| Exclusão que pode ser desfeita | ✅ Lixeira + desfazer | ❌ | ❌ | ❌ | ❌ |
| Lixo do sistema Windows | ✅ itens comuns | ✅ bem completo | ✅ | ✅ bem completo | ❌ |
| Limpeza do registro | ❌ de propósito | ✅ | ✅ | ✅ | ❌ |
| Gratuito e de código aberto | ✅ MIT | ❌ versão grátis limitada | ✅ GPL | grátis, sem manutenção | grátis para uso pessoal |

Resumindo: para o lixo clássico do sistema, qualquer um serve. Para o lixo das ferramentas de IA e de desenvolvimento, esta é a ferramenta feita para isso. Dá para usar os dois juntos.

<a id="faq"></a>
## Perguntas frequentes

### Meu disco C: (ou D:) está cheio. Como vejo o que está ocupando o espaço?
Abra "Para onde foi meu espaço?", escolha uma unidade e veja a divisão por pasta (`Users\<você>\AppData\Local`, `Downloads` etc. aparecem separados). Depois rode uma análise para ver o uso por ferramenta e por modelo.

### Onde o Ollama guarda os modelos e como movo para outra unidade?
Por padrão em `C:\Users\<você>\.ollama\models`. Nos resultados, escolha o item do Ollama, depois "Mover para outra unidade" e uma pasta no D:. O aplicativo copia, verifica e define `OLLAMA_MODELS`. Reinicie o Ollama e ele passa a ler do novo local.

### Posso apagar o cache do Hugging Face (`.cache\huggingface`)?
Pode. Ele fica marcado como "Pode baixar de novo": os modelos são baixados outra vez quando necessário. Se não quiser baixar de novo, mova-o (o aplicativo define `HF_HUB_CACHE`) ou una os duplicados com o LM Studio / ComfyUI usando a desduplicação de modelos.

### O Cursor está usando dezenas de GB. Posso apagar o `state.vscdb`?
Os caches, registros e o cache de downloads de extensões do Cursor podem ser apagados com segurança. Mas o `state.vscdb` guarda **todo o seu histórico de conversas com a IA** (há relatos de 77 GB). Ele fica marcado como "Revisar", não vem marcado por padrão e pode ser exportado antes.

### Posso apagar a pasta `.claude` do Claude Code?
O `~/.claude/projects` guarda as transcrições das sessões (jsonl) e fica marcado como "Revisar"; exporte antes se quiser mantê-las. Arquivos do tipo cache podem ser limpos com segurança.

### Como limpo o cache do pip / conda / npm?
Eles aparecem no grupo Python / caches de desenvolvimento como "Seguro": são recriados na próxima instalação. Os ambientes conda aparecem com nome e data do último uso, então você também pode remover os que não usa mais.

### Como reduzo um `ext4.vhdx` do WSL ou do Docker?
Discos virtuais nunca encolhem sozinhos. Selecione o vhdx e clique em "Compactar": o aplicativo executa `wsl --shutdown` e depois o `compact vdisk` do diskpart (precisa de permissão de administrador; funciona no Windows Home). Os dados dentro dele não são tocados.

### Ele só limpa o C:? Consegue analisar o D: e o E:?
Analisa qualquer unidade. Marque as unidades em "Abrangência da análise" (nas Configurações e no topo dos resultados). Bibliotecas de modelos, caches, discos virtuais, saídas de build e arquivos grandes em outras unidades são todos encontrados. Quando o C: aperta, você também pode mover modelos grandes para outra unidade.

### Apaguei algo por engano. Dá para recuperar?
Dá. Tudo vai para a Lixeira. Use "Desfazer a última limpeza" ou restaure itens individuais pelo histórico.

### Ele pode estragar o Windows?
A análise é somente leitura. Só são limpas as pastas que estão nas regras, cada uma com a consequência descrita. Por padrão só "Seguro" vem marcado, o registro nunca é tocado e as ferramentas em execução são sinalizadas.

### É uma alternativa ao CCleaner?
Ele cobre o que o CCleaner não cobre: modelos de IA, ambientes Python, caches de editores com IA. Também limpa o lixo comum do Windows. Usar os dois juntos não tem problema.

### É grátis? Tem anúncios ou telemetria?
Grátis, licença MIT, sem anúncios e sem programas embutidos. A análise e a limpeza são feitas localmente; as atualizações do aplicativo e das regras acessam o GitHub. A verificação de versões MCP na página Ferramentas de IA vem ativada por padrão e pode ser desativada nas Configurações. Ela envia apenas os nomes dos pacotes ao npm / PyPI, nunca o conteúdo das configurações.

### macOS / Linux?
Por enquanto só Windows 10 / 11. O motor de análise é escrito em Rust multiplataforma, então outras plataformas podem vir depois.

## Contribuir

Falta alguma ferramenta de IA que você usa? [Abra uma issue](https://github.com/nethero23/ai-disk-cleaner/issues) ou adicione uma regra (algumas linhas de TOML). Build e formato das regras: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

Se ele te economizou alguns gigas, uma ⭐ ajuda outras pessoas com o disco cheio a encontrá-lo.

## Licença

[MIT](LICENSE)
