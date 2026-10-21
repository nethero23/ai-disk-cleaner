import "./settings.css";
import type { ReactNode } from "react";
import type { Drive, RuleLibrary, Settings } from "../../types";
import { coreLang, type Lang, type LangPref, type Strings } from "../../i18n";
import type { Theme } from "../../theme";
import { LanguageSelect } from "../../LanguageSelect";
import { DrivePicker } from "../../DrivePicker";
import { Button, Card, Icon, SegmentedButton, Switch, type IconName } from "../../ui";

const MB = 1024 * 1024;
const GB = 1024 * MB;

function Section({ icon, title, hint, children }: { icon: IconName; title: string; hint?: string; children: ReactNode }) {
  return (
    <Card variant="low" className="set-section">
      <h2>
        <Icon name={icon} />
        {title}
      </h2>
      {hint && <p className="set-hint">{hint}</p>}
      {children}
    </Card>
  );
}

/// A settings line: title and optional description on the left, the control on the right.
function Row({ title, desc, children }: { title: ReactNode; desc?: ReactNode; children?: ReactNode }) {
  return (
    <div className="set-row">
      <div className="set-text">
        <span className="t">{title}</span>
        {desc && <span className="d">{desc}</span>}
      </div>
      {children}
    </div>
  );
}

function SwitchRow({ title, desc, checked, onChange }: { title: string; desc?: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <div className="set-row">
      <Switch checked={checked} onChange={onChange}>
        <span className="set-text">
          <span className="t">{title}</span>
          {desc && <span className="d">{desc}</span>}
        </span>
      </Switch>
    </div>
  );
}

/// 设置 page: grouped cards of rows, every change saved immediately.
export function SettingsPage(p: {
  s: Strings;
  lang: Lang;
  langPref: LangPref;
  onLangPref: (v: LangPref) => void;
  theme: Theme;
  onTheme: (v: Theme) => void;
  settings: Settings;
  save: (next: Settings) => void;
  /** Local edit without saving (text fields save on blur). */
  edit: (next: Settings) => void;
  drives: Drive[];
  portable: boolean;
  checkingUpdate: boolean;
  onCheckUpdate: () => void;
  onOpenReleases: () => void;
  onAddExcluded: () => void;
  library: RuleLibrary | null;
  updatingRules: boolean;
  onUpdateRules: () => void;
  onPickRulesDir: () => void;
  onClearRulesDir: () => void;
  onToggleRule: (id: string, enabled: boolean) => void;
}) {
  const { s, settings: st, save } = p;
  const sizeSelect = (value: number, options: [number, string][], onChange: (v: number) => void, label: string) => (
    <select aria-label={label} value={value} onChange={(e) => onChange(Number(e.target.value))}>
      {options.map(([v, l]) => (
        <option key={v} value={v}>
          {l}
        </option>
      ))}
    </select>
  );

  return (
    <section className="settings-page">
      <Section icon="palette" title={s.setAppearance}>
        <Row title={s.language}>
          <LanguageSelect value={p.langPref} onChange={p.onLangPref} label={s.language} autoLabel={s.languageAuto} />
        </Row>
        <Row title={s.theme}>
          <SegmentedButton
            label={s.theme}
            small
            value={p.theme}
            onChange={p.onTheme}
            options={[
              { value: "system", label: s.themeSystem },
              { value: "light", label: s.themeLight },
              { value: "dark", label: s.themeDark },
            ]}
          />
        </Row>
      </Section>

      <Section icon="radar" title={s.setScanSection}>
        <Row title={s.scanDrives} desc={p.drives.length < 2 ? s.scanDrivesOnlyOne : s.scanDrivesHint}>
          <DrivePicker drives={p.drives} settings={st} s={s} onChange={save} align="start" />
        </Row>
        <Row title={s.setMinSize}>
          {sizeSelect(st.min_item_bytes, [[0, "0"], [10 * MB, "10 MB"], [100 * MB, "100 MB"], [GB, "1 GB"]], (v) => save({ ...st, min_item_bytes: v }), s.setMinSize)}
        </Row>
        <Row title={s.setExcluded} desc={s.setExcludedBody}>
          <Button variant="tonal" small icon="add" onClick={p.onAddExcluded}>
            {s.addFolder}
          </Button>
          {st.excluded_paths.length === 0 ? (
            <span className="d set-text">{s.setExcludedEmpty}</span>
          ) : (
            <ul className="set-paths">
              {st.excluded_paths.map((path) => (
                <li key={path}>
                  <Icon name="folder" />
                  <span title={path} className="md-ltr">{path}</span>
                  <Button variant="text" small onClick={() => save({ ...st, excluded_paths: st.excluded_paths.filter((x) => x !== path) })}>
                    {s.remove}
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </Row>
      </Section>

      <Section icon="schedule" title={s.setBackground}>
        <SwitchRow title={s.setCloseToTray} checked={st.close_to_tray} onChange={(v) => save({ ...st, close_to_tray: v })} />
        <SwitchRow title={s.setAutostart} checked={st.autostart} onChange={(v) => save({ ...st, autostart: v })} />
        <Row title={s.setSchedule} desc={st.last_background_scan ? s.lastBackground(new Date(st.last_background_scan).toLocaleString()) : undefined}>
          {sizeSelect(
            st.scheduled_scan_hours,
            [[0, s.schedOff], ...[6, 12, 24, 72, 168].map((h): [number, string] => [h, s.schedHours(h)])],
            (v) => save({ ...st, scheduled_scan_hours: v }),
            s.setSchedule,
          )}
        </Row>
        <Row title={s.setNotify}>
          {sizeSelect(st.notify_min_bytes, [[0, "0"], [500 * MB, "500 MB"], [GB, "1 GB"], [5 * GB, "5 GB"]], (v) => save({ ...st, notify_min_bytes: v }), s.setNotify)}
        </Row>
        <SwitchRow title={s.setAutoClean} checked={st.auto_clean_safe} onChange={(v) => save({ ...st, auto_clean_safe: v })} />
        <Row title={s.setIdleDays}>
          {sizeSelect(st.idle_model_days, [30, 60, 90, 180, 365].map((d): [number, string] => [d, s.days(d)]), (v) => save({ ...st, idle_model_days: v }), s.setIdleDays)}
        </Row>
        <Row title={s.setGrowthNotify}>
          {sizeSelect(
            st.growth_notify_bytes,
            [[0, s.growthOff], [GB, "1 GB"], [2 * GB, "2 GB"], [5 * GB, "5 GB"], [10 * GB, "10 GB"]],
            (v) => save({ ...st, growth_notify_bytes: v }),
            s.setGrowthNotify,
          )}
        </Row>
      </Section>

      <Section icon="update" title={s.setAppUpdate}>
        {p.portable ? (
          <Row title={s.portableNoUpdate}>
            <Button variant="text" small icon="open_in_new" onClick={p.onOpenReleases}>
              {s.portableReleases}
            </Button>
          </Row>
        ) : (
          <>
            <SwitchRow title={s.setAutoUpdateCheck} checked={st.auto_update_check} onChange={(v) => save({ ...st, auto_update_check: v })} />
            <Row title={s.checkUpdate}>
              <Button variant="outlined" small disabled={p.checkingUpdate} onClick={p.onCheckUpdate}>
                {p.checkingUpdate ? s.checkingUpdate : s.checkUpdate}
              </Button>
            </Row>
          </>
        )}
      </Section>

      <Section icon="rule" title={s.setRules} hint={s.setRulesBody}>
        <Row title={s.setUpdate} desc={st.rules_updated_at ? s.updatedAt(new Date(st.rules_updated_at).toLocaleString()) : s.neverUpdated}>
          <Button variant="outlined" small disabled={p.updatingRules} onClick={p.onUpdateRules}>
            {p.updatingRules ? s.updating : s.updateNow}
          </Button>
        </Row>
        <Row title={s.setUpdateBody}>
          <input
            className="wide md-ltr"
            aria-label={s.setUpdate}
            value={st.rules_update_url}
            onChange={(e) => p.edit({ ...st, rules_update_url: e.target.value })}
            onBlur={() => save(st)}
          />
        </Row>
        <Row title={s.setCustomDir} desc={<span className="md-ltr">{st.custom_rules_dir ?? s.setCustomDirBody}</span>}>
          <Button variant="text" small icon="folder_open" onClick={p.onPickRulesDir}>
            {s.chooseFolder}
          </Button>
          {st.custom_rules_dir && (
            <Button variant="text" small onClick={p.onClearRulesDir}>
              {s.clearFolder}
            </Button>
          )}
        </Row>
        {p.library && p.library.errors.length > 0 && (
          <div className="banner error set-errors">
            {s.setRuleErrors}
            <ul className="plain">
              {p.library.errors.map(([path, e]) => (
                <li key={path}>
                  {path}: {e}
                </li>
              ))}
            </ul>
          </div>
        )}
        {p.library && (
          <ul className="set-rules">
            {p.library.rules.map((r) => (
              <li key={r.id}>
                <span className="tool">{r.tool}</span>
                <span className="name">
                  {coreLang(p.lang) === "zh" ? r.name_zh : r.name_en} <span className="id md-ltr">· {r.id}</span>
                </span>
                <span className={`md-safety ${r.safety}`}>{s.safety[r.safety]}</span>
                <Switch checked={r.enabled} onChange={(v) => p.onToggleRule(r.id, v)}>
                  <span className="md-visually-hidden">{r.id}</span>
                </Switch>
              </li>
            ))}
          </ul>
        )}
      </Section>
    </section>
  );
}
