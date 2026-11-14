/*
@cypher schema=1 source_sha256=1882ccd8f0c61f2e368c81e54b2d8a65c6a6a3ddf53982454176cbf56e1f2e9e
MERGE (self:File {path:"frontend/src/app/(app)/analytics/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/analytics/page.tsx::AnalyticsPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /analytics — 分析 (per 0·页面合并图 & Analytics.dc.html mockup)
// 3 子 tab: 概览 / 运维 / 审计日志
// 概览: 4 KPI 卡片 + IPA 阶段进度条 (P2 需求定义 → P6.2 集成测试)
// =====================================================================
"use client";

import { useState, useMemo } from "react";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { useStore } from "@/lib/store";
import {
  BarChart3,
  CheckCircle2,
  TrendingUp,
  Cpu,
  AlertTriangle,
  FileText,
  GitBranch,
  ChevronDown,
} from "lucide-react";
import { useTranslation } from "@/lib/i18n";

const ANALYTICS_TABS = ["概览", "运维", "审计日志"] as const;

const IPA_PHASES = [
  { id: "P2", label: "需求定义", done: 2, total: 20 },
  { id: "P3", label: "基本设计", done: 1, total: 21 },
  { id: "P4", label: "详细设计", done: 2, total: 20 },
  { id: "P5", label: "实现", done: 2, total: 15 },
  { id: "P6.1", label: "单元测试", done: 2, total: 11 },
  { id: "P6.2", label: "集成测试", done: 1, total: 12 },
] as const;

export default function AnalyticsPage() {
  const { t, tx } = useTranslation();
  const [activeTab, setActiveTab] = useState<typeof ANALYTICS_TABS[number]>("概览");

  // 真实 store 数据
  const workItems = useStore((s) => s.workItems);
  const worktrees = useStore((s) => s.worktrees);
  const agents = useStore((s) => s.agentSessions);

  // 计算 KPI (真实数据 + fallback)
  const kpiData = useMemo(() => {
    const completed = workItems.filter((w) => w.status === "done").length;
    const total = workItems.length || 1;
    const completionRate = Math.round((completed / total) * 100);
    const totalTokens = agents.reduce((sum, a) => sum + (a.token_usage?.total || 0), 0);
    const avgTokens = agents.length > 0 ? Math.round(totalTokens / agents.length / 1000) : 0;
    const ipaSkills = workItems.filter((w) => w.ipa_skill_ids && w.ipa_skill_ids.length > 0).length;
    const ipaCoverage = workItems.length > 0 ? Math.round((ipaSkills / workItems.length) * 100) : 0;

    return [
      { label: "完成任务", value: `${completed} / ${total}`, icon: CheckCircle2, trend: "+12%", trendUp: true },
      { label: "验证通过率", value: `${completionRate}%`, icon: TrendingUp, trend: "+5%", trendUp: true },
      { label: "Agent 运行时长", value: `${avgTokens}K tokens 平均`, icon: Cpu, trend: "-3%", trendUp: false },
      { label: "IPA 技能覆盖", value: `${ipaCoverage}%`, icon: FileText, trend: "+8%", trendUp: true },
    ];
  }, [workItems, agents]);

  return (
    <div className="flex h-full flex-col bg-bg">
      {/* 头部: 标题 + 租户/时间范围选择器 */}
      <header className="border-b border-line bg-bg-card px-4 py-3 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <h1 className="text-base font-bold text-ink">分析</h1>
          <span className="text-[10px] px-1.5 py-0.5 rounded border border-line text-ink-dim">全局</span>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={activeTab}
            onChange={(e) => setActiveTab(e.target.value as typeof ANALYTICS_TABS[number])}
            className="text-xs px-2 py-1.5 rounded border border-line bg-bg-soft text-ink"
          >
            {ANALYTICS_TABS.map((tab) => (
              <option key={tab} value={tab}>{tab}</option>
            ))}
          </select>
          <select className="text-xs px-2 py-1.5 rounded border border-line bg-bg-soft text-ink">
            <option>Physis / GVPE</option>
            <option>StarGate Dashboard</option>
            <option>Mobile Companion</option>
          </select>
          <select className="text-xs px-2 py-1.5 rounded border border-line bg-bg-soft text-ink">
            <option>近 14 天</option>
            <option>近 30 天</option>
            <option>本季度</option>
          </select>
        </div>
      </header>

      {/* 主内容 */}
      <main className="flex-1 overflow-auto bg-bg p-4">
        {activeTab === "概览" && (
          <div className="max-w-7xl mx-auto space-y-4">
            {/* 4 KPI 卡片 (真实 store 数据) */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
              {kpiData.map((kpi, i) => (
                <div key={i} data-testid={`analytics-kpi-${i}`} className="bg-bg-card rounded-xl border border-line p-4">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-ink-dim">{kpi.label}</span>
                    <kpi.icon size={14} className={i === 0 ? "text-ok" : i === 1 ? "text-accent" : i === 2 ? "text-info" : "text-warn"} />
                  </div>
                  <div className="mt-2 flex items-baseline justify-between">
                    <span className="text-2xl font-bold text-ink">{kpi.value}</span>
                    <span className={`text-xs font-semibold ${kpi.trendUp ? "text-ok" : "text-err"}`}>{kpi.trend}</span>
                  </div>
                </div>
              ))}
            </div>

            {/* IPA 阶段进度 */}
            <section data-testid="analytics-ipa-phases" className="bg-bg-card rounded-xl border border-line p-4">
              <div className="flex items-center justify-between mb-3">
                <h2 className="text-sm font-bold text-ink">IPA 阶段进度</h2>
                <span className="text-[10px] px-1.5 py-0.5 rounded border border-line text-ink-dim">Branch: feat/relation-bfs</span>
              </div>
              <div className="space-y-2">
                {IPA_PHASES.map((phase, i) => {
                  const pct = phase.total > 0 ? Math.round((phase.done / phase.total) * 100) : 0;
                  return (
                    <div key={phase.id} className="flex items-center gap-3">
                      <span className="text-[10px] font-mono text-ink-dim w-10 text-right">{phase.id}</span>
                      <span className="text-xs text-ink-dim w-24">{phase.label}</span>
                      <div className="flex-1 h-3 bg-bg-soft rounded-full overflow-hidden">
                        <div
                          className="h-full bg-accent rounded-full transition-all duration-500"
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                      <span className="text-[10px] font-mono text-ink-dim w-20 text-right">
                        {phase.done} / {phase.total} ({pct}%)
                      </span>
                    </div>
                  );
                })}
              </div>
            </section>

            {/* 租户/项目概览 */}
            <section data-testid="analytics-scope" className="bg-bg-card rounded-xl border border-line p-4">
              <h2 className="text-sm font-bold text-ink mb-3">范围: Physis / GVPE</h2>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs">
                <div className="p-3 rounded-lg bg-bg-soft border border-line">
                  <span className="text-ink-dim">项目</span>
                  <p className="font-mono text-ink mt-1">Physis / GVPE · StarGate Dashboard · Mobile Companion</p>
                </div>
                <div className="p-3 rounded-lg bg-bg-soft border border-line">
                  <span className="text-ink-dim">时间范围</span>
                  <p className="font-mono text-ink mt-1">近 14 天</p>
                </div>
                <div className="p-3 rounded-lg bg-bg-soft border border-line">
                  <span className="text-ink-dim">当前 Branch</span>
                  <p className="font-mono text-ink mt-1">feat/relation-bfs</p>
                </div>
              </div>
            </section>
          </div>
        )}

        {activeTab === "运维" && (
          <div className="max-w-7xl mx-auto space-y-4">
            <section className="bg-bg-card rounded-xl border border-line p-4">
              <h2 className="text-sm font-bold text-ink mb-3">运维指标 (对接 MetricsTab / LogAITab)</h2>
              <p className="text-xs text-ink-dim">集成 /ops (运维控制台) 与 star-api-rest / canvas-engine 等服务指标</p>
              <div className="mt-3 grid grid-cols-2 md:grid-cols-4 gap-3 text-xs">
                {[
                  { label: "API 可用性", value: "99.97%", color: "text-ok" },
                  { label: "P95 延迟", value: "142ms", color: "text-info" },
                  { label: "错误率", value: "0.03%", color: "text-ok" },
                  { label: "事件积压", value: "12", color: "text-warn" },
                ].map((m, i) => (
                  <div key={i} className="p-3 rounded-lg bg-bg-soft border border-line">
                    <span className="text-ink-dim">{m.label}</span>
                    <p className={`font-mono text-ink ${m.color} mt-0.5`}>{m.value}</p>
                  </div>
                ))}
              </div>
            </section>
          </div>
        )}

        {activeTab === "审计日志" && (
          <div className="max-w-7xl mx-auto space-y-4">
            <section className="bg-bg-card rounded-xl border border-line p-4">
              <h2 className="text-sm font-bold text-ink mb-3">审计日志</h2>
              <p className="text-xs text-ink-dim mb-3">系统关键操作记录 (登录 / 权限变更 / 部署 / 删除 等)</p>
              <div className="overflow-x-auto">
                <table className="w-full text-xs">
                  <thead>
                    <tr className="text-left text-ink-dim border-b border-line">
                      <th className="py-2 px-3">时间</th>
                      <th className="py-2 px-3">用户</th>
                      <th className="py-2 px-3">操作</th>
                      <th className="py-2 px-3">资源</th>
                      <th className="py-2 px-3">结果</th>
                    </tr>
                  </thead>
                  <tbody>
                    {[
                      { time: "2026-11-14 10:23", user: "Ulysses", action: "部署", resource: "canvas-engine 0.3.2", result: "成功" },
                      { time: "2026-11-14 09:45", user: "Agent A1", action: "权限变更", resource: "worktree wt-001", result: "成功" },
                      { time: "2026-11-14 08:12", user: "Ulysses", action: "删除", resource: "work-item T-098", result: "失败" },
                      { time: "2026-11-13 18:30", user: "System", action: "定时备份", resource: "全库", result: "成功" },
                    ].map((log, i) => (
                      <tr key={i} className="border-b border-line/50 hover:bg-bg-soft">
                        <td className="py-2 px-3 font-mono text-ink-dim">{log.time}</td>
                        <td className="py-2 px-3 text-ink">{log.user}</td>
                        <td className="py-2 px-3 text-ink">{log.action}</td>
                        <td className="py-2 px-3 font-mono text-ink-dim">{log.resource}</td>
                        <td className="py-2 px-3">
                          <span className={`text-[10px] px-1.5 py-0.5 rounded ${log.result === "成功" ? "bg-ok/10 text-ok" : "bg-err/10 text-err"}`}>{log.result}</span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          </div>
        )}
      </main>
    </div>
  );
}
