// Vite-only frontend fixture: /tools/ui-smoke.html. Never calls native commands
// or registries. This checks UI behavior, NOT the native IPC implementation.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

const defaults = {
  excluded_paths: [], disabled_rules: [], custom_rules_dir: null,
  rules_update_url: "https://example.invalid/rules", rules_updated_at: null,
  min_item_bytes: 0, lang: "zh", close_to_tray: false, autostart: false,
  scheduled_scan_hours: 0, notify_min_bytes: 1073741824, last_background_scan: null,
  auto_clean_safe: false, auto_update_check: false, scan_drives: [],
  idle_model_days: 90, growth_notify_bytes: 2147483648, onboarded: true,
  mcp_version_check: true,
};
const saved = sessionStorage.getItem("clearwisp-smoke-settings");
const state = window.__clearwispSmoke = {
  settings: saved ? JSON.parse(saved) : defaults,
  calls: [],
  mode: "success", // Set to "offline" or "slow" for failure/late-reply checks.
};
const selectionItems = [
  ["safe", "Selection Safe", 4096, "safe", null],
  ["review", "Selection Review", 8192, "review", null],
  ["busy", "Selection In Use", 16384, "safe", "fixture.exe"],
  ["blocked", "Selection All Busy", 32768, "safe", "fixture.exe"],
].map(([id, name, size_bytes, safety, in_use_by]) => ({
  id, rule_id: `fixture.${id}`, name, size_bytes, safety, in_use_by,
  tool: "Selection Fixture", category: id === "blocked" ? "ai_agent" : "package_cache", path: `C:/fixtures/${id}`,
  file_count: 1, last_used: null, consequence: "Synthetic fixture only",
  detail: null, auto_clean: true,
}));
const inventory = {
  tools: [{
    id: "cursor", name: "Cursor (fixture)", present: true, verified: true, running: false,
    configs: [{ tool: "cursor", path: "C:/fixtures/mcp.json", exists: true, bytes: 100, error: null, servers: 1 }],
    mcp: [{
      tool: "cursor", config_path: "C:/fixtures/mcp.json", scope: "user",
      name: "Demo MCP", transport: "stdio", command: "npx", args: ["demo-package@1.0.0"],
      url: null, package: "demo-package", version: "1.0.0", identity: "npm:demo-package",
      env_keys: [], secrets: [], disabled: false,
    }],
    skills: [], plugins: [], extensions: [],
  }],
  analysis: { findings: [], mcp_total: 1, mcp_unique: 1, skills_total: 0, skills_unique: 0, reclaimable_bytes: 0 },
  computed_at: "2026-10-22T00:00:00Z",
};
mockWindows("main");
mockIPC(async (cmd, args = {}) => {
  state.calls.push({ cmd, args: structuredClone(args) });
  switch (cmd) {
    case "get_settings": return structuredClone(state.settings);
    case "save_settings":
      state.settings = structuredClone(args.settings);
      sessionStorage.setItem("clearwisp-smoke-settings", JSON.stringify(state.settings));
      return;
    case "rules": return { rules: [], errors: [] };
    case "is_portable": return true;
    case "set_lang": return;
    case "scan": return {
      items: structuredClone(selectionItems), total_bytes: 61440, safe_bytes: 53248,
      review_bytes: 8192, redownload_bytes: 0, started_at: null, finished_at: null,
      unreadable: [], timings: [], cancelled: false,
    };
    case "growth_report": return [];
    case "relocatable": case "compactable": case "exportable": case "drives": return [];
    case "ai_inventory": return structuredClone(inventory);
    case "ai_version_check": {
      if (!state.settings.mcp_version_check) throw new Error("UI requested a disabled version check");
      const mode = state.mode;
      if (mode === "slow") await new Promise((resolve) => setTimeout(resolve, 1500));
      return args.identities.map((identity) => ({
        identity, latest: mode === "offline" ? null : "2.0.0",
        error: mode === "offline" ? "simulated offline" : null,
      }));
    }
    default: throw new Error(`Smoke fixture refuses command: ${cmd}`);
  }
}, { shouldMockEvents: true });

await import("/src/main.tsx");
