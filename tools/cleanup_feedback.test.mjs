import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";
import * as feedback from "../src/cleanupFeedback.ts";
import { afterClean, withItems } from "../src/scanReport.ts";

const item = (id, rule_id = id) => ({ id, rule_id, name: id, path: `C:/fixtures/${id}`, size_bytes: 4, safety: "review", in_use_by: null });
const items = [item("done"), item("failed")];
const summary = { freed_bytes: 4, records: items.map((i, n) => ({ ...i, ok: n === 0, error: n ? "Access denied" : null })) };
const source = ts.createSourceFile("App.tsx", readFileSync(new URL("../src/App.tsx", import.meta.url), "utf8"), ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
function appFunction(name) {
  let found;
  const visit = (node) => {
    if (ts.isFunctionDeclaration(node) && node.name?.text === name) found = node;
    ts.forEachChild(node, visit);
  };
  visit(source);
  assert.ok(found, `${name} missing`);
  return ts.transpileModule(found.getText(source), { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
}

const old = { ...item("old", "windows.old"), path: "C:/Windows.old", size_bytes: 8 };
const oldTask = { id: "windows_old", title: "Windows.old", applicable: true };
async function request(selected, options = {}) {
  const state = { calls: [], error: null, busy: [], result: "stale", notice: "stale" };
  const context = {
    ...feedback, items: selected, selectedItems: selected, bigSelectedItems: selected,
    deepBusy: options.busy ?? null, lang: "zh", coreLang: () => "zh",
    s: { deepNotApplicable: "not applicable" }, allDrives: options.drives ?? [{ path: "C:/", system: true }],
    invoke: async (command) => { state.calls.push(command); if (options.error) throw new Error(options.error); return options.tasks ?? [options.task ?? oldTask]; },
    setError: (v) => { state.error = v; }, setResult: (v) => { state.result = v; },
    setNotice: (v) => { state.notice = v; }, setDeepBusy: (v) => { state.busy.push(v); },
    setConfirming: (v) => { state.recycleConfirm = v; }, setBigConfirm: (v) => { state.bigConfirm = v; },
    setDeepTasks() {}, setDeepResult() {}, setDeepAnalysis() {},
    setDeepSel: (v) => { state.deepSelection = [...v]; }, setView: (v) => { state.view = v; },
    setDeepConfirm: (v) => { state.systemConfirm = v; }, big: options.big ?? false,
  };
  const entry = options.entry ?? "requestClean";
  await vm.runInNewContext(`${appFunction("requestClean")}; ${entry === "requestClean" ? "" : appFunction(entry)}; ${entry}(items, big);`, context);
  return state;
}

test("mixed Windows.old selection opens only separate system confirmation, without any deletion", async () => {
  const state = await request([items[0], old]);
  assert.deepEqual(state.calls, ["deep_tasks"]);
  assert.deepEqual(state.deepSelection, ["windows_old"]);
  assert.equal(state.systemConfirm, true);
  assert.equal(state.recycleConfirm, undefined);
  assert.equal(state.view, "deep");
});

test("Windows scanner template paths with repeated separators open system confirmation", async () => {
  for (const path of ["C:\\\\Windows.old", "C:\\/Windows.old", "\\\\?\\C:\\\\Windows.old"]) {
    const state = await request([items[0], { ...old, path }]);
    assert.equal(state.error, null, path);
    assert.equal(state.systemConfirm, true, path);
    assert.deepEqual(state.calls, ["deep_tasks"]);
  }
});

test("ordinary and large-file cleanup retain their original recycle confirmations", async () => {
  const scan = await request(items);
  assert.equal(scan.recycleConfirm, true);
  assert.deepEqual(scan.calls, []);
  const big = await request(items, { big: true });
  assert.equal(big.bigConfirm, true);
  assert.deepEqual(big.calls, []);
});

test("different-volume, redirected or unavailable Windows.old cannot start system cleanup", async () => {
  for (const path of ["D:/Windows.old", "C:/fixtures/Windows.old"]) {
    const state = await request([{ ...old, path }]);
    assert.match(state.error, /not applicable/);
    assert.deepEqual(state.calls, []);
    assert.equal(state.systemConfirm, undefined);
  }
  const unavailable = await request([old], { task: { ...oldTask, applicable: false } });
  assert.match(unavailable.error, /not applicable/);
  assert.equal(unavailable.systemConfirm, undefined);
});

for (const ok of [true, false]) {
  test(`system cleanup ${ok ? "success removes only Windows.old" : "failure preserves items and is not green success"}`, async () => {
    const otherOld = { ...old, id: "other-old", path: "D:/Windows.old" };
    const state = { report: { items: [items[0], old, otherOld] }, selected: new Set(["done", "old", "other-old"]), notice: null };
    const context = {
      ...feedback, withItems, report: state.report, deepSel: new Set(["windows_old"]), deepTasks: [oldTask], lang: "zh", coreLang: () => "zh", humanSize: String,
      allDrives: [{ path: "C:/", system: true }],
      s: { deepDone: (n) => `freed ${n}`, deepFailed: String },
      invoke: async (command, args) => command === "deep_run"
        ? (assert.deepEqual([...args.ids], ["windows_old"]), { freed_bytes: ok ? 8 : 0, results: [{ id: "windows_old", ok, message: ok ? "" : "Windows.old still exists" }] }) : [],
      setDeepConfirm() {}, setDeepBusy() {}, setDeepResult() {}, setDeepAnalysis() {}, setDeepTasks() {},
      setNotice: (v) => { state.notice = v; }, setError: (v) => { state.error = v; },
      setDeepSel: (v) => { state.deepSelection = [...v]; },
      setReport: (update) => { state.report = update(state.report); },
      setSelected: (update) => { state.selected = update(state.selected); },
    };
    await vm.runInNewContext(`${appFunction("runDeep")}; runDeep();`, context);
    assert.deepEqual(state.report.items.map((i) => i.id), ok ? ["done", "other-old"] : ["done", "old", "other-old"]);
    assert.deepEqual([...state.selected], ok ? ["done", "other-old"] : ["done", "old", "other-old"]);
    assert.deepEqual(state.deepSelection, ok ? [] : ["windows_old"]);
    if (!ok) { assert.equal(state.notice, null); assert.match(state.error, /Windows.old still exists/); }
  });
}

test("cleanup feedback includes item errors, audit failures and localized fallback", () => {
  assert.match(feedback.cleanupError(summary, "未能删除"), /failed\nC:\/fixtures\/failed\nAccess denied/);
  assert.match(feedback.cleanupError({ ...summary, log_error: "journal unavailable" }, "失败"), /journal unavailable/);
  assert.equal(feedback.cleanupError({ records: [{ ...items[0], ok: false, error: null }] }, "未能删除"), "done\nC:/fixtures/done\n未能删除");
  assert.equal(feedback.cleanupError({ freed_bytes: 4, records: [summary.records[0]] }, "失败"), null);
});

test("only confirmed matching successes leave the selection", () => {
  assert.deepEqual([...feedback.remainingSelection([...items, item("unreported")], summary)], ["failed", "unreported"]);
  assert.deepEqual([...feedback.remainingSelection([items[0]], { ...summary, records: [{ ...summary.records[0], rule_id: "other" }] })], ["done"]);
});

test("Windows.old routes only to the system drive, including normalized Windows paths", () => {
  assert.ok(feedback.needsSystemCleanup([item("old", "windows.old")]));
  assert.ok(!feedback.needsSystemCleanup(items));
  for (const path of ["C:/Windows.old", "c:\\WINDOWS.OLD\\", "\\\\?\\C:\\Windows.old", "C:\\\\Windows.old", "C:\\/Windows.old"]) assert.ok(feedback.isSystemWindowsOld(path, "C:\\"), path);
  for (const path of ["D:/Windows.old", "C:/fixtures/Windows.old", "C:/Windows.old/Users", "C:/Windows", "\\\\server\\C$\\Windows.old", "\\\\C:\\Windows.old", "C:/fixtures/../Windows.old"]) assert.ok(!feedback.isSystemWindowsOld(path, "C:\\"), path);
});

for (const name of ["runClean", "runBigClean"]) {
  test(`${name} surfaces item failure and keeps failed selection`, async () => {
    const state = { selection: null, error: null, notice: "previous cleanup succeeded" };
    const context = {
      ...feedback, selectedItems: items, bigSelectedItems: items, selectedExportable: [], exportOn: false,
      s: { failed: () => "Could not delete" },
      bigSelected: new Set(items.map((i) => i.id)),
      setConfirming() {}, setBigConfirm() {}, setPhase() {}, setResult() {},
      setError(value) { state.error = value; },
      setNotice(value) { state.notice = value; },
      setSelected(value) { state.selection = value; }, setBigSelected(value) { state.selection = value; },
      setReport(update) { update(null); }, setBigItems(update) { update(null); }, afterClean,
      invoke: async (command) => { assert.equal(command, "clean_items"); return summary; },
    };
    await vm.runInNewContext(`${appFunction(name)}; ${name}();`, context);
    assert.match(state.error ?? "", /Access denied/);
    assert.equal(state.notice, null);
    assert.deepEqual([...state.selection], ["failed"]);
  });
}

test("system preflight stops while busy and releases busy after missing-drive/catalog failures", async () => {
  const busy = await request([old], { busy: "run" });
  assert.deepEqual(busy.calls, []);
  assert.deepEqual(busy.busy, []);
  assert.equal(busy.result, "stale");
  for (const options of [{ drives: [] }, { error: "catalog offline" }, { tasks: [] }]) {
    const state = await request([old], options);
    assert.match(state.error, /not applicable|catalog offline/);
    assert.deepEqual(state.busy, ["analyze", null]);
    assert.equal(state.systemConfirm, undefined);
    assert.equal(state.result, null);
    assert.equal(state.notice, null);
    assert.deepEqual(state.calls, options.drives ? [] : ["deep_tasks"]);
  }
});

for (const name of ["runClean", "runBigClean"]) {
  test(`${name} defensively redirects mixed selections without ordinary cleanup`, async () => {
    const state = await request([items[0], old], { entry: name });
    assert.deepEqual(state.calls, ["deep_tasks"]);
    assert.deepEqual(state.deepSelection, ["windows_old"]);
    assert.equal(state.systemConfirm, true);
    assert.equal(name === "runClean" ? state.recycleConfirm : state.bigConfirm, false);
  });
}

function deepHarness(sum, options = {}) {
  const state = {
    report: options.report === undefined ? withItems({}, [old, items[0]]) : options.report,
    selected: new Set(["old", "done"]), selection: new Set(options.ids ?? ["windows_old"]),
    calls: [], busy: [], error: "stale", notice: "stale", result: "stale",
  };
  const context = {
    ...feedback, withItems, report: state.report, deepSel: state.selection,
    deepTasks: options.tasks ?? [oldTask], allDrives: options.drives ?? [{ path: "C:/", system: true }],
    lang: "zh", coreLang: () => "zh", humanSize: String,
    s: { deepDone: (n) => `freed ${n}`, deepFailed: (n) => `failed ${n}` },
    invoke: async (command, args) => {
      state.calls.push(command);
      return options.invoke ? options.invoke(command, args) : command === "deep_run" ? sum : [oldTask];
    },
    setDeepConfirm: (v) => { state.confirm = v; }, setDeepBusy: (v) => { state.busy.push(v); },
    setError: (v) => { state.error = v; }, setNotice: (v) => { state.notice = v; },
    setDeepResult: (v) => { state.result = v; }, setDeepAnalysis() {},
    setDeepTasks: (v) => { context.deepTasks = v; },
    setDeepSel: (v) => { context.deepSel = state.selection = v; },
    setReport: (fn) => { context.report = state.report = fn(state.report); },
    setSelected: (fn) => { state.selected = fn(state.selected); },
  };
  return { state, run: vm.runInNewContext(`${appFunction("runDeep")}; runDeep;`, context) };
}
const deepSuccess = { freed_bytes: 8, results: [{ id: "windows_old", ok: true, message: "" }] };

test("mixed system results retain failed tasks with title/id fallback and remove only confirmed items", async () => {
  const sum = { freed_bytes: 8, results: [...deepSuccess.results,
    { id: "winsxs", ok: false, message: "denied" }, { id: "unknown", ok: false, message: "unsupported" }] };
  const { state, run } = deepHarness(sum, {
    ids: ["windows_old", "winsxs", "unknown"], tasks: [oldTask, { id: "winsxs", title: "Component store" }],
  });
  await run();
  assert.equal(state.notice, "freed 8 · failed 2");
  assert.equal(state.error, "Component store: denied\nunknown: unsupported");
  assert.deepEqual([...state.selection], ["winsxs", "unknown"]);
  assert.deepEqual(state.report.items.map((i) => i.id), ["done"]);
  assert.deepEqual([...state.selected], ["done"]);
  assert.equal(state.report.total_bytes, 4);
  assert.deepEqual(state.busy, ["run", null]);
});

test("empty system results preserve all items/tasks without stale success or error", async () => {
  const sum = { freed_bytes: 0, results: [] };
  const { state, run } = deepHarness(sum);
  await run();
  assert.deepEqual(state.report.items.map((i) => i.id), ["old", "done"]);
  assert.deepEqual([...state.selected], ["old", "done"]);
  assert.deepEqual([...state.selection], ["windows_old"]);
  assert.equal(state.notice, null);
  assert.equal(state.error, null);
  assert.equal(state.result, sum);
  assert.deepEqual(state.busy, ["run", null]);
});

test("system success tolerates absent report or drive without removing unrelated selections", async () => {
  for (const options of [{ report: null }, { drives: [] }]) {
    const { state, run } = deepHarness(deepSuccess, options);
    await run();
    assert.equal(state.error, null);
    assert.deepEqual([...state.selected], ["old", "done"]);
    assert.deepEqual([...state.selection], []);
    if (options.report === null) assert.equal(state.report, null);
    else assert.deepEqual(state.report.items.map((i) => i.id), ["old", "done"]);
    assert.deepEqual(state.busy, ["run", null]);
  }
});

test("catalog refresh rejection preserves confirmed system success and unlocks retry", async () => {
  const { state, run } = deepHarness(deepSuccess, { invoke: async (command) => {
    if (command === "deep_run") return deepSuccess;
    throw new Error("refresh offline");
  } });
  await run();
  assert.match(state.error, /refresh offline/);
  assert.equal(state.notice, "freed 8");
  assert.equal(state.result, deepSuccess);
  assert.deepEqual(state.report.items.map((i) => i.id), ["done"]);
  assert.deepEqual([...state.selected], ["done"]);
  assert.deepEqual([...state.selection], []);
  assert.deepEqual(state.busy, ["run", null]);
});

test("deep_run rejection clears stale feedback, retains selection and permits a successful retry", async () => {
  let attempts = 0;
  const { state, run } = deepHarness(deepSuccess, { invoke: async (command, args) => {
    if (command !== "deep_run") return [oldTask];
    assert.deepEqual([...args.ids], ["windows_old"]);
    if (++attempts === 1) throw new Error("UAC declined");
    return deepSuccess;
  } });
  await run();
  assert.match(state.error, /UAC declined/);
  assert.equal(state.notice, null);
  assert.equal(state.result, null);
  assert.equal(state.confirm, false);
  assert.deepEqual([...state.selection], ["windows_old"]);
  assert.deepEqual([...state.selected], ["old", "done"]);
  assert.deepEqual(state.busy, ["run", null]);
  await run();
  assert.equal(state.error, null);
  assert.deepEqual([...state.selected], ["done"]);
  assert.deepEqual(state.busy, ["run", null, "run", null]);
  assert.deepEqual(state.calls, ["deep_run", "deep_run", "deep_tasks"]);
});

for (const name of ["runClean", "runBigClean"]) {
  test(`${name} reconciles real arrays for partial/all/empty outcomes and IPC rejection`, async () => {
    const record = (i, ok) => ({ ...i, ok, error: ok ? null : "Access denied" });
    for (const records of [summary.records, items.map((i) => record(i, true)), items.map((i) => record(i, false)), [], null]) {
      const untouched = item("untouched");
      const state = { list: [...items, untouched], selection: new Set(items.map((i) => i.id)), phases: [], error: "stale", result: "stale", notice: "stale" };
      const sum = { freed_bytes: 0, records };
      const context = {
        ...feedback, afterClean, selectedItems: items, bigSelectedItems: items, bigSelected: state.selection,
        selectedExportable: [], exportOn: false, s: { failed: () => "Could not delete" },
        setConfirming() {}, setBigConfirm() {}, setPhase: (v) => { state.phases.push(v); },
        setError: (v) => { state.error = v; }, setResult: (v) => { state.result = v; }, setNotice: (v) => { state.notice = v; },
        setSelected: (v) => { state.selection = v; }, setBigSelected: (v) => { state.selection = v; },
        setReport: (fn) => { state.list = fn(withItems({}, state.list)).items; },
        setBigItems: (fn) => { state.list = fn(state.list); },
        invoke: async (command, args) => {
          assert.equal(command, "clean_items"); assert.equal(args.items, items);
          if (records === null) throw new Error("IPC unavailable");
          return sum;
        },
      };
      await vm.runInNewContext(`${appFunction(name)}; ${name}();`, context);
      const removed = new Set((records ?? []).filter((r) => r.ok).map((r) => r.id));
      assert.deepEqual(state.list.map((i) => i.id), [...items, untouched].filter((i) => !removed.has(i.id)).map((i) => i.id));
      assert.deepEqual([...state.selection], items.filter((i) => !removed.has(i.id)).map((i) => i.id));
      assert.deepEqual(state.phases, ["cleaning", "ready"]);
      assert.equal(state.notice, null);
      assert.equal(state.result, records === null ? null : sum);
      if (records === null) assert.match(state.error, /IPC unavailable/);
      else if (records.some((r) => !r.ok)) assert.match(state.error, /Access denied/);
      else assert.equal(state.error, null);
    }
  });
}

test("success-only deep feedback scrolls into view through the real effect callback", () => {
  let callback;
  const visit = (node) => {
    if (ts.isCallExpression(node) && node.expression.getText(source) === "useEffect" &&
        node.arguments[1] && ts.isArrayLiteralExpression(node.arguments[1]) &&
        node.arguments[1].elements.some((dep) => dep.getText(source) === "deepResult")) callback = node.arguments[0];
    ts.forEachChild(node, visit);
  };
  visit(source);
  assert.ok(callback, "feedback effect missing");
  const code = ts.transpileModule(`(${callback.getText(source)})();`, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
  const calls = [];
  vm.runInNewContext(code, {
    error: null, result: null, deepResult: deepSuccess,
    feedbackRef: { current: { scrollIntoView: (options) => { calls.push(options.block); } } },
  });
  assert.deepEqual(calls, ["start"]);
});
