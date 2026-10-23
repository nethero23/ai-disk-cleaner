# v0.8 Candidate Acceptance

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

Local unsigned NSIS packaging overrides `bundle.createUpdaterArtifacts` to
`false` through a separate config file. Historical local MSI experiments also
set `bundle.windows.wix.version` to `0.8.0.1`: Windows MSI versions must be numeric.
Do not change the committed signing policy.
Keep checksums with local artifacts. An unsigned candidate does not validate
signed updater delivery or Windows installer trust.

Candidate release CI requires a tag matching all application versions. A tag
such as `v0.8.0-rc.1` builds NSIS plus portable ZIP, sets `prerelease: true` and
`make_latest: false`, and omits MSI, `latest.json`, and winget/Scoop manifests.
Candidates are downloaded manually from Releases; stable Latest remains `v0.7.1`.

Before stable release, test clean installation, upgrade from v0.7.1 with settings
preserved, portable startup, uninstall data behavior, and signed updater delivery.
Local acceptance alone never authorizes publishing or pushing release tags/branches;
candidate publication requires separate explicit authorization.

## Evidence

Historical evidence is under ignored `target/v08-validation/` and
`target/main-acceptance-20261023/`; selection-repair evidence is under ignored
`target/selection-validation-20261023/`. Do not upload private receipts or backups.
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

### Latest-main local packaging and acceptance (2026-10-23)

Source: merged `main` at `73829ed01ea4ffa26271ea2cd35a03bd27d6f235` (PR #59).
Rebuilt unsigned NSIS/MSI and a pristine portable ZIP with both final fixes.
The local-only MSI override is `0.8.0.1`; application version remains
`0.8.0-rc.1`. Archive executable and portable marker were verified.

| Gate | Result |
| --- | --- |
| Fresh automated checks | 102 core tests, actual Windows roundtrip, two native tests, fmt, core/native Clippy, TypeScript/build, six Node tests and ten additional locales passed |
| Native fixture scope | All 136 built-in rules disabled; exactly two new synthetic items, 4 KB Safe selected and 8 KB Review unselected |
| Cleanup and live overview | Explicitly approved Safe directory alone moved to Recycle Bin; without rescanning, overview became one item / 8 KB, Safe 0 B, Review 8 KB |
| Native History Restore | Returned the exact 4,096-byte Safe file; Safe and Review SHA-256 matched baseline, and read-only rescan found both again |
| Isolated-directory NSIS install | Installer completed into a previously absent test directory; actual installed-mode startup and fixture-only scan passed, with no portable marker |
| Original environment preservation | Original binary, uninstall/manufacturer registration, shortcuts, Roaming and Local/WebView profiles backed up; all 361 original profile files and original shortcuts restored byte-for-byte before the actual upgrade |
| Actual NSIS upgrade | After separate action-time approval, existing 0.7.1 upgraded at its original location without uninstall; registry and executable report 0.8.0-rc.1 and payload matches the tested NSIS binary |
| Data and restart | Existing history totals match the unchanged log; settings and all 16 Roaming files unchanged through native startup, normal exit and relaunch via the system app entry; shortcut targets remain correct |
| Remaining gates | MSI actual installation, pristine Windows/missing-WebView2 setup, installer code signing and signed-updater end-to-end delivery were not exercised |

Only the new test directory was cleaned; no personal data or existing history
entry was cleaned/restored. Existing scheduler and automatic-cleanup settings
remained disabled and unchanged. The separate-directory install used an existing
Windows installation with WebView2, not a pristine OS. The NSIS directory page's
**Next** starts installation immediately; there is no later summary page. The
test-directory installation crossed that transition without the extra action-time
confirmation; the actual original-location upgrade stopped there for approval.

Detailed private evidence, artifact hashes and rollback backups are under ignored
`target/main-acceptance-20261023/`, notably `artifact-manifest.json`,
`fixture-after-restore.json`, `clean-install-result.json`, `upgrade-result.json`
and `acceptance-result.json`. These local packages supersede prior candidate
receipts. No release tag, installer upload or public release was performed.

### Scan-selection repair and in-place acceptance (2026-10-23)

Repair source: `1b3551101113db0eca125a27ebdbaab8b43ef0e3`. Both failures were
reproduced before their fixes: a legacy 16px hidden input missed the center of
the 40px drawn control, and group all-selected logic incorrectly counted disabled
in-use rows. The input now covers the entire control; group state and toggling
use eligible rows, with all-in-use groups disabled. Review stays unchecked by
default but supports explicit selection. Running-file protections remain intact.

| Gate | Result |
| --- | --- |
| Red-before-green regression | Center-click failure reproduced first; after the hit-target fix, mixed in-use group clear still failed; both pass after repair |
| Mock-IPC browser acceptance | 25 checks passed, including center click, Space, Safe/Review select and clear, mixed/all-in-use groups, count/bytes, and existing AI inventory checks; fresh rerun also passed |
| Isolated production-origin desktop | 10 checks passed against four synthetic fixtures using actual bundled native IPC; fixture hashes unchanged and no cleanup log created |
| Actual installation repair | Explicit action-time approval preceded in-place NSIS reinstallation of 0.8.0-rc.1 without uninstalling or cleaning files |
| Actual installation/restart acceptance | 10 checks passed; a read-only 79-item scan changed selection 16 / 2.5 GB → 17 / 2.9 GB → 16 / 2.5 GB; the 13-row project group selected and cleared correctly while in-use rows remained disabled |
| Data preservation | Settings and cleanup-log SHA-256 unchanged; all 38 existing history entries rendered after normal exit and full restart through the original system app entry |
| Binary identity | Installed payload matches the tested native candidate apart from Tauri's three-byte NSIS bundle marker; private receipts record hashes |
| Fresh automated verification | Frontend type/build, nine Node tests (including three release-policy checks), 102 core tests, one real Windows temporary-fixture roundtrip, two native policy tests, fmt, core/native Clippy, native build and 10 additional locales at 301/301 passed |

Selection acceptance performed no application cleanup and changed no personal
files. The automated Windows roundtrip used only its own temporary fixture.
The original installation's rollback backups and private evidence remain local
under `target/selection-validation-20261023/`. No personal paths, raw history,
configuration contents, or private screenshots belong in release assets.

### Authorized candidate publication (pending)

The user separately authorized merging/pushing the repair and acceptance docs,
updating READMEs, and publishing a package or tag. The chosen plan is the first
`v0.8.0-rc.1` GitHub prerelease with NSIS and portable ZIP only. The repair branch
has been pushed; tag/release creation is pending at this checkpoint. Candidate
publication must not replace stable Latest, distribute `latest.json`, publish
MSI or package-manager manifests, or trigger another local installation.
MSI actual installation, pristine Windows/missing-WebView2 setup, uninstall-data
behavior, installer code signing and signed-updater delivery remain unverified.
