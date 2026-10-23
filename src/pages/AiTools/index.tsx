// AI tools page (v0.8.0): read-only inventory of what every AI tool on this
// PC has configured, and what overlaps or is broken. Standalone component,
// built on the Material 3 layer in src/ui; strings come from i18n.ts.
import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import "../../ui";
import { Button, Card, Dialog, FilterChip, Icon, ListItem, SegmentedButton } from "../../ui";
import { humanSize, t, type Lang } from "../../i18n";
import type { AiFinding, AiFindingKind, AiInventory, AiMcpServer, AiToolInventory, AiVersionInfo } from "../../types";
import { isBehind } from "./versions";
import "./style.css";

type View = "tools" | "mcp" | "findings";

const KINDS: AiFindingKind[] = [
  "secret_exposed",
  "mcp_broken",
  "mcp_version_conflict",
  "mcp_duplicate",
  "skill_diverged",
  "skill_copies",
  "stale_versions",
];

const KIND_ICON: Record<AiFindingKind, "lock" | "error" | "warning" | "content_copy" | "history"> = {
  secret_exposed: "lock",
  mcp_broken: "error",
  mcp_version_conflict: "warning",
  mcp_duplicate: "content_copy",
  skill_diverged: "warning",
  skill_copies: "content_copy",
  stale_versions: "history",
};

type Detail =
  | { kind: "finding"; finding: AiFinding }
  | { kind: "server"; servers: AiMcpServer[] }
  | { kind: "tool"; tool: AiToolInventory };

export default function AiTools({ lang, versionCheck }: { lang: Lang; versionCheck: boolean }) {
  const s = t(lang);
  const [inv, setInv] = useState<AiInventory | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<View>("tools");
  const [kind, setKind] = useState<AiFindingKind | "all">("all");
  const [detail, setDetail] = useState<Detail | null>(null);
  const [latest, setLatest] = useState<Map<string, string>>(new Map());
  const [checking, setChecking] = useState(false);
  const [checkFailed, setCheckFailed] = useState(false);
  const loadId = useRef(0);

  async function load() {
    const id = ++loadId.current;
    setBusy(true);
    setError(null);
    try {
      const next = await invoke<AiInventory>("ai_inventory");
      if (id === loadId.current) setInv(next);
    } catch (e) {
      if (id === loadId.current) setError(String(e));
    } finally {
      if (id === loadId.current) setBusy(false);
    }
  }

  useEffect(() => {
    void load();
    return () => { loadId.current++; };
  }, []);

  // Ignore stale replies after a refresh, preference change or navigation away.
  useEffect(() => {
    let cancelled = false;
    setLatest(new Map());
    setChecking(false);
    setCheckFailed(false);
    if (!inv || !versionCheck) return;
    const ids = [...new Set(inv.tools.flatMap((x) => x.mcp.map((m) => m.identity)))].filter((id) => id.startsWith("npm:") || id.startsWith("pypi:"));
    if (ids.length === 0) return;
    setChecking(true);
    void invoke<AiVersionInfo[]>("ai_version_check", { identities: ids })
      .then((infos) => {
        if (cancelled) return;
        setLatest(new Map(infos.filter((i) => i.latest).map((i) => [i.identity, i.latest as string])));
        setCheckFailed(infos.some((i) => i.error));
      })
      .catch(() => { if (!cancelled) setCheckFailed(true); })
      .finally(() => { if (!cancelled) setChecking(false); });
    return () => { cancelled = true; };
  }, [inv, versionCheck]);

  const present = useMemo(() => (inv ? inv.tools.filter((x) => x.present) : []), [inv]);
  const byIdentity = useMemo(() => {
    const m = new Map<string, AiMcpServer[]>();
    for (const tool of present) for (const srv of tool.mcp) {
      const list = m.get(srv.identity) ?? [];
      list.push(srv);
      m.set(srv.identity, list);
    }
    return [...m.entries()].sort((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]));
  }, [present]);
  const toolName = (id: string) => inv?.tools.find((x) => x.id === id)?.name ?? id;
  const findings = inv?.analysis.findings ?? [];
  const shown = kind === "all" ? findings : findings.filter((f) => f.kind === kind);
  const counts = new Map<string, number>();
  for (const f of findings) counts.set(f.kind, (counts.get(f.kind) ?? 0) + 1);

  return (
    <section className="ai-page">
      <header className="ai-head">
        <div>
          <h2>{s.aiTitle}</h2>
          <p className="ai-hint">{s.aiHint}</p>
        </div>
        <Button variant="tonal" icon="refresh" disabled={busy || checking} onClick={load}>
          {busy ? s.aiLoading : inv ? s.aiRefresh : s.aiLoad}
        </Button>
      </header>

      {error && <p className="ai-error">{error}</p>}

      {inv && (
        <>
          <Card variant="filled" className="ai-summary">
            <p>{s.aiSummary(present.length, inv.analysis.mcp_total, inv.analysis.mcp_unique, inv.analysis.skills_total, inv.analysis.skills_unique)}</p>
            {inv.analysis.reclaimable_bytes > 0 && <p className="ai-reclaim">{s.aiReclaim(humanSize(inv.analysis.reclaimable_bytes))}</p>}
            {checking && <p className="ai-muted" role="status">{s.aiChecking}</p>}
            {checkFailed && <p className="ai-muted" role="status">{s.aiCheckFailed}</p>}
          </Card>

          <div className="ai-toolbar">
            <SegmentedButton<View>
              label={s.aiViewLabel}
              value={view}
              onChange={setView}
              options={[
                { value: "tools", label: s.aiViewTools },
                { value: "mcp", label: s.aiViewMcp },
                { value: "findings", label: `${s.aiViewFindings}${findings.length ? ` (${findings.length})` : ""}` },
              ]}
            />
          </div>

          {view === "tools" && (present.length === 0 ? <p className="ai-empty">{s.aiNoTools}</p> : (
            <div className="ai-grid">
              {present.map((tool) => (
                <Card key={tool.id} variant="outlined" className="ai-tool">
                  <div className="ai-tool-head">
                    <h3>{tool.name}</h3>
                    {tool.running && <span className="ai-tag run">{s.aiRunning}</span>}
                    {!tool.verified && <span className="ai-tag">{s.aiUnverified}</span>}
                  </div>
                  <ul className="ai-stats">
                    <li><b>{tool.mcp.length}</b> {s.aiMcp}</li>
                    <li><b>{tool.skills.length}</b> {s.aiSkills}</li>
                    {tool.plugins.length > 0 && <li><b>{tool.plugins.length}</b> {s.aiPlugins}</li>}
                    {tool.extensions.length > 0 && <li><b>{tool.extensions.length}</b> {s.aiExtensions}</li>}
                  </ul>
                  <Button variant="text" icon="chevron_right" onClick={() => setDetail({ kind: "tool", tool })}>{s.aiDetails}</Button>
                </Card>
              ))}
            </div>
          ))}

          {view === "mcp" && (byIdentity.length === 0 ? <p className="ai-empty">{s.aiNoMcp}</p> : (
            <ul className="md-list">
              {byIdentity.map(([identity, servers]) => {
                const tools = [...new Set(servers.map((x) => x.tool))];
                const versions = [...new Set(servers.map((x) => x.version).filter(Boolean))];
                const secret = servers.some((x) => x.secrets.length > 0);
                const lat = latest.get(identity);
                const behindPins = lat ? versions.filter((v) => v && isBehind(v, lat)) : [];
                return (
                  <ListItem
                    key={identity}
                    avatar={<Icon name={servers[0].transport === "remote" ? "language" : "hub"} />}
                    headline={<>{servers[0].name}{secret && <Icon name="lock" className="ai-inline-icon" />}</>}
                    supporting={<>{identity}{versions.length > 1 ? ` · ${versions.join(" / ")}` : ""}{lat ? <span className={behindPins.length ? "ai-behind" : "ai-muted"}> · {behindPins.length ? s.aiNewer(behindPins.join(" / "), lat) : s.aiLatest(lat)}</span> : null}</>}
                    trailing={<span className="ai-trail">{tools.length > 1 ? s.aiInTools(tools.length) : toolName(tools[0])}</span>}
                    onOpen={() => setDetail({ kind: "server", servers })}
                  />
                );
              })}
            </ul>
          ))}

          {view === "findings" && (
            <>
              <div className="ai-chips">
                <FilterChip selected={kind === "all"} onClick={() => setKind("all")}>{s.aiKindAll}</FilterChip>
                {KINDS.filter((k) => counts.has(k)).map((k) => (
                  <FilterChip key={k} selected={kind === k} icon={KIND_ICON[k]} onClick={() => setKind(k)}>
                    {s.aiKind[k]} · {counts.get(k)}
                  </FilterChip>
                ))}
              </div>
              {shown.length === 0 ? <p className="ai-empty">{s.aiNoFindings}</p> : (
                <ul className="md-list">
                  {shown.map((f, i) => (
                    <ListItem
                      key={`${f.kind}-${f.subject}-${i}`}
                      avatar={<Icon name={KIND_ICON[f.kind]} />}
                      headline={<>{s.aiKind[f.kind]}: {f.subject}</>}
                      supporting={f.detail}
                      trailing={<span className="ai-trail">{f.reclaimable_bytes > 0 ? s.aiReclaimable(humanSize(f.reclaimable_bytes)) : f.tools.map(toolName).join(", ")}</span>}
                      onOpen={() => setDetail({ kind: "finding", finding: f })}
                    />
                  ))}
                </ul>
              )}
            </>
          )}
        </>
      )}

      <Dialog
        open={detail !== null}
        title={detail?.kind === "finding" ? `${s.aiKind[detail.finding.kind]}: ${detail.finding.subject}` : detail?.kind === "server" ? detail.servers[0].name : detail?.kind === "tool" ? detail.tool.name : ""}
        onClose={() => setDetail(null)}
        actions={<Button variant="text" onClick={() => setDetail(null)}>{s.aiClose}</Button>}
      >
        {detail?.kind === "finding" && <FindingDetail f={detail.finding} s={s} toolName={toolName} />}
        {detail?.kind === "server" && <ServerDetail servers={detail.servers} s={s} toolName={toolName} />}
        {detail?.kind === "tool" && <ToolDetail tool={detail.tool} s={s} />}
      </Dialog>
    </section>
  );
}

type S = ReturnType<typeof t>;

function PathList({ paths, s }: { paths: string[]; s: S }) {
  const [copied, setCopied] = useState<string | null>(null);
  return (
    <ul className="ai-paths">
      {paths.map((p) => (
        <li key={p}>
          <code>{p}</code>
          <span className="ai-path-actions">
            <button className="linkish" onClick={() => revealItemInDir(p)}>{s.aiReveal}</button>
            <button
              className="linkish"
              onClick={() => { void navigator.clipboard.writeText(p); setCopied(p); }}
            >
              {copied === p ? s.aiCopied : s.aiCopyPath}
            </button>
          </span>
        </li>
      ))}
    </ul>
  );
}

function FindingDetail({ f, s, toolName }: { f: AiFinding; s: S; toolName: (id: string) => string }) {
  return (
    <>
      <p className="ai-kind-hint">{s.aiKindHint[f.kind]}</p>
      <p>{f.detail}</p>
      {f.tools.length > 0 && <p className="ai-muted">{f.tools.map(toolName).join(" · ")}</p>}
      {f.reclaimable_bytes > 0 && <p><b>{s.aiReclaimable(humanSize(f.reclaimable_bytes))}</b></p>}
      <h4>{s.aiPaths}</h4>
      <PathList paths={f.paths} s={s} />
    </>
  );
}

function ServerDetail({ servers, s, toolName }: { servers: AiMcpServer[]; s: S; toolName: (id: string) => string }) {
  return (
    <>
      <p className="ai-muted"><code>{servers[0].identity}</code></p>
      {servers.map((srv, i) => (
        <div key={i} className="ai-srv">
          <div className="ai-srv-head">
            <b>{toolName(srv.tool)}</b>
            {srv.scope.startsWith("project:") && <span className="ai-tag">{s.aiScopeProject} {srv.scope.slice(8)}</span>}
            {srv.scope.startsWith("extension:") && <span className="ai-tag">{s.aiScopeExtension} {srv.scope.slice(10)}</span>}
            {srv.disabled && <span className="ai-tag">{s.aiOff}</span>}
            {srv.version && <span className="ai-tag">{srv.version}</span>}
          </div>
          <code className="ai-cmd">{srv.transport === "remote" ? srv.url : [srv.command, ...srv.args].filter(Boolean).join(" ")}</code>
          {srv.env_keys.length > 0 && <p className="ai-muted">env: {srv.env_keys.join(", ")}</p>}
          {srv.secrets.map((h) => (
            <p key={h.location} className="ai-secret"><Icon name="lock" className="ai-inline-icon" /> {h.location} = {h.masked}</p>
          ))}
          <PathList paths={[srv.config_path]} s={s} />
        </div>
      ))}
    </>
  );
}

function ToolDetail({ tool, s }: { tool: AiToolInventory; s: S }) {
  const existing = tool.configs.filter((c) => c.exists);
  return (
    <>
      {existing.length > 0 && (
        <>
          <h4>{s.aiConfigs}</h4>
          <ul className="ai-paths">
            {existing.map((c) => (
              <li key={c.path}>
                <code>{c.path}</code>
                <span className="ai-muted"> {humanSize(c.bytes)}{c.error ? ` · ${s.aiConfigError}` : ` · ${c.servers} ${s.aiMcp}`}</span>
              </li>
            ))}
          </ul>
        </>
      )}
      {tool.skills.length > 0 && (
        <>
          <h4>{s.aiSkills}</h4>
          <ul className="ai-paths">
            {tool.skills.map((k) => (
              <li key={k.path}><b>{k.name}</b>{k.description ? <span className="ai-muted"> {k.description}</span> : null}<span className="ai-muted"> · {humanSize(k.bytes)}</span></li>
            ))}
          </ul>
        </>
      )}
      {tool.plugins.length > 0 && (
        <>
          <h4>{s.aiPlugins}</h4>
          <ul className="ai-paths">
            {tool.plugins.map((p) => (
              <li key={p.path}><b>{p.name}</b> {p.version ?? ""}<span className="ai-muted"> · {humanSize(p.bytes)}{p.stale_versions.length ? ` · ${s.aiKind.stale_versions}: ${p.stale_versions.join(", ")}` : ""}</span></li>
            ))}
          </ul>
        </>
      )}
      {tool.extensions.length > 0 && (
        <>
          <h4>{s.aiExtensions}</h4>
          <ul className="ai-paths">
            {tool.extensions.map((p) => (
              <li key={p.path}><b>{p.name}</b> {p.version ?? ""}<span className="ai-muted"> · {humanSize(p.bytes)}{p.stale_versions.length ? ` · ${s.aiKind.stale_versions}: ${p.stale_versions.join(", ")}` : ""}</span></li>
            ))}
          </ul>
        </>
      )}
    </>
  );
}
