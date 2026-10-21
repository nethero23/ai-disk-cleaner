import type { Category, SafetyLevel } from "./types";

export type Lang = "zh" | "en";

const strings = {
  zh: {
    appName: "AI Disk Cleaner",
    tagline: "先看清每一 GB 是什么，再决定删不删",
    scan: "开始扫描",
    rescan: "重新扫描",
    scanning: "正在扫描…",
    found: (n: number, size: string) => `找到 ${n} 项，共 ${size}`,
    selected: (n: number, size: string) => `已选 ${n} 项，${size}`,
    clean: "清理所选",
    cleaning: "正在移入回收站…",
    confirmTitle: "确认清理",
    confirmBody: (n: number, size: string) =>
      `将把 ${n} 项（${size}）移入回收站。可以从回收站或本软件的清理记录还原。`,
    confirmReview: (n: number) => `其中 ${n} 项标记为「需确认」，可能包含你的对话记录或正在使用的环境。`,
    cancel: "取消",
    confirm: "移入回收站",
    done: (size: string) => `已释放 ${size}`,
    failed: (n: number) => `${n} 项未能删除（可能正在使用）`,
    emptyTitle: "还没有扫描",
    emptyBody: "点击「开始扫描」，我们会检查 60 多个 AI 工具和系统的缓存位置。只读取，不删除。",
    nothing: "没有发现可清理的内容。",
    lastUsed: "最后使用",
    never: "未知",
    files: (n: number) => `${n} 个文件`,
    openFolder: "打开所在文件夹",
    unreadable: (n: number) => `${n} 个位置无法读取（权限不足）`,
    selectAllSafe: "只选安全项",
    selectNone: "全不选",
    safety: {
      safe: "安全",
      redownload: "可重下",
      review: "需确认",
    } as Record<SafetyLevel, string>,
    safetyHint: {
      safe: "纯缓存，工具会自动重建",
      redownload: "删了能恢复，但要重新下载",
      review: "含用户数据或可能还在用，请看清再删",
    } as Record<SafetyLevel, string>,
    category: {
      models: "本地模型",
      package_cache: "包缓存",
      python_env: "Python 环境",
      ai_editor: "AI 编辑器",
      ai_agent: "AI 编程助手",
      ai_desktop_app: "AI 桌面应用",
      browser: "浏览器",
      container: "容器 / WSL",
      project_artifacts: "项目产物",
      gpu: "GPU 缓存",
      system: "系统",
    } as Record<Category, string>,
    history: "清理记录",
    backToScan: "返回扫描",
    historyEmpty: "还没有清理过任何东西。",
    historyTotal: (n: number, size: string) => `共清理 ${n} 项，释放 ${size}。文件在回收站里，可以从那里还原。`,
    inUse: (p: string) => `${p} 正在运行，关闭后才能清理`,
    language: "English",
  },
  en: {
    appName: "AI Disk Cleaner",
    tagline: "See what every GB is before you decide",
    scan: "Scan",
    rescan: "Rescan",
    scanning: "Scanning…",
    found: (n: number, size: string) => `${n} items, ${size}`,
    selected: (n: number, size: string) => `${n} selected, ${size}`,
    clean: "Clean selected",
    cleaning: "Moving to Recycle Bin…",
    confirmTitle: "Confirm clean",
    confirmBody: (n: number, size: string) =>
      `${n} items (${size}) will be moved to the Recycle Bin. You can restore them from there or from this app's history.`,
    confirmReview: (n: number) => `${n} of them are marked "Review" and may hold chat history or an environment still in use.`,
    cancel: "Cancel",
    confirm: "Move to Recycle Bin",
    done: (size: string) => `Freed ${size}`,
    failed: (n: number) => `${n} items could not be deleted (maybe in use)`,
    emptyTitle: "No scan yet",
    emptyBody: "Press Scan to check 60+ cache locations of AI tools and Windows. Read-only.",
    nothing: "Nothing to clean.",
    lastUsed: "Last used",
    never: "unknown",
    files: (n: number) => `${n} files`,
    openFolder: "Open folder",
    unreadable: (n: number) => `${n} locations unreadable (permissions)`,
    selectAllSafe: "Select safe only",
    selectNone: "Select none",
    safety: {
      safe: "Safe",
      redownload: "Re-download",
      review: "Review",
    } as Record<SafetyLevel, string>,
    safetyHint: {
      safe: "Pure cache; the tool rebuilds it",
      redownload: "Recoverable, but must be downloaded again",
      review: "Holds user data or may still be in use",
    } as Record<SafetyLevel, string>,
    category: {
      models: "Local models",
      package_cache: "Package caches",
      python_env: "Python environments",
      ai_editor: "AI editors",
      ai_agent: "AI coding agents",
      ai_desktop_app: "AI desktop apps",
      browser: "Browsers",
      container: "Containers / WSL",
      project_artifacts: "Project artifacts",
      gpu: "GPU caches",
      system: "System",
    } as Record<Category, string>,
    history: "History",
    backToScan: "Back to scan",
    historyEmpty: "Nothing has been cleaned yet.",
    historyTotal: (n: number, size: string) => `${n} items cleaned, ${size} freed. Files are in the Recycle Bin and can be restored from there.`,
    inUse: (p: string) => `${p} is running; close it to clean this`,
    language: "中文",
  },
};

export type Strings = (typeof strings)["zh"];

export function t(lang: Lang): Strings {
  return strings[lang] as Strings;
}

export function humanSize(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = bytes;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return i === 0 ? `${bytes} B` : `${v.toFixed(1)} ${units[i]}`;
}

export function relativeDate(iso: string | null, lang: Lang): string {
  if (!iso) return t(lang).never;
  const days = Math.floor((Date.now() - new Date(iso).getTime()) / 86_400_000);
  if (days <= 0) return lang === "zh" ? "今天" : "today";
  if (days < 30) return lang === "zh" ? `${days} 天前` : `${days}d ago`;
  if (days < 365) return lang === "zh" ? `${Math.floor(days / 30)} 个月前` : `${Math.floor(days / 30)}mo ago`;
  return lang === "zh" ? `${Math.floor(days / 365)} 年前` : `${Math.floor(days / 365)}y ago`;
}
