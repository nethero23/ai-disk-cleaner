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
    exportFirst: (n: number) => `清理前先把 ${n} 项对话记录打包成 zip 备份`,
    exportPick: "选择保存位置…",
    exportTo: (p: string) => `保存到：${p}`,
    exporting: "正在打包对话记录…",
    exported: (files: number, zip: string) => `已导出 ${files} 个文件到 ${zip}`,
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
    move: "搬到其他盘…",
    moveTitle: "搬家",
    moveBody: (name: string, size: string, dest: string) =>
      `将把「${name}」（${size}）完整复制到 ${dest}，核对无误后删除原位置，并让对应工具指向新位置。原位置不会先删。`,
    moveWhole: (p: string) => `这个工具通过环境变量找数据，只能整个目录一起搬：${p}`,
    moveEnv: (v: string) => `会设置用户环境变量 ${v}`,
    moveJunction: "会在原位置留一个目录联接，程序无需改设置",
    moving: "正在复制…",
    moveDone: (to: string) => `已搬到 ${to}`,
    moveNote: { "restart Ollama": "请重启 Ollama", "open a new terminal for the variable to take effect": "新开的终端才会读到新变量" } as Record<string, string>,
    movedTitle: "已搬走的目录",
    movedBody: "这些目录被搬到了其他盘。搬回来会先复制回原位置并校验，再撤销环境变量或联接，最后删除搬走的那份。",
    moveBack: "搬回来",
    movingBack: "正在搬回…",
    movedBack: (to: string) => `已搬回 ${to}`,
    compact: "压缩虚拟磁盘…",
    compactTitle: "压缩虚拟磁盘",
    compactBody: "WSL 的虚拟磁盘只会变大：在 Linux 里删文件（或 docker system prune）并不会把空间还给 Windows。压缩会把磁盘收缩到实际占用的大小，里面的数据不受影响。",
    compactSteps: "会先执行 wsl --shutdown（关闭所有 WSL 发行版和 Docker Desktop 的容器），然后弹出管理员权限确认（UAC）运行 diskpart。请先保存 WSL 里正在进行的工作。",
    compactNone: "这里没有找到 .vhdx 文件。",
    compactTip: "提示：先在 Linux 里清掉不要的文件（Docker 用 docker system prune），压缩效果才明显。",
    compacting: "正在压缩，可能需要几分钟…",
    compactDone: (before: string, after: string, freed: string) => `${before} → ${after}，释放 ${freed}`,
    compactNoGain: "大小没有变化。可能是磁盘里没有可回收空间，或者 diskpart 没能拿到管理员权限。",
    findDups: "查找重复模型",
    findingDups: "正在比对模型文件…",
    dupsTitle: (n: number, size: string) => `发现 ${n} 组重复模型文件，可释放 ${size}`,
    dupsNone: "没有发现跨工具重复的模型文件。",
    dupsHow: "合并后多个工具共用同一份数据（硬链接），每个工具都照常工作；删除其中一个不影响其他。",
    merge: "合并为硬链接（保留第一个）",
    mergeDiffVolume: "不在同一个盘，无法硬链接",
    merged: (size: string) => `已合并，释放 ${size}`,
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
    exportFirst: (n: number) => `Back up ${n} chat-history item(s) to a zip before cleaning`,
    exportPick: "Choose where to save…",
    exportTo: (p: string) => `Save to: ${p}`,
    exporting: "Packing chat history…",
    exported: (files: number, zip: string) => `Exported ${files} files to ${zip}`,
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
    move: "Move to another drive…",
    moveTitle: "Move",
    moveBody: (name: string, size: string, dest: string) =>
      `"${name}" (${size}) will be copied to ${dest}, verified, then removed from its old place, and the tool pointed at the new location. The original is not touched until the copy is verified.`,
    moveWhole: (p: string) => `This tool finds its data through an environment variable, so the whole folder moves: ${p}`,
    moveEnv: (v: string) => `Sets user environment variable ${v}`,
    moveJunction: "Leaves a directory junction at the old path, so nothing needs reconfiguring",
    moving: "Copying…",
    moveDone: (to: string) => `Moved to ${to}`,
    moveNote: { "restart Ollama": "Restart Ollama", "open a new terminal for the variable to take effect": "Open a new terminal for the variable to take effect" } as Record<string, string>,
    movedTitle: "Moved folders",
    movedBody: "These folders were moved to another drive. Moving back copies them to the original place, verifies, undoes the variable or junction, then removes the moved copy.",
    moveBack: "Move back",
    movingBack: "Moving back…",
    movedBack: (to: string) => `Moved back to ${to}`,
    compact: "Compact virtual disk…",
    compactTitle: "Compact virtual disk",
    compactBody: "A WSL virtual disk only grows: deleting files inside Linux (or docker system prune) does not give the space back to Windows. Compacting shrinks the disk to what is actually used; the data inside is untouched.",
    compactSteps: "This runs wsl --shutdown first (stops every WSL distribution and Docker Desktop's containers), then asks for administrator rights (UAC) to run diskpart. Save any work in WSL first.",
    compactNone: "No .vhdx file found here.",
    compactTip: "Tip: delete what you do not need inside Linux first (docker system prune for Docker) so there is something to reclaim.",
    compacting: "Compacting, this can take a few minutes…",
    compactDone: (before: string, after: string, freed: string) => `${before} → ${after}, freed ${freed}`,
    compactNoGain: "The size did not change. Either nothing was reclaimable or diskpart did not get administrator rights.",
    findDups: "Find duplicate models",
    findingDups: "Comparing model files…",
    dupsTitle: (n: number, size: string) => `${n} groups of duplicate model files, ${size} reclaimable`,
    dupsNone: "No duplicate model files across tools.",
    dupsHow: "Merging makes the tools share one copy (hard link); every tool keeps working, and deleting one copy does not affect the others.",
    merge: "Merge into hard links (keep first)",
    mergeDiffVolume: "On different drives; cannot hard-link",
    merged: (size: string) => `Merged, freed ${size}`,
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
