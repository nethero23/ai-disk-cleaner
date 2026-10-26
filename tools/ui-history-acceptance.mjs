// Offline history regression lane through /browse. No cleanup/restore calls.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const binary = process.argv[2];
assert.ok(binary, "Pass the gstack browse binary path");
const out = resolve(process.env.ACCEPTANCE_OUT ?? "target/history-validation");
mkdirSync(out, { recursive: true });
const passed = [], transcript = [];
function browse(...args) {
  const value = execFileSync(binary, args, { encoding: "utf8", timeout: 35000, windowsHide: true }).trim();
  transcript.push({ command: args[0], output: value });
  return value;
}
function click(selector) { browse("snapshot", "-i"); browse("click", selector); }
function js(code) { return browse("js", code); }
function check(name, expression) {
  assert.equal(JSON.parse(js(`JSON.stringify(Boolean(${expression}))`)), true, name);
  passed.push(name);
}
function mode(value) { js(`window.__clearwispSmoke.historyMode=${JSON.stringify(value)}; "fixture mode"`); }
const origin = process.env.ACCEPTANCE_ORIGIN ?? "http://127.0.0.1:1420";
const tab = JSON.parse(browse("newtab", `${origin}/tools/ui-smoke.html`, "--json"));
try {
  browse("wait", "nav");
  js('localStorage.setItem("uiLang","zh"); "fixture language"');
  browse("reload");
  browse("wait", 'button:text-is("记录")');
  mode("pending");
  click('button:text-is("记录")');
  check("pending history never claims no records", '!document.querySelector(".history").textContent.includes("还没有清理过")');
  check("pending history announces loading", 'document.querySelector(".history [role=status]")?.textContent.includes("加载")');
  js('window.__clearwispSmoke.historyPending.splice(0).forEach(resolve=>resolve()); "release fixture read"');
  browse("wait", '.history .log');
  check("loaded history displays original records", 'document.querySelector(".history").textContent.includes("History fixture")');
  for (const size of ["1100x800", "800x700"]) {
    browse("viewport", size);
    check(`history table fits ${size}`, 'document.querySelector(".app-main").scrollWidth <= document.querySelector(".app-main").clientWidth + 1');
    check(`restore action stays visible at ${size}`, '[...document.querySelectorAll(".history .log button")].every(b=>{const r=b.getBoundingClientRect(),m=document.querySelector(".app-main").getBoundingClientRect();return r.left>=m.left && r.right<=m.right})');
  }
  browse("screenshot", "--viewport", resolve(out, "history-layout.png"));
  for (const lang of ["zh", "en", "zh-TW", "ja", "ko", "de", "fr", "es", "pt", "ru", "ar", "hi"]) {
    js(`localStorage.setItem("uiLang",${JSON.stringify(lang)}); "fixture language"`);
    browse("reload");
    browse("wait", "nav");
    click('button[data-v="history"]');
    browse("wait", '.history .log button');
    check(`history layout fits in ${lang}`, 'document.querySelector(".app-main").scrollWidth <= document.querySelector(".app-main").clientWidth + 1');
  }
  browse("screenshot", "--viewport", resolve(out, "history-hi-layout.png"));
  js('localStorage.setItem("uiLang","zh"); "restore fixture language"');
  browse("reload");
  browse("wait", "nav");
  click('button[data-v="history"]');
  browse("wait", '.history .log button');
  mode("error");
  click('button:text-is("记录")');
  browse("wait", '.history [role=alert]');
  check("failed refresh retains loaded records", 'document.querySelector(".history").textContent.includes("History fixture")');
  check("failed refresh disables stale restore actions", '[...document.querySelectorAll(".history .log button")].every(b=>b.disabled)');
  mode("success");
  click('.history button:text-is("重试")');
  browse("wait", '.history .log button:enabled');
  check("retry clears the local error", '!document.querySelector(".history [role=alert]")');
  mode("pending");
  click('button:text-is("记录")');
  check("refresh disables restore while checking status", '[...document.querySelectorAll(".history .log button")].every(b=>b.disabled)');
  mode("empty");
  click('button:text-is("记录")');
  browse("wait", '.history .empty');
  js('window.__clearwispSmoke.historyPending.splice(0).forEach(resolve=>resolve()); "release stale fixture read"');
  check("late history reply cannot overwrite a newer empty result", 'document.querySelector(".history .empty")?.textContent.includes("还没有清理过") && !document.querySelector(".history .log")');
  mode("status-error");
  click('button:text-is("记录")');
  browse("wait", '.history [role=alert]');
  check("status failure retains readable history without offering restore", 'document.querySelector(".history").textContent.includes("History fixture") && !document.querySelector(".history .log button:enabled")');
  mode("move-error");
  click('button:text-is("记录")');
  browse("wait", '.history [role=alert]');
  check("relocation read failure retains cleanup history", 'document.querySelector(".history").textContent.includes("History fixture")');
  check("relocation read failure disables recovery actions", '!document.querySelector(".history .log button:enabled")');
  browse("reload");
  browse("wait", "nav");
  mode("error");
  click('button:text-is("记录")');
  browse("wait", '.history [role=alert]');
  check("first-load failure is not misreported as empty history", '!document.querySelector(".history .empty")');
  check("history checks never invoke mutation", '!window.__clearwispSmoke.calls.some(x=>["clean_items","restore_items","relocate","deduplicate","compact"].includes(x.cmd))');
  assert.match(browse("console", "--errors"), /no console errors/);
  passed.push("no browser console errors");
  console.log(JSON.stringify({ passed }, null, 2));
} finally {
  writeFileSync(resolve(out, "history-browser-acceptance.json"), JSON.stringify({ passed, transcript }, null, 2));
  browse("viewport", "1280x900");
  browse("closetab", String(tab.tabId));
}
