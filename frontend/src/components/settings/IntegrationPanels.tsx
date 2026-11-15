/*
@cypher schema=1 source_sha256=5673d6cdd61f7f10ac408322668c6602675fbd477f829ccbe9b05ccfac0bc013
MERGE (self:File {path:"frontend/src/components/settings/IntegrationPanels.tsx"})
MERGE (react:ExternalService {name:"react"})
MERGE (lucide:ExternalService {name:"lucide-react"})
MERGE (integrationPanels:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::IntegrationPanels",kind:"component"})
MERGE (scmPanel:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::ScmPanel",kind:"component"})
MERGE (automationPanel:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::AutomationPanel",kind:"component"})
MERGE (webhooksPanel:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::WebhooksPanel",kind:"component"})
MERGE (skillsPanel:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::SkillsPanel",kind:"component"})
MERGE (pluginsPanel:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::PluginsPanel",kind:"component"})
MERGE (hooksPanel:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::HooksPanel",kind:"component"})
MERGE (flagsPanel:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::FlagsPanel",kind:"component"})
MERGE (toggle:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::Toggle",kind:"component"})
MERGE (notConnectedBadge:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::NotConnectedBadge",kind:"component"})
MERGE (previewBadge:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::PreviewBadge",kind:"component"})
MERGE (panelIntro:Symbol {id:"frontend/src/components/settings/IntegrationPanels.tsx::PanelIntro",kind:"component"})
MERGE (self)-[:IMPORTS]->(react)
MERGE (self)-[:IMPORTS]->(lucide)
MERGE (self)-[:DEFINES]->(integrationPanels)
MERGE (self)-[:DEFINES]->(scmPanel)
MERGE (self)-[:DEFINES]->(automationPanel)
MERGE (self)-[:DEFINES]->(webhooksPanel)
MERGE (self)-[:DEFINES]->(skillsPanel)
MERGE (self)-[:DEFINES]->(pluginsPanel)
MERGE (self)-[:DEFINES]->(hooksPanel)
MERGE (self)-[:DEFINES]->(flagsPanel)
MERGE (self)-[:DEFINES]->(toggle)
MERGE (self)-[:DEFINES]->(notConnectedBadge)
MERGE (self)-[:DEFINES]->(previewBadge)
MERGE (self)-[:DEFINES]->(panelIntro)
MERGE (integrationPanels)-[:CALLS]->(scmPanel)
MERGE (integrationPanels)-[:CALLS]->(automationPanel)
MERGE (integrationPanels)-[:CALLS]->(webhooksPanel)
MERGE (integrationPanels)-[:CALLS]->(skillsPanel)
MERGE (integrationPanels)-[:CALLS]->(pluginsPanel)
MERGE (integrationPanels)-[:CALLS]->(hooksPanel)
MERGE (integrationPanels)-[:CALLS]->(flagsPanel)
MERGE (scmPanel)-[:CALLS]->(notConnectedBadge)
MERGE (scmPanel)-[:CALLS]->(previewBadge)
MERGE (automationPanel)-[:CALLS]->(notConnectedBadge)
MERGE (automationPanel)-[:CALLS]->(previewBadge)
MERGE (webhooksPanel)-[:CALLS]->(notConnectedBadge)
MERGE (webhooksPanel)-[:CALLS]->(previewBadge)
MERGE (skillsPanel)-[:CALLS]->(previewBadge)
MERGE (pluginsPanel)-[:CALLS]->(previewBadge)
MERGE (pluginsPanel)-[:CALLS]->(toggle)
MERGE (hooksPanel)-[:CALLS]->(previewBadge)
MERGE (flagsPanel)-[:CALLS]->(previewBadge)
MERGE (flagsPanel)-[:CALLS]->(toggle)
MERGE (scmPanel)-[:CALLS]->(panelIntro)
MERGE (automationPanel)-[:CALLS]->(panelIntro)
MERGE (webhooksPanel)-[:CALLS]->(panelIntro)
MERGE (skillsPanel)-[:CALLS]->(panelIntro)
MERGE (pluginsPanel)-[:CALLS]->(panelIntro)
MERGE (hooksPanel)-[:CALLS]->(panelIntro)
MERGE (flagsPanel)-[:CALLS]->(panelIntro)
@endcypher
*/
"use client";

// =====================================================================
// IntegrationPanels — /settings 集成面板组 (7 个面板, prop 切换)
// =====================================================================
// 诚实标注 (per 用户约束 缺标比错标):
//   - 后端未接入的能力 → 按钮禁用 + 「未接入」徽标 + 原因说明
//   - 本地可实现/展示的数据 → 本地常量渲染 + 「预览数据」徽标
//   - 本地 toggle (插件/开关) 仅本地 UI 状态, 不冒充后端持久化
// 面板: ScmPanel / AutomationPanel / WebhooksPanel / SkillsPanel /
//       PluginsPanel / HooksPanel / FlagsPanel
// 容器 IntegrationPanels: active prop 选择面板 (默认 scm = 第一面板),
//   受控 (active + onActiveChange) 或非受控 (内部 state + 顶部 tab)
// =====================================================================

import { useState } from "react";
import type { ComponentType, ReactNode } from "react";
import {
  GitBranch,
  Zap,
  Globe,
  Sparkles,
  Plug,
  Webhook,
  Flag,
  KeyRound,
  Plus,
  Check,
} from "lucide-react";

export type IntegrationPanelId =
  | "scm"
  | "automation"
  | "webhooks"
  | "skills"
  | "plugins"
  | "hooks"
  | "flags";

// =====================================================================
// 本地预览数据 (接入后端后替换为服务端投影)
// =====================================================================

const SCM_REPOS = [
  { id: "git-001", name: "star/star", provider: "GitHub", branch: "dev", updatedAt: "2026-11-15" },
  { id: "git-002", name: "acme/portal-web", provider: "GitLab", branch: "main", updatedAt: "2026-11-12" },
  { id: "git-003", name: "acme/api-gateway", provider: "GitHub", branch: "main", updatedAt: "2026-11-08" },
];

const AUTOMATION_TRIGGERS = [
  { id: "trg-001", name: "PR 创建 → 自动 code review", event: "pull_request.opened", action: "run /review", enabled: true },
  { id: "trg-002", name: "Worktree 归档 → 通知发布频道", event: "worktree.archived", action: "notify #release", enabled: true },
  { id: "trg-003", name: "每日 09:00 → 打开逾期任务看板", event: "cron.daily_09:00", action: "open /kanban?filter=overdue", enabled: false },
];

const WEBHOOK_ENDPOINTS = [
  { id: "wh-001", method: "POST", path: "/api/v1/webhooks/workitems/status", secret: "whsec_***REDACTED***", deliveries24h: "128" },
  { id: "wh-002", method: "POST", path: "/api/v1/webhooks/sprints/updated", secret: "whsec_***REDACTED***", deliveries24h: "41" },
  { id: "wh-003", method: "GET", path: "/api/v1/webhooks/deliveries", secret: "—", deliveries24h: "—" },
];

const SKILLS = [
  { id: "skl-001", name: "cypher-graph-guardian", scope: "全局", version: "1.0.0" },
  { id: "skl-002", name: "code-review", scope: "全局", version: "1.2.0" },
  { id: "skl-003", name: "diagnosing-bugs", scope: "全局", version: "0.9.0" },
  { id: "skl-004", name: "tdd", scope: "项目", version: "0.8.1" },
];

const PLUGINS = [
  { id: "plg-001", name: "star-frontend-dev", version: "0.4.1", description: "前端看板/画布开发约定", enabled: true },
  { id: "plg-002", name: "mcp-postgres", version: "1.2.0", description: "PostgreSQL 只读查询与表结构", enabled: true },
  { id: "plg-003", name: "browser-automation", version: "0.9.3", description: "浏览器页面操作与截图", enabled: false },
];

const HOOKS = [
  { id: "hk-001", name: "lint-staged", point: "pre-commit", target: "pnpm lint-staged", status: "启用" },
  { id: "hk-002", name: "typecheck", point: "pre-push", target: "pnpm typecheck", status: "启用" },
  { id: "hk-003", name: "graph reconcile", point: "post-merge", target: "cgg reconcile", status: "未接入" },
];

const FLAGS = [
  { id: "flag-001", key: "settings.integrations", description: "设置集成面板", defaultEnabled: true },
  { id: "flag-002", key: "canvas.group-editing", description: "画布分组编辑", defaultEnabled: false },
  { id: "flag-003", key: "agent.loop-protection", description: "Agent 循环防护", defaultEnabled: true },
];

// =====================================================================
// 共用徽标与开关
// =====================================================================

function NotConnectedBadge() {
  return (
    <span
      data-testid="badge-not-connected"
      className="inline-flex items-center rounded border border-warn/40 bg-warn/10 px-1.5 py-0.5 text-[9px] font-semibold text-warn"
    >
      未接入
    </span>
  );
}

function PreviewBadge() {
  return (
    <span
      data-testid="badge-preview"
      className="inline-flex items-center rounded border border-ok/40 bg-ok/10 px-1.5 py-0.5 text-[9px] font-semibold text-ok"
    >
      预览数据
    </span>
  );
}

function Toggle({
  checked,
  onChange,
  testId,
  label,
}: {
  checked: boolean;
  onChange: (next: boolean) => void;
  testId: string;
  label: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      data-testid={testId}
      onClick={() => onChange(!checked)}
      className={`relative h-4 w-7 shrink-0 rounded-full transition-colors ${
        checked ? "bg-accent" : "border border-line bg-bg-soft"
      }`}
    >
      <span
        className={`absolute left-0.5 top-0.5 size-3 rounded-full bg-white transition-transform ${
          checked ? "translate-x-3" : ""
        }`}
      />
    </button>
  );
}

function PanelIntro({ title, description }: { title: string; description: string }) {
  return (
    <div className="flex items-start gap-2">
      <h2 className="text-sm font-bold text-ink">{title}</h2>
      <PreviewBadge />
      <p className="min-w-0 flex-1 text-[10px] leading-relaxed text-ink-dim">{description}</p>
    </div>
  );
}

// =====================================================================
// 1. ScmPanel — 代码托管
// =====================================================================

export function ScmPanel() {
  return (
    <div data-testid="settings-scm-panel" className="space-y-3">
      <PanelIntro title="代码托管" description="绑定 Git provider 后按仓库同步 PR / 分支 / 提交。当前仓库列表为本地预览数据。" />

      <div className="flex items-center gap-2 flex-wrap">
        <button
          type="button"
          disabled
          data-testid="settings-scm-bind-provider"
          className="inline-flex items-center gap-1.5 rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[11px] text-ink-mute cursor-not-allowed"
        >
          <GitBranch size={12} /> 绑定 Git Provider
        </button>
        <NotConnectedBadge />
        <span className="text-[10px] text-warn" data-testid="settings-scm-disabled-reason">
          Git provider OAuth 后端未接入 — 接入后启用绑定
        </span>
      </div>

      <ul className="divide-y divide-line overflow-hidden rounded-xl border border-line">
        {SCM_REPOS.map((repo) => (
          <li
            key={repo.id}
            data-testid={`settings-scm-repo-${repo.id}`}
            className="flex items-center gap-3 bg-bg-card px-3 py-2"
          >
            <GitBranch size={13} className="shrink-0 text-ink-mute" />
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-ink">{repo.name}</div>
              <div className="text-[10px] text-ink-mute">
                {repo.id} · {repo.provider} · {repo.branch}
              </div>
            </div>
            <span className="shrink-0 text-[10px] text-ink-mute">更新 {repo.updatedAt}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

// =====================================================================
// 2. AutomationPanel — 自动化触发
// =====================================================================

export function AutomationPanel() {
  return (
    <div data-testid="settings-automation-panel" className="space-y-3">
      <PanelIntro title="自动化触发" description="事件驱动的自动化规则。当前触发器列表为本地预览数据。" />

      <div className="flex items-center gap-2 flex-wrap">
        <button
          type="button"
          disabled
          data-testid="settings-automation-new-trigger"
          className="inline-flex items-center gap-1.5 rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[11px] text-ink-mute cursor-not-allowed"
        >
          <Plus size={12} /> 新建触发器
        </button>
        <NotConnectedBadge />
        <span className="text-[10px] text-warn" data-testid="settings-automation-disabled-reason">
          触发器执行后端未接入 — 接入后启用新建/编辑
        </span>
      </div>

      <ul className="divide-y divide-line overflow-hidden rounded-xl border border-line">
        {AUTOMATION_TRIGGERS.map((t) => (
          <li
            key={t.id}
            data-testid={`settings-automation-trigger-${t.id}`}
            className="flex items-center gap-3 bg-bg-card px-3 py-2"
          >
            <Zap size={13} className="shrink-0 text-ink-mute" />
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-ink">{t.name}</div>
              <div className="font-mono text-[10px] text-ink-mute">
                {t.event} → {t.action}
              </div>
            </div>
            {t.enabled ? (
              <span className="inline-flex shrink-0 items-center gap-0.5 text-[10px] font-semibold text-ok">
                <Check size={10} /> 已启用
              </span>
            ) : (
              <span className="shrink-0 text-[10px] text-ink-mute">已停用</span>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

// =====================================================================
// 3. WebhooksPanel — API 与 Webhook
// =====================================================================

export function WebhooksPanel() {
  return (
    <div data-testid="settings-webhooks-panel" className="space-y-3">
      <PanelIntro title="API 与 Webhook" description="对外事件端点。当前端点表为本地预览数据。" />

      <div className="flex items-center gap-2 flex-wrap">
        <button
          type="button"
          disabled
          data-testid="settings-webhooks-generate-secret"
          className="inline-flex items-center gap-1.5 rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[11px] text-ink-mute cursor-not-allowed"
        >
          <KeyRound size={12} /> 生成密钥
        </button>
        <NotConnectedBadge />
        <span className="text-[10px] text-warn" data-testid="settings-webhooks-disabled-reason">
          密钥管理服务未接入 — 接入后启用生成/轮换
        </span>
      </div>

      <div className="overflow-hidden rounded-xl border border-line">
        <div className="grid grid-cols-[3.5rem_1fr_8.5rem_4rem] gap-2 border-b border-line bg-bg-soft/60 px-3 py-2 text-[10px] font-semibold text-ink-mute">
          <span>方法</span>
          <span>端点</span>
          <span>密钥</span>
          <span>24h 投递</span>
        </div>
        {WEBHOOK_ENDPOINTS.map((ep) => (
          <div
            key={ep.id}
            data-testid={`settings-webhooks-endpoint-${ep.id}`}
            className="grid grid-cols-[3.5rem_1fr_8.5rem_4rem] items-center gap-2 border-b border-line bg-bg-card px-3 py-2 last:border-b-0"
          >
            <span
              className={`w-fit rounded border px-1.5 py-0.5 text-[9px] font-bold ${
                ep.method === "POST" ? "border-accent/40 bg-accent/10 text-accent" : "border-ok/40 bg-ok/10 text-ok"
              }`}
            >
              {ep.method}
            </span>
            <span className="truncate font-mono text-[11px] text-ink">{ep.path}</span>
            <span className="truncate font-mono text-[10px] text-ink-mute">{ep.secret}</span>
            <span className="text-[10px] text-ink-dim">{ep.deliveries24h}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// =====================================================================
// 4. SkillsPanel — Skills
// =====================================================================

export function SkillsPanel() {
  return (
    <div data-testid="settings-skills-panel" className="space-y-3">
      <PanelIntro title="Skills" description="可复用技能列表（本地预览数据）。" />

      <ul className="divide-y divide-line overflow-hidden rounded-xl border border-line">
        {SKILLS.map((s) => (
          <li
            key={s.id}
            data-testid={`settings-skills-item-${s.id}`}
            className="flex items-center gap-3 bg-bg-card px-3 py-2"
          >
            <Sparkles size={13} className="shrink-0 text-ink-mute" />
            <span className="min-w-0 flex-1 truncate text-xs font-semibold text-ink">{s.name}</span>
            <span className="shrink-0 text-[10px] text-ink-mute">{s.scope}</span>
            <span className="shrink-0 font-mono text-[10px] text-ink-dim">v{s.version}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

// =====================================================================
// 5. PluginsPanel — Plugins
// =====================================================================

export function PluginsPanel() {
  const [enabled, setEnabled] = useState<Record<string, boolean>>(() =>
    Object.fromEntries(PLUGINS.map((p) => [p.id, p.enabled])),
  );

  return (
    <div data-testid="settings-plugins-panel" className="space-y-3">
      <PanelIntro title="Plugins" description="插件列表（本地预览数据）。启停仅本地 UI 状态 — 插件运行时未接入，不持久化。" />

      <ul className="divide-y divide-line overflow-hidden rounded-xl border border-line">
        {PLUGINS.map((p) => (
          <li
            key={p.id}
            data-testid={`settings-plugins-item-${p.id}`}
            className="flex items-center gap-3 bg-bg-card px-3 py-2"
          >
            <Plug size={13} className="shrink-0 text-ink-mute" />
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-ink">
                {p.name} <span className="ml-1 font-mono text-[9px] font-normal text-ink-mute">v{p.version}</span>
              </div>
              <div className="text-[10px] text-ink-dim">{p.description}</div>
            </div>
            <Toggle
              checked={enabled[p.id] ?? false}
              onChange={(next) => setEnabled((prev) => ({ ...prev, [p.id]: next }))}
              testId={`settings-plugins-toggle-${p.id}`}
              label={`切换插件 ${p.name}`}
            />
          </li>
        ))}
      </ul>
    </div>
  );
}

// =====================================================================
// 6. HooksPanel — Hooks
// =====================================================================

export function HooksPanel() {
  return (
    <div data-testid="settings-hooks-panel" className="space-y-3">
      <PanelIntro title="Hooks" description="Git 生命周期 hook 列表（本地预览数据）。" />

      <ul className="divide-y divide-line overflow-hidden rounded-xl border border-line">
        {HOOKS.map((h) => (
          <li
            key={h.id}
            data-testid={`settings-hooks-item-${h.id}`}
            className="flex items-center gap-3 bg-bg-card px-3 py-2"
          >
            <Webhook size={13} className="shrink-0 text-ink-mute" />
            <span className="text-xs font-semibold text-ink">{h.name}</span>
            <span className="rounded border border-line bg-bg-soft/60 px-1.5 py-0.5 font-mono text-[9px] text-ink-dim">
              {h.point}
            </span>
            <span className="min-w-0 flex-1 truncate font-mono text-[10px] text-ink-mute">{h.target}</span>
            {h.status === "启用" ? (
              <span className="shrink-0 text-[10px] font-semibold text-ok">{h.status}</span>
            ) : (
              <span className="shrink-0 text-[10px] text-warn">{h.status}</span>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

// =====================================================================
// 7. FlagsPanel — Feature Flags
// =====================================================================

export function FlagsPanel() {
  const [enabled, setEnabled] = useState<Record<string, boolean>>(() =>
    Object.fromEntries(FLAGS.map((f) => [f.id, f.defaultEnabled])),
  );

  return (
    <div data-testid="settings-flags-panel" className="space-y-3">
      <PanelIntro title="Feature Flags" description="功能开关（本地预览数据）。开关仅本地会话状态 — 后端 flags 服务未接入，刷新后重置。" />

      <ul className="divide-y divide-line overflow-hidden rounded-xl border border-line">
        {FLAGS.map((f) => (
          <li
            key={f.id}
            data-testid={`settings-flags-item-${f.id}`}
            className="flex items-center gap-3 bg-bg-card px-3 py-2"
          >
            <Flag size={13} className="shrink-0 text-ink-mute" />
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-ink">
                <span className="font-mono">{f.key}</span>
                <span className="ml-1.5 text-[9px] font-normal text-ink-mute">
                  默认 {f.defaultEnabled ? "开" : "关"}
                </span>
              </div>
              <div className="text-[10px] text-ink-dim">{f.description}</div>
            </div>
            <Toggle
              checked={enabled[f.id] ?? false}
              onChange={(next) => setEnabled((prev) => ({ ...prev, [f.id]: next }))}
              testId={`settings-flags-toggle-${f.id}`}
              label={`切换 flag ${f.key}`}
            />
          </li>
        ))}
      </ul>
    </div>
  );
}

// =====================================================================
// 容器 — prop 切换 (默认 scm = 第一面板)
// =====================================================================

const PANEL_TABS: { id: IntegrationPanelId; label: string; icon: ReactNode }[] = [
  { id: "scm", label: "代码托管", icon: <GitBranch size={12} /> },
  { id: "automation", label: "自动化触发", icon: <Zap size={12} /> },
  { id: "webhooks", label: "API与Webhook", icon: <Globe size={12} /> },
  { id: "skills", label: "Skills", icon: <Sparkles size={12} /> },
  { id: "plugins", label: "Plugins", icon: <Plug size={12} /> },
  { id: "hooks", label: "Hooks", icon: <Webhook size={12} /> },
  { id: "flags", label: "Feature Flags", icon: <Flag size={12} /> },
];

const PANEL_COMPONENTS: Record<IntegrationPanelId, ComponentType> = {
  scm: ScmPanel,
  automation: AutomationPanel,
  webhooks: WebhooksPanel,
  skills: SkillsPanel,
  plugins: PluginsPanel,
  hooks: HooksPanel,
  flags: FlagsPanel,
};

export interface IntegrationPanelsProps {
  /** 选中面板 id; 缺省时内部 state 控制 (默认 "scm" = 第一面板) */
  active?: IntegrationPanelId;
  onActiveChange?: (id: IntegrationPanelId) => void;
}

export function IntegrationPanels({ active, onActiveChange }: IntegrationPanelsProps) {
  const [internalActive, setInternalActive] = useState<IntegrationPanelId>("scm");
  const current = active ?? internalActive;
  const ActivePanel = PANEL_COMPONENTS[current];

  const select = (id: IntegrationPanelId) => {
    setInternalActive(id);
    onActiveChange?.(id);
  };

  return (
    <div data-testid="settings-integration-panels" className="space-y-3">
      <div
        data-testid="settings-integration-tabs"
        className="flex items-center gap-1 overflow-x-auto border-b border-line"
      >
        {PANEL_TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            data-testid={`settings-integration-tab-${t.id}`}
            onClick={() => select(t.id)}
            className={`-mb-px flex shrink-0 items-center gap-1.5 whitespace-nowrap border-b-2 px-2.5 py-1.5 text-[11px] transition-colors ${
              current === t.id
                ? "border-accent font-semibold text-accent"
                : "border-transparent text-ink-dim hover:text-ink"
            }`}
          >
            <span className="text-ink-mute">{t.icon}</span>
            {t.label}
          </button>
        ))}
      </div>
      <ActivePanel />
    </div>
  );
}

export default IntegrationPanels;
