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
  auto_clean: boolean;
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
  timings: [string, number][];
  cancelled: boolean;
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
  batch?: string;
}

export type Whereabouts = "present" | "in_bin" | "gone";

export interface RestoreStatus {
  path: string;
  state: Whereabouts;
}

export interface RestoreSummary {
  restored: number;
  restored_bytes: number;
  results: { path: string; ok: boolean; error: string | null }[];
}

export interface CleanSummary {
  freed_bytes: number;
  records: CleanRecord[];
  log_error?: string | null;
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
  auto_clean_safe: boolean;
  auto_update_check: boolean;
  scan_drives: string[];
  idle_model_days: number;
  growth_notify_bytes: number;
  onboarded: boolean;
  mcp_version_check: boolean;
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

export interface OverviewEntry {
  name: string;
  path: string;
  bytes: number;
  files: number;
  children: OverviewEntry[];
}

export interface Overview {
  root: string;
  total_bytes: number;
  free_bytes: number;
  scanned_bytes: number;
  entries: OverviewEntry[];
  computed_at: string;
  cancelled: boolean;
}

export interface Drive {
  path: string;
  name: string;
  total_bytes: number;
  free_bytes: number;
  system: boolean;
}

export interface Unrecognized {
  name: string;
  path: string;
  bytes: number;
  files: number;
  newest: string | null;
}

export interface DeepTask {
  id: string;
  title: string;
  body: string;
  consequence: string;
  estimate_bytes: number | null;
  needs_analysis: boolean;
  applicable: boolean;
}

export interface DeepAnalysis {
  winsxs_actual_bytes: number | null;
  winsxs_reclaimable_bytes: number | null;
  winsxs_cleanup_recommended: boolean | null;
  shadow_used_bytes: number | null;
  shadow_count: number | null;
}

export interface DeepResult {
  id: string;
  ok: boolean;
  message: string;
}

export interface DeepSummary {
  results: DeepResult[];
  freed_bytes: number;
}

export interface Growth {
  tool: string;
  now_bytes: number;
  before_bytes: number;
  delta_bytes: number;
  since: string;
}

// ---- AI tool configuration inventory (crates/cleaner-core/src/aiconfig) ----

export interface AiConfigFile {
  tool: string;
  path: string;
  exists: boolean;
  bytes: number;
  error: string | null;
  servers: number;
}

export interface AiSecretHit {
  location: string;
  masked: string;
}

export interface AiMcpServer {
  tool: string;
  config_path: string;
  scope: string;
  name: string;
  transport: "stdio" | "remote";
  command: string | null;
  args: string[];
  url: string | null;
  package: string | null;
  version: string | null;
  identity: string;
  env_keys: string[];
  secrets: AiSecretHit[];
  disabled: boolean;
}

export interface AiSkill {
  tool: string;
  name: string;
  path: string;
  bytes: number;
  files: number;
  modified: string | null;
  hash: string;
  description: string | null;
  link: boolean;
}

export interface AiPlugin {
  tool: string;
  name: string;
  version: string | null;
  path: string;
  bytes: number;
  stale_versions: string[];
  stale_bytes: number;
}

export type AiExtension = AiPlugin;

export interface AiToolInventory {
  id: string;
  name: string;
  present: boolean;
  verified: boolean;
  running: boolean;
  configs: AiConfigFile[];
  mcp: AiMcpServer[];
  skills: AiSkill[];
  plugins: AiPlugin[];
  extensions: AiExtension[];
}

export type AiFindingKind =
  | "mcp_duplicate"
  | "mcp_version_conflict"
  | "skill_copies"
  | "skill_diverged"
  | "mcp_broken"
  | "secret_exposed"
  | "stale_versions";

export interface AiFinding {
  kind: AiFindingKind;
  subject: string;
  tools: string[];
  paths: string[];
  detail: string;
  reclaimable_bytes: number;
}

export interface AiAnalysis {
  findings: AiFinding[];
  mcp_total: number;
  mcp_unique: number;
  skills_total: number;
  skills_unique: number;
  reclaimable_bytes: number;
}

export interface AiInventory {
  tools: AiToolInventory[];
  analysis: AiAnalysis;
  computed_at: string | null;
}

export interface AiVersionInfo {
  identity: string;
  latest: string | null;
  error: string | null;
}
