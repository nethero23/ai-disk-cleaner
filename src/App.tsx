import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import type { Category, CleanRecord, CleanSummary, ScanItem, ScanReport } from "./types";
import { humanSize, relativeDate, t, type Lang } from "./i18n";

type Phase = "idle" | "scanning" | "ready" | "cleaning";

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
  return i.safety === "safe" && !i.in_use_by;
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
  const [collapsed, setCollapsed] = useState<Set<Category>>(new Set());
  const [view, setView] = useState<"scan" | "history">("scan");
  const [history, setHistory] = useState<CleanRecord[]>([]);

  async function openHistory() {
    try {
      setHistory(await invoke<CleanRecord[]>("clean_log"));
    } catch (e) {
      setError(String(e));
    }
    setView("history");
  }

  useEffect(() => {
    localStorage.setItem("lang", lang);
    document.documentElement.lang = lang === "zh" ? "zh-CN" : "en";
  }, [lang]);

  async function runScan() {
    setPhase("scanning");
    setError(null);
    setResult(null);
    try {
      const r = await invoke<ScanReport>("scan", { lang });
      setReport(r);
      // Default selection: only what is safe. The user opts into everything else.
      setSelected(new Set(r.items.filter(isDefaultTicked).map((i) => i.id)));
      setPhase("ready");
    } catch (e) {
      setError(String(e));
      setPhase("idle");
    }
  }

  const groups = useMemo(() => {
    if (!report) return [];
    const by = new Map<Category, ScanItem[]>();
    for (const it of report.items) {
      if (!by.has(it.category)) by.set(it.category, []);
      by.get(it.category)!.push(it);
    }
    return CATEGORY_ORDER.filter((c) => by.has(c)).map((c) => ({
      category: c,
      items: by.get(c)!,
      bytes: by.get(c)!.reduce((a, i) => a + i.size_bytes, 0),
    }));
  }, [report]);

  const selectedItems = useMemo(
    () => (report ? report.items.filter((i) => selected.has(i.id)) : []),
    [report, selected],
  );
  const selectedBytes = selectedItems.reduce((a, i) => a + i.size_bytes, 0);
  const selectedReview = selectedItems.filter((i) => i.safety === "review").length;

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

  async function runClean() {
    setConfirming(false);
    setPhase("cleaning");
    try {
      const summary = await invoke<CleanSummary>("clean_items", { items: selectedItems });
      setResult(summary);
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
      <header className="topbar">
        <div>
          <h1>{s.appName}</h1>
          <p className="tagline">{s.tagline}</p>
        </div>
        <div className="topbar-actions">
          <button className="ghost" onClick={() => setLang(lang === "zh" ? "en" : "zh")}>
            {s.language}
          </button>
          <button className="ghost" onClick={() => (view === "history" ? setView("scan") : openHistory())}>
            {view === "history" ? s.backToScan : s.history}
          </button>
          <button
            className="primary"
            disabled={phase === "scanning" || phase === "cleaning"}
            onClick={runScan}
          >
            {phase === "scanning" ? s.scanning : report ? s.rescan : s.scan}
          </button>
        </div>
      </header>

      {error && <div className="banner error">{error}</div>}
      {result && (
        <div className="banner ok">
          {s.done(humanSize(result.freed_bytes))}
          {result.records.some((r) => !r.ok) && (
            <span> · {s.failed(result.records.filter((r) => !r.ok).length)}</span>
          )}
        </div>
      )}

      {view === "history" && (
        <section className="history">
          {history.length === 0 ? (
            <p className="empty">{s.historyEmpty}</p>
          ) : (
            <>
              <p className="muted">
                {s.historyTotal(
                  history.filter((r) => r.ok).length,
                  humanSize(history.filter((r) => r.ok).reduce((a, r) => a + r.size_bytes, 0)),
                )}
              </p>
              <table className="log">
                <tbody>
                  {[...history].reverse().map((r, i) => (
                    <tr key={i} className={r.ok ? "" : "failed"}>
                      <td className="muted">{new Date(r.at).toLocaleString()}</td>
                      <td>{r.name}</td>
                      <td className="muted" title={r.path}>{r.path}</td>
                      <td className="size">{humanSize(r.size_bytes)}</td>
                      <td className="muted">{r.ok ? "" : r.error}</td>
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
        </section>
      )}
      {view === "scan" && phase === "scanning" && (
        <section className="empty">
          <div className="spinner" />
          <p>{s.scanning}</p>
        </section>
      )}

      {view === "scan" && report && phase !== "scanning" && (
        <>
          <section className="summary">
            <div className="stat">
              <span className="stat-label">{s.found(report.items.length, humanSize(report.total_bytes))}</span>
            </div>
            <div className="stat-row">
              <Pill level="safe" label={s.safety.safe} size={humanSize(report.safe_bytes)} hint={s.safetyHint.safe} />
              <Pill level="redownload" label={s.safety.redownload} size={humanSize(report.redownload_bytes)} hint={s.safetyHint.redownload} />
              <Pill level="review" label={s.safety.review} size={humanSize(report.review_bytes)} hint={s.safetyHint.review} />
            </div>
            <div className="quick">
              <button className="ghost" onClick={() => setSelected(new Set(report.items.filter(isDefaultTicked).map((i) => i.id)))}>
                {s.selectAllSafe}
              </button>
              <button className="ghost" onClick={() => setSelected(new Set())}>
                {s.selectNone}
              </button>
            </div>
            {report.unreadable.length > 0 && (
              <p className="muted" title={report.unreadable.join("\n")}>
                {s.unreadable(report.unreadable.length)}
              </p>
            )}
          </section>

          {report.items.length === 0 && <p className="empty">{s.nothing}</p>}

          <main className="groups">
            {groups.map((g) => {
              const open = !collapsed.has(g.category);
              const allOn = g.items.every((i) => selected.has(i.id));
              const someOn = g.items.some((i) => selected.has(i.id));
              return (
                <section key={g.category} className="group">
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
                          if (next.has(g.category)) next.delete(g.category);
                          else next.add(g.category);
                          return next;
                        })
                      }
                    >
                      <span className={`chev ${open ? "open" : ""}`}>▸</span>
                      {s.category[g.category]}
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
                            </div>
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
              {phase === "cleaning" ? s.cleaning : s.clean}
            </button>
          </footer>
        </>
      )}

      {confirming && (
        <div className="modal-backdrop" onClick={() => setConfirming(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{s.confirmTitle}</h3>
            <p>{s.confirmBody(selectedItems.length, humanSize(selectedBytes))}</p>
            {selectedReview > 0 && <p className="warn">{s.confirmReview(selectedReview)}</p>}
            <div className="modal-actions">
              <button className="ghost" onClick={() => setConfirming(false)}>
                {s.cancel}
              </button>
              <button className="primary danger" onClick={runClean}>
                {s.confirm}
              </button>
            </div>
          </div>
        </div>
      )}
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
