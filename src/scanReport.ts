import type { CleanSummary, ScanItem, ScanReport } from "./types";

/** Keep the remaining items and their overview in one state update. */
export function withItems(report: ScanReport, items: ScanItem[]): ScanReport {
  const totals = { total_bytes: 0, safe_bytes: 0, redownload_bytes: 0, review_bytes: 0 };
  for (const item of items) {
    totals.total_bytes += item.size_bytes;
    totals[`${item.safety}_bytes`] += item.size_bytes;
  }
  return { ...report, ...totals, items };
}

/** Only confirmed successes disappear; failed or unreported items remain. */
export function afterClean(report: ScanReport, summary: CleanSummary): ScanReport {
  const cleaned = new Set(summary.records.filter((r) => r.ok).map((r) => JSON.stringify([r.rule_id, r.path])));
  return withItems(report, report.items.filter((i) => !cleaned.has(JSON.stringify([i.rule_id, i.path]))));
}
