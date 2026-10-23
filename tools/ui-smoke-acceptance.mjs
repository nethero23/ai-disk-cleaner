// Repeatable UI acceptance through the /browse driver, with synthetic IPC only.
// Start `npm run dev`, then: node tools/ui-smoke-acceptance.mjs <browse binary>
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const binary = process.argv[2];
assert.ok(binary, "Pass the gstack browse binary path");
const out = resolve(process.env.ACCEPTANCE_OUT ?? "target/v08-validation");
mkdirSync(out, { recursive: true });
const transcript = [];
const passed = [];
function browse(...args) {
  const result = execFileSync(binary, args, { encoding: "utf8", timeout: 35000, windowsHide: true });
  transcript.push({ command: args[0], output: result });
  return result.trim();
}
function click(selector) {
  browse("snapshot", "-i");
  browse("click", selector);
}
function check(name, expression) {
  const result = JSON.parse(browse("js", `JSON.stringify({pass:Boolean(${expression})})`));
  assert.equal(result.pass, true, name);
  passed.push(name);
}
function language(value) {
  const tree = browse("snapshot", "-i");
  const ref = tree.match(/(@e\d+) \[combobox\]/)?.[1];
  assert.ok(ref, "Settings language selector is present");
  browse("select", ref, value);
}

const origin = process.env.ACCEPTANCE_ORIGIN ?? "http://127.0.0.1:1420";
const tab = JSON.parse(browse("newtab", `${origin}/tools/ui-smoke.html`, "--json"));
try {
  browse("wait", "nav");
  browse("js", 'localStorage.setItem("uiLang","zh"); localStorage.setItem("theme","system"); localStorage.setItem("groupBy","category"); "reset fixture-only preferences"');
  browse("reload");
  browse("wait", 'button:text-is("AI 工具")');
  click('button[aria-label="开始扫描"]');
  browse("wait", '.ri input[aria-label="Selection Safe"]');
  const safe = '.ri:has(input[aria-label="Selection Safe"]) .md-checkbox';
  const review = '.ri:has(input[aria-label="Selection Review"]) .md-checkbox';
  const group = '.cg:has(input[aria-label="Selection Safe"]) .cg-head .md-checkbox';
  const checked = (label) => `document.querySelector('input[aria-label="${label}"]').checked`;
  check("only eligible Safe fixture is selected by default", `${checked("Selection Safe")} && !${checked("Selection Review")} && !${checked("Selection In Use")}`);
  browse("js", 'JSON.stringify([...document.querySelectorAll(".md-checkbox")].map(e=>({control:e.getBoundingClientRect().toJSON(),input:e.querySelector("input").getBoundingClientRect().toJSON()})))');
  click(safe); // Click the visible 40px control's center, NOT the tiny native input.
  check("mouse center deselects the Safe item", `!${checked("Selection Safe")}`);
  check("deselection clears the selection toolbar", '!document.querySelector(\'[role="toolbar"]\')');
  click(safe);
  check("mouse center selects the Safe item", checked("Selection Safe"));
  check("selection toolbar follows selected count and bytes", 'document.querySelector(\'[role="toolbar"]\').textContent.includes("已选 1 项，4.0 KB")');
  browse("js", 'document.querySelector(\'.ri input[aria-label="Selection Safe"]\').focus(); "focus fixture checkbox"');
  browse("press", "Space");
  check("keyboard Space deselects the item", `!${checked("Selection Safe")}`);
  browse("press", "Space");
  check("keyboard Space selects the item", checked("Selection Safe"));
  click(review);
  check("manual Review selection works", checked("Selection Review"));
  click(review);
  check("manual Review deselection works", `!${checked("Selection Review")}`);
  click(group);
  check("group selects eligible rows only", `${checked("Selection Safe")} && ${checked("Selection Review")} && !${checked("Selection In Use")}`);
  click(group);
  check("mixed in-use group can clear eligible rows", `!${checked("Selection Safe")} && !${checked("Selection Review")} && !${checked("Selection In Use")}`);
  check("in-use input remains disabled", 'document.querySelector(\'input[aria-label="Selection In Use"]\').disabled');
  check("fully in-use group is disabled and unchecked", 'document.querySelector(\'.cg:has(input[aria-label="Selection All Busy"]) .cg-head input\').disabled && !document.querySelector(\'.cg:has(input[aria-label="Selection All Busy"]) .cg-head input\').checked');
  check("row checkboxes do not expand details", '!document.querySelector(".ri.open")');
  click('button:text-is("只选安全项")');
  check("safe-only action retains in-use protection", `${checked("Selection Safe")} && !${checked("Selection Review")} && !${checked("Selection In Use")}`);
  click('button:text-is("AI 工具")');
  browse("wait", ".ai-summary");
  click('[role="radio"]:text-is("按 MCP")');
  browse("wait", ".ai-behind");
  check("newer version is displayed", 'document.querySelector(".ai-behind").textContent.includes("2.0.0")');
  browse("screenshot", "--viewport", resolve(out, "browser-ai-versions.png"));

  browse("js", 'window.__clearwispSmoke.mode="offline"; "offline fixture"');
  click('button:text-is("重新盘点")');
  browse("wait", '.ai-page [role="status"]');
  check("offline failure preserves inventory", 'document.querySelector(".ai-page").textContent.includes("暂时无法获取") && document.querySelector(".ai-page").textContent.includes("Demo MCP") && !document.querySelector(".ai-behind")');

  click('button:text-is("设置")');
  click('.md-switch:has-text("AI 工具")');
  check("version-check setting is disabled", '!window.__clearwispSmoke.settings.mcp_version_check');
  browse("js", 'window.__clearwispSmoke.calls=[]; "reset fixture log"');
  click('button:text-is("AI 工具")');
  browse("wait", ".ai-summary");
  check("disabled checks issue no IPC query", '!window.__clearwispSmoke.calls.some(x=>x.cmd==="ai_version_check")');
  browse("reload");
  browse("wait", 'button:text-is("AI 工具")');
  click('button:text-is("AI 工具")');
  browse("wait", ".ai-summary");
  check("disabled setting survives reload", '!window.__clearwispSmoke.settings.mcp_version_check && !window.__clearwispSmoke.calls.some(x=>x.cmd==="ai_version_check")');

  click('button:text-is("设置")');
  click('.md-switch:has-text("AI 工具")');
  browse("js", 'window.__clearwispSmoke.mode="slow"; "slow fixture"');
  click('button:text-is("AI 工具")');
  browse("wait", '.ai-page [role="status"]');
  click('button:text-is("设置")');
  click('.md-switch:has-text("AI 工具")');
  browse("js", 'window.__clearwispSmoke.calls=[]; "reset after disable"');
  click('button:text-is("AI 工具")');
  browse("wait", ".ai-summary");
  browse("js", '(async()=>{await new Promise(r=>setTimeout(r,1800));return "late reply settled"})()');
  check("late response cannot re-enable version display", '!window.__clearwispSmoke.settings.mcp_version_check && !window.__clearwispSmoke.calls.some(x=>x.cmd==="ai_version_check") && !document.querySelector(".ai-behind") && !document.querySelector(".ai-page [role=status]")');

  click('button:text-is("设置")');
  click('[role="radio"]:text-is("深色")');
  language("ar");
  check("Arabic layout and dark theme apply", 'document.documentElement.dir==="rtl" && document.documentElement.dataset.theme==="dark"');
  browse("screenshot", "--viewport", resolve(out, "browser-ar-dark.png"));
  browse("reload");
  browse("wait", "nav");
  check("theme and Arabic survive reload", 'document.documentElement.dir==="rtl" && document.documentElement.dataset.theme==="dark"');
  check("fixture never calls mutating native operations", '!window.__clearwispSmoke.calls.some(x=>["clean_items","relocate","deduplicate","compact"].includes(x.cmd))');
  assert.match(browse("console", "--errors"), /no console errors/);
  passed.push("no browser console errors");
  console.log(JSON.stringify({ passed }, null, 2));
} finally {
  writeFileSync(resolve(out, "browser-acceptance.json"), JSON.stringify({ passed, transcript }, null, 2));
  browse("closetab", String(tab.tabId));
}
