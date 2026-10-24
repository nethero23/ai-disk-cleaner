import type { CleanSummary, ScanItem } from "./types";

export const WINDOWS_OLD_RULE = "windows.old";
export const WINDOWS_OLD_TASK = "windows_old";

export function needsSystemCleanup(items: readonly ScanItem[]): boolean {
  return items.some((item) => item.rule_id === WINDOWS_OLD_RULE);
}

export function isSystemWindowsOld(path: string, drive: string): boolean {
  const normalize = (value: string) => value.replace(/^\\\\\?\\/, "").replace(/\\/g, "/").replace(/\/+/g, "/").replace(/\/+$/, "").toLowerCase();
  return normalize(path) === `${normalize(drive)}/windows.old`;
}

export function cleanupError(summary: CleanSummary, fallback: string): string | null {
  const errors = summary.records.filter((record) => !record.ok)
    .map((record) => `${record.name}\n${record.path}\n${record.error || fallback}`);
  return [summary.log_error, ...errors].filter(Boolean).join("\n\n") || null;
}

export function remainingSelection(items: readonly ScanItem[], summary: CleanSummary): Set<string> {
  return new Set(items.filter((item) => !summary.records.some((record) =>
    record.ok && record.rule_id === item.rule_id && record.path === item.path,
  )).map((item) => item.id));
}
