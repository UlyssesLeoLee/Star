/*
@cypher schema=1 source_sha256=16468e525781b4e6c8c591619b6c75ed8f25fd16ec8e5a616086dfc91ade68da
MERGE (self:File {path:"frontend/src/app/(app)/settings/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/settings/page.tsx::SettingsPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /settings — 6 · 设置 (per 2026-11-14 参考 mockup: Settings.dc.html)
// 左导航 5 组: 账户/组织与权限/集成/扩展/开发者; 主区 = 分组面板,
// 默认 扩展 → MCP 服务器 (官方目录/自定义 + 已连接服务器卡片)
// 深链: ?tab=<panel-id> (兼容旧 profile/apikeys/billing 等)
// =====================================================================
"use client";

import { useEffect, useState } from "react";
import {
  User,
  Key,
  Terminal,
  Users,
  Shield,
  Building2,
  GitBranch,
  Zap,
  Puzzle,
  Server,
  Sparkles,
  Plug,
  Code2,
  Bug,
  Globe,
  Flag,
  Search,
  Check,
} from "lucide-react";
import { clsx } from "clsx";

interface NavItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  href: string;
}

const NAV_GROUPS: { group: string; items: NavItem[] }[] = [
  {
    group: "账户",
    items: [
      { id: "profile", label: "个人资料", icon: <User size={13} />, href: "" },
      { id: "apikeys", label: "凭据与 API Key", icon: <Key size={13} />, href: "/settings/api-keys" },
      { id: "cli-profiles", label: "CLI 配置", icon: <Terminal size={13} />, href: "/settings/cli-profiles" },
    ],
  },
  {
    group: "组织与权限",
    items: [
      { id: "identity", label: "成员与身份", icon: <Users size={13} />, href: "" },
      { id: "permission", label: "角色与权限", icon: <Shield size={13} />, href: "" },
      { id: "tenant", label: "租户", icon: <Building2 size={13} />, href: "" },
    ],
  },
  {
    group: "集成",
    items: [
      { id: "scm", label: "代码托管", icon: <GitBranch size={13} />, href: "" },
      { id: "automation", label: "自动化触发", icon: <Zap size={13} />, href: "" },
    ],
  },
  {
    group: "扩展",
    items: [
      { id: "mod", label: "Mod", icon: <Puzzle size={13} />, href: "" },
      { id: "mcp", label: "MCP 服务器", icon: <Server size={13} />, href: "" },
      { id: "skills", label: "Skills", icon: <Sparkles size={13} />, href: "" },
      { id: "plugins", label: "Plugins", icon: <Plug size={13} />, href: "" },
      { id: "hooks", label: "Hooks", icon: <Plug size={13} />, href: "" },
    ],
  },
  {
    group: "开发者",
    items: [
      { id: "dev-mode", label: "开发者模式", icon: <Code2 size={13} />, href: "/settings/developer" },
      { id: "debug", label: "调试工具", icon: <Bug size={13} />, href: "/settings/developer" },
      { id: "webhooks", label: "API 与 Webhook", icon: <Globe size={13} />, href: "" },
      { id: "flags", label: "Feature Flags", icon: <Flag size={13} />, href: "" },
    ],
  },
];

const MCP_SERVERS = [
  { name: "GitHub", auth: "OAuth", verified: "仓库、Issue、PR 读写", kind: "官方 · 14 个工具", connected: true },
  { name: "PostgreSQL", auth: "连接串", verified: "只读查询与表结构", kind: "官方 · 6 个工具", connected: true },
  { name: "文件系统", auth: "本地", verified: "受限目录读写", kind: "官方 · 8 个工具", connected: true },
  { name: "浏览器自动化", auth: "本地", verified: "打开页面、截图、表单", kind: "官方 · 12 个工具", connected: true },
  { name: "Slack", auth: "OAuth", verified: "频道消息读取与发送", kind: "官方 · 9 个工具", connected: false },
  { name: "Sentry", auth: "Token", verified: "错误事件与发布追踪", kind: "官方 · 7 个工具", connected: false },
];

function McpPanel() {
  const [source, setSource] = useState<"official" | "custom">("official");
  return (
    <div data-testid="settings-panel-mcp">
      <h2 className="text-base font-bold text-ink">MCP 服务器</h2>
      <p className="text-xs text-ink-dim mt-1">官方目录一键安装，或自定义接入任意 MCP 服务器（原 /integration）</p>

      <div className="mt-3 flex gap-1 border-b border-line">
        {(["official", "custom"] as const).map((s) => (
          <button
            key={s}
            type="button"
            onClick={() => setSource(s)}
            className={clsx(
              "px-3 py-2 text-sm border-b-2 -mb-px",
              source === s ? "border-accent text-accent font-semibold" : "border-transparent text-ink-dim",
            )}
          >
            {s === "official" ? "官方目录" : "自定义"}
          </button>
        ))}
      </div>

      <ul className="mt-3 grid gap-2">
        {MCP_SERVERS.map((srv) => (
          <li key={srv.name} data-testid={`settings-mcp-${srv.name}`} className="flex items-center gap-3 px-3.5 py-3 rounded-xl border border-line bg-bg-card">
            <div className="size-9 rounded-lg bg-accent/15 grid place-items-center text-accent shrink-0">
              <Server size={15} />
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold text-ink">{srv.name}</span>
                <span className="text-[10px] text-ink-mute">{srv.kind}</span>
                {srv.connected && (
                  <span className="text-[10px] px-1.5 py-0.5 rounded bg-ok/10 text-ok font-semibold flex items-center gap-0.5">
                    <Check size={10} /> 已连接
                  </span>
                )}
              </div>
              <div className="text-[11px] text-ink-dim mt-0.5">
                {srv.verified} · <span className="font-mono">{srv.auth}</span>
              </div>
            </div>
            <button type="button" className={clsx("text-xs px-2.5 py-1.5 rounded-lg", srv.connected ? "border border-line text-ink-dim" : "bg-accent text-white hover:bg-accent-soft")}>
              {srv.connected ? "管理" : "安装"}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

function StubPanel({ id, label, realHref }: { id: string; label: string; realHref?: string }) {
  return (
    <div data-testid={`settings-panel-${id}`} className="bg-bg-card rounded-xl border border-line p-6 text-center">
      <h2 className="text-sm font-bold text-ink">{label}</h2>
      {realHref ? (
        <>
          <p className="text-xs text-ink-dim mt-1">该面板的完整实现在子路由 {realHref}</p>
          <a href={realHref} data-testid={`settings-panel-${id}-go`} className="inline-flex items-center gap-1.5 mt-3 text-xs px-3 py-2 rounded-lg bg-accent text-white hover:bg-accent-soft">
            前往 {realHref}
          </a>
        </>
      ) : (
        <p className="text-xs text-ink-dim mt-1">该面板待接入（P2 功能缺口，per 缺标比错标）</p>
      )}
    </div>
  );
}

const LEGACY_TAB_MAP: Record<string, { id: string; label: string }> = {
  profile: { id: "profile", label: "个人资料" },
  account: { id: "profile", label: "个人资料" },
  team: { id: "identity", label: "成员与身份" },
  billing: { id: "tenant", label: "租户" },
  apikeys: { id: "apikeys", label: "凭据与 API Key" },
  mcp: { id: "mcp", label: "MCP 服务器" },
  identity: { id: "identity", label: "成员与身份" },
  permission: { id: "permission", label: "角色与权限" },
  tenant: { id: "tenant", label: "租户" },
  scm: { id: "scm", label: "代码托管" },
  automation: { id: "automation", label: "自动化触发" },
  mod: { id: "mod", label: "Mod" },
  skills: { id: "skills", label: "Skills" },
  plugins: { id: "plugins", label: "Plugins" },
  hooks: { id: "hooks", label: "Hooks" },
  "dev-mode": { id: "dev-mode", label: "开发者模式" },
  debug: { id: "debug", label: "调试工具" },
  webhooks: { id: "webhooks", label: "API 与 Webhook" },
  flags: { id: "flags", label: "Feature Flags" },
  "cli-profiles": { id: "cli-profiles", label: "CLI 配置" },
};

function SettingsInner() {
  // 深链 ?tab= (client-only 解析; 不用 useSearchParams — dev 下双份 DOM)
  const [rawTab, setRawTab] = useState("");
  useEffect(() => {
    if (typeof window === "undefined") return;
    setRawTab(new URLSearchParams(window.location.search).get("tab") ?? "");
  }, []);
  const resolved = LEGACY_TAB_MAP[rawTab] ?? { id: "mcp", label: "MCP 服务器" };
  const activeId = resolved.id;

  return (
    <div data-testid="settings-page" className="flex h-full bg-bg">
      {/* 左导航 */}
      <aside data-testid="settings-nav" className="w-64 shrink-0 border-r border-line bg-bg-card overflow-y-auto p-3 space-y-4">
        {NAV_GROUPS.map((g) => (
          <div key={g.group}>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1.5 px-1">{g.group}</div>
            <ul className="space-y-0.5">
              {g.items.map((item) => (
                <li key={item.id}>
                  <a
                    href={`/settings?tab=${item.id}`}
                    data-testid={`settings-nav-${item.id}`}
                    className={clsx(
                      "flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs transition-colors",
                      activeId === item.id ? "bg-accent/10 text-accent font-semibold" : "text-ink-dim hover:bg-bg-soft/60",
                    )}
                  >
                    <span className="text-ink-mute">{item.icon}</span>
                    {item.label}
                  </a>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </aside>

      {/* 主区 */}
      <main className="flex-1 min-w-0 overflow-y-auto p-5">
        <div className="max-w-3xl space-y-4">
          <div className="flex items-center gap-2 text-xs text-ink-mute">
            <span className="font-semibold text-ink-dim">设置</span>
            <span>›</span>
            <span>{resolved.label}</span>
            <div className="flex-1" />
            <button type="button" data-testid="settings-dev-mode-toggle" className="flex items-center gap-1 text-ink-dim hover:text-accent">
              <Code2 size={12} /> 开发者模式
            </button>
            <button type="button" className="flex items-center gap-1 text-ink-dim hover:text-accent">
              <Search size={12} /> 搜索设置
            </button>
          </div>

          {activeId === "mcp" ? (
            <McpPanel />
          ) : (
            <StubPanel
              id={activeId}
              label={resolved.label}
              realHref={
                activeId === "apikeys" ? "/settings/api-keys" :
                activeId === "cli-profiles" ? "/settings/cli-profiles" :
                activeId === "dev-mode" || activeId === "debug" ? "/settings/developer" :
                undefined
              }
            />
          )}
        </div>
      </main>
    </div>
  );
}

export default function SettingsPage() {
  return <SettingsInner />;
}
