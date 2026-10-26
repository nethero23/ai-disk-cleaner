/** Order persisted language/settings writes and ignore obsolete rule responses. */
export function createLanguageSync<T>(
  initialLanguage: string,
  setLanguage: (language: string) => Promise<unknown>,
  loadRules: (language: string) => Promise<T>,
  publishRules: (rules: T) => void,
) {
  let language = initialLanguage;
  let generation = 0;
  let ruleRequest = 0;
  let writes: Promise<unknown> = Promise.resolve();

  function write<R>(action: (language: string) => Promise<R>): Promise<R> {
    // Snapshot at enqueue time, so a settings write cannot preempt a later
    // set_lang and cause its native tray relabel to be skipped as "unchanged".
    const requestedLanguage = language;
    const result = writes.then(() => action(requestedLanguage));
    // A failed write must not prevent a subsequent language change or retry.
    writes = result.catch(() => {});
    return result;
  }

  async function refresh() {
    const request = ++ruleRequest;
    const version = generation;
    const rules = await loadRules(language);
    if (version === generation && request === ruleRequest) publishRules(rules);
  }

  async function select(next: string) {
    language = next;
    const version = ++generation;
    await write(() => version === generation ? setLanguage(next) : Promise.resolve());
    if (version === generation) await refresh();
  }

  return { select, refresh, write };
}

/** Legacy reports only supported zh/en; new reports carry the scan's snapshot. */
export function reportMatchesLanguage(report: { ui_lang?: string | null }, language: string) {
  return report.ui_lang ? report.ui_lang === language : language === "en" || language === "zh";
}
