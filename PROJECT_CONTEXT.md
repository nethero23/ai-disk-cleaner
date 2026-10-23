# Clearwisp Project Context

## Current milestone

v0.8.0-rc.1 local integration candidate on `feat/v0.8-integration`.
This is not a published release. The latest local release tag remains `v0.7.1`.

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
compilation. Earlier unsigned bundles are local and predate this permission fix.

The first GitHub Windows roundtrip exposed short-name parent paths not matching
the Recycle Bin's canonical spelling. Restore/status/dedup now normalize existing
ancestors only when no reparse/symlink ancestor is present. The original failure
was reproduced locally with an 8.3 TEMP alias, then the full suite passed there.

Full-engine measurement improved from 1.503s to 0.863s in a same-host comparison;
tools and analysis serialized identically. The earlier two-minute observation
was not reproduced, so this is not a demonstrated cold-cache fix. Native first
page entry showed the complete inventory by the 6.2-second observation. Native
post-clean overview reacceptance awaits specific fixture approval.
Installation/upgrade and signed-updater checks remain open. See
`docs/RELEASE_CHECKLIST.md` for evidence.

## Deferred work

- Writing, merging, or synchronizing AI tool configurations: requires a separate
  design with change preview, backup, and recovery.
- winget/Scoop submissions and installer code signing.
- Source integration on GitHub is separate from release acceptance. Public package
  publication, release tags, installation/upgrade, and signed updates remain separate gates.
