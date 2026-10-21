// Lists keys each non-source locale is missing or has extra, compared with
// the zh block of src/i18n.ts. Missing keys fall back to English at runtime.
// Usage: node tools/i18n_check.mjs  (exit code 1 if anything is missing)
import { readFileSync, readdirSync } from "node:fs";

const root = new URL("../src/", import.meta.url);

// Top-level keys of an object literal: "  key:" at the first indent level.
function keys(text, indent) {
  const re = new RegExp(`^ {${indent}}(?:"([^"]+)"|([A-Za-z_][\\w]*)):`, "gm");
  return new Set([...text.matchAll(re)].map((m) => m[1] ?? m[2]));
}

const src = readFileSync(new URL("i18n.ts", root), "utf8");
const zhBlock = src.slice(src.indexOf("  zh: {"), src.indexOf("\n  en: {"));
const want = keys(zhBlock, 4);

let bad = false;
for (const f of readdirSync(new URL("locales/", root)).sort()) {
  const have = keys(readFileSync(new URL(`locales/${f}`, root), "utf8"), 2);
  const missing = [...want].filter((k) => !have.has(k));
  const extra = [...have].filter((k) => !want.has(k));
  if (missing.length) bad = true;
  console.log(`${f}: ${want.size - missing.length}/${want.size}` +
    (missing.length ? `\n  missing: ${missing.join(", ")}` : "") +
    (extra.length ? `\n  unknown: ${extra.join(", ")}` : ""));
}
process.exit(bad ? 1 : 0);
