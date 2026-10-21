import { invoke } from "@tauri-apps/api/core";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { coreLang, humanSize, relativeDate, t, type Lang } from "./i18n";
import type { Unrecognized } from "./types";

/// "Unrecognized large folders": profile folders no rule claims, with a
/// one-click prefilled GitHub issue so the user can tell us what they are.
export function UnrecognizedPanel({ lang, onError }: { lang: Lang; onError: (msg: string) => void }) {
  const s = t(lang);
  const [items, setItems] = useState<Unrecognized[] | null>(null);
  const [busy, setBusy] = useState(false);

  async function find() {
    setBusy(true);
    try {
      setItems(await invoke<Unrecognized[]>("unrecognized"));
    } catch (e) {
      onError(String(e));
    }
    setBusy(false);
  }

  async function suggest(entry: Unrecognized) {
    try {
      const url = await invoke<string>("suggest_rule_url", { entry, lang: coreLang(lang) });
      await openUrl(url);
    } catch (e) {
      onError(String(e));
    }
  }

  return (
    <div className="unrec" title={s.unrecHint}>
      <div className="unrec-head">
        <strong>{s.unrecTitle}</strong>
        <button className="ghost" disabled={busy} onClick={find}>
          {busy ? s.unrecFinding : s.unrecFind}
        </button>
      </div>
      {items && items.length === 0 && <p className="muted">{s.unrecNone}</p>}
      {items && items.length > 0 && (
        <>
          <p className="muted">{s.unrecHint}</p>
          <table className="log">
            <tbody>
              {items.map((u) => (
                <tr key={u.path}>
                  <td title={u.path}>{u.name}</td>
                  <td className="size">{humanSize(u.bytes)}</td>
                  <td className="muted">{s.files(u.files)}</td>
                  <td className="muted">
                    {s.unrecNewest} {relativeDate(u.newest, lang)}
                  </td>
                  <td>
                    <button className="link" onClick={() => revealItemInDir(u.path)}>{s.openFolder}</button>
                    <button className="link" onClick={() => suggest(u)}>{s.unrecSuggest}</button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
    </div>
  );
}
