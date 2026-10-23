# Changelog

## [0.8.0-rc.1] - 2026-10-23

Development candidate only; merging this source does not publish an installer.

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
- Older plugin, extension, and browser versions require manual review and cannot
  be selected for automatic cleanup; unknown/equal versions remain visible.
- Remaining-space and safety totals refresh together after successful cleanup;
  failed or unreported items stay visible. Relocation/exclusion totals stay consistent.
- Portable startup leaves the installed application's autostart registration alone.

### Verification
- Added offline version/safety/overview regressions and native startup-policy tests.
- Added a Windows fixture scan, Recycle Bin cleanup, and byte-identical restore test.
- Repeatable mock-IPC acceptance covers offline, disabled checks, late replies,
  language, and theme behavior. See [acceptance status](docs/RELEASE_CHECKLIST.md)
  for native observations and remaining gates; mocks do not replace desktop acceptance.

Earlier public releases: [GitHub Releases](https://github.com/nethero23/ai-disk-cleaner/releases).
