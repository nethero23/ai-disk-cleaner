import type { ReactNode } from "react";
import type { Preview, ScanItem } from "../../types";
import { humanSize, relativeDate, type Lang, type Strings } from "../../i18n";
import { Avatar, Button, Checkbox, Icon, IconButton, SafetyChip } from "../../ui";

/// One scan result row. Clicking the row (or the chevron) expands the details:
/// what deleting it means, the path, a peek at the largest files and the actions.
export function ResultItem({
  it,
  s,
  lang,
  selected,
  expanded,
  peek,
  canMove,
  canCompact,
  onToggle,
  onExpand,
  onReveal,
  onPeek,
  onExclude,
  onMove,
  onCompact,
}: {
  it: ScanItem;
  s: Strings;
  lang: Lang;
  selected: boolean;
  expanded: boolean;
  peek: Preview | "loading" | undefined;
  canMove: boolean;
  canCompact: boolean;
  onToggle: () => void;
  onExpand: () => void;
  onReveal: (path: string) => void;
  onPeek: () => void;
  onExclude: () => void;
  onMove: () => void;
  onCompact: () => void;
}) {
  const inUse = !!it.in_use_by;
  let peekBody: ReactNode = null;
  if (peek === "loading") peekBody = <span className="muted">{s.peekLoading}</span>;
  else if (peek && peek.files === 0) peekBody = <span className="muted">{s.peekEmpty}</span>;
  else if (peek)
    peekBody = (
      <>
        <span className="muted">{s.peekSummary(peek.files, peek.dirs, peek.largest.length)}</span>
        <ul className="ri-peek">
          {peek.largest.map((f) => (
            <li key={f.path} title={f.path}>
              <span className="md-mono ri-rel">{f.rel}</span>
              <span className="md-ltr">{humanSize(f.size_bytes)}</span>
              <span className="muted">{relativeDate(f.modified, lang)}</span>
              <IconButton icon="folder_open" label={s.openFolder} onClick={() => onReveal(f.path)} />
            </li>
          ))}
        </ul>
      </>
    );
  return (
    <li className={`ri${selected ? " on" : ""}${inUse ? " inuse" : ""}${expanded ? " open" : ""}`}>
      <div className="ri-row" onClick={onExpand}>
        <span onClick={(e) => e.stopPropagation()}>
          <Checkbox label={it.name} checked={selected} disabled={inUse} onChange={onToggle} />
        </span>
        <Avatar text={it.tool} />
        <div className="ri-main">
          <div className="ri-head">
            <span className="ri-name">{it.name}</span>
            <SafetyChip level={it.safety} label={s.safety[it.safety]} hint={s.safetyHint[it.safety]} />
            {inUse && <SafetyChip level="inuse" label={s.inUse(it.in_use_by!)} />}
          </div>
          <div className="ri-sub">
            {it.tool} · {s.lastUsed} {relativeDate(it.last_used, lang)} · {s.files(it.file_count)}
          </div>
        </div>
        <span className="ri-size md-ltr">{humanSize(it.size_bytes)}</span>
        <button
          type="button"
          className="md-interactive md-icon-btn ri-chev"
          aria-expanded={expanded}
          aria-label={s.details}
          title={s.details}
          onClick={(e) => {
            e.stopPropagation();
            onExpand();
          }}
        >
          <Icon name="chevron_right" />
        </button>
      </div>
      {expanded && (
        <div className="ri-detail">
          <p className={`ri-consequence ${it.safety}`}>
            {it.consequence}
            {it.detail && <span className="muted"> · {it.detail}</span>}
          </p>
          <p className="md-mono ri-path" title={it.path}>{it.path}</p>
          {peekBody && <div className="ri-peekbox">{peekBody}</div>}
          <div className="ri-actions">
            <Button small variant="outlined" icon="folder_open" onClick={() => onReveal(it.path)}>{s.openFolder}</Button>
            <Button small variant="outlined" icon="visibility" onClick={onPeek}>{peek ? s.peekHide : s.peek}</Button>
            {canMove && <Button small variant="outlined" icon="drive_file_move" onClick={onMove}>{s.move}</Button>}
            {canCompact && <Button small variant="outlined" icon="compress" onClick={onCompact}>{s.compact}</Button>}
            <Button small variant="text" icon="visibility_off" onClick={onExclude}>{s.exclude}</Button>
          </div>
        </div>
      )}
    </li>
  );
}
