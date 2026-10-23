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
| Native cleanup / history restore | After explicit approval, only the 4,121-byte Safe fixture was moved to the Recycle Bin; its original path disappeared, then History Restore returned identical SHA-256 bytes; Review control unchanged throughout |
| Post-restore read-only scan | Both fixture items returned, with Safe alone selected |
| Post-clean overview | Open issue: item list removed the cleaned fixture, but overview retained 12.1 KB total / 4.0 KB Safe until rescan; do not treat overview as current remaining space |
| Frontend regression recheck | Build/type-check, two Node tests and 10 additional locales passed again |
| Rust recheck | Existing compiled tests passed again: 97 core, one Windows roundtrip, two native startup-policy tests; fresh offline Cargo resolution failed because the dependency cache lacks `chrono` |
| Installation / upgrade / signed updater | Still not executed |

Do not equate the existing test-binary rerun with fresh Rust compilation. The
October 22 build/Clippy evidence remains historical; no Rust source changed in
this desktop follow-up. No personal file was cleaned.

Native cleanup batch: `1792750088015`. Exact synthetic target:
`target/v08-validation/native-20261023/fixture-safe`. The isolated app was
normally exited after restore and rescan. Detailed before/after file hashes are
in `native-20261023/after-native-clean.json` and `after-native-restore.json`
under the ignored validation evidence directory. No application source changed
during this cleanup/restore check; the overview issue is recorded, not fixed.

### October 23 overview/performance fixes and rebuild

- `dc02466`: rebuild total and all safety amounts together with remaining items.
  Remove only backend-confirmed successes; keep failed/unreported items visible.
  Regression first reproduced three stale-total failures, then all six Node tests passed.
- `d529643`: up to four independent tool readers, retaining catalog order before
  analysis. No persistent cache, skipped hashes, new dependency, or IPC change.
  Overlap regression failed under serial code and passed after optimization.
- Fresh checks: 98 core tests, one actual Windows roundtrip, two native startup
  tests, Rust fmt, core/native Clippy, frontend type/build and locale parity pass.
  Locked dependencies were re-fetched, resolving the earlier empty-cache blocker.
- Full-engine timing: 1.503s before, 0.863s first optimized run (about 43% less
  elapsed time in one same-host comparison); subsequent runs 0.988/0.726/0.712s.
  All four optimized tools/analysis outputs matched baseline bytes exactly.
  This is not an OS cold-cache benchmark; the earlier two-minute UI latency was
  not reproduced. Native first-page entry showed 12 tools / 106 MCP entries /
  371 skills by the 6.2-second observation, which is an upper bound, not exact load time.
- New production-origin native build scanned only the same two test fixtures,
  Safe alone selected. Final UI cleanup/overview check awaits action-time approval.
- Unsigned NSIS/MSI were rebuilt. Updated portable:
  `target/release/bundle/Clearwisp_0.8.0-rc.1_x64-portable-recheck.zip`.
  Current checksums: `target/v08-validation/recheck-artifact-manifest.json`;
  old receipts remain historical and must not be used for the rebuilt installers.

Installation, upgrade, signed updater, remote push and publication were not performed.

### Source-integration privacy review

- Independent review found that ordinary settings loading fell back to enabled
  MCP checks when a concurrent settings write was temporarily unreadable.
  Network authorization now uses strict reads and denies new requests on any
  read/parse error, both at batch entry and per request. Already-sent requests
  are not canceled; valid older settings retain their documented defaults.
- The regression failed with the old fallback. Two new offline tests cover
  missing/unreadable/invalid settings and a partial settings write mid-batch.
  Fresh verification: 100 core tests, one Windows roundtrip, two native policy
  tests, six Node tests, locale parity, fmt, core/native Clippy, frontend build,
  and native compilation passed. Independent re-review approved the fix.
- README translations and the candidate CHANGELOG are synchronized for source
  integration. No public installer release or tag is implied. The prior local
  bundles predate this privacy fix and must be rebuilt before release.
- The final native overview-after-clean recheck, installation/upgrade and signed
  updater gates remain open; source integration does not certify these gates.

### GitHub Windows short-path regression

The first PR run passed Linux core and the Windows app build. Windows core unit
tests passed, but the real roundtrip failed because its temporary parent used an
8.3 short name while the Recycle Bin stored canonical paths. Pointing only a local
test process's TEMP/TMP at a new short-name fixture reproduced the same failure.

Restore selection, status and dedup now use equivalent existing-ancestor spelling
without requiring the deleted tail to exist. Reparse/symlink ancestors and
unexpected metadata errors retain literal matching, so a redirected ancestor
does not match another original path. Cutoff, newest-entry and collision guards
remain intact. Added canonical/absent-parent and junction/symlink negative tests.
All 102 core tests plus the actual roundtrip passed under the 8.3 TEMP alias;
independent re-review approved the narrowed fix. No test was skipped or weakened.
