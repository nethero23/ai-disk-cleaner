/// Light / dark / follow the OS. Stored per machine in localStorage and
/// applied as <html data-theme>, which src/ui/tokens.css reads.
export type Theme = "system" | "light" | "dark";

export function loadTheme(): Theme {
  try {
    const v = localStorage.getItem("theme");
    if (v === "light" || v === "dark") return v;
  } catch {
    /* storage blocked: follow the OS */
  }
  return "system";
}

export function applyTheme(theme: Theme) {
  if (theme === "system") document.documentElement.removeAttribute("data-theme");
  else document.documentElement.setAttribute("data-theme", theme);
  try {
    localStorage.setItem("theme", theme);
  } catch {
    /* not persisted, still applied */
  }
}
