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

Interactive Windows UI acceptance is incomplete: the desktop helper returned a
black screenshot and could not activate the isolated validation window. No native
cleanup click was attempted. Installation/upgrade and signed-updater checks also
remain open. See `docs/RELEASE_CHECKLIST.md` for evidence and release gates.

## Deferred work

- Writing, merging, or synchronizing AI tool configurations: requires a separate
  design with change preview, backup, and recovery.
- winget/Scoop submissions and installer code signing.
- No production publication, remote push, tag creation, or installation is authorized in this validation run.
