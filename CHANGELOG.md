# Changelog

## [0.9.0-rc.1] - 2026-10-24

Recovery reliability candidate; stable downloads and automatic updates are not promoted.

### Fixed
- Cleanup and relocation history now distinguish a missing log from read, UTF-8,
  schema and JSON errors. Real backend failures reach the existing retry UI,
  preserving loaded rows and disabling stale recovery actions.
- Recycle Bin enumeration failures are reported instead of marking absent paths
  permanently gone. Failed cleanup records cannot be restored as successful ones.
- Cleanup validates and opens the history log before touching files. An unavailable
  or corrupt log blocks the batch; append/sync failures stop further items while
  preserving actual successful deletions and untouched items in the results.
- Foreground and scheduled cleanup expose audit failures. Legacy valid records
  without a batch ID or final newline remain readable. No log migration is required.

### Verification and limits
- Offline tests cover history errors, missing/legacy logs, unavailable bin listing,
  preflight refusal and injected mid-batch write failure; browser checks cover
  retry, stale-action guards, visible audit warnings and correct remaining totals.
- This is not a crash-safe write-ahead journal. Relocation log writes remain a
  separate transactionality follow-up. No installed app replacement is included.

## [0.8.0-rc.2] - 2026-10-24

Release candidate; stable downloads and automatic updates remain unchanged.

### Fixed
- History displays a loading state rather than an empty-history message while
  requests are pending. Rejected IPC requests preserve loaded rows and offer retry.
- Overlapping history requests cannot overwrite newer results. Restore and
  move-back controls wait for current status and remain unavailable on load errors.
- Long history paths and failure messages no longer push the action column
  outside the window. Loading/error/retry text is translated in all 12 UI languages.

### Verification
- Loading and layout failures reproduced before repair; offline browser checks
  cover pending/error/retry/late replies, narrow layouts, and existing selection flows.
- OS-level log-read errors that the existing backend maps to an empty list are
  not covered by the new IPC-error message. No cleanup or IPC contract changed.

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
