import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { check as checkUpdate, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { UnrecognizedPanel } from "./Unrecognized";
import type { Growth, Overview, OverviewEntry, Category, CleanRecord, CleanSummary, RestoreStatus, RestoreSummary, Whereabouts, CompactResult, Drive, DuplicateGroup, ExportResult, Preview, RelocateRecord, RelocateResult, RuleLibrary, RulesUpdate, ScanItem, ScanReport, Settings, Vhdx, DeepAnalysis, DeepSummary, DeepTask } from "./types";
import { humanSize, relativeDate, t, type Lang, sinceDuration } from "./i18n";
import { DrivePicker } from "./DrivePicker";
import { Fab, NavigationRail, Tabs } from "./ui";

type Phase = "idle" | "scanning" | "ready" | "cleaning";
type Page = "clean" | "space" | "history" | "settings";

const CATEGORY_ORDER: Category[] = [
  "models",
  "python_env",
  "package_cache",
  "ai_editor",
  "ai_agent",
  "ai_desktop_app",
  "container",
  "gpu",
  "browser",
  "project_artifacts",
  "system",
];

/** Ticked by default: pure caches whose owning tool is not running. */
function isDefaultTicked(i: ScanItem): boolean {
  return i.safety === "safe" && !i.in_use_by && i.auto_clean !== false;
}

export function App() {
  const [lang, setLang] = useState<Lang>(() => (localStorage.getItem("lang") as Lang) || "zh");
  const s = t(lang);
  const [phase, setPhase] = useState<Phase>("idle");
  const [report, setReport] = useState<ScanReport | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [confirming, setConfirming] = useState(false);
  const [result, setResult] = useState<CleanSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [query, setQuery] = useState("");
  const [groupBy, setGroupBy] = useState<"category" | "tool">(() => (localStorage.getItem("groupBy") as "category" | "tool") || "category");
  const [sortBy, setSortBy] = useState<"size" | "recent" | "safety" | "name">(() => (localStorage.getItem("sortBy") as "size" | "recent" | "safety" | "name") || "size");
  const [view, setView] = useState<"scan" | "history" | "settings" | "bigfiles" | "overview" | "deep">("scan");
  const page: Page = view === "scan" || view === "deep" ? "clean" : view === "overview" || view === "bigfiles" ? "space" : view;
  const [bigDrives, setBigDrives] = useState<Drive[]>([]);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [ovRoot, setOvRoot] = useState<string | null>(null);
  const [ovBusy, setOvBusy] = useState(false);
  const [ovProgress, setOvProgress] = useState<{ seen: number; bytes: number; dir: string } | null>(null);
  const [ovOpen, setOvOpen] = useState<Set<string>>(new Set());
  const [allDrives, setAllDrives] = useState<Drive[]>([]);
  const [bigPick, setBigPick] = useState<Set<string>>(new Set());
  const [bigMin, setBigMin] = useState(500 * 1024 * 1024);
  const [bigScanning, setBigScanning] = useState(false);
  const [bigProgress, setBigProgress] = useState<{ seen: number; bytes: number; dir: string } | null>(null);
  const autoConfirmRef = useRef(false);
  const [peeks, setPeeks] = useState<Map<string, Preview | "loading">>(new Map());
  const [scanStep, setScanStep] = useState<{ done: number; total: number; rule_name: string } | null>(null);
  const [bigItems, setBigItems] = useState<ScanItem[] | null>(null);
  const [bigSelected, setBigSelected] = useState<Set<string>>(new Set());
  const [bigConfirm, setBigConfirm] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [library, setLibrary] = useState<RuleLibrary | null>(null);
  const [updatingRules, setUpdatingRules] = useState(false);
  const [history, setHistory] = useState<CleanRecord[]>([]);
  const [relocations, setRelocations] = useState<RelocateRecord[]>([]);
  const [movingBack, setMovingBack] = useState<{ rec: RelocateRecord; progress?: [number, number] } | null>(null);
  const [whereabouts, setWhereabouts] = useState<Map<string, Whereabouts>>(new Map());
  const [restoring, setRestoring] = useState(false);
  const [relocatable, setRelocatable] = useState<Set<string>>(new Set());
  const [compactable, setCompactable] = useState<Set<string>>(new Set());
  const [compacting, setCompacting] = useState<{ item: ScanItem; disks: Vhdx[]; busy?: string; results: CompactResult[] } | null>(null);
  const [exportable, setExportable] = useState<Set<string>>(new Set());
  const [exportDest, setExportDest] = useState<string | null>(null);
  const [exportOn, setExportOn] = useState(true);
  const [exporting, setExporting] = useState<[number, number] | null>(null);
  const [moving, setMoving] = useState<{ item: ScanItem; dest: string; source: string; progress?: [number, number]; result?: RelocateResult } | null>(null);
  const [dups, setDups] = useState<DuplicateGroup[] | null>(null);
  const [dupsBusy, setDupsBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [growth, setGrowth] = useState<Growth[]>([]);
  const [obStep, setObStep] = useState<number | null>(null);
  const [portable, setPortable] = useState(false);
  const [update, setUpdate] = useState<Update | null>(null);
  const [updateState, setUpdateState] = useState<"idle" | "checking" | "downloading" | "installed">("idle");
  const [updatePct, setUpdatePct] = useState(0);
  const [updateMsg, setUpdateMsg] = useState<string | null>(null);

  useEffect(() => {
    invoke<string[]>("relocatable").then((ids) => setRelocatable(new Set(ids))).catch(() => {});
    invoke<string[]>("compactable").then((ids) => setCompactable(new Set(ids))).catch(() => {});
    invoke<string[]>("exportable").then((ids) => setExportable(new Set(ids))).catch(() => {});
  }, []);

  async function pickMoveTarget(item: ScanItem) {
    const dest = await openDialog({ directory: true, multiple: false, title: s.moveTitle });
    if (typeof dest !== "string") return;
    const source = await invoke<string>("move_source", { ruleId: item.rule_id, path: item.path }).catch(() => item.path);
    setMoving({ item, dest, source });
  }

  async function runMove() {
    if (!moving) return;
    const { item, dest, source } = moving;
    setMoving({ item, dest, source, progress: [0, item.size_bytes] });
    const unlisten = await listen<{ copied: number; total: number }>("relocate-progress", (e) => {
      setMoving((m) => (m ? { ...m, progress: [e.payload.copied, e.payload.total] } : m));
    });
    try {
      const result = await invoke<RelocateResult>("relocate_item", { ruleId: item.rule_id, path: item.path, destRoot: dest });
      setMoving({ item, dest, source, result });
      // A whole store may have moved, taking its other items with it.
      const moved = (i: ScanItem) =>
        i.id === item.id || i.path === result.from || i.path.startsWith(result.from + "\\") || i.path.startsWith(result.from + "/");
      setReport((prev) => (prev ? { ...prev, items: prev.items.filter((i) => !moved(i)) } : prev));
      setSelected((prev) => {
        const next = new Set(prev);
        next.delete(item.id);
        return next;
      });
    } catch (e) {
      setError(String(e));
      setMoving(null);
    } finally {
      unlisten();
    }
  }

  async function runFindDups() {
    if (!report) return;
    setDupsBusy(true);
    try {
      setDups(await invoke<DuplicateGroup[]>("find_duplicates", { items: report.items }));
    } catch (e) {
      setError(String(e));
    }
    setDupsBusy(false);
  }

  async function runMerge(group: DuplicateGroup) {
    try {
      const freed = await invoke<number>("merge_duplicates", { group, keep: group.files[0].path });
      setNotice(s.merged(humanSize(freed)));
      setDups((prev) => (prev ? prev.filter((g) => g !== group) : prev));
    } catch (e) {
      setError(String(e));
    }
  }

  async function openCompact(item: ScanItem) {
    try {
      const disks = await invoke<Vhdx[]>("vhdx_files", { path: item.path });
      setCompacting({ item, disks, results: [] });
    } catch (e) {
      setError(String(e));
    }
  }

  async function openSettings() {
    try {
      setSettings(await invoke<Settings>("get_settings"));
      setLibrary(await invoke<RuleLibrary>("rules"));
    } catch (e) {
      setError(String(e));
    }
    setView("settings");
  }

  async function runCheckUpdate(manual: boolean) {
    setUpdateState("checking");
    setUpdateMsg(null);
    try {
      const u = await checkUpdate({ timeout: 15000 });
      setUpdate(u);
      if (!u && manual) setUpdateMsg(s.upToDate(await getVersion().catch(() => "")));
    } catch (e) {
      if (manual) setUpdateMsg(s.updateFailed(String(e)));
    }
    setUpdateState("idle");
  }

  async function installUpdate() {
    if (!update) return;
    setUpdateState("downloading");
    setUpdatePct(0);
    let total = 0;
    let got = 0;
    try {
      await update.downloadAndInstall((ev) => {
        if (ev.event === "Started") total = ev.data.contentLength ?? 0;
        else if (ev.event === "Progress") {
          got += ev.data.chunkLength;
          if (total) setUpdatePct(Math.min(100, Math.round((got / total) * 100)));
        }
      });
      setUpdateState("installed");
    } catch (e) {
      setUpdateMsg(s.updateFailed(String(e)));
      setUpdateState("idle");
    }
  }

  useEffect(() => {
    invoke<Drive[]>("drives").then(setAllDrives).catch(() => {});
    invoke<Settings>("get_settings")
      .then(async (st) => {
        setSettings(st);
        if (!st.onboarded) setObStep(0);
        const p = await invoke<boolean>("is_portable").catch(() => false);
        setPortable(p);
        if (st.auto_update_check && !p) runCheckUpdate(false);
      })
      .catch(() => {});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Models nobody has touched for a while: the AI-specific "what can go" list.
  const idleModels = useMemo(() => {
    if (!report) return [];
    const days = settings?.idle_model_days ?? 90;
    const cutoff = Date.now() - days * 86_400_000;
    return report.items
      .filter((i) => i.category === "models" && i.last_used && new Date(i.last_used).getTime() < cutoff)
      .sort((a, b) => b.size_bytes - a.size_bytes);
  }, [report, settings]);
  function finishOnboarding() {
    setObStep(null);
    if (settings && !settings.onboarded) saveSettings({ ...settings, onboarded: true });
  }

  // One sentence after a scan: what can go safely and who is the biggest offender.
  const summaryLine = useMemo(() => {
    if (!report) return "";
    if (report.items.length === 0) return s.sumNothing;
    const parts: string[] = [];
    if (report.safe_bytes > 0) {
      parts.push(s.sumSafe(humanSize(report.safe_bytes)));
      const byTool = new Map<string, number>();
      for (const i of report.items) if (i.safety === "safe") byTool.set(i.tool, (byTool.get(i.tool) ?? 0) + i.size_bytes);
      const top = [...byTool].sort((a, b) => b[1] - a[1])[0];
      if (top && top[1] >= report.safe_bytes * 0.2) parts.push(s.sumTop(top[0], humanSize(top[1])));
    }
    if (report.redownload_bytes > 0) parts.push(s.sumRedownload(humanSize(report.redownload_bytes)));
    if (report.review_bytes > 0) parts.push(s.sumReview(humanSize(report.review_bytes)));
    return parts.join(lang === "zh" ? "，" : ", ") + (lang === "zh" ? "。" : ".");
  }, [report, s, lang]);

  async function saveSettings(next: Settings) {
    setSettings(next);
    try {
      await invoke("save_settings", { settings: next });
    } catch (e) {
      setError(String(e));
    }
  }

  async function runCompact() {
    if (!compacting) return;
    setError(null);
    for (const d of compacting.disks) {
      setCompacting((c) => (c ? { ...c, busy: d.path } : c));
      try {
        const r = await invoke<CompactResult>("compact_vhdx", { path: d.path });
        setCompacting((c) => (c ? { ...c, busy: undefined, results: [...c.results, r] } : c));
      } catch (e) {
        setError(String(e));
        setCompacting((c) => (c ? { ...c, busy: undefined } : c));
        return;
      }
    }
  }

  async function excludeItem(it: ScanItem) {
    const current = settings ?? (await invoke<Settings>("get_settings"));
    if (!current.excluded_paths.includes(it.path)) {
      await saveSettings({ ...current, excluded_paths: [...current.excluded_paths, it.path] });
    }
    setReport((prev) => (prev ? { ...prev, items: prev.items.filter((i) => !i.path.startsWith(it.path)) } : prev));
    setSelected((prev) => {
      const next = new Set(prev);
      next.delete(it.id);
      return next;
    });
    setNotice(s.excluded(it.path));
  }

  async function addExcludedFolder() {
    if (!settings) return;
    const dir = await openDialog({ directory: true, multiple: false, title: s.addFolder });
    if (typeof dir !== "string" || settings.excluded_paths.includes(dir)) return;
    await saveSettings({ ...settings, excluded_paths: [...settings.excluded_paths, dir] });
  }

  async function pickCustomRulesDir() {
    if (!settings) return;
    const dir = await openDialog({ directory: true, multiple: false, title: s.setCustomDir });
    if (typeof dir !== "string") return;
    await saveSettings({ ...settings, custom_rules_dir: dir });
    setLibrary(await invoke<RuleLibrary>("rules").catch(() => library));
  }

  async function toggleRule(id: string, enabled: boolean) {
    if (!settings) return;
    const disabled = enabled ? settings.disabled_rules.filter((r) => r !== id) : [...settings.disabled_rules, id];
    await saveSettings({ ...settings, disabled_rules: disabled });
    setLibrary((lib) => (lib ? { ...lib, rules: lib.rules.map((r) => (r.id === id ? { ...r, enabled } : r)) } : lib));
  }

  async function runRulesUpdate() {
    if (!settings) return;
    setUpdatingRules(true);
    setError(null);
    try {
      await invoke("save_settings", { settings });
      const r = await invoke<RulesUpdate>("update_rules");
      setNotice(s.updateDone(r.files, r.rules));
      setSettings({ ...settings, rules_updated_at: r.updated_at });
      setLibrary(await invoke<RuleLibrary>("rules"));
    } catch (e) {
      setError(String(e));
    }
    setUpdatingRules(false);
  }

  const [deepTasks, setDeepTasks] = useState<DeepTask[]>([]);
  const [deepSel, setDeepSel] = useState<Set<string>>(new Set());
  const [deepAnalysis, setDeepAnalysis] = useState<DeepAnalysis | null>(null);
  const [deepBusy, setDeepBusy] = useState<"analyze" | "run" | "hiber" | null>(null);
  const [deepConfirm, setDeepConfirm] = useState(false);
  const [deepResult, setDeepResult] = useState<DeepSummary | null>(null);

  async function openDeep() {
    setError(null);
    setView("deep");
    try {
      setDeepTasks(await invoke<DeepTask[]>("deep_tasks", { lang }));
    } catch (e) {
      setError(String(e));
    }
  }

  async function runDeepAnalyze() {
    setDeepBusy("analyze");
    setError(null);
    try {
      setDeepAnalysis(await invoke<DeepAnalysis>("deep_analyze"));
    } catch (e) {
      setError(String(e));
    }
    setDeepBusy(null);
  }

  async function runDeep() {
    setDeepConfirm(false);
    setDeepBusy("run");
    setError(null);
    try {
      const sum = await invoke<DeepSummary>("deep_run", { ids: [...deepSel] });
      setDeepResult(sum);
      const failed = sum.results.filter((r) => !r.ok);
      setNotice(s.deepDone(humanSize(sum.freed_bytes)) + (failed.length ? ` · ${s.deepFailed(failed.length)}` : ""));
      if (failed.length) setError(failed.map((r) => `${r.id}: ${r.message}`).join("\n"));
      setDeepSel(new Set());
      setDeepAnalysis(null);
      setDeepTasks(await invoke<DeepTask[]>("deep_tasks", { lang }));
    } catch (e) {
      setError(String(e));
    }
    setDeepBusy(null);
  }

  async function runHibernateOn() {
    setDeepBusy("hiber");
    setError(null);
    try {
      const r = await invoke<{ ok: boolean; message: string }>("hibernate_on");
      if (r.ok) setNotice(s.hiberOnDone);
      else setError(r.message);
      setDeepTasks(await invoke<DeepTask[]>("deep_tasks", { lang }));
    } catch (e) {
      setError(String(e));
    }
    setDeepBusy(null);
  }

  async function openOverview() {
    setError(null);
    try {
      const last = await invoke<Overview | null>("last_overview");
      if (last) {
        setOverview(last);
        setOvRoot(last.root);
      }
      if (!allDrives.length) setAllDrives(await invoke<Drive[]>("drives"));
    } catch (e) {
      setError(String(e));
    }
    setView("overview");
  }

  async function runOverview(root: string) {
    setError(null);
    setOvBusy(true);
    setOvRoot(root);
    setOvProgress({ seen: 0, bytes: 0, dir: "" });
    const unlisten = await listen<{ seen: number; bytes: number; dir: string }>("overview-progress", (e) => {
      setOvProgress(e.payload);
    });
    try {
      const ov = await invoke<Overview>("drive_overview", { root });
      setOverview(ov);
      setOvOpen(new Set());
    } catch (e) {
      setError(String(e));
    } finally {
      unlisten();
      setOvBusy(false);
      setOvProgress(null);
    }
  }

  function OverviewRows({ entries, base, depth }: { entries: OverviewEntry[]; base: number; depth: number }) {
    return (
      <>
        {entries.slice(0, depth === 0 ? 40 : 12).map((en) => {
          const open = ovOpen.has(en.path);
          const pct = base ? (en.bytes / base) * 100 : 0;
          return (
            <div key={en.path}>
              <div className="bar-row ov-row" style={{ paddingLeft: depth * 18 }} title={`${en.path} · ${s.ovFiles(en.files)}`}>
                <span className="bar-label">
                  {en.children.length > 0 ? (
                    <button
                      className="linkish"
                      onClick={() =>
                        setOvOpen((prev) => {
                          const next = new Set(prev);
                          if (next.has(en.path)) next.delete(en.path);
                          else next.add(en.path);
                          return next;
                        })
                      }
                    >
                      <span className={`chev ${open ? "open" : ""}`}>▸</span> {en.name}
                    </button>
                  ) : (
                    <span>{en.name}</span>
                  )}
                </span>
                <span className="bar"><span className="bar-fill" style={{ width: `${Math.max(0.5, pct)}%` }} /></span>
                <span className="bar-value">{humanSize(en.bytes)}</span>
                <button className="linkish muted" onClick={() => revealItemInDir(en.path)}>{s.openFolder}</button>
              </div>
              {open && <OverviewRows entries={en.children} base={base} depth={depth + 1} />}
            </div>
          );
        })}
      </>
    );
  }

  async function openBigFiles() {
    try {
      const d = await invoke<Drive[]>("drives");
      setBigDrives(d);
      setBigPick((prev) => (prev.size ? prev : new Set(d.map((x) => x.path))));
    } catch (e) {
      setError(String(e));
    }
    setView("bigfiles");
  }

  async function runBigScan() {
    if (bigPick.size === 0) {
      setError(s.bigPickDrive);
      return;
    }
    setError(null);
    setBigScanning(true);
    setBigProgress({ seen: 0, bytes: 0, dir: "" });
    const unlisten = await listen<{ seen: number; bytes: number; dir: string }>("bigfiles-progress", (e) => {
      setBigProgress(e.payload);
    });
    try {
      const items = await invoke<ScanItem[]>("scan_big_files", { roots: [...bigPick], minBytes: bigMin, lang });
      setBigItems(items);
      setBigSelected(new Set());
    } catch (e) {
      setError(String(e));
    } finally {
      unlisten();
      setBigScanning(false);
      setBigProgress(null);
    }
  }

  const bigSelectedItems = useMemo(() => (bigItems ?? []).filter((i) => bigSelected.has(i.id)), [bigItems, bigSelected]);

  async function runBigClean() {
    setBigConfirm(false);
    setPhase("cleaning");
    try {
      const summary = await invoke<CleanSummary>("clean_items", { items: bigSelectedItems });
      setResult(summary);
      const failed = new Set(summary.records.filter((r) => !r.ok).map((r) => r.path));
      setBigItems((prev) => (prev ? prev.filter((i) => !bigSelected.has(i.id) || failed.has(i.path)) : prev));
      setBigSelected(new Set());
    } catch (e) {
      setError(String(e));
    }
    setPhase("ready");
  }

  async function togglePeek(it: ScanItem) {
    if (peeks.has(it.id)) {
      setPeeks((m) => {
        const n = new Map(m);
        n.delete(it.id);
        return n;
      });
      return;
    }
    setPeeks((m) => new Map(m).set(it.id, "loading"));
    try {
      const p = await invoke<Preview>("preview_item", { path: it.path, limit: 8 });
      setPeeks((m) => (m.has(it.id) ? new Map(m).set(it.id, p) : m));
    } catch (e) {
      setError(String(e));
      setPeeks((m) => {
        const n = new Map(m);
        n.delete(it.id);
        return n;
      });
    }
  }

  async function openHistory() {
    setView("history");
    try {
      const log = await invoke<CleanRecord[]>("clean_log");
      setHistory(log);
      setRelocations(await invoke<RelocateRecord[]>("relocations"));
      if (!library) setLibrary(await invoke<RuleLibrary>("rules"));
      const paths = [...new Set(log.filter((r) => r.ok).map((r) => r.path))];
      const st = await invoke<RestoreStatus[]>("restore_status", { paths });
      setWhereabouts(new Map(st.map((x) => [x.path, x.state])));
    } catch (e) {
      setError(String(e));
    }
  }

  // Records of the most recent clean that are still in the Recycle Bin.
  const lastBatch = useMemo(() => {
    const ok = history.filter((r) => r.ok);
    if (ok.length === 0) return [];
    const last = ok[ok.length - 1];
    const same = last.batch
      ? ok.filter((r) => r.batch === last.batch)
      : ok.filter((r) => r.at.slice(0, 19) === last.at.slice(0, 19));
    return same.filter((r) => whereabouts.get(r.path) === "in_bin");
  }, [history, whereabouts]);

  async function restoreRecords(records: CleanRecord[]) {
    if (records.length === 0 || restoring) return;
    setRestoring(true);
    setError(null);
    try {
      const sum = await invoke<RestoreSummary>("restore_items", { records });
      const failed = sum.results.filter((r) => !r.ok).length;
      setNotice(s.restored(sum.restored, humanSize(sum.restored_bytes)) + (failed ? ` · ${s.restoreFailed(failed)}` : ""));
      if (failed) setError(sum.results.filter((r) => !r.ok).map((r) => `${r.path}: ${r.error}`).join("\n"));
      setResult(null);
      if (view === "history") await openHistory();
    } catch (e) {
      setError(String(e));
    }
    setRestoring(false);
  }

  // Freed bytes per owning tool and per month, for the history page.
  const stats = useMemo(() => {
    const ok = history.filter((r) => r.ok);
    const toolOf = (ruleId: string) =>
      ruleId.startsWith("bigfiles") ? s.bigFiles : (library?.rules.find((r) => r.id === ruleId)?.tool ?? ruleId.split(".")[0]);
    const byTool = new Map<string, number>();
    const byMonth = new Map<string, number>();
    for (const r of ok) {
      const tool = toolOf(r.rule_id);
      byTool.set(tool, (byTool.get(tool) ?? 0) + r.size_bytes);
      const m = r.at.slice(0, 7);
      byMonth.set(m, (byMonth.get(m) ?? 0) + r.size_bytes);
    }
    const tools = [...byTool].sort((a, b) => b[1] - a[1]).slice(0, 8);
    const months = [...byMonth].sort((a, b) => a[0].localeCompare(b[0])).slice(-12);
    return { tools, months, count: ok.length };
  }, [history, library, s]);

  async function moveBack(rec: RelocateRecord) {
    setMovingBack({ rec });
    setError(null);
    const unlisten = await listen<{ copied: number; total: number }>("relocate-progress", (e) => {
      setMovingBack((m) => (m ? { ...m, progress: [e.payload.copied, e.payload.total] } : m));
    });
    try {
      await invoke("move_back", { record: rec });
      setNotice(s.movedBack(rec.from));
      setRelocations((prev) => prev.filter((r) => r !== rec));
    } catch (e) {
      setError(String(e));
    } finally {
      unlisten();
      setMovingBack(null);
    }
  }

  useEffect(() => {
    localStorage.setItem("lang", lang);
    document.documentElement.lang = lang === "zh" ? "zh-CN" : "en";
    invoke("set_lang", { lang }).catch(() => {});
  }, [lang]);

  useEffect(() => {
    const c = listen<{ cleaned: number; freed_bytes: number }>("background-clean", (e) => {
      if (e.payload.cleaned > 0) {
        setNotice(t((localStorage.getItem("lang") as Lang) || "zh").bgCleaned(e.payload.cleaned, humanSize(e.payload.freed_bytes)));
      }
    });
    const p = listen<ScanReport>("background-scan", (e) => {
      setReport(e.payload);
      invoke<Growth[]>("growth_report").then(setGrowth).catch(() => {});
      setSelected(new Set(e.payload.items.filter(isDefaultTicked).map((i) => i.id)));
      setPhase("ready");
      setNotice(t((localStorage.getItem("lang") as Lang) || "zh").bgBanner(humanSize(e.payload.total_bytes)));
    });
    return () => {
      p.then((un) => un());
      c.then((un) => un());
    };
  }, []);

  async function runScan() {
    setPhase("scanning");
    setError(null);
    setResult(null);
    setScanStep(null);
    const unlisten = await listen<{ done: number; total: number; rule_name: string }>("scan-progress", (e) => {
      setScanStep(e.payload);
    });
    try {
      const r = await invoke<ScanReport>("scan", { lang });
      setReport(r);
      invoke<Growth[]>("growth_report").then(setGrowth).catch(() => {});
      // Default selection: only what is safe. The user opts into everything else.
      const safe = r.items.filter(isDefaultTicked).map((i) => i.id);
      setSelected(new Set(safe));
      setPhase("ready");
      if (autoConfirmRef.current && safe.length > 0) setConfirming(true);
      autoConfirmRef.current = false;
    } catch (e) {
      autoConfirmRef.current = false;
      setError(String(e));
      setPhase("idle");
    } finally {
      unlisten();
      setScanStep(null);
    }
  }

  const visibleItems = useMemo(() => {
    if (!report) return [];
    const q = query.trim().toLowerCase();
    if (!q) return report.items;
    return report.items.filter((i) => `${i.tool} ${i.name} ${i.path} ${i.detail ?? ""}`.toLowerCase().includes(q));
  }, [report, query]);

  const groups = useMemo(() => {
    const SAFETY_RANK = { safe: 0, redownload: 1, review: 2 };
    const cmp = (a: ScanItem, b: ScanItem) => {
      switch (sortBy) {
        case "recent":
          return (b.last_used ?? "").localeCompare(a.last_used ?? "") || b.size_bytes - a.size_bytes;
        case "safety":
          return SAFETY_RANK[a.safety] - SAFETY_RANK[b.safety] || b.size_bytes - a.size_bytes;
        case "name":
          return a.tool.localeCompare(b.tool) || a.name.localeCompare(b.name);
        default:
          return b.size_bytes - a.size_bytes;
      }
    };
    const by = new Map<string, ScanItem[]>();
    for (const it of visibleItems) {
      const key = groupBy === "tool" ? it.tool : it.category;
      if (!by.has(key)) by.set(key, []);
      by.get(key)!.push(it);
    }
    const keys =
      groupBy === "tool"
        ? [...by.keys()].sort((a, b) => by.get(b)!.reduce((x, i) => x + i.size_bytes, 0) - by.get(a)!.reduce((x, i) => x + i.size_bytes, 0))
        : CATEGORY_ORDER.filter((c) => by.has(c));
    return keys.map((k) => ({
      key: k,
      title: groupBy === "tool" ? k : s.category[k as Category],
      items: [...by.get(k)!].sort(cmp),
      bytes: by.get(k)!.reduce((a, i) => a + i.size_bytes, 0),
    }));
  }, [visibleItems, groupBy, sortBy, s]);

  const selectedItems = useMemo(
    () => (report ? report.items.filter((i) => selected.has(i.id)) : []),
    [report, selected],
  );
  const selectedBytes = selectedItems.reduce((a, i) => a + i.size_bytes, 0);
  const selectedReview = selectedItems.filter((i) => i.safety === "review").length;
  const selectedExportable = selectedItems.filter((i) => exportable.has(i.rule_id));

  function toggle(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function toggleGroup(items: ScanItem[]) {
    const all = items.every((i) => selected.has(i.id));
    setSelected((prev) => {
      const next = new Set(prev);
      for (const i of items) {
        if (all || i.in_use_by) next.delete(i.id);
        else next.add(i.id);
      }
      return next;
    });
  }

  async function pickExportDest() {
    const dest = await openDialog({ directory: true, multiple: false, title: s.exportPick });
    if (typeof dest === "string") setExportDest(dest);
  }

  async function runClean() {
    setConfirming(false);
    setPhase("cleaning");
    try {
      let exported: ExportResult | null = null;
      if (exportOn && exportDest && selectedExportable.length > 0) {
        setExporting([0, selectedExportable.reduce((a, i) => a + i.size_bytes, 0)]);
        const unlisten = await listen<{ copied: number; total: number }>("export-progress", (e) => {
          setExporting([e.payload.copied, e.payload.total]);
        });
        try {
          exported = await invoke<ExportResult>("export_items", { items: selectedExportable, destDir: exportDest });
        } finally {
          unlisten();
          setExporting(null);
        }
      }
      const summary = await invoke<CleanSummary>("clean_items", { items: selectedItems });
      setResult(summary);
      if (exported) setNotice(s.exported(exported.files, exported.zip_path));
      // Drop what was deleted; keep what failed so the user sees it.
      const failed = new Set(summary.records.filter((r) => !r.ok).map((r) => r.path));
      setReport((prev) =>
        prev
          ? {
              ...prev,
              items: prev.items.filter((i) => !selected.has(i.id) || failed.has(i.path)),
            }
          : prev,
      );
      setSelected(new Set());
    } catch (e) {
      setError(String(e));
    }
    setPhase("ready");
  }

  return (
    <div className="app">
      <NavigationRail<Page>
        fab={
          <Fab
            icon="radar"
            label={phase === "scanning" ? s.scanning : report ? s.rescan : s.scan}
            disabled={phase === "scanning" || phase === "cleaning"}
            onClick={() => {
              setView("scan");
              runScan();
            }}
          />
        }
        items={[
          { value: "clean", label: s.navClean, icon: "cleaning_services" },
          { value: "space", label: s.navSpace, icon: "donut_large" },
          { value: "history", label: s.navHistory, icon: "history" },
        ]}
        bottom={[{ value: "settings", label: s.settings, icon: "settings", badge: update !== null }]}
        value={page}
        onChange={(p) => {
          if (p === "clean") setView("scan");
          else if (p === "space") openOverview();
          else if (p === "history") openHistory();
          else openSettings();
        }}
      />
      <div className="app-main">
      <header className="topbar">
        <h1>{{ clean: s.navClean, space: s.navSpace, history: s.navHistory, settings: s.settings }[page]}</h1>
        <div className="topbar-actions">
          <button className="ghost" onClick={() => setLang(lang === "zh" ? "en" : "zh")}>
            {s.language}
          </button>
        </div>
      </header>
      {page === "clean" && (
        <div className="page-tabs">
          <Tabs
            label={s.navClean}
            value={view === "deep" ? "deep" : "scan"}
            onChange={(v) => (v === "deep" ? openDeep() : setView("scan"))}
            tabs={[
              { value: "scan", label: s.tabScan, icon: "radar" },
              { value: "deep", label: s.deep, icon: "admin_panel_settings" },
            ]}
          />
        </div>
      )}
      {page === "space" && (
        <div className="page-tabs">
          <Tabs
            label={s.navSpace}
            value={view === "bigfiles" ? "bigfiles" : "overview"}
            onChange={(v) => (v === "bigfiles" ? openBigFiles() : openOverview())}
            tabs={[
              { value: "overview", label: s.tabOverview, icon: "donut_large" },
              { value: "bigfiles", label: s.bigFiles, icon: "description" },
            ]}
          />
        </div>
      )}

      {error && <div className="banner error">{error}</div>}
      {notice && <div className="banner ok">{notice}</div>}
      {update && (
        <div className="banner update">
          <span>
            <strong>{s.updateAvailable(update.version)}</strong>
            {update.body && <span className="muted"> · {update.body.split("\n")[0].slice(0, 120)}</span>}
          </span>
          <span style={{ flex: 1 }} />
          {updateState === "downloading" && <span className="muted">{s.updateDownloading(`${updatePct}%`)}</span>}
          {updateState === "installed" ? (
            <>
              <span>{s.updateInstalled}</span>
              <button className="primary" onClick={() => relaunch()}>{s.updateRelaunch}</button>
            </>
          ) : (
            <>
              <button className="primary" disabled={updateState === "downloading"} onClick={installUpdate}>{s.updateInstall}</button>
              <button className="ghost" disabled={updateState === "downloading"} onClick={() => setUpdate(null)}>{s.updateLater}</button>
            </>
          )}
        </div>
      )}
      {updateMsg && <div className="banner ok">{updateMsg}</div>}
      {result && (
        <div className="banner ok">
          {s.done(humanSize(result.freed_bytes))}
          {result.records.some((r) => !r.ok) && (
            <span> · {s.failed(result.records.filter((r) => !r.ok).length)}</span>
          )}
          {result.records.some((r) => r.ok) && (
            <button className="ghost linkish" disabled={restoring} onClick={() => restoreRecords(result.records.filter((r) => r.ok))}>
              {restoring ? s.restoring : s.undo}
            </button>
          )}
        </div>
      )}

      {view === "deep" && (
        <section className="bigfiles deep">
          <h2>{s.deepTitle}</h2>
          <p className="muted">{s.deepBody}</p>
          <div className="big-controls">
            <button className="ghost" disabled={deepBusy !== null} onClick={runDeepAnalyze}>
              {deepBusy === "analyze" ? s.deepAnalyzing : s.deepAnalyze}
            </button>
            <button className="primary" disabled={deepBusy !== null || deepSel.size === 0} onClick={() => setDeepConfirm(true)}>
              {deepBusy === "run" ? s.deepRunning : s.deepRun(deepSel.size)}
            </button>
          </div>
          <ul className="deep-list">
            {deepTasks.map((t) => {
              const est =
                t.id === "winsxs" && deepAnalysis?.winsxs_reclaimable_bytes != null
                  ? s.deepWinsxs(humanSize(deepAnalysis.winsxs_actual_bytes ?? 0), humanSize(deepAnalysis.winsxs_reclaimable_bytes), !!deepAnalysis.winsxs_cleanup_recommended)
                  : t.id === "restore_points" && deepAnalysis?.shadow_used_bytes != null
                    ? s.deepShadow(deepAnalysis.shadow_count ?? 0, humanSize(deepAnalysis.shadow_used_bytes))
                    : t.estimate_bytes != null
                      ? s.deepEstimate(humanSize(t.estimate_bytes))
                      : t.needs_analysis
                        ? s.deepUnknown
                        : "";
              const hiberOff = t.id === "hibernation" && !t.applicable;
              return (
                <li key={t.id} className={`deep-item${t.applicable ? "" : " muted"}`}>
                  <label>
                    <input
                      type="checkbox"
                      disabled={!t.applicable || deepBusy !== null}
                      checked={deepSel.has(t.id)}
                      onChange={(e) =>
                        setDeepSel((prev) => {
                          const n = new Set(prev);
                          if (e.target.checked) n.add(t.id);
                          else n.delete(t.id);
                          return n;
                        })
                      }
                    />
                    <strong>{t.title}</strong>
                    <span className="size">{hiberOff ? "" : est || (t.applicable ? "" : s.deepNotApplicable)}</span>
                  </label>
                  <p>{hiberOff ? s.hiberOff : t.body}</p>
                  {!hiberOff && <p className="warn">{t.consequence}</p>}
                  {hiberOff && (
                    <button className="ghost" disabled={deepBusy !== null} onClick={runHibernateOn}>{s.hiberOn}</button>
                  )}
                  {deepResult?.results.find((r) => r.id === t.id && !r.ok) && (
                    <p className="warn">{deepResult.results.find((r) => r.id === t.id)?.message}</p>
                  )}
                </li>
              );
            })}
          </ul>
        </section>
      )}

      {deepConfirm && (
        <div className="modal-backdrop" onClick={() => setDeepConfirm(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{s.deepConfirmTitle}</h3>
            <p>{s.deepConfirmBody}</p>
            <ul>
              {deepTasks.filter((t) => deepSel.has(t.id)).map((t) => (
                <li key={t.id}>
                  <strong>{t.title}</strong>
                  <span className="warn"> · {t.consequence}</span>
                </li>
              ))}
            </ul>
            <div className="modal-actions">
              <button className="ghost" onClick={() => setDeepConfirm(false)}>{s.cancel}</button>
              <button className="primary" onClick={runDeep}>{s.deepRun(deepSel.size)}</button>
            </div>
          </div>
        </div>
      )}

      {view === "overview" && (
        <section className="bigfiles overview">
          <h2>{s.ovTitle}</h2>
          <p className="muted">{s.ovBody}</p>
          <div className="big-controls">
            <span className="muted">{s.ovDrive}</span>
            {allDrives.map((d) => (
              <label key={d.path}>
                <input type="radio" name="ov-drive" checked={(ovRoot ?? allDrives.find((x) => x.system)?.path) === d.path} disabled={ovBusy} onChange={() => setOvRoot(d.path)} />
                {d.path}{d.name ? ` ${d.name}` : ""}
              </label>
            ))}
            {ovBusy ? (
              <button className="ghost" onClick={() => invoke("cancel_overview")}>{s.ovCancel}</button>
            ) : (
              <button className="primary" onClick={() => runOverview(ovRoot ?? allDrives.find((x) => x.system)?.path ?? allDrives[0]?.path ?? "C:\\")}>
                {overview && overview.root === ovRoot ? s.ovRerun : s.ovRun}
              </button>
            )}
          </div>
          {ovBusy && ovProgress && (
            <p className="muted">{s.ovRunning} {ovProgress.seen.toLocaleString()} · {humanSize(ovProgress.bytes)} · {ovProgress.dir}</p>
          )}
          {overview && !ovBusy && (
            <>
              <p>
                <strong>{overview.root}</strong>{" "}
                {s.ovSummary(humanSize(overview.total_bytes), humanSize(overview.total_bytes - overview.free_bytes), humanSize(overview.free_bytes))}
                <span className="muted"> · {s.ovComputedAt(new Date(overview.computed_at).toLocaleString())}</span>
              </p>
              {overview.cancelled && <div className="banner">{s.ovPartial}</div>}
              <div className="ov-stack" title={s.ovSummary(humanSize(overview.total_bytes), humanSize(overview.total_bytes - overview.free_bytes), humanSize(overview.free_bytes))}>
                {overview.entries.slice(0, 8).map((en, i) => (
                  <span key={en.path} className={`ov-seg c${i % 8}`} style={{ width: `${(en.bytes / Math.max(1, overview.total_bytes)) * 100}%` }} title={`${en.name} ${humanSize(en.bytes)}`} />
                ))}
                <span className="ov-seg free" style={{ width: `${(overview.free_bytes / Math.max(1, overview.total_bytes)) * 100}%` }} />
              </div>
              <p className="muted">
                {report
                  ? <>{s.ovCleanable(humanSize(report.total_bytes))} <span title={s.ovCleanableHint}>ⓘ</span></>
                  : s.ovNoScan}
                {overview.total_bytes - overview.free_bytes > overview.scanned_bytes + 1024 * 1024 * 1024 && (
                  <> · {s.ovUnread(humanSize(overview.total_bytes - overview.free_bytes - overview.scanned_bytes))}</>
                )}
              </p>
              <div className="stat-panel">
                <OverviewRows entries={overview.entries} base={Math.max(1, overview.total_bytes - overview.free_bytes)} depth={0} />
              </div>
            </>
          )}
        </section>
      )}

      {view === "bigfiles" && (
        <section className="bigfiles">
          <h2>{s.bigTitle}</h2>
          <p className="muted">{s.bigBody}</p>
          <div className="big-controls">
            <div>
              <span className="muted">{s.bigDrives}</span>
              {bigDrives.map((d) => (
                <label key={d.path} className="drive">
                  <input
                    type="checkbox"
                    checked={bigPick.has(d.path)}
                    disabled={bigScanning}
                    onChange={(e) => {
                      const next = new Set(bigPick);
                      if (e.target.checked) next.add(d.path);
                      else next.delete(d.path);
                      setBigPick(next);
                    }}
                  />{" "}
                  <b>{d.path}</b> {d.name && <span className="muted">{d.name}</span>}{" "}
                  <span className="muted">
                    {humanSize(d.total_bytes - d.free_bytes)} / {humanSize(d.total_bytes)}
                  </span>
                </label>
              ))}
            </div>
            <div>
              {s.bigMin}{" "}
              <select value={bigMin} disabled={bigScanning} onChange={(e) => setBigMin(Number(e.target.value))}>
                <option value={100 * 1024 * 1024}>100 MB</option>
                <option value={500 * 1024 * 1024}>500 MB</option>
                <option value={1024 * 1024 * 1024}>1 GB</option>
                <option value={5 * 1024 * 1024 * 1024}>5 GB</option>
              </select>{" "}
              {bigScanning ? (
                <button className="ghost" onClick={() => invoke("cancel_big_scan")}>{s.bigCancel}</button>
              ) : (
                <button className="primary" onClick={runBigScan}>{s.bigScan}</button>
              )}
            </div>
          </div>
          {bigScanning && bigProgress && (
            <p className="muted big-progress">{s.bigScanning(bigProgress.seen, bigProgress.dir)}</p>
          )}
          {!bigScanning && bigItems && (
            <>
              <p className="stat-label">
                {bigItems.length === 0 ? s.bigNone : s.bigFound(bigItems.length, humanSize(bigItems.reduce((a, i) => a + i.size_bytes, 0)))}
              </p>
              <main className="groups">
                {[...new Set(bigItems.map((i) => i.tool))].map((kind) => {
                  const items = bigItems.filter((i) => i.tool === kind);
                  const allOn = items.every((i) => bigSelected.has(i.id));
                  return (
                    <section key={kind} className="group">
                      <header className="group-head">
                        <input
                          type="checkbox"
                          checked={allOn}
                          onChange={() => {
                            const next = new Set(bigSelected);
                            for (const i of items) {
                              if (allOn) next.delete(i.id);
                              else next.add(i.id);
                            }
                            setBigSelected(next);
                          }}
                        />
                        <span className="group-title">{kind} · {items.length}</span>
                        <span className="group-size">{humanSize(items.reduce((a, i) => a + i.size_bytes, 0))}</span>
                      </header>
                      <ul className="items">
                        {items.map((it) => (
                          <li key={it.id} className={`item ${bigSelected.has(it.id) ? "on" : ""}`}>
                            <input
                              type="checkbox"
                              checked={bigSelected.has(it.id)}
                              disabled={it.rule_id === "bigfiles.hiberfil"}
                              onChange={() => {
                                const next = new Set(bigSelected);
                                if (next.has(it.id)) next.delete(it.id);
                                else next.add(it.id);
                                setBigSelected(next);
                              }}
                            />
                            <div className="item-main">
                              <div className="item-line">
                                <span className="name">{it.name}</span>
                              </div>
                              {it.rule_id !== "bigfiles" && (
                                <div className="item-sub">
                                  <span className="consequence warn">{it.consequence}</span>
                                  {it.rule_id === "bigfiles.vdisk" && it.path.toLowerCase().endsWith(".vhdx") && (
                                    <button className="link" onClick={() => openCompact(it)}>{s.compactTitle}</button>
                                  )}
                                </div>
                              )}
                              <div className="item-path" title={it.path}>
                                {it.detail}
                                <button className="link" onClick={() => revealItemInDir(it.path)}>
                                  {s.openFolder}
                                </button>
                              </div>
                            </div>
                            <div className="item-meta">
                              <div className="size">{humanSize(it.size_bytes)}</div>
                              <div className="muted">
                                {s.lastUsed} {relativeDate(it.last_used, lang)}
                              </div>
                            </div>
                          </li>
                        ))}
                      </ul>
                    </section>
                  );
                })}
              </main>
              {bigSelectedItems.length > 0 && (
                <footer className="actionbar">
                  <span>{s.selected(bigSelectedItems.length, humanSize(bigSelectedItems.reduce((a, i) => a + i.size_bytes, 0)))}</span>
                  <button className="primary" disabled={phase === "cleaning"} onClick={() => setBigConfirm(true)}>
                    {phase === "cleaning" ? s.cleaning : s.clean}
                  </button>
                </footer>
              )}
            </>
          )}
        </section>
      )}

      {bigConfirm && (
        <div className="modal-backdrop" onClick={() => setBigConfirm(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{s.confirmTitle}</h3>
            <p>{s.confirmBody(bigSelectedItems.length, humanSize(bigSelectedItems.reduce((a, i) => a + i.size_bytes, 0)))}</p>
            <div className="modal-actions">
              <button className="ghost" onClick={() => setBigConfirm(false)}>{s.cancel}</button>
              <button className="primary danger" onClick={runBigClean}>{s.confirm}</button>
            </div>
          </div>
        </div>
      )}

      {view === "settings" && settings && (
        <section className="settings">
          <h2>{s.setExcluded}</h2>
          <p className="muted">{s.setExcludedBody}</p>
          {settings.excluded_paths.length === 0 ? (
            <p className="empty">{s.setExcludedEmpty}</p>
          ) : (
            <ul className="plain">
              {settings.excluded_paths.map((p) => (
                <li key={p}>
                  <span title={p}>{p}</span>
                  <button className="link" onClick={() => saveSettings({ ...settings, excluded_paths: settings.excluded_paths.filter((x) => x !== p) })}>
                    {s.remove}
                  </button>
                </li>
              ))}
            </ul>
          )}
          <p>
            <button className="ghost" onClick={addExcludedFolder}>{s.addFolder}</button>
          </p>
          <h2>{s.scanDrives}</h2>
          <p className="muted">{s.scanDrivesHint}</p>
          <DrivePicker drives={allDrives} settings={settings} s={s} onChange={saveSettings} align="start" />
          {allDrives.length < 2 && <p className="empty">{s.scanDrivesOnlyOne}</p>}
          <p>
            {s.setMinSize}{" "}
            <select
              value={settings.min_item_bytes}
              onChange={(e) => saveSettings({ ...settings, min_item_bytes: Number(e.target.value) })}
            >
              <option value={0}>0</option>
              <option value={10 * 1024 * 1024}>10 MB</option>
              <option value={100 * 1024 * 1024}>100 MB</option>
              <option value={1024 * 1024 * 1024}>1 GB</option>
            </select>
          </p>

          <h2>{s.setBackground}</h2>
          <p>
            <label>
              <input type="checkbox" checked={settings.close_to_tray} onChange={(e) => saveSettings({ ...settings, close_to_tray: e.target.checked })} />{" "}
              {s.setCloseToTray}
            </label>
          </p>
          <p>
            <label>
              <input type="checkbox" checked={settings.autostart} onChange={(e) => saveSettings({ ...settings, autostart: e.target.checked })} />{" "}
              {s.setAutostart}
            </label>
          </p>
          <p>
            {s.setSchedule}{" "}
            <select
              value={settings.scheduled_scan_hours}
              onChange={(e) => saveSettings({ ...settings, scheduled_scan_hours: Number(e.target.value) })}
            >
              <option value={0}>{s.schedOff}</option>
              {[6, 12, 24, 72, 168].map((h) => (
                <option key={h} value={h}>{s.schedHours(h)}</option>
              ))}
            </select>{" "}
            {s.setNotify}{" "}
            <select
              value={settings.notify_min_bytes}
              onChange={(e) => saveSettings({ ...settings, notify_min_bytes: Number(e.target.value) })}
            >
              <option value={0}>0</option>
              <option value={500 * 1024 * 1024}>500 MB</option>
              <option value={1024 * 1024 * 1024}>1 GB</option>
              <option value={5 * 1024 * 1024 * 1024}>5 GB</option>
            </select>
          </p>
          <p>
            <label>
              <input type="checkbox" checked={settings.auto_clean_safe} onChange={(e) => saveSettings({ ...settings, auto_clean_safe: e.target.checked })} />{" "}
              {s.setAutoClean}
            </label>
          </p>
          <p>
            {s.setIdleDays}{" "}
            <select value={settings.idle_model_days} onChange={(e) => saveSettings({ ...settings, idle_model_days: Number(e.target.value) })}>
              {[30, 60, 90, 180, 365].map((d) => (
                <option key={d} value={d}>{s.days(d)}</option>
              ))}
            </select>
          </p>
          <p>
            {s.setGrowthNotify}{" "}
            <select
              value={settings.growth_notify_bytes}
              onChange={(e) => saveSettings({ ...settings, growth_notify_bytes: Number(e.target.value) })}
            >
              <option value={0}>{s.growthOff}</option>
              <option value={1024 * 1024 * 1024}>1 GB</option>
              <option value={2 * 1024 * 1024 * 1024}>2 GB</option>
              <option value={5 * 1024 * 1024 * 1024}>5 GB</option>
              <option value={10 * 1024 * 1024 * 1024}>10 GB</option>
            </select>
          </p>
          {settings.last_background_scan && (
            <p className="muted">{s.lastBackground(new Date(settings.last_background_scan).toLocaleString())}</p>
          )}

          <h2>{s.setAppUpdate}</h2>
          {portable ? (
            <p className="muted">
              {s.portableNoUpdate}{" "}
              <button className="linkish" onClick={() => openUrl("https://github.com/nethero23/ai-disk-cleaner/releases")}>{s.portableReleases}</button>
            </p>
          ) : (
            <>
              <p>
                <label>
                  <input type="checkbox" checked={settings.auto_update_check} onChange={(e) => saveSettings({ ...settings, auto_update_check: e.target.checked })} />{" "}
                  {s.setAutoUpdateCheck}
                </label>
              </p>
              <p>
                <button className="ghost" disabled={updateState === "checking"} onClick={() => runCheckUpdate(true)}>
                  {updateState === "checking" ? s.checkingUpdate : s.checkUpdate}
                </button>
              </p>
            </>
          )}

          <h2>{s.setUpdate}</h2>
          <p className="muted">{s.setUpdateBody}</p>
          <p>
            <input
              className="wide"
              value={settings.rules_update_url}
              onChange={(e) => setSettings({ ...settings, rules_update_url: e.target.value })}
              onBlur={() => saveSettings(settings)}
            />
          </p>
          <p>
            <button className="ghost" disabled={updatingRules} onClick={runRulesUpdate}>
              {updatingRules ? s.updating : s.updateNow}
            </button>{" "}
            <span className="muted">
              {settings.rules_updated_at ? s.updatedAt(new Date(settings.rules_updated_at).toLocaleString()) : s.neverUpdated}
            </span>
          </p>

          <h2>{s.setCustomDir}</h2>
          <p className="muted">{s.setCustomDirBody}</p>
          <p>
            <span className="muted">{settings.custom_rules_dir ?? "—"}</span>{" "}
            <button className="link" onClick={pickCustomRulesDir}>{s.chooseFolder}</button>
            {settings.custom_rules_dir && (
              <button
                className="link"
                onClick={async () => {
                  await saveSettings({ ...settings, custom_rules_dir: null });
                  setLibrary(await invoke<RuleLibrary>("rules").catch(() => library));
                }}
              >
                {s.clearFolder}
              </button>
            )}
          </p>
          {library && library.errors.length > 0 && (
            <div className="banner error">
              {s.setRuleErrors}
              <ul className="plain">
                {library.errors.map(([p, e]) => (
                  <li key={p}>
                    {p}: {e}
                  </li>
                ))}
              </ul>
            </div>
          )}

          <h2>{s.setRules}</h2>
          <p className="muted">{s.setRulesBody}</p>
          {library && (
            <ul className="plain rules">
              {library.rules.map((r) => (
                <li key={r.id}>
                  <label>
                    <input type="checkbox" checked={r.enabled} onChange={(e) => toggleRule(r.id, e.target.checked)} />{" "}
                    <span className="tool">{r.tool}</span> {lang === "zh" ? r.name_zh : r.name_en}{" "}
                    <span className={`tag ${r.safety}`}>{s.safety[r.safety]}</span>
                    <span className="muted"> · {r.id}</span>
                  </label>
                </li>
              ))}
            </ul>
          )}
        </section>
      )}

      {view === "history" && (
        <section className="history">
          {relocations.length > 0 && (
            <div className="moved">
              <h3>{s.movedTitle}</h3>
              <p className="muted">{s.movedBody}</p>
              <table className="log">
                <tbody>
                  {relocations.map((r) => (
                    <tr key={r.from + r.to}>
                      <td className="muted">{new Date(r.at).toLocaleString()}</td>
                      <td title={r.from}>{r.from}</td>
                      <td className="muted" title={r.to}>→ {r.to}</td>
                      <td className="size">{humanSize(r.bytes)}</td>
                      <td>
                        {movingBack?.rec === r ? (
                          <span className="muted">
                            {s.movingBack}
                            {movingBack.progress && movingBack.progress[1] > 0 && ` ${Math.round((movingBack.progress[0] / movingBack.progress[1]) * 100)}%`}
                          </span>
                        ) : (
                          <button className="ghost" disabled={movingBack !== null} onClick={() => moveBack(r)}>
                            {s.moveBack}
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          {history.length === 0 ? (
            <p className="empty">{s.historyEmpty}</p>
          ) : (
            <>
              <p className="muted">
                {s.historyTotal(
                  history.filter((r) => r.ok).length,
                  humanSize(history.filter((r) => r.ok).reduce((a, r) => a + r.size_bytes, 0)),
                )}
                {lastBatch.length > 0 && (
                  <button
                    className="ghost"
                    disabled={restoring}
                    title={s.undoLastHint(lastBatch.length, humanSize(lastBatch.reduce((a, r) => a + r.size_bytes, 0)))}
                    onClick={() => restoreRecords(lastBatch)}
                  >
                    {restoring ? s.restoring : s.undoLast}
                  </button>
                )}
              </p>
              <div className="stats">
                <div className="stat-panel">
                  <h3>{s.statByTool}</h3>
                  {stats.tools.map(([tool, bytes]) => (
                    <div key={tool} className="bar-row">
                      <span className="bar-label" title={tool}>{tool}</span>
                      <span className="bar"><span className="bar-fill" style={{ width: `${Math.max(2, (bytes / stats.tools[0][1]) * 100)}%` }} /></span>
                      <span className="bar-value">{humanSize(bytes)}</span>
                    </div>
                  ))}
                </div>
                <div className="stat-panel">
                  <h3>{s.statByMonth}</h3>
                  {stats.months.map(([month, bytes]) => (
                    <div key={month} className="bar-row">
                      <span className="bar-label">{month}</span>
                      <span className="bar"><span className="bar-fill" style={{ width: `${Math.max(2, (bytes / Math.max(...stats.months.map((m) => m[1]))) * 100)}%` }} /></span>
                      <span className="bar-value">{humanSize(bytes)}</span>
                    </div>
                  ))}
                </div>
              </div>
              <table className="log">
                <tbody>
                  {[...history].reverse().map((r, i) => (
                    <tr key={i} className={r.ok ? "" : "failed"}>
                      <td className="muted">{new Date(r.at).toLocaleString()}</td>
                      <td>{r.name}</td>
                      <td className="muted" title={r.path}>{r.path}</td>
                      <td className="size">{humanSize(r.size_bytes)}</td>
                      <td className="muted">
                        {!r.ok ? r.error : whereabouts.get(r.path) === "in_bin" ? (
                          <button className="ghost" disabled={restoring} onClick={() => restoreRecords([r])}>{s.restore}</button>
                        ) : whereabouts.get(r.path) === "present" ? s.statePresent : whereabouts.has(r.path) ? s.stateGone : ""}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </section>
      )}

      {view === "scan" && !report && phase !== "scanning" && (
        <section className="empty">
          <h2>{s.emptyTitle}</h2>
          <p>{s.emptyBody}</p>
          {settings && <DrivePicker drives={allDrives} settings={settings} s={s} onChange={saveSettings} />}
          <p>
            <button
              className="ghost"
              onClick={() => {
                autoConfirmRef.current = true;
                runScan();
              }}
            >
              {s.scanAndCleanSafe}
            </button>
          </p>
        </section>
      )}
      {view === "scan" && phase === "scanning" && (
        <section className="empty">
          <div className="spinner" />
          <p>{s.scanning}</p>
          {scanStep && (
            <>
              <progress className="scan-bar" value={scanStep.done} max={scanStep.total} />
              <p className="muted">{s.scanStep(scanStep.done, scanStep.total, scanStep.rule_name)}</p>
            </>
          )}
          <p>
            <button className="ghost" onClick={() => invoke("cancel_scan")}>{s.scanCancel}</button>
          </p>
        </section>
      )}

      {view === "scan" && report && phase !== "scanning" && (
        <>
          <section className="summary">
            <div className="stat">
              <span className="stat-label">{s.found(report.items.length, humanSize(report.total_bytes))}</span>
              {report.started_at && report.finished_at && (
                <span className="muted scan-took" title={report.timings ? `${s.slowest} ${report.timings.slice(0, 5).map(([id, ms]) => `${id} ${(ms / 1000).toFixed(1)}s`).join(", ")}` : undefined}>
                  {" "}{s.scanTook(((new Date(report.finished_at).getTime() - new Date(report.started_at).getTime()) / 1000).toFixed(1))}
                </span>
              )}
            </div>
            {summaryLine && <p className="summary-line">{summaryLine}</p>}
            {settings && <DrivePicker drives={allDrives} settings={settings} s={s} onChange={saveSettings} note={s.scanDrivesNext} align="start" />}
            {report.cancelled && <div className="banner">{s.scanPartial}</div>}
            {growth.some((g) => Math.abs(g.delta_bytes) >= 100 * 1024 * 1024) && (
              <div className="growth" title={s.growthHint}>
                <span className="muted">{s.growthTitle}</span>
                {growth
                  .filter((g) => Math.abs(g.delta_bytes) >= 100 * 1024 * 1024)
                  .slice(0, 5)
                  .map((g) => (
                    <span key={g.tool} className={`growth-chip ${g.delta_bytes > 0 ? "up" : "down"}`}>
                      {g.delta_bytes > 0
                        ? s.growthRow(g.tool, humanSize(g.delta_bytes), new Date(g.since).toLocaleDateString())
                        : s.growthShrank(g.tool, humanSize(-g.delta_bytes))}
                    </span>
                  ))}
              </div>
            )}
            <div className="stat-row">
              <Pill level="safe" label={s.safety.safe} size={humanSize(report.safe_bytes)} hint={s.safetyHint.safe} />
              <Pill level="redownload" label={s.safety.redownload} size={humanSize(report.redownload_bytes)} hint={s.safetyHint.redownload} />
              <Pill level="review" label={s.safety.review} size={humanSize(report.review_bytes)} hint={s.safetyHint.review} />
            </div>
            <div className="quick">
              <button className="ghost" onClick={() => setSelected(new Set(report.items.filter(isDefaultTicked).map((i) => i.id)))}>
                {s.selectAllSafe}
              </button>
              {report.items.some(isDefaultTicked) && (
                <button
                  className="primary"
                  disabled={phase === "cleaning"}
                  onClick={() => {
                    setSelected(new Set(report.items.filter(isDefaultTicked).map((i) => i.id)));
                    setConfirming(true);
                  }}
                >
                  {s.cleanSafeNow}
                </button>
              )}
              <button className="ghost" onClick={() => setSelected(new Set())}>
                {s.selectNone}
              </button>
              <button className="ghost" disabled={dupsBusy} onClick={runFindDups}>
                {dupsBusy ? s.findingDups : s.findDups}
              </button>
            </div>
            {dups && (
              <div className="dups">
                {dups.length === 0 ? (
                  <p className="muted">{s.dupsNone}</p>
                ) : (
                  <>
                    <p className="stat-label">
                      {s.dupsTitle(dups.length, humanSize(dups.reduce((a, g) => a + g.reclaimable_bytes, 0)))}
                    </p>
                    <p className="muted">{s.dupsHow}</p>
                    {dups.map((g, i) => (
                      <div key={i} className="dup-group">
                        <div className="dup-head">
                          <span className="size">{humanSize(g.size_bytes)}</span>
                          <span className="muted"> × {g.files.length}</span>
                          <span style={{ flex: 1 }} />
                          {g.same_volume ? (
                            <button className="ghost" onClick={() => runMerge(g)}>{s.merge}</button>
                          ) : (
                            <span className="muted">{s.mergeDiffVolume}</span>
                          )}
                        </div>
                        <ul className="dup-files">
                          {g.files.map((f) => (
                            <li key={f.path} title={f.path}>
                              <span className="tool">{f.tool}</span> {f.path}
                            </li>
                          ))}
                        </ul>
                      </div>
                    ))}
                  </>
                )}
              </div>
            )}
            <UnrecognizedPanel lang={lang} onError={setError} />
            {idleModels.length > 0 && (
              <div className="idle" title={s.idleHint}>
                <div className="idle-head">
                  <strong>{s.idleTitle(idleModels.length, humanSize(idleModels.reduce((a, i) => a + i.size_bytes, 0)))}</strong>
                  <button className="link" onClick={() => setSelected((prev) => new Set([...prev, ...idleModels.map((i) => i.id)]))}>{s.idleTickAll}</button>
                </div>
                <p className="muted">{s.idleHint}</p>
                <table className="log">
                  <tbody>
                    {idleModels.slice(0, 10).map((it) => (
                      <tr key={it.id}>
                        <td>{it.name}</td>
                        <td className="muted">{it.tool}</td>
                        <td className="size">{humanSize(it.size_bytes)}</td>
                        <td className="muted">{it.last_used && s.idleSince(sinceDuration(it.last_used, lang))}</td>
                        <td>
                          <button
                            className="link"
                            onClick={() =>
                              setSelected((prev) => {
                                const n = new Set(prev);
                                if (n.has(it.id)) n.delete(it.id);
                                else n.add(it.id);
                                return n;
                              })
                            }
                          >
                            {selected.has(it.id) ? s.idleUntick : s.idleTick}
                          </button>
                          {relocatable.has(it.rule_id) && !it.in_use_by && (
                            <button className="link" onClick={() => pickMoveTarget(it)}>{s.move}</button>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {idleModels.length > 10 && <p className="muted">{s.idleMore(idleModels.length - 10)}</p>}
              </div>
            )}
            {report.unreadable.length > 0 && (
              <p className="muted" title={report.unreadable.join("\n")}>
                {s.unreadable(report.unreadable.length)}
              </p>
            )}
          </section>

          {report.items.length > 0 && (
            <div className="toolbar">
              <input
                type="search"
                className="search"
                placeholder={s.searchPlaceholder}
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
              <label>
                {s.groupBy}
                <select value={groupBy} onChange={(e) => { const v = e.target.value as "category" | "tool"; setGroupBy(v); localStorage.setItem("groupBy", v); setCollapsed(new Set()); }}>
                  <option value="category">{s.groupByCategory}</option>
                  <option value="tool">{s.groupByTool}</option>
                </select>
              </label>
              <label>
                {s.sortBy}
                <select value={sortBy} onChange={(e) => { const v = e.target.value as "size" | "recent" | "safety" | "name"; setSortBy(v); localStorage.setItem("sortBy", v); }}>
                  <option value="size">{s.sortSize}</option>
                  <option value="recent">{s.sortRecent}</option>
                  <option value="safety">{s.sortSafety}</option>
                  <option value="name">{s.sortName}</option>
                </select>
              </label>
              {query && (
                <span className="muted">{s.matched(visibleItems.length, humanSize(visibleItems.reduce((a, i) => a + i.size_bytes, 0)))}</span>
              )}
            </div>
          )}

          {report.items.length === 0 && <p className="empty">{s.nothing}</p>}
          {report.items.length > 0 && visibleItems.length === 0 && <p className="empty">{s.noMatch}</p>}

          <main className="groups">
            {groups.map((g) => {
              const open = !collapsed.has(g.key);
              const allOn = g.items.every((i) => selected.has(i.id));
              const someOn = g.items.some((i) => selected.has(i.id));
              return (
                <section key={g.key} className="group">
                  <header className="group-head">
                    <input
                      type="checkbox"
                      checked={allOn}
                      ref={(el) => el && (el.indeterminate = !allOn && someOn)}
                      onChange={() => toggleGroup(g.items)}
                    />
                    <button
                      className="group-title"
                      onClick={() =>
                        setCollapsed((prev) => {
                          const next = new Set(prev);
                          if (next.has(g.key)) next.delete(g.key);
                          else next.add(g.key);
                          return next;
                        })
                      }
                    >
                      <span className={`chev ${open ? "open" : ""}`}>▸</span>
                      {g.title}
                      <span className="count">{g.items.length}</span>
                    </button>
                    <span className="group-size">{humanSize(g.bytes)}</span>
                  </header>
                  {open && (
                    <ul className="items">
                      {g.items.map((it) => (
                        <li key={it.id} className={`item ${selected.has(it.id) ? "on" : ""} ${it.in_use_by ? "inuse" : ""}`}>
                          <input
                            type="checkbox"
                            checked={selected.has(it.id)}
                            disabled={!!it.in_use_by}
                            title={it.in_use_by ? s.inUse(it.in_use_by) : undefined}
                            onChange={() => toggle(it.id)}
                          />
                          <div className="item-main">
                            <div className="item-line">
                              <span className="tool">{it.tool}</span>
                              <span className="name">{it.name}</span>
                              <span className={`tag ${it.safety}`} title={s.safetyHint[it.safety]}>
                                {s.safety[it.safety]}
                              </span>
                              {it.in_use_by && (
                                <span className="tag inuse" title={s.inUse(it.in_use_by)}>
                                  ● {it.in_use_by}
                                </span>
                              )}
                            </div>
                            <div className="item-sub">
                              <span className="consequence">{it.consequence}</span>
                              {it.detail && <span className="detail"> · {it.detail}</span>}
                            </div>
                            <div className="item-path" title={it.path}>
                              {it.path}
                              <button className="link" onClick={() => revealItemInDir(it.path)}>
                                {s.openFolder}
                              </button>
                              <button className="link" onClick={() => togglePeek(it)}>
                                {peeks.has(it.id) ? s.peekHide : s.peek}
                              </button>
                              <button className="link" onClick={() => excludeItem(it)}>
                                {s.exclude}
                              </button>
                              {relocatable.has(it.rule_id) && !it.in_use_by && (
                                <button className="link" onClick={() => pickMoveTarget(it)}>
                                  {s.move}
                                </button>
                              )}
                              {compactable.has(it.rule_id) && (
                                <button className="link" onClick={() => openCompact(it)}>
                                  {s.compact}
                                </button>
                              )}
                            </div>
                            {peeks.has(it.id) && (
                              <div className="peek">
                                {peeks.get(it.id) === "loading" ? (
                                  <span className="muted">{s.peekLoading}</span>
                                ) : (
                                  (() => {
                                    const p = peeks.get(it.id) as Preview;
                                    return p.files === 0 ? (
                                      <span className="muted">{s.peekEmpty}</span>
                                    ) : (
                                      <>
                                        <div className="muted">{s.peekSummary(p.files, p.dirs, p.largest.length)}</div>
                                        <ul className="plain peek-list">
                                          {p.largest.map((f) => (
                                            <li key={f.path} title={f.path}>
                                              <span className="peek-rel">{f.rel}</span>
                                              <span className="peek-size">{humanSize(f.size_bytes)}</span>
                                              <span className="muted">{relativeDate(f.modified, lang)}</span>
                                              <button className="link" onClick={() => revealItemInDir(f.path)}>{s.openFolder}</button>
                                            </li>
                                          ))}
                                        </ul>
                                      </>
                                    );
                                  })()
                                )}
                              </div>
                            )}
                          </div>
                          <div className="item-meta">
                            <div className="size">{humanSize(it.size_bytes)}</div>
                            <div className="muted">{s.files(it.file_count)}</div>
                            <div className="muted">
                              {s.lastUsed} {relativeDate(it.last_used, lang)}
                            </div>
                          </div>
                        </li>
                      ))}
                    </ul>
                  )}
                </section>
              );
            })}
          </main>

          <footer className="actionbar">
            <span>{s.selected(selectedItems.length, humanSize(selectedBytes))}</span>
            <button
              className="primary"
              disabled={selectedItems.length === 0 || phase === "cleaning"}
              onClick={() => setConfirming(true)}
            >
              {phase === "cleaning" ? (exporting ? s.exporting : s.cleaning) : s.clean}
            </button>
          </footer>
        </>
      )}

      {moving && (
        <div className="modal-backdrop" onClick={() => !moving.progress && setMoving(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{s.moveTitle}</h3>
            {moving.result ? (
              <>
                <p>{s.moveDone(moving.result.to)}</p>
                {moving.result.note && <p className="warn">{s.moveNote[moving.result.note] ?? moving.result.note}</p>}
                <div className="modal-actions">
                  <button className="primary" onClick={() => setMoving(null)}>OK</button>
                </div>
              </>
            ) : moving.progress ? (
              <>
                <p>{s.moving}</p>
                <progress max={moving.progress[1] || 1} value={moving.progress[0]} style={{ width: "100%" }} />
                <p className="muted">
                  {humanSize(moving.progress[0])} / {humanSize(moving.progress[1])}
                </p>
              </>
            ) : (
              <>
                <p>{s.moveBody(moving.item.name, humanSize(moving.item.size_bytes), moving.dest)}</p>
                {moving.source !== moving.item.path && <p className="warn">{s.moveWhole(moving.source)}</p>}
                <p className="muted">{moving.source}</p>
                <div className="modal-actions">
                  <button className="ghost" onClick={() => setMoving(null)}>{s.cancel}</button>
                  <button className="primary" onClick={runMove}>{s.moveTitle}</button>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      {compacting && (
        <div className="modal-backdrop" onClick={() => !compacting.busy && setCompacting(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{s.compactTitle}</h3>
            {compacting.disks.length === 0 ? (
              <>
                <p>{s.compactNone}</p>
                <div className="modal-actions">
                  <button className="primary" onClick={() => setCompacting(null)}>OK</button>
                </div>
              </>
            ) : compacting.results.length === compacting.disks.length ? (
              <>
                {compacting.results.map((r) => (
                  <p key={r.path}>
                    <span className="muted">{r.path}</span>
                    <br />
                    {r.after < r.before
                      ? s.compactDone(humanSize(r.before), humanSize(r.after), humanSize(r.before - r.after))
                      : s.compactNoGain}
                  </p>
                ))}
                <div className="modal-actions">
                  <button className="primary" onClick={() => setCompacting(null)}>OK</button>
                </div>
              </>
            ) : compacting.busy ? (
              <>
                <p>{s.compacting}</p>
                <p className="muted">{compacting.busy}</p>
              </>
            ) : (
              <>
                <p>{s.compactBody}</p>
                <p className="warn">{s.compactSteps}</p>
                <p className="muted">{s.compactTip}</p>
                <ul className="plain">
                  {compacting.disks.map((d) => (
                    <li key={d.path}>
                      <span className="muted">{d.path}</span> · {humanSize(d.size_bytes)}
                    </li>
                  ))}
                </ul>
                <div className="modal-actions">
                  <button className="ghost" onClick={() => setCompacting(null)}>{s.cancel}</button>
                  <button className="primary" onClick={runCompact}>{s.compactTitle}</button>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      {obStep !== null && (
        <div className="modal-backdrop">
          <div className="modal onboarding">
            <p className="muted">{s.obStep(obStep + 1, s.obTitles.length)}</p>
            <h3>{s.obTitles[obStep]}</h3>
            <p>{s.obBodies[obStep]}</p>
            {obStep === 1 && (
              <div className="stat-row">
                <Pill level="safe" label={s.safety.safe} size="" hint={s.safetyHint.safe} />
                <Pill level="redownload" label={s.safety.redownload} size="" hint={s.safetyHint.redownload} />
                <Pill level="review" label={s.safety.review} size="" hint={s.safetyHint.review} />
              </div>
            )}
            <div className="modal-actions">
              <button className="ghost" onClick={finishOnboarding}>{s.obSkip}</button>
              {obStep + 1 < s.obTitles.length ? (
                <button className="primary" onClick={() => setObStep(obStep + 1)}>{s.obNext}</button>
              ) : (
                <button className="primary" onClick={finishOnboarding}>{s.obDone}</button>
              )}
            </div>
          </div>
        </div>
      )}
      {confirming && (
        <div className="modal-backdrop" onClick={() => setConfirming(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{s.confirmTitle}</h3>
            <p>{s.confirmBody(selectedItems.length, humanSize(selectedBytes))}</p>
            {selectedReview > 0 && <p className="warn">{s.confirmReview(selectedReview)}</p>}
            {selectedExportable.length > 0 && (
              <div className="export-opt">
                <label>
                  <input type="checkbox" checked={exportOn} onChange={(e) => setExportOn(e.target.checked)} />{" "}
                  {s.exportFirst(selectedExportable.length)}
                </label>
                {exportOn && (
                  <p className="muted">
                    {exportDest ? s.exportTo(exportDest) : null}{" "}
                    <button className="link" onClick={pickExportDest}>{s.exportPick}</button>
                  </p>
                )}
              </div>
            )}
            <div className="modal-actions">
              <button className="ghost" onClick={() => setConfirming(false)}>
                {s.cancel}
              </button>
              <button
                className="primary danger"
                disabled={exportOn && selectedExportable.length > 0 && !exportDest}
                onClick={runClean}
              >
                {s.confirm}
              </button>
            </div>
          </div>
        </div>
      )}
      </div>
    </div>
  );
}

function Pill({ level, label, size, hint }: { level: string; label: string; size: string; hint: string }) {
  return (
    <div className={`pill ${level}`} title={hint}>
      <span className="pill-label">{label}</span>
      <span className="pill-size">{size}</span>
    </div>
  );
}
