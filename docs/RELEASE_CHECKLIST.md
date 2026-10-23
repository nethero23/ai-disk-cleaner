# v0.8 Local Candidate Acceptance

## Scope

Version: `0.8.0-rc.1`. AI tool management is read-only inventory and diagnosis,
not automatic configuration editing, consolidation, or synchronization.
Keep package/executable identifiers unchanged for compatibility.

## Automated gates

Run from the repository root:

```sh
cargo fmt --all --check
cargo clippy -p cleaner-core --all-targets -- -D warnings
cargo test -p cleaner-core
npm run build
node tools/i18n_check.mjs
node --test tools/*.test.mjs
cargo build -p ai-disk-cleaner
cargo test -p ai-disk-cleaner --lib
```

Node regression tests cover numeric version display and review-only stale-version
messaging in every UI language. Registry tests use injected responses, not live requests.

## Windows native acceptance

- Use a separate portable directory; never replace the installed app's data.
- Disable every built-in rule and supply custom rules targeting only newly
  created safe/review acceptance fixtures. Disable scheduled cleanup and update/version requests.
- Verify scan results contain only fixtures; scan does not modify fixture content.
- After explicit confirmation, verify Recycle Bin cleanup and undo restore identical bytes.
- Verify settings, theme, and language persist after a complete restart.
- A portable launch must not overwrite the installed app's autostart registration;
  registration changes still require an explicit autostart-setting change.
- Verify AI inventory never changes tool configuration files or exposes raw keys.
- Check offline/version-query errors and late replies without losing inventory data.

## Packaging and release boundaries

Local unsigned packaging overrides `bundle.createUpdaterArtifacts` to `false`
through a separate config file. For `0.8.0-rc.1`, also set
`bundle.windows.wix.version` to `0.8.0.1` in that local override: Windows MSI
versions must be numeric. Do not change the committed signing policy.
Keep checksums with local artifacts. An unsigned candidate does not validate
signed updater delivery or Windows installer trust.

Before public release, test clean installation, upgrade from v0.7.1 with settings
preserved, portable startup, uninstall data behavior, and signed updater delivery.
Do not publish or push release tags/branches as part of local acceptance.

## Evidence

Validation evidence is kept under ignored `target/v08-validation/`.
Run the repeatable mock-IPC browser flow with Vite running:

```sh
node tools/ui-smoke-acceptance.mjs <path-to-gstack-browse>
```

Native tests requiring an interactive Windows desktop are not complete until
their actual observations are recorded; frontend mock-IPC tests are not substitutes.

### 2026-10-22 local result

| Gate | Result |
| --- | --- |
| Rust formatting / Clippy | Pass |
| Core unit tests | 97 passed |
| Windows scan → Recycle Bin → restore integration | 1 passed; restored bytes identical |
| Frontend type-check and production build | Pass; non-blocking bundle-size warning |
| Node regression tests | 2 passed |
| Additional locale completeness | 10 languages, each 301/301 |
| Mock-IPC browser acceptance | 10 checks passed; includes offline, disabled checks, late replies, RTL/theme persistence |
| Windows desktop compilation | Pass |
| Native startup-policy tests | 2 passed; portable startup never invokes registration changes |
| Local unsigned NSIS / MSI packaging | Pass; MSI ProductVersion verified as 0.8.0.1 |
| Interactive Windows UI | Incomplete: black screenshot and window activation failure; awaiting an unlocked interactive desktop |
| Installation / upgrade / signed updater | Not executed |

The isolated native fixture was verified unchanged. No real tool configuration
was modified and no personal file was cleaned. These results are a candidate
readiness record, not approval to publish.

### 2026-10-23 native desktop follow-up

The actual portable ZIP was hash-verified and extracted into a separate validation
directory. A byte-identical, uniquely named executable avoids confusing it with
the installed app. The native window used bundled `http://tauri.localhost/`, not
the mock-IPC browser page. All 136 built-in rules were disabled.

| Gate | Observation |
| --- | --- |
| Native scan | Exactly two synthetic items, 12.1 KB; only the 4.0 KB Safe item selected, 8.0 KB Review item not selected |
| Scan read-only | Both fixture content hashes unchanged |
| Settings persistence | English and light theme survived complete exit/relaunch; external checks and scheduled cleanup stayed disabled |
| Real AI inventory | Completed: 12 tools, 106 MCP entries, 371 skills; first load took roughly two minutes, requiring performance follow-up |
| Configuration preservation | Four sampled tool configuration files had identical SHA-256 before/after inventory; this is not an exhaustive file audit |
| Portable startup | Matching installed-app autostart entries unchanged after launch/relaunch |
| Native cleanup confirmation | Correctly offers one 4.0 KB item to the Recycle Bin; final action awaits explicit test-fixture permission |
| Frontend regression recheck | Build/type-check, two Node tests and 10 additional locales passed again |
| Rust recheck | Existing compiled tests passed again: 97 core, one Windows roundtrip, two native startup-policy tests; fresh offline Cargo resolution failed because the dependency cache lacks `chrono` |
| Installation / upgrade / signed updater | Still not executed |

Do not equate the existing test-binary rerun with fresh Rust compilation. The
October 22 build/Clippy evidence remains historical; no Rust source changed in
this desktop follow-up. No personal file was cleaned.
