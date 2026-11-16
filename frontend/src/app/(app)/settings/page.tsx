/*
@cypher schema=1 source_sha256=dc949fdbc278f1ffad5ef74402f44aae7bb57d05be76adec9c2fc53c5c8cecac
MERGE (self:File {path:"frontend/src/app/(app)/settings/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/settings/page.tsx::SettingsPage",kind:"component"})
MERGE (hooksSettingsPanel:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HooksSettingsPanel",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(hooksSettingsPanel)
MERGE (page)-[:CALLS]->(hooksSettingsPanel)
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
import { useStore } from "@/lib/store";
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
  Brain,
} from "lucide-react";
import { clsx } from "clsx";
import { IdentityPanel, ProfilePanel, PermissionPanel, TenantPanel } from "@/components/settings/IdentityPanels";
import { IntegrationPanels } from "@/components/settings/IntegrationPanels";
import { ModDirectory } from "@/components/settings/ModDirectory";
import { MemorySettingsPanel } from "@/components/settings/MemorySettingsPanel";
import { HooksSettingsPanel } from "@/components/settings/HooksSettingsPanel";
import { McpSettingsPanel } from "@/components/settings/McpSettingsPanel";
import { SkillsSettingsPanel } from "@/components/settings/SkillsSettingsPanel";

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
    group: "Agent",
    items: [
      { id: "memory", label: "总记忆", icon: <Brain size={13} />, href: "" },
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
  memory: { id: "memory", label: "总记忆" },
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
            <McpSettingsPanel />
          ) : activeId === "mod" ? (
            <ModDirectory />
          ) : activeId === "identity" ? (
            <IdentityPanel />
          ) : activeId === "profile" ? (
            <ProfilePanel />
          ) : activeId === "permission" ? (
            <PermissionPanel />
          ) : activeId === "tenant" ? (
            <TenantPanel />
          ) : activeId === "memory" ? (
            <MemorySettingsPanel projects={useStore((s) => s.projects)} />
          ) : activeId === "hooks" ? (
            <HooksSettingsPanel />
          ) : activeId === "skills" ? (
            <SkillsSettingsPanel />
          ) : activeId === "scm" || activeId === "automation" || activeId === "webhooks" ||
            activeId === "plugins" || activeId === "flags" ? (
            <IntegrationPanels active={activeId} />
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

// =====================================================================
// ModPanel — Mod 目录管理 (per 2026-11-15 docs/prototypes/6 设置 mockup:
//   浏览 Mod 目录 / 从文件安装 / 用模板新建 + Mod 表 + 隔离运行时说明)
//   目录后端未接入 → 操作禁用并说明原因 (per 用户约束: 不接通界面交互)
// =====================================================================
function ModPanel() {
  const integrations = useStore((s) => s.integrations);
  const [sourceTab, setSourceTab] = useState<"official" | "custom">("official");

  return (
    <div data-testid="settings-mod-panel" className="space-y-3">
      {/* 目录源切换 + 已连接徽标 */}
      <div className="flex items-center gap-2">
        {(["official", "custom"] as const).map((src) => (
          <button
            key={src}
            type="button"
            onClick={() => setSourceTab(src)}
            data-testid={`settings-mod-src-${src}`}
            className={`rounded-full border px-3 py-1 text-[11px] font-semibold transition-colors ${
              sourceTab === src ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-mute hover:text-ink"
            }`}
          >
            {src === "official" ? "官方目录" : "自定义"}
          </button>
        ))}
        <span className="ml-auto rounded-full border border-ok/40 bg-ok/10 px-2 py-0.5 text-[10px] text-ok" data-testid="settings-mod-connected">
          已连接 · {integrations.length}
        </span>
      </div>

      {/* 3 按钮 (目录后端未接入 → 禁用 + 原因) */}
      <div className="flex items-center gap-2 flex-wrap">
        <button type="button" disabled data-testid="settings-mod-browse" className="rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[11px] text-ink-mute cursor-not-allowed">
          浏览 Mod 目录
        </button>
        <button type="button" disabled data-testid="settings-mod-install" className="rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[11px] text-ink-mute cursor-not-allowed">
          从文件安装
        </button>
        <button type="button" disabled data-testid="settings-mod-template" className="rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[11px] text-ink-mute cursor-not-allowed">
          用模板新建 Mod
        </button>
        <span className="text-[10px] text-warn" data-testid="settings-mod-disabled-reason">
          目录服务未接入 — 接入后启用安装/新建
        </span>
      </div>

      {/* Mod 表 */}
      <div className="rounded-xl border border-line overflow-hidden">
        <div className="grid grid-cols-[1.2fr_0.8fr_1fr_0.5fr_0.5fr] gap-2 border-b border-line bg-bg-soft/60 px-3 py-2 text-[10px] font-semibold text-ink-mute">
          <span>Mod</span><span>类型</span><span>声明的能力</span><span>版本</span><span>启用</span>
        </div>
        <div className="px-3 py-6 text-center text-[11px] text-ink-mute" data-testid="settings-mod-empty">
          暂无已安装 Mod — 浏览 Mod 目录安装 (目录服务接入后)
        </div>
      </div>

      {/* 隔离运行时说明 (mockup 原文) */}
      <div className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn" data-testid="settings-mod-isolation">
        Mod 在隔离运行时中执行，只能使用声明的能力；不会在主进程加载原生代码。
        安全相关 Hook 由 Star 核心执行，Mod 不能替代或减弱。
      </div>
    </div>
  );
}

export default function SettingsPage() {
  return <SettingsInner />;
}
