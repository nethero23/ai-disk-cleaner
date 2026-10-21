// Mirrors crates/cleaner-core/src/model.rs. Keep the two in step.

export type SafetyLevel = "safe" | "redownload" | "review";

export type Category =
  | "models"
  | "package_cache"
  | "python_env"
  | "ai_editor"
  | "ai_agent"
  | "ai_desktop_app"
  | "browser"
  | "container"
  | "project_artifacts"
  | "gpu"
  | "system";

export interface ScanItem {
  id: string;
  rule_id: string;
  tool: string;
  category: Category;
  name: string;
  path: string;
  size_bytes: number;
  file_count: number;
  last_used: string | null;
  safety: SafetyLevel;
  consequence: string;
  detail: string | null;
  in_use_by: string | null;
}

export interface ScanReport {
  items: ScanItem[];
  total_bytes: number;
  safe_bytes: number;
  redownload_bytes: number;
  review_bytes: number;
  started_at: string | null;
  finished_at: string | null;
  unreadable: string[];
}

export interface CleanRecord {
  at: string;
  rule_id: string;
  name: string;
  path: string;
  size_bytes: number;
  safety: SafetyLevel;
  ok: boolean;
  error: string | null;
}

export interface CleanSummary {
  freed_bytes: number;
  records: CleanRecord[];
}

export interface RelocateRecord {
  at: string;
  rule_id: string;
  from: string;
  to: string;
  bytes: number;
  redirect: { kind: "env_var"; value: string } | { kind: "junction" };
  active: boolean;
  previous_env?: string | null;
}

export interface RelocateResult {
  from: string;
  to: string;
  bytes: number;
  files: number;
  redirect: { kind: "env_var"; value: string } | { kind: "junction" };
  note: string;
}

export interface DupFile {
  path: string;
  tool: string;
  item_id: string;
}

export interface DuplicateGroup {
  size_bytes: number;
  files: DupFile[];
  reclaimable_bytes: number;
  same_volume: boolean;
}

export interface Vhdx {
  path: string;
  size_bytes: number;
}

export interface CompactResult {
  path: string;
  before: number;
  after: number;
}

export interface ExportResult {
  zip_path: string;
  files: number;
  bytes: number;
}

export interface Settings {
  excluded_paths: string[];
  disabled_rules: string[];
  custom_rules_dir: string | null;
  rules_update_url: string;
  rules_updated_at: string | null;
  min_item_bytes: number;
  lang: string;
  close_to_tray: boolean;
  autostart: boolean;
  scheduled_scan_hours: number;
  notify_min_bytes: number;
  last_background_scan: string | null;
}

export interface RuleInfo {
  id: string;
  tool: string;
  name_zh: string;
  name_en: string;
  safety: SafetyLevel;
  enabled: boolean;
}

export interface RuleLibrary {
  rules: RuleInfo[];
  errors: [string, string][];
}

export interface RulesUpdate {
  files: number;
  rules: number;
  updated_at: string;
}

export interface PreviewEntry {
  rel: string;
  path: string;
  size_bytes: number;
  modified: string | null;
}

export interface Preview {
  files: number;
  dirs: number;
  largest: PreviewEntry[];
}

export interface Drive {
  path: string;
  name: string;
  total_bytes: number;
  free_bytes: number;
}
