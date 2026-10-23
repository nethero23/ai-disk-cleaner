# Repository Guidelines

## Project Structure & Module Organization

Clearwisp is a Windows disk-cleaning app built with React 18, TypeScript, Vite, Tauri 2, and Rust.

- `src/`: frontend; `pages/` contains screens, `ui/` shared components, and `locales/` translations.
- `src-tauri/`: native commands, desktop configuration, and `icons/` assets.
- `crates/cleaner-core/`: reusable scanning/cleanup engine, CLI, and `rules/*.toml` definitions.
- `tools/`: maintenance/checking scripts. See `docs/DEVELOPMENT.md` for development details and `.github/workflows/ci.yml` for CI checks.
- `dist/`, `target/`, and `node_modules/` are generated; do not commit them.

## Build, Test, and Development Commands

Run from the repository root with Node 22 and stable Rust. Desktop development requires Windows and WebView2.

```sh
npm ci                             # Install locked frontend dependencies
npm run tauri dev                  # Desktop app with hot reload
npm run dev                        # Frontend only; no native IPC
npm run build                      # Type-check and bundle frontend
cargo build -p ai-disk-cleaner      # Compile desktop app after frontend build
cargo fmt --all --check             # Check Rust formatting
cargo clippy -p cleaner-core --all-targets -- -D warnings # Reject Rust warnings
cargo test -p cleaner-core          # Run engine tests
```

## Coding Style & Naming Conventions

Use rustfmt defaults (four-space indentation), `snake_case` Rust functions/modules, and `PascalCase` types. TypeScript uses two spaces, double quotes, semicolons, `PascalCase` components, and `camelCase` functions/variables.

Keep filesystem logic in the engine. Synchronize Rust IPC models with `src/types.ts`. Use stable dotted rule IDs such as `pip.cache`; register new rule files in `crates/cleaner-core/src/rules.rs`.

## Testing Guidelines

Rust tests live beside implementations in `#[cfg(test)]` modules with descriptive `snake_case` names. Use `tempfile` fixtures, keep tests offline, and cover safety guards and rule changes. CI enforces no numeric coverage threshold.

There is no configured `npm test` or frontend lint script; run `npm run build` and `node --test tools/*.test.mjs`. For translations, also run `node tools/i18n_check.mjs`. Test native startup policy with `cargo test -p ai-disk-cleaner --lib`. Validate desktop behavior manually for native/UI changes.

## Commit & Pull Request Guidelines

History mixes English/Chinese summaries, version prefixes, and scoped subjects such as `feat(rules): ...`. Keep commits focused. PRs should explain rationale, link relevant issues, list verification results, and include screenshots for UI changes. Update Chinese/English READMEs before synchronizing translations.

## Safety & Configuration

Keep scans read-only. Preserve Recycle Bin cleanup, protected-path/symlink checks, and explicit selection. Default/automatic cleanup requires safe, not-in-use, auto-clean-eligible items; preserve separate relocation/deduplication confirmations. Old-version candidates require review. Never commit credentials, signing keys, or personal configuration.
