import { LANGS, type LangPref } from "./i18n";

/** Language picker. Self-contained so the redesign can move it anywhere. */
export function LanguageSelect({
  value,
  onChange,
  label,
  autoLabel,
}: {
  value: LangPref;
  onChange: (v: LangPref) => void;
  label: string;
  autoLabel: string;
}) {
  return (
    <select className="lang-select" aria-label={label} title={label} value={value} onChange={(e) => onChange(e.target.value as LangPref)}>
      <option value="auto">🌐 {autoLabel}</option>
      {LANGS.map((l) => (
        <option key={l.code} value={l.code} lang={l.code}>
          {l.name}
        </option>
      ))}
    </select>
  );
}
