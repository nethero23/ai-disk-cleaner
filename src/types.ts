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
