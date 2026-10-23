import { test } from "node:test";
import assert from "node:assert/strict";
import { afterClean, withItems } from "../src/scanReport.ts";

const item = (id, size_bytes, safety = "safe") => ({
  id, rule_id: id, path: `C:/fixtures/${id}`, size_bytes, safety,
  tool: id, category: "package_cache", name: id, file_count: 1,
  last_used: null, consequence: "fixture", detail: null, in_use_by: null, auto_clean: true,
});
const report = {
  items: [item("safe", 4), item("failed", 2), item("download", 8, "redownload"), item("review", 16, "review")],
  total_bytes: 30, safe_bytes: 6, redownload_bytes: 8, review_bytes: 16,
  started_at: "2026-10-23T00:00:00Z", finished_at: "2026-10-23T00:00:01Z",
  unreadable: ["C:/unreadable"], timings: [["safe", 1]], cancelled: false,
};
const record = (i, ok) => ({ ...i, at: "2026-10-23T00:00:02Z", ok, error: ok ? null : "in use" });
const amounts = (r) => [r.total_bytes, r.safe_bytes, r.redownload_bytes, r.review_bytes];

test("partial cleanup refreshes every overview amount and retains failures/unreported items", () => {
  const next = afterClean(report, { freed_bytes: 4, records: [record(report.items[0], true), record(report.items[1], false)] });
  assert.deepEqual(next.items.map((i) => i.id), ["failed", "download", "review"]);
  assert.deepEqual(amounts(next), [26, 2, 8, 16]);
  assert.equal(next.started_at, report.started_at);
  assert.equal(next.timings, report.timings);
  assert.equal(next.unreadable, report.unreadable);
  assert.deepEqual(amounts(report), [30, 6, 8, 16]);
  assert.equal(report.items.length, 4);
});

test("all cleaned items leave a zero overview without a rescan", () => {
  const next = afterClean(report, { freed_bytes: 30, records: report.items.map((i) => record(i, true)) });
  assert.equal(next.items.length, 0);
  assert.deepEqual(amounts(next), [0, 0, 0, 0]);
});

test("matching requires both rule and path, and missing success is never assumed", () => {
  const next = afterClean(report, { freed_bytes: 4, records: [{ ...record(report.items[0], true), rule_id: "unrelated" }] });
  assert.deepEqual(next.items, report.items);
  assert.deepEqual(amounts(next), amounts(report));
});

test("other item removals also rebuild all safety buckets", () => {
  assert.deepEqual(amounts(withItems(report, [report.items[2]])), [8, 0, 8, 0]);
});
