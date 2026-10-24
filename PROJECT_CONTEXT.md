# Clearwisp Project Context

## Current milestone

`0.9.0-rc.1` is published as a [GitHub prerelease](https://github.com/nethero23/ai-disk-cleaner/releases/tag/v0.9.0-rc.1).
It is not a stable promotion; publication does not automatically replace an
installed application.
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

After the desktop was unlocked, the isolated bundled v0.9 app passed native
history-error/retry checks for an exclusively locked cleanup log and malformed
cleanup/relocation logs, retaining both synthetic rows. Fixture-only scanning
found four items (137 B); ten mouse/keyboard/group/manual-review/disabled-state
selection checks passed. Both rows survived complete exit/restart. The candidate
was then exited and the original installed app reopened; all twelve baseline
file hashes stayed unchanged. No native UI cleanup/restore or installer upgrade
ran in that isolated lane. Private evidence stays in ignored `target/v09-release-20261024/`.

[PR #65](https://github.com/nethero23/ai-disk-cleaner/pull/65) merged at
`72872087f6a183ae2d06ef421495a234e465a40c`; the new annotated release tag targets
that exact reviewed source tree. [Tag CI](https://github.com/nethero23/ai-disk-cleaner/actions?query=branch%3Av0.9.0-rc.1)
passed on Windows/Linux. Both public downloads match GitHub's SHA-256 digests.
The portable ZIP contains only its executable, README, empty data directory and
marker; product version is `0.9.0-rc.1`. Both Windows binaries are `NotSigned`.
At publication, cloud binaries were inspected, not executed. Exactly NSIS and portable
ZIP were published: no MSI, updater or package-manager manifests. Stable Latest
remains `v0.7.1`; the then-installed `0.8.0-rc.1` was unchanged at that checkpoint. Package hashes
are recorded in the [acceptance checklist](docs/RELEASE_CHECKLIST.md).

This is not a crash-safe write-ahead journal. Unexpected post-deletion append
failure is visible and stops the batch, but cannot guarantee a durable success
record; transactional relocation log writes remain a separate follow-up.

### Windows.old merge-only source follow-up (2026-10-24)

The owner chose repair-source/docs merge and push only, with no new package,
tag, updater or stable/Latest promotion. Main retains version `0.9.0-rc.1`.
The pre-existing stable-version edits in `Cargo.toml`, `Cargo.lock`, `package.json`,
`package-lock.json` and `src-tauri/tauri.conf.json` are excluded from this merge
and remain local, uncommitted `0.9.0` preparation. The accepted installed local
`0.9.0` R2 executable is unchanged.

After R2 acceptance, pre-merge source hardening rejects `.`/`..` path segments
before generic recycling and scrolls success-only system-cleanup feedback into
view. Fresh local offline tests/build checks passed for committed source
`e67ce35`; counts and remaining behavior-group gaps are in the focused acceptance
record. This is not hosted CI, a new installation or real system cleanup. The R2
native receipts and artifact hashes below do not certify these later source
bytes or future cloud packages.

### Stable v0.9 acceptance in progress (2026-10-24)

`0.9.0` is a local validation build, not a published stable release. Its five
existing version files are prepared. The originally frozen acceptance packages
changed no runtime behavior, IPC or log schemas; automated gates and unsigned
NSIS/MSI packaging passed for those bytes.

The subsequent Windows.old incident adds a separate native cleanup task and
visible per-item failure feedback. At the local R2 checkpoint, the repair passed
20 Node tests, 116 core unit tests, two Windows integration tests (fixture recycle/restore and
mocked PowerShell), two native policy tests and 17 mock-IPC browser assertions.
Frontend/native compilation, fmt, Clippy and locale parity passed. This approval
covers no real Windows.old removal or cleanup elevation. A subsequent local
in-place NSIS update retained all 371 profile files before first launch, including
the 95 history records; its real scan exposed repeated path separators that the
original mocks omitted. Frontend/backend matching is now corrected with RED/GREEN
tests. After separate action-time approval, the final rebuilt package completed
same-version Add/Reinstall Components at the original location, without uninstall.
All 374 then-current profile files stayed identical before first launch. Its
installed 18,895,872-byte executable matches the frozen NSIS-marker reference.
Actual scan-to-Windows.old-only confirmation and Cancel passed, retaining the
other ten Safe selections. Complete exit and restart retained all 95 records
(81 successes), all fourteen critical configuration/history/rule hashes and both
shortcut hashes. Refusal feedback passed with isolated fixtures and the same
binary bytes. This closes the local repaired-NSIS acceptance lane only, not actual
Windows.old removal or the remaining stable-release gates. No publication ran.
Original configuration and history remain in place. Earlier installer/runtime
acceptance does not certify these new source changes; stable gates remain open. See
`docs/WINDOWS_OLD_CLEANUP_ACCEPTANCE.md`.

The isolated bundled native app scanned only two new fixtures (12 KB). After
action-time approval, it recycled only the 4,096-byte Safe directory. Without
rescanning, the overview changed to one item / 8 KB, Safe 0 B and Review 8 KB.
Per-record history restore returned identical bytes; the 8,192-byte Review
control never changed. Complete exit reached zero processes and relaunch one;
all three isolated history rows, including a synthetic legacy success and
failure, loaded normally and the restored row reported AtOriginal. A later
read-only rescan found both fixtures and restored the Safe-only default selection.
Original installed program/profile backups (370 files) and two shortcuts remained
hash-identical at this checkpoint; no personal history was used for cleanup tests.

The local NSIS installed successfully into a new test directory with Do not
uninstall selected; actual extraction and success were observed. The original
installation's two registration keys and two shortcuts were restored and verified.
The installed payload differs from the local portable payload only by Tauri's
three-byte NSIS marker. The desktop launcher resolved the standard filename to
the registered original app, so installed-mode runtime checks used a byte-identical
uniquely named companion in the test installation, without a portable marker;
its physical process path/version were verified before testing. Original history
(95 records / 81 successes / 10.0 GB) loaded before and after full exit/restart.
The original program, fourteen non-scan configuration/rule files and shortcuts
stayed unchanged; mutable WebView/scan metadata is not claimed byte-identical.
No original-profile scan, cleanup or restore ran. This is a new-directory install
with an existing profile, not a pristine-system install or an in-place upgrade.

The verified public `v0.7.1` NSIS package also installed into a separate test
directory with Do not uninstall selected; original registration and shortcuts
were restored. Its byte-identical, uniquely named payload used an independent
portable profile: 136 built-in rules disabled, only two new fixtures (12 KB),
and two synthetic legacy history rows. The old runtime displayed both rows
(one success / 1 KB and one failure), scanned exactly the two unselected items,
and exited normally without cleanup or restore. Nine critical test files stayed
unchanged, and all 187 test files were backed up and hash-verified before upgrade.
The local stable NSIS then upgraded that registered test installation from
`0.7.1` to `0.9.0`, again without uninstalling. Only the executable and uninstaller
changed; the other 185 test files stayed identical before first launch. The
upgraded payload matched the earlier clean-directory NSIS installation. Both
legacy rows loaded normally; a read-only scan still found exactly two fixtures
/ 12 KB. Eight mouse/Space/group selection-and-clear checks passed and ended
with nothing selected. Complete exit/restart retained both history rows and all
ten recorded critical file hashes. Original registration/shortcuts were restored;
original configuration, 95 history records and verified backups stayed intact.
No cleanup or restore ran in this upgrade lane. Runtime checks used the same
byte-identical unique-filename workaround, with physical process verification.
This closes the isolated v0.7.1 upgrade case, not the installed-candidate or
pristine-system cases.

The public `v0.9.0-rc.1` NSIS package, rechecked against GitHub's asset digest,
has now installed into another separate test directory. Its same-version
maintenance page used Add/reinstall components, not Uninstall; the destination
was explicitly redirected away from the original installation. Automatic launch
and desktop shortcut creation were disabled, and original registration and
shortcuts were restored. The 18,881,024-byte payload matches the existing verified
public NSIS installation. Its byte-identical uniquely named runtime used only an
independent portable profile: two synthetic batched history rows displayed
normally (one success / 1 KB and one failure), and a read-only scan found exactly
two unselected fixtures / 12 KB. It exited normally without cleanup or restore.
All nine critical files stayed unchanged; all 172 test files were backed up and
hash-verified. The local stable NSIS subsequently upgraded this registered test
installation to `0.9.0` with Do not uninstall selected. Only executable and
uninstaller changed; the other 170 files matched before launch. Both batched
history rows and the same two fixtures / 12 KB loaded normally. Eight
mouse/Space/group select-and-clear checks passed, ending with nothing selected.
Full exit/restart preserved both rows and ten critical file hashes; all 172
candidate rollback files remained verified. Original registration/shortcuts were
restored and original program/configuration/95 history records stayed intact.
No cleanup or restore ran; this was a separate candidate installation, not an
upgrade of the original application.

Uninstall acceptance is still pending. Generated NSIS ordinary-uninstall logic
also clears Windows jump-list/recent-destination entries for the shared bundle
ID, even when Delete app data is unchecked. Do not equate that operation with
test-directory-only deletion on a host carrying the original installation.
The owner retained the original test-directory-only scope; ordinary uninstall
has not run and its acceptance gate remains unverified. Do not clear shared
Windows recent destinations or treat this deliberate hold as a passing test.
No pristine Windows/WebView2
test environment is available, and MSI administrative install and signed-updater
delivery are not certified. Keep these stable gates open:
do not create a stable tag/release or promote Latest from this local result.
Private receipts/backups stay in ignored `target/v09-stable-20261024/`; see the
[stable acceptance gates](docs/RELEASE_CHECKLIST.md#v090-stable-acceptance-in-progress).

### Published v0.9 NSIS installation acceptance (2026-10-24)

After separate action-time approval and a hash-verified backup of 368 program/
profile files plus two shortcuts, the actual GitHub NSIS package upgraded
`0.8.0-rc.1` to `0.9.0-rc.1` at the existing location without uninstalling or
resetting data. Extraction and successful completion were observed. The installed
18,881,024-byte executable matches the public cloud payload except Tauri's
documented three-byte NSIS bundle marker; the portable cloud executable was not run.

Ten installed-native checks passed: mouse-center/Space item select and clear,
eight-item project-group select and clear, clicks on disabled busy group/item,
and default Safe clear/reselect. This read-only scan found 49 items (69.4 GB,
3.6 s); default selection was restored to 15 items / 2.8 GB. Full exit reached
zero processes; relaunch through the original app entry loaded history normally.
Settings, all 95 history records (81 successful), fourteen non-scan Roaming
configuration/rule files and two shortcuts retained their baseline hashes.
Scan/growth and WebView runtime metadata may change; whole-profile byte identity
is not claimed. No cleanup or restore ran. Private receipts/backups stay in
ignored `target/github-v09-install-acceptance-20261024/`; see the
[installed acceptance record](docs/RELEASE_CHECKLIST.md#github-v09-in-place-upgrade-and-retest-2026-10-24).
This one upgrade does not close the remaining stable-release gates or republish
any package or tag.

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
