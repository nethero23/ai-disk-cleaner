# Clearwisp Project Context

## Current milestone

v0.8.0-rc.1 source merged to `main` through PR #59; tested source is
`73829ed01ea4ffa26271ea2cd35a03bd27d6f235`. Latest-main local packaging and
NSIS desktop/install/upgrade acceptance passed. This is not a published release;
the latest local release tag remains `v0.7.1`.

### Scan-selection follow-up (local only)

`1b35511` fixes manual selection on a local follow-up branch, not yet merged or
publicly released. Legacy 16px native-input sizing missed the center of the 40px
drawn checkbox; the input now covers the control. Group selection/clear and its
checked state now consider only eligible rows, with all-in-use groups disabled.
Both original failures were reproduced before their fixes. All 25 browser
interaction checks passed, plus a rebuilt native fixture run covering visible
mouse-center selection/clear, keyboard Space, explicit Review selection and
mixed in-use group toggles. Busy protections remain intact; no cleanup ran and
fixture/configuration/history hashes stayed unchanged. Fresh automated checks
and the unsigned local NSIS build passed. After explicit action-time approval,
the existing installation was updated in place without uninstalling. Its binary
matches the tested candidate apart from Tauri's three-byte NSIS bundle marker.
On an actual 79-item scan, mouse-center and Space toggles changed selection from
16 / 2.5 GB to 17 / 2.9 GB and back. The 13-row project group selected and cleared
correctly; busy Chrome rows remained disabled. A full exit/restart through the
original app entry succeeded, and the existing history still rendered. Settings
and cleanup history remained byte-identical. No cleanup, GitHub push, tag or
public package release was performed; original binaries/profiles are backed up.

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
temporary fixture. The mock-IPC browser flow passes 10 checks. Local unsigned
NSIS/MSI packaging succeeds with an explicit numeric MSI-version override.

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
`docs/RELEASE_CHECKLIST.md` for evidence; nothing was publicly released.

## Deferred work

- Writing, merging, or synchronizing AI tool configurations: requires a separate
  design with change preview, backup, and recovery.
- winget/Scoop submissions and installer code signing.
- Source integration on GitHub is separate from release acceptance. Public package
  publication, release tags, installation/upgrade, and signed updates remain separate gates.
