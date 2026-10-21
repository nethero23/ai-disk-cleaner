import type { Drive, Settings } from "./types";
import type { Strings } from "./i18n";
import { FilterChip } from "./ui";

/// The "scan range" checkboxes: the system drive is always on, other
/// drives toggle `settings.scan_drives`. Shown on the home page, above
/// scan results and in settings, so the choice can be changed any time.
export function DrivePicker({
  drives,
  settings,
  s,
  onChange,
  note,
  align,
}: {
  drives: Drive[];
  settings: Settings;
  s: Strings;
  onChange: (next: Settings) => void;
  note?: string;
  align?: "center" | "start";
}) {
  if (drives.length < 2) return null;
  return (
    <div className={`drive-pick${align === "start" ? " start" : ""}`} title={s.scanDrivesHint}>
      <span className="muted">{s.scanDrives}</span>
      {drives.map((d) => {
        const on = d.system || settings.scan_drives.includes(d.path);
        return (
          <FilterChip
            key={d.path}
            selected={on}
            disabled={d.system}
            title={d.system ? s.systemDrive : undefined}
            onClick={() =>
              onChange({
                ...settings,
                scan_drives: on ? settings.scan_drives.filter((p) => p !== d.path) : [...settings.scan_drives, d.path],
              })
            }
          >
            {d.path.replace(/\\$/, "")}{d.name ? ` ${d.name}` : ""}
          </FilterChip>
        );
      })}
      {note && <span className="muted">{note}</span>}
    </div>
  );
}
