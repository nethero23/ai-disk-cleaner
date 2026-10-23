# Clearwisp Project Context

## Current milestone

`0.9.0-rc.1` is the next recovery-reliability candidate. Publication is authorized;
it is not a stable promotion and does not replace the installed application.
Cleanup and relocation log read/UTF-8/parse errors now reject IPC rather than
masquerading as empty history. Recycle Bin listing errors no longer mark paths
gone. Cleanup checks the audit log before deletion; a write/sync failure stops
subsequent items, exposes `log_error`, and preserves actual successes and skipped
items in foreground/background results. Legacy valid JSONL needs no migration.

Fresh local checks passed: nine Node tests, 111 core unit tests, one Windows
temporary-fixture Recycle Bin/restore roundtrip, two native policy tests, locale
parity, fmt, core/native Clippy, frontend build, native debug/release and unsigned
NSIS packaging. All 67 mock-IPC browser checks passed (25 selection/inventory,
30 history, 12 audit-failure/result checks). Independent in-host production
review found no new P1/P2 issues; external-model tooling is unavailable.

The v0.9 interactive desktop check is currently blocked: capture is black and
activation failed after fresh window selection. No UI input, candidate launch,
installation or personal-file cleanup followed that failure. Isolated fixtures
are prepared; prior rc.2 desktop acceptance is not v0.9 acceptance. Private
evidence stays in ignored `target/v09-release-20261024/`.

This is not a crash-safe write-ahead journal. Unexpected post-deletion append
failure is visible and stops the batch, but cannot guarantee a durable success
record; transactional relocation log writes remain a separate follow-up.

### Published rc.2 baseline

`0.8.0-rc.2` is published as a [GitHub prerelease](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.8.0-rc.2).
[PR #63](https://github.com/nethero23/ai-disk-cleaner/pull/63) merged at
`3cbef24861d50757b9b11ca35ab199665effb63a`; its annotated tag targets that source.
[Tag CI](https://github.com/nethero23/ai-disk-cleaner/actions/runs/39994807644)
completed successfully. Exactly two public assets, NSIS and portable ZIP, were
downloaded and hash-verified. Stable Latest remains `v0.7.1`; no MSI, updater
or package-manager manifests were published. The installed rc.1 was not replaced.
It fixes pending
history appearing empty, rejected-IPC retry/preservation, stale replies and
narrow history layout. Loading/error states disable restore and move-back;
all three new status strings cover 12 UI languages. The two original defects
were reproduced before repair. All 53 browser checks (28 history plus 25
existing) and isolated production-origin desktop checks passed. No installed
application was replaced for this candidate. See the [acceptance checklist](docs/RELEASE_CHECKLIST.md).

### Published rc.1 baseline

v0.8.0-rc.1 selection repair, all 11 READMEs and acceptance documentation merged
to `main` through [PR #61](https://github.com/nethero23/ai-disk-cleaner/pull/61)
at `bbb48441bb2ce67366aebf356c21391209976839`. The annotated `v0.8.0-rc.1`
tag targets that exact source. The separately authorized
[GitHub prerelease](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.8.0-rc.1)
is published after successful tag CI: NSIS and portable ZIP only, no MSI,
`latest.json`, or winget/Scoop manifests. Both public downloads match GitHub's
SHA-256 digests; the portable archive has empty data, a portable marker and
application version `0.8.0-rc.1`. Latest stable remains `v0.7.1`.
Windows installer Authenticode is `NotSigned`. At the October 23 delivery
checkpoint, downloaded cloud binaries were inspected but not executed.
On October 24, the actual GitHub NSIS package was reinstalled in place and
fully restarted. Its installed payload matches the cloud build including
Tauri's NSIS marker; all 10 selection checks passed. Settings, the 95-line
history and both shortcut hashes stayed unchanged; no cleanup ran. Private
receipts and rollback backups stay under ignored `target/github-install-acceptance-20261024/`.

### rc.2 readiness and limits

Fresh checks passed: nine Node tests, 102 core tests, one Windows temporary-fixture
Recycle Bin/restore roundtrip, two native policy tests, locale parity, Rust fmt,
core/native Clippy, frontend type/build, native debug/release and unsigned NSIS
packaging. The isolated bundled `tauri.localhost` app displayed loading and two
synthetic history rows without horizontal overflow. Read-only scanning found
only four new fixtures (201 B); Safe alone was selected, Review could be
selected/cleared manually, and in-use controls stayed disabled. No application
cleanup or restore ran; installed and fixture configuration/history hashes
stayed unchanged. Evidence remains private under ignored `target/next-release-20261024/`.

The error UI covers rejected IPC requests only. Existing backend log-read
errors can still be mapped to empty lists; this candidate does not change that
behavior or the IPC contract. An independent in-host review approved the fix;
unavailable external-model tooling is not additional review coverage.

### Scan-selection follow-up

`1b35511` fixes manual selection and is included in the merged/tagged candidate.
Legacy 16px native-input sizing missed the center of the 40px
drawn checkbox; the input now covers the control. Group selection/clear and its
checked state now consider only eligible rows, with all-in-use groups disabled.
Both original failures were reproduced before their fixes. All 25 browser
interaction checks passed, plus a rebuilt native fixture run covering visible
mouse-center selection/clear, keyboard Space, explicit Review selection and
mixed in-use group toggles. Busy protections remain intact; no cleanup ran and
fixture/configuration/history hashes stayed unchanged. Fresh automated checks
and the unsigned local NSIS build passed. After explicit action-time approval,
installer finalization and repair verification were completed without uninstalling.
The first resumed observation already showed installation complete; initial
write timing was not independently observed. The installed binary
matches the tested candidate apart from Tauri's three-byte NSIS bundle marker.
On an actual 79-item scan, mouse-center and Space toggles changed selection from
16 / 2.5 GB to 17 / 2.9 GB and back. The 13-row project group selected and cleared
correctly; in-use rows remained disabled. A full exit/restart through the
original app entry succeeded, and existing history still rendered with the same
summary of 38 successful cleanups.
Settings and cleanup history remained byte-identical. Selection acceptance
performed no cleanup. Original binaries/profiles are backed up locally; private
receipts remain under ignored `target/selection-validation-20261023/`.
Fresh verification passed: 25 browser, 10 isolated native, 10 actual-install/
restart checks; nine Node tests, 102 core tests, one Windows fixture roundtrip,
two native policy tests, type/build, locale parity, fmt and Clippy. Source
integration and candidate publication do not certify the remaining stable gates.

## Approved v0.8 scope

- Material Design 3 UI, light/dark/system theme, and localized screens.
- Read-only inventory of MCP servers, skills, plugins, and extensions.
- Duplicate/conflict/broken-configuration hints with masked secrets.
- Optional, bounded MCP package-version lookups to npm/PyPI; no configuration contents transmitted.
- Older-version folders are manual-review candidates, never evidence that a version is unused.
- Portable startup does not synchronize the installed app's autostart registration;
  explicit autostart-setting changes retain their existing behavior.

## Release gate

Automated checks pass, including a real Windows Recycle Bin roundtrip using a
temporary fixture. The baseline mock-IPC browser flow passed 10 checks before
the selection regression suite expanded it to 25. Historical local unsigned
NSIS/MSI packaging succeeded with an explicit numeric MSI-version override;
public candidate CI intentionally excludes MSI.

On October 23, the unlocked Windows desktop passed native portable startup,
fixture-only scanning, safe-only default selection, and language/light-theme
persistence after restart. Real AI inventory completed; four sampled tool
configuration hashes and matching autostart entries remained unchanged.

After explicit test-fixture approval, native UI cleanup moved only the 4,121-byte
Safe fixture to the Recycle Bin. History restore returned identical bytes; the
Review control stayed unchanged, and a read-only rescan found both fixtures again.

Follow-up fixes: dc02466 rebuilds remaining overview amounts atomically after
confirmed cleanup successes; d529643 overlaps independent AI tool reads with at
most four workers, preserving catalog order and fresh results without a cache.
MCP network permission now fails closed on unreadable or malformed settings,
both at batch entry and before each request. Six frontend regressions, 102 core
tests, one Windows roundtrip and two native policy tests pass after fresh
compilation. Unsigned NSIS/MSI and a pristine portable ZIP were rebuilt from the
merged main, including both the permission and short-path restore fixes.

The first GitHub Windows roundtrip exposed short-name parent paths not matching
the Recycle Bin's canonical spelling. Restore/status/dedup now normalize existing
ancestors only when no reparse/symlink ancestor is present. The original failure
was reproduced locally with an 8.3 TEMP alias, then the full suite passed there.

Full-engine measurement improved from 1.503s to 0.863s in a same-host comparison;
tools and analysis serialized identically. The earlier two-minute observation
was not reproduced, so this is not a demonstrated cold-cache fix. Native first
page entry showed the complete inventory by the 6.2-second observation.

Latest-main native cleanup, after action-time approval, moved only the new 4 KB
test directory to the Recycle Bin. Without a rescan, the overview changed from
12 KB / two items to 8 KB / one item, Safe 0 B and Review 8 KB. History Restore
returned identical bytes; the Review control never changed.

An isolated-directory NSIS install and installed-mode fixture scan passed. The
existing 0.7.1 installation then upgraded in place without uninstalling. Its
settings, history and all 16 Roaming data files stayed byte-identical through
normal exit and full restart; shortcuts still target the original installation.
Original profiles and binaries are backed up locally. MSI installation, pristine
Windows/WebView2 setup and signed-updater delivery remain untested. See
`docs/RELEASE_CHECKLIST.md` for evidence; that acceptance did not publish a release.

## Deferred work

- Writing, merging, or synchronizing AI tool configurations: requires a separate
  design with change preview, backup, and recovery.
- winget/Scoop submissions and installer code signing.
- Stable promotion, MSI actual installation, pristine Windows/WebView2 setup,
  uninstall-data behavior and signed updates remain separate gates. The approved
  prerelease plan does not close them or authorize another local installation.
- Crash-safe cleanup audit journaling and transactional relocation log writes.
  v0.9 fixes swallowed history reads and preflight/write-error handling, not those
  separate persistence guarantees.
