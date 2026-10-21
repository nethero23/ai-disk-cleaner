// Dev-only preview of the v0.8 components: `npm run dev`, then open http://localhost:1420/ui-gallery.html
// Not part of the production build (vite only bundles index.html).
import { useState } from "react";
import ReactDOM from "react-dom/client";
import {
  Avatar, Button, Card, Checkbox, Dialog, Fab, FilterChip, FloatingToolbar, Icon, IconButton, LinearProgress,
  ListItem, NavigationRail, SafetyChip, SearchBar, SegmentedButton, SnackbarHost, Switch, Tabs, Tooltip, useSnackbar,
} from ".";

type Dest = "clean" | "space" | "history" | "settings";
const ITEMS = [
  { id: "a", tool: "Ollama", name: "模型分块缓存", size: "5.4 GB", level: "safe" as const, sub: "最后使用 3 天前" },
  { id: "b", tool: "Hugging Face", name: "下载临时文件", size: "2.7 GB", level: "safe" as const, sub: "最后使用 12 天前" },
  { id: "c", tool: "pip", name: "wheel 缓存", size: "1.9 GB", level: "redownload" as const, sub: "最后使用 1 天前" },
  { id: "d", tool: "Claude Code", name: "对话记录", size: "0.8 GB", level: "review" as const, sub: "最后使用 今天" },
];
const LABEL = { safe: "安全", redownload: "可重下", review: "需确认", inuse: "正在使用" };

function Gallery() {
  const [dest, setDest] = useState<Dest>("clean");
  const [theme, setTheme] = useState<"system" | "light" | "dark">("system");
  const [sel, setSel] = useState<Set<string>>(new Set(["a", "b"]));
  const [q, setQ] = useState("");
  const [group, setGroup] = useState<"category" | "tool">("category");
  const [tab, setTab] = useState<"overview" | "big">("overview");
  const [onlySafe, setOnlySafe] = useState(true);
  const [tray, setTray] = useState(true);
  const [confirm, setConfirm] = useState(false);
  const snack = useSnackbar();

  const applyTheme = (t: typeof theme) => {
    setTheme(t);
    if (t === "system") document.documentElement.removeAttribute("data-theme");
    else document.documentElement.setAttribute("data-theme", t);
  };
  const toggle = (id: string) => {
    const n = new Set(sel);
    if (n.has(id)) n.delete(id);
    else n.add(id);
    setSel(n);
  };

  return (
    <div className="md-root" style={{ display: "grid", gridTemplateColumns: "88px 1fr", height: "100vh" }}>
      <NavigationRail<Dest>
        fab={<Fab icon="radar" label="扫描" onClick={() => snack({ message: "开始扫描…" })} />}
        items={[
          { value: "clean", label: "清理", icon: "cleaning_services" },
          { value: "space", label: "空间", icon: "donut_large" },
          { value: "history", label: "记录", icon: "history" },
        ]}
        bottom={[{ value: "settings", label: "设置", icon: "settings", badge: true }]}
        value={dest}
        onChange={setDest}
      />
      <main style={{ background: "var(--md-surface)", borderRadius: "16px 0 0 0", overflow: "auto", padding: "16px 24px 24px", display: "grid", gap: 16, alignContent: "start" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <h1 style={{ font: "var(--md-title-l)", margin: 0, flex: 1 }}>组件预览</h1>
          <SegmentedButton small label="主题" value={theme} onChange={applyTheme} options={[{ value: "system", label: "跟随" }, { value: "light", label: "浅色" }, { value: "dark", label: "深色" }]} />
        </div>

        <Card>
          <span style={{ font: "var(--md-label-m)", color: "var(--md-on-surface-variant)" }}>可以释放</span>
          <span style={{ font: "var(--md-display-m)" }}>38.6 GB</span>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            <Button icon="wand_stars">清理 21 个安全项 · 12.4 GB</Button>
            <Button variant="tonal" icon="drive_file_move">搬到 D 盘</Button>
            <Button variant="outlined" icon="folder_open">打开文件夹</Button>
            <Button variant="text">逐项查看</Button>
            <Button variant="danger" icon="delete">移入回收站</Button>
            <Button disabled>不可用</Button>
          </div>
          <LinearProgress value={0.42} label="扫描进度" />
          <LinearProgress label="正在统计" />
        </Card>

        <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
          <div style={{ flex: "1 1 280px" }}><SearchBar value={q} onChange={setQ} placeholder="搜索工具、名称或路径" clearLabel="清除" /></div>
          <FilterChip selected={onlySafe} onClick={() => setOnlySafe(!onlySafe)}>安全 21</FilterChip>
          <FilterChip selected={false} icon="block">隐藏正在使用的</FilterChip>
          <SegmentedButton small label="分组" value={group} onChange={setGroup} options={[{ value: "category", label: "按类别" }, { value: "tool", label: "按工具" }]} />
          <Tooltip text="按大小排序"><IconButton icon="sort" label="排序" /></Tooltip>
        </div>

        <ul className="md-list">
          <li className="md-list-sub">
            <Checkbox label="全选本地模型" checked={ITEMS.every((i) => sel.has(i.id))} indeterminate={sel.size > 0} onChange={(v) => setSel(v ? new Set(ITEMS.map((i) => i.id)) : new Set())} />
            本地模型
          </li>
          {ITEMS.map((it) => (
            <ListItem
              key={it.id}
              selected={sel.has(it.id)}
              onOpen={() => snack({ message: `打开 ${it.name} 的详情` })}
              lead={<Checkbox label={it.name} checked={sel.has(it.id)} onChange={() => toggle(it.id)} />}
              avatar={<Avatar text={it.tool} />}
              headline={<>{it.name}<SafetyChip level={it.level} label={LABEL[it.level]} /></>}
              supporting={`${it.tool} · ${it.sub}`}
              trailing={it.size}
            />
          ))}
        </ul>

        <Tabs label="空间" value={tab} onChange={setTab} tabs={[{ value: "overview", label: "概览", icon: "donut_large" }, { value: "big", label: "大文件", icon: "description" }]} />

        <Card variant="outlined">
          <Switch checked={tray} onChange={setTray}>关闭窗口时缩到托盘，不退出</Switch>
          <div style={{ display: "flex", gap: 8 }}>
            <SafetyChip level="safe" label="安全" /><SafetyChip level="redownload" label="可重下" /><SafetyChip level="review" label="需确认" /><SafetyChip level="inuse" label="正在使用" />
            <Icon name="info" />
          </div>
        </Card>

        {sel.size > 0 && (
          <FloatingToolbar label="已选操作">
            <span>已选 <b>{sel.size} 项</b></span>
            <Button variant="text" onClick={() => setSel(new Set())}>取消选择</Button>
            <Button icon="delete" onClick={() => setConfirm(true)}>移入回收站</Button>
          </FloatingToolbar>
        )}
      </main>

      <Dialog
        open={confirm}
        icon="delete"
        title={`把 ${sel.size} 项移入回收站？`}
        onClose={() => setConfirm(false)}
        actions={
          <>
            <Button variant="text" onClick={() => setConfirm(false)}>取消</Button>
            <Button variant="danger" onClick={() => { setConfirm(false); setSel(new Set()); snack({ message: "已释放 8.1 GB", action: "撤销", onAction: () => snack({ message: "已还原" }) }); }}>移入回收站</Button>
          </>
        }
      >
        <p style={{ margin: 0 }}>可以在「记录」里一键撤销，或者从回收站还原。</p>
      </Dialog>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <SnackbarHost closeLabel="关闭">
    <Gallery />
  </SnackbarHost>,
);
