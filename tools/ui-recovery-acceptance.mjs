// Runs only against the Vite mock-IPC entry. No native commands or real files.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const binary = process.argv[2];
assert.ok(binary, "Pass the gstack browse binary path");
const out = resolve(process.env.ACCEPTANCE_OUT ?? "target/recovery-validation");
mkdirSync(out, { recursive: true });
const passed = [], transcript = [];
function browse(...args) {
  const output = execFileSync(binary, args, { encoding: "utf8", timeout: 35000, windowsHide: true }).trim();
  transcript.push({ command: args[0], output });
  return output;
}
function js(code) { return browse("js", code); }
function click(selector) { browse("snapshot", "-i"); browse("click", selector); }
function check(name, expression) {
  assert.equal(JSON.parse(js(`JSON.stringify(Boolean(${expression}))`)), true, name);
  passed.push(name);
}
const tab = JSON.parse(browse("newtab", `${process.env.ACCEPTANCE_ORIGIN ?? "http://127.0.0.1:1420"}/tools/ui-smoke.html`, "--json"));
try {
  browse("wait", "nav");
  js('localStorage.setItem("uiLang","zh"); localStorage.setItem("groupBy","category"); "fixture preferences"');
  browse("reload");
  browse("wait", "nav");
  for (const mode of ["blocked", "partial"]) {
    js(`window.__clearwispSmoke.cleanMode=${JSON.stringify(mode)}; "synthetic result only"`);
    click('button[aria-label="开始扫描"], button[aria-label="重新扫描"]');
    browse("wait", '.ri input[aria-label="Selection Safe"]');
    if (mode === "partial") click('.ri:has(input[aria-label="Selection Review"]) .md-checkbox');
    click('[role=toolbar] button:has-text("清理所选")');
    browse("wait", ".modal .danger");
    click(".modal .danger");
    browse("wait", ".banner.error");
    check(`${mode}: history write warning is visible`, 'document.querySelector(".banner.error").textContent.includes("Synthetic history persistence failure")');
    check(`${mode}: skipped Review remains visible`, '!!document.querySelector(\'.ri input[aria-label="Selection Review"]\')');
    check(`${mode}: both in-use rows remain disabled`, 'document.querySelectorAll(".ri input:disabled").length===2');
    check(`${mode}: Safe result follows actual success only`, `${mode === "blocked" ? "!!" : "!"}document.querySelector('.ri input[aria-label="Selection Safe"]')`);
    check(`${mode}: overview matches only successful records`, `document.querySelector(".hero-big").textContent===${JSON.stringify(mode === "blocked" ? "60.0 KB" : "56.0 KB")}`);
  }
  check("every cleanup call is synthetic fixture-only", 'window.__clearwispSmoke.calls.filter(c=>c.cmd==="clean_items").every(c=>c.args.items.every(i=>i.path.startsWith("C:/fixtures/")))');
  browse("screenshot", "--viewport", resolve(out, "recovery-log-warning.png"));
  assert.match(browse("console", "--errors"), /no console errors/);
  passed.push("no browser console errors");
  console.log(JSON.stringify({ passed }, null, 2));
} finally {
  writeFileSync(resolve(out, "recovery-browser-acceptance.json"), JSON.stringify({ passed, transcript }, null, 2));
  browse("closetab", String(tab.tabId));
}
