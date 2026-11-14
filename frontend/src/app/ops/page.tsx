/*
@cypher schema=1 source_sha256=c077c58e86720270352e15eba25fc03f07558e6d6817aefc0bb542f1a6f1654b
MERGE (self:File {path:"frontend/src/app/ops/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/ops/page.tsx::OpsPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
"use client";

/**
 * /ops — 7 · 运维控制台（仅管理员）
 * per 2026-11-14 参考 mockup: Ops.dc.html
 * 5 tab: 运维数据 / App 集群(原子化 App 表 + 兼容性检查 + 晋级/渐进发布) /
 * 租户 / 用户与权限 / 审计 — 环境选择器 + 管理员标识
 * 既有实装保留: MetricsTab/LogAITab(运维数据), DocsTab(审计), F-02 端到端
 */

import { useState } from "react";
import { Server, Brain, BarChart3, FileText, ShieldAlert, ChevronDown, Check, AlertTriangle } from "lucide-react";
import { clsx } from "clsx";
import { LogAITab } from "./components/LogAITab";
import { MetricsTab } from "./components/MetricsTab";
import { DocsTab } from "./components/DocsTab";
import { ClusterTab } from "./components/ClusterTab";

const OPS_TABS = ["运维数据", "App 集群", "租户", "用户与权限", "审计"] as const;

interface AppRow {
  app: string;
  kind: string;
  current: string;
  upgrade: string | null;
  contract: string;
  health: string;
  healthTone: string;
}

const APPS: AppRow[] = [
  { app: "star-api-rest", kind: "服务 · REST API", current: "0.4.2", upgrade: "0.4.3", contract: "v3 兼容", health: "健康", healthTone: "bg-ok/10 text-ok" },
  { app: "star-bff", kind: "服务 · 前端网关", current: "0.2.0", upgrade: null, contract: "v3 兼容", health: "健康", healthTone: "bg-ok/10 text-ok" },
  { app: "canvas-engine", kind: "服务 · 画布平台", current: "0.3.1", upgrade: "0.3.2", contract: "v2 兼容", health: "健康", healthTone: "bg-ok/10 text-ok" },
  { app: "domain-canvas", kind: "服务 · 画布业务域", current: "0.3.0", upgrade: "0.4.0", contract: "需迁移", health: "健康", healthTone: "bg-ok/10 text-ok" },
  { app: "canvas-realtime", kind: "服务 · 实时协作", current: "0.2.4", upgrade: "0.2.5", contract: "v1 兼容", health: "降级", healthTone: "bg-warn/10 text-warn" },
  { app: "web", kind: "前端 · Vercel 独立发布", current: "2026.11.14", upgrade: null, contract: "—", health: "健康", healthTone: "bg-ok/10 text-ok" },
  { app: "star-desktop", kind: "桌面 · 独立发布链", current: "0.9.0", upgrade: "0.9.1", contract: "—", health: "—", healthTone: "bg-bg-soft text-ink-dim" },
];

const CHECKS = [
  { ok: true, text: "镜像已签名，SBOM 无新增阻断许可证" },
  { ok: true, text: "对外 API v3 不变，新增字段向后兼容" },
  { ok: false, text: "数据库迁移含删列：需先发布「扩展」步骤，旧版本仍可读写" },
  { ok: true, text: "依赖方 canvas-engine 0.3.1 满足 requires ≥ 0.3.0" },
  { ok: true, text: "事件 schema canvas.element.v2 兼容旧消费者" },
];

function AppClusterPanel() {
  return (
    <div data-testid="ops-app-cluster" className="space-y-4">
      <div className="flex items-center gap-2">
        <h2 className="text-sm font-bold text-ink">原子化 App</h2>
        <span className="text-[11px] text-ink-dim">每个 App 独立构建、独立晋级、独立回滚</span>
      </div>

      {/* App 表 */}
      <div className="bg-bg-card rounded-xl border border-line shadow-soft overflow-hidden">
        <table data-testid="ops-app-table" className="w-full text-xs">
          <thead>
            <tr className="text-left text-ink-mute border-b border-line bg-bg">
              <th className="py-2.5 px-3 font-medium">App</th>
              <th className="py-2.5 px-3 font-medium">当前版本</th>
              <th className="py-2.5 px-3 font-medium">可升级到</th>
              <th className="py-2.5 px-3 font-medium">契约</th>
              <th className="py-2.5 px-3 font-medium">健康</th>
              <th className="py-2.5 px-3 font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            {APPS.map((row) => (
              <tr key={row.app} data-testid={`ops-app-${row.app}`} className="border-b border-line/60 hover:bg-bg-soft/60">
                <td className="py-2.5 px-3">
                  <div className="font-mono font-semibold text-ink">{row.app}</div>
                  <div className="text-[10px] text-ink-mute">{row.kind}</div>
                </td>
                <td className="py-2.5 px-3 font-mono text-ink-dim">{row.current}</td>
                <td className="py-2.5 px-3 font-mono text-accent">{row.upgrade ?? "—"}</td>
                <td className="py-2.5 px-3 text-ink-dim">{row.contract}</td>
                <td className="py-2.5 px-3">
                  <span className={clsx("text-[10px] px-1.5 py-0.5 rounded font-semibold", row.healthTone)}>{row.health}</span>
                </td>
                <td className="py-2.5 px-3">
                  <span className="flex items-center gap-2">
                    <button type="button" className="text-accent hover:underline">单独更新</button>
                    <button type="button" className="text-ink-dim hover:underline">回滚</button>
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* domain-canvas 兼容性检查 */}
      <section data-testid="ops-release-check" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
        <h3 className="text-sm font-bold text-ink">domain-canvas 0.3.0 → 0.4.0</h3>
        <p className="text-xs text-ink-dim mt-0.5">兼容性检查（晋级前必过）</p>
        <ul className="mt-3 space-y-1.5">
          {CHECKS.map((c, i) => (
            <li key={i} className="flex items-start gap-2 text-xs">
              {c.ok ? (
                <Check size={13} className="text-ok mt-0.5 shrink-0" />
              ) : (
                <AlertTriangle size={13} className="text-warn mt-0.5 shrink-0" />
              )}
              <span className={c.ok ? "text-ink-dim" : "text-warn"}>{c.text}</span>
            </li>
          ))}
        </ul>

        <div className="mt-4 flex items-center gap-2 text-xs">
          <span className="text-ink-dim">晋级路径</span>
          <span className="px-2 py-1 rounded border border-line bg-bg font-mono">dev</span>
          <span className="text-ink-mute">→</span>
          <span className="px-2 py-1 rounded border border-line bg-bg font-mono">staging</span>
          <span className="text-ink-mute">→</span>
          <span className="px-2 py-1 rounded border border-accent/50 bg-accent/10 font-mono text-accent">prod（需审批）</span>
        </div>

        <div className="mt-2 flex items-center gap-2 text-xs">
          <span className="text-ink-dim">渐进发布</span>
          {["10%", "30%", "60%", "100%"].map((p, i) => (
            <span key={p} className={clsx("px-2 py-1 rounded border font-mono", i === 0 ? "border-accent/60 bg-accent/10 text-accent font-semibold" : "border-line bg-bg text-ink-dim")}>
              {p}
            </span>
          ))}
          <span className="text-[11px] text-ink-dim ml-2">每一步跑分析门禁（错误率、P95 延迟、事件积压），失败自动回滚，只影响这一个 App</span>
        </div>

        <div className="mt-3 flex items-center gap-2">
          <button type="button" data-testid="ops-release-go" className="text-xs px-3 py-2 rounded-lg bg-accent text-white hover:bg-accent-soft">
            先发布扩展迁移，再更新
          </button>
          <button type="button" className="text-xs px-3 py-2 rounded-lg border border-line text-ink-dim hover:border-accent/60">
            暂停
          </button>
          <button type="button" className="text-xs px-3 py-2 rounded-lg border border-line text-ink-dim hover:border-accent/60">
            查看发布记录
          </button>
        </div>
      </section>
    </div>
  );
}

function StubPanel({ label }: { label: string }) {
  return (
    <div className="bg-bg-card rounded-xl border border-line p-6 text-xs text-ink-dim">
      {label} 面板待接入（P2 功能缺口，per 缺标比错标）
    </div>
  );
}

export default function OpsPage() {
  const [activeTab, setActiveTab] = useState<(typeof OPS_TABS)[number]>("App 集群");

  return (
    <div data-testid="ops-page" className="flex flex-col h-full bg-bg">
      {/* 头部 */}
      <header className="bg-bg-card border-b border-line px-4 py-3 flex items-center gap-3">
        <div className="flex items-center gap-2.5">
          <div className="size-8 rounded-lg bg-accent/15 grid place-items-center text-accent">
            <ShieldAlert size={15} />
          </div>
          <div>
            <h1 className="text-sm font-bold text-ink">运维控制台</h1>
            <p className="text-[10px] text-ink-mute">仅管理员</p>
          </div>
        </div>

        <div className="flex items-center gap-1.5 ml-4 border-b border-transparent">
          {OPS_TABS.map((tab) => (
            <button
              key={tab}
              type="button"
              onClick={() => setActiveTab(tab)}
              data-testid={`ops-tab-${tab}`}
              className={clsx(
                "px-3 py-1.5 text-sm rounded-t-lg border-b-2 transition-colors",
                activeTab === tab ? "border-accent text-accent font-semibold" : "border-transparent text-ink-dim hover:text-ink",
              )}
            >
              {tab}
            </button>
          ))}
        </div>

        <div className="flex-1" />

        <button type="button" data-testid="ops-env" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim">
          环境 <span className="font-mono font-semibold">staging</span> <ChevronDown size={12} />
        </button>
        <span className="size-7 rounded-full bg-bg-soft text-ink-dim font-bold grid place-items-center text-[10px]">L</span>
        <span className="text-xs text-ink-dim">lidian727@gmail.com</span>
      </header>

      {/* 内容 */}
      <main className="flex-1 overflow-y-auto p-4">
        {activeTab === "App 集群" && (
          <div className="space-y-4">
            <AppClusterPanel />
            <section data-testid="ops-cluster-tab" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
              <h2 className="text-sm font-bold text-ink mb-2">集群更新（F-01 实装）</h2>
              <ClusterTab />
            </section>
          </div>
        )}
        {activeTab === "运维数据" && (
          <div className="space-y-4">
            <MetricsTab />
            <LogAITab />
          </div>
        )}
        {activeTab === "审计" && <DocsTab />}
        {activeTab === "租户" && <StubPanel label="租户" />}
        {activeTab === "用户与权限" && <StubPanel label="用户与权限" />}
      </main>
    </div>
  );
}
