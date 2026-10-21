import "./clean.css";
import type { ReactNode } from "react";
import type { ScanReport } from "../../types";
import { humanSize, type Strings } from "../../i18n";
import { Button, Card } from "../../ui";

/// Top of the 清理 page after a scan: how much can be freed, split by safety level,
/// with the one-click "clean safe items" action next to it.
export function Hero({
  report,
  s,
  meta,
  summaryLine,
  safeCount,
  safeBytes,
  busy,
  onCleanSafe,
  onViewAll,
  children,
}: {
  report: ScanReport;
  s: Strings;
  meta: ReactNode;
  summaryLine: string | null;
  safeCount: number;
  safeBytes: number;
  busy: boolean;
  onCleanSafe: () => void;
  onViewAll: () => void;
  children?: ReactNode;
}) {
  const total = Math.max(1, report.total_bytes);
  const parts = [
    { level: "safe" as const, bytes: report.safe_bytes },
    { level: "redownload" as const, bytes: report.redownload_bytes },
    { level: "review" as const, bytes: report.review_bytes },
  ];
  return (
    <Card className="hero">
      <div className="hero-top">
        <div className="hero-figure">
          <span className="hero-label">{s.heroLabel}</span>
          <span className="hero-big md-ltr">{humanSize(report.total_bytes)}</span>
          <span className="hero-meta">{meta}</span>
        </div>
        <div className="hero-actions">
          {safeCount > 0 && (
            <Button icon="wand_stars" disabled={busy} onClick={onCleanSafe}>
              {s.cleanSafeN(safeCount, humanSize(safeBytes))}
            </Button>
          )}
          <Button variant="text" onClick={onViewAll}>
            {s.viewAll(report.items.length)}
          </Button>
        </div>
      </div>
      <div className="hero-meter" role="img" aria-label={parts.map((p) => `${s.safety[p.level]} ${humanSize(p.bytes)}`).join("，")}>
        {parts.map((p) => p.bytes > 0 && <i key={p.level} className={p.level} style={{ flexGrow: p.bytes / total }} />)}
      </div>
      <div className="hero-legend">
        {parts.map((p) => (
          <span key={p.level}>
            <i className={p.level} />
            <b>
              {s.safety[p.level]} <span className="md-ltr">{humanSize(p.bytes)}</span>
            </b>
            <span className="muted">{s.safetyHint[p.level]}</span>
          </span>
        ))}
      </div>
      {summaryLine && <p className="hero-summary">{summaryLine}</p>}
      {children}
    </Card>
  );
}
