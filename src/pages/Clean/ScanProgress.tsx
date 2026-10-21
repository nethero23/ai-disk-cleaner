import type { Strings } from "../../i18n";
import { Button, Card, LinearProgress } from "../../ui";

export function ScanProgress({
  s,
  step,
  onCancel,
}: {
  s: Strings;
  step: { done: number; total: number; rule_name: string } | null;
  onCancel: () => void;
}) {
  return (
    <Card className="scan-progress">
      <span className="hero-label">{s.scanning}</span>
      <LinearProgress label={s.scanning} value={step && step.total > 0 ? step.done / step.total : undefined} />
      <span className="muted" aria-live="polite">{step ? s.scanStep(step.done, step.total, step.rule_name) : " "}</span>
      <div>
        <Button variant="outlined" icon="close" onClick={onCancel}>
          {s.scanCancel}
        </Button>
      </div>
    </Card>
  );
}
