# Changelog

## [0.8.0-rc.1] - 2026-10-23

Release candidate, not a stable release. Tagged candidates publish NSIS and
portable packages only; stable downloads and automatic updates are not promoted.

### Added
- Optional MCP package-version checks against npm/PyPI, with registry allowlists,
  bounded request counts and timeouts, and protection against stale UI replies.
- AI-tool inventory translations across all 12 interface languages.

### Changed
- AI-tool inventory reads up to four tools concurrently while preserving catalog
  order, content hashes, masked values, and fresh results without a persistent cache.
- Inventory descriptions distinguish the highest detected version from the active
  version, and describe duplicate/older-version space as potential savings.

### Fixed
- Manual selection responds across the entire drawn checkbox, not a hidden
  16px corner. Group select/clear and checked state ignore in-use rows; groups
  containing only in-use items remain disabled.
- Candidate releases cannot become GitHub's latest stable release or publish
  unvalidated MSI/registry manifests; release tags must match application versions.
- Recycle Bin history and restore recognize Windows short-name parent paths;
  redirected ancestors never broaden matching to an unrelated original path.
- MCP version checks stop starting requests if the settings file cannot be read
  or parsed, rather than falling back to an enabled network permission.
- Older plugin, extension, and browser versions require manual review and cannot
  be selected for automatic cleanup; unknown/equal versions remain visible.
- Remaining-space and safety totals refresh together after successful cleanup;
  failed or unreported items stay visible. Relocation/exclusion totals stay consistent.
- Portable startup leaves the installed application's autostart registration alone.

### Verification
- Selection regressions reproduced before repair; 25 mock-browser checks,
  10 isolated native checks and 10 actual-install/restart checks passed. Existing
  settings/history stayed byte-identical; selection tests performed no cleanup.
- Added offline version/safety/overview regressions and native startup-policy tests.
- Added a Windows fixture scan, Recycle Bin cleanup, and byte-identical restore test.
- Repeatable mock-IPC acceptance covers offline, disabled checks, late replies,
  language, and theme behavior. See [acceptance status](docs/RELEASE_CHECKLIST.md)
  for native observations and remaining gates; mocks do not replace desktop acceptance.

Earlier public releases: [GitHub Releases](https://github.com/nethero23/ai-disk-cleaner/releases).
