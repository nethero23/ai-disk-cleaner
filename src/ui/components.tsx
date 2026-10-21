import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useId,
  useRef,
  useState,
  type ButtonHTMLAttributes,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from "react";
import { ICONS, type IconName } from "./icons.gen";

export type { IconName };

const cx = (...parts: (string | false | null | undefined)[]) => parts.filter(Boolean).join(" ");

// Icons that point along the reading direction; mirrored when dir="rtl".
const DIRECTIONAL = new Set<IconName>(["arrow_back", "arrow_forward", "chevron_right", "undo", "trending_up", "drive_file_move"]);

export function Icon({ name, filled, className }: { name: IconName; filled?: boolean; className?: string }) {
  return (
    <svg className={cx("md-icon", DIRECTIONAL.has(name) && "md-icon-dir", className)} viewBox="0 -960 960 960" aria-hidden="true" focusable="false">
      <path d={ICONS[name][filled ? 1 : 0]} />
    </svg>
  );
}

type BtnProps = ButtonHTMLAttributes<HTMLButtonElement>;

export function Button({
  variant = "filled",
  icon,
  small,
  className,
  children,
  ...rest
}: BtnProps & { variant?: "filled" | "tonal" | "outlined" | "text" | "danger"; icon?: IconName; small?: boolean }) {
  return (
    <button type="button" className={cx("md-interactive md-btn", variant, small && "sm", className)} {...rest}>
      {icon && <Icon name={icon} />}
      {children}
    </button>
  );
}

/** Icon-only button. `label` is required: it is the accessible name and the tooltip. */
export function IconButton({ icon, label, className, ...rest }: BtnProps & { icon: IconName; label: string }) {
  return (
    <button type="button" className={cx("md-interactive md-icon-btn", className)} aria-label={label} title={label} {...rest}>
      <Icon name={icon} />
    </button>
  );
}

export function Fab({ icon, label, extended, className, ...rest }: BtnProps & { icon: IconName; label: string; extended?: boolean }) {
  return (
    <button
      type="button"
      className={cx("md-interactive md-fab", extended && "extended", className)}
      aria-label={extended ? undefined : label}
      title={extended ? undefined : label}
      {...rest}
    >
      <Icon name={icon} />
      {extended && label}
    </button>
  );
}

export function FilterChip({ selected, icon, children, ...rest }: BtnProps & { selected: boolean; icon?: IconName }) {
  return (
    <button type="button" className="md-interactive md-chip" aria-pressed={selected} {...rest}>
      {selected ? <Icon name="check" /> : icon && <Icon name={icon} />}
      {children}
    </button>
  );
}

export type Safety = "safe" | "redownload" | "review" | "inuse";
const SAFETY_ICON: Record<Safety, IconName> = { safe: "verified", redownload: "cloud_download", review: "warning", inuse: "lock" };

/** Read-only safety label. Always icon + text, never color alone. */
export function SafetyChip({ level, label, hint }: { level: Safety; label: string; hint?: string }) {
  return (
    <span className={cx("md-safety", level)} title={hint}>
      <Icon name={SAFETY_ICON[level]} />
      {label}
    </span>
  );
}

export function Checkbox({
  checked,
  indeterminate,
  onChange,
  label,
  disabled,
}: {
  checked: boolean;
  indeterminate?: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  disabled?: boolean;
}) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (ref.current) ref.current.indeterminate = !!indeterminate && !checked;
  }, [indeterminate, checked]);
  const on = checked || indeterminate;
  return (
    <span className={cx("md-interactive md-checkbox", on && "on")}>
      <input ref={ref} type="checkbox" checked={checked} disabled={disabled} aria-label={label} onChange={(e) => onChange(e.target.checked)} />
      <span className="box">{checked ? <Icon name="check" /> : indeterminate ? <span style={{ width: 10, height: 2, background: "currentColor" }} /> : null}</span>
    </span>
  );
}

export function Switch({ checked, onChange, children, disabled }: { checked: boolean; onChange: (v: boolean) => void; children: ReactNode; disabled?: boolean }) {
  return (
    <label className="md-switch">
      <span style={{ flex: 1 }}>{children}</span>
      <input type="checkbox" role="switch" checked={checked} disabled={disabled} onChange={(e) => onChange(e.target.checked)} />
      <span className="track">
        <span className="thumb" />
      </span>
    </label>
  );
}

/** Search bar. Ctrl+F focuses it while it is mounted. */
export function SearchBar({ value, onChange, placeholder, clearLabel }: { value: string; onChange: (v: string) => void; placeholder: string; clearLabel: string }) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key.toLowerCase() === "f") {
        e.preventDefault();
        ref.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  return (
    <label className="md-search">
      <Icon name="search" />
      <input ref={ref} type="search" value={value} placeholder={placeholder} aria-label={placeholder} onChange={(e) => onChange(e.target.value)} />
      {value && <IconButton icon="close" label={clearLabel} onClick={() => onChange("")} />}
    </label>
  );
}

export function SegmentedButton<T extends string>({
  options,
  value,
  onChange,
  label,
  small,
}: {
  options: { value: T; label: string }[];
  value: T;
  onChange: (v: T) => void;
  label: string;
  small?: boolean;
}) {
  return (
    <div className={cx("md-segmented", small && "sm")} role="radiogroup" aria-label={label}>
      {options.map((o) => (
        <button key={o.value} type="button" role="radio" className="md-interactive" aria-checked={o.value === value} onClick={() => onChange(o.value)}>
          {o.value === value && <Icon name="check" />}
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function ListItem({
  lead,
  avatar,
  headline,
  supporting,
  trailing,
  selected,
  onOpen,
}: {
  lead?: ReactNode;
  avatar?: ReactNode;
  headline: ReactNode;
  supporting?: ReactNode;
  trailing?: ReactNode;
  selected?: boolean;
  onOpen?: () => void;
}) {
  return (
    <li
      className={cx("md-list-item", onOpen && "md-interactive")}
      aria-selected={selected}
      tabIndex={onOpen ? 0 : undefined}
      onClick={onOpen}
      onKeyDown={onOpen ? (e) => e.key === "Enter" && e.target === e.currentTarget && onOpen() : undefined}
    >
      <span className="lead" onClick={(e) => e.stopPropagation()}>{lead}</span>
      <span className="avatar">{avatar}</span>
      <span className="body">
        <span className="headline">{headline}</span>
        {supporting && <span className="supporting" style={{ display: "block" }}>{supporting}</span>}
      </span>
      <span className="trailing">{trailing}</span>
    </li>
  );
}

export function Avatar({ text }: { text: string }) {
  return <span className="md-avatar" aria-hidden="true">{text.slice(0, 2)}</span>;
}

export function Card({ variant = "low", className, children }: { variant?: "low" | "filled" | "outlined"; className?: string; children: ReactNode }) {
  return <div className={cx("md-card", variant, className)}>{children}</div>;
}

/** Modal dialog: Esc and scrim click close it, focus moves in and is restored on close. */
export function Dialog({
  open,
  icon,
  title,
  children,
  actions,
  onClose,
}: {
  open: boolean;
  icon?: IconName;
  title: string;
  children: ReactNode;
  actions: ReactNode;
  onClose: () => void;
}) {
  const id = useId();
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const prev = document.activeElement as HTMLElement | null;
    ref.current?.querySelector<HTMLElement>(".actions button")?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
      if (e.key === "Tab" && ref.current) {
        const f = ref.current.querySelectorAll<HTMLElement>("button, input, select, a[href], [tabindex='0']");
        if (f.length === 0) return;
        const first = f[0];
        const last = f[f.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      prev?.focus();
    };
  }, [open, onClose]);
  if (!open) return null;
  return (
    <div className="md-scrim" onClick={onClose}>
      <div ref={ref} className="md-dialog" role="dialog" aria-modal="true" aria-labelledby={id} onClick={(e) => e.stopPropagation()}>
        {icon && <span className="icon"><Icon name={icon} /></span>}
        <h2 id={id}>{title}</h2>
        <div className="content">{children}</div>
        <div className="actions">{actions}</div>
      </div>
    </div>
  );
}

type Snack = { id: number; message: string; action?: string; onAction?: () => void; duration?: number };
const SnackCtx = createContext<(s: Omit<Snack, "id">) => void>(() => {});

/** Wrap the app once; call useSnackbar() anywhere to show a message. Announced to screen readers. */
export function SnackbarHost({ children, closeLabel }: { children: ReactNode; closeLabel: string }) {
  const [snacks, setSnacks] = useState<Snack[]>([]);
  const next = useRef(1);
  const dismiss = useCallback((id: number) => setSnacks((s) => s.filter((x) => x.id !== id)), []);
  const show = useCallback(
    (s: Omit<Snack, "id">) => {
      const id = next.current++;
      setSnacks([{ ...s, id }]);
      window.setTimeout(() => dismiss(id), s.duration ?? (s.action ? 10000 : 4000));
    },
    [dismiss],
  );
  return (
    <SnackCtx.Provider value={show}>
      {children}
      <div className="md-snackbar-host" role="status" aria-live="polite">
        {snacks.map((s) => (
          <div key={s.id} className="md-snackbar">
            <span className="msg">{s.message}</span>
            {s.action && (
              <Button variant="text" onClick={() => { s.onAction?.(); dismiss(s.id); }}>
                {s.action}
              </Button>
            )}
            <IconButton icon="close" label={closeLabel} onClick={() => dismiss(s.id)} />
          </div>
        ))}
      </div>
    </SnackCtx.Provider>
  );
}

export const useSnackbar = () => useContext(SnackCtx);

export type RailItem<T extends string> = { value: T; label: string; icon: IconName; badge?: boolean };

/** Navigation rail. Up/Down arrows move between destinations. */
export function NavigationRail<T extends string>({
  fab,
  items,
  bottom,
  value,
  onChange,
}: {
  fab?: ReactNode;
  items: RailItem<T>[];
  bottom?: RailItem<T>[];
  value: T;
  onChange: (v: T) => void;
}) {
  const all = [...items, ...(bottom ?? [])];
  const onKey = (e: ReactKeyboardEvent) => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const i = all.findIndex((x) => x.value === value);
    const n = all[(i + (e.key === "ArrowDown" ? 1 : all.length - 1)) % all.length];
    onChange(n.value);
    (e.currentTarget.querySelector(`[data-v="${n.value}"]`) as HTMLElement | null)?.focus();
  };
  const render = (it: RailItem<T>) => (
    <button
      key={it.value}
      type="button"
      data-v={it.value}
      className="md-rail-item"
      aria-current={it.value === value ? "page" : undefined}
      tabIndex={it.value === value ? 0 : -1}
      onClick={() => onChange(it.value)}
    >
      <span className="ind">
        <Icon name={it.icon} filled={it.value === value} />
        {it.badge && <span className="md-badge" />}
      </span>
      {it.label}
    </button>
  );
  return (
    <nav className="md-rail" onKeyDown={onKey}>
      {fab && <div className="fab-slot">{fab}</div>}
      {items.map(render)}
      <span className="spacer" />
      {bottom?.map(render)}
    </nav>
  );
}

export function Tabs<T extends string>({
  tabs,
  value,
  onChange,
  label,
}: {
  tabs: { value: T; label: string; icon?: IconName; badge?: ReactNode }[];
  value: T;
  onChange: (v: T) => void;
  label: string;
}) {
  return (
    <div className="md-tabs" role="tablist" aria-label={label}>
      {tabs.map((t) => (
        <button key={t.value} type="button" role="tab" className="md-interactive md-tab" aria-selected={t.value === value} onClick={() => onChange(t.value)}>
          {t.icon && <Icon name={t.icon} />}
          {t.label}
          {t.badge}
        </button>
      ))}
    </div>
  );
}

/** Determinate when `value` (0..1) is given, indeterminate otherwise. */
export function LinearProgress({ value, label }: { value?: number; label: string }) {
  const det = value !== undefined;
  return (
    <div
      className={cx("md-progress", !det && "indeterminate")}
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={det ? Math.round(value * 100) : undefined}
    >
      <span className="bar" style={det ? { width: `${Math.min(100, value * 100)}%` } : undefined} />
    </div>
  );
}

/** Plain tooltip shown on hover and on keyboard focus. */
export function Tooltip({ text, children }: { text: string; children: ReactNode }) {
  const id = useId();
  return (
    <span className="md-tooltip-anchor" aria-describedby={id}>
      {children}
      <span id={id} role="tooltip" className="md-tooltip">{text}</span>
    </span>
  );
}

export function FloatingToolbar({ children, label }: { children: ReactNode; label: string }) {
  return (
    <div className="md-floating-toolbar" role="toolbar" aria-label={label}>
      {children}
    </div>
  );
}
