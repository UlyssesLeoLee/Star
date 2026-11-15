/*
@cypher schema=1 source_sha256=16477096ca343be917ff597a9944c5bccd3524df348d4050ef98878af3bb7cdf
MERGE (self:File {path:"frontend/src/app/(app)/analytics/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/analytics/page.tsx::AnalyticsPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /analytics — 分析 (per 2026-11-15 用户发令: 所有分析/BI 类都在分析界面,
//   燃尽图等图表合并进本页, 标签页切换 (不用下拉菜单))
// =====================================================================
"use client";

import { useState, useMemo } from "react";
import { useStore } from "@/lib/store";
import { CheckCircle2, TrendingUp, Cpu, AlertTriangle, FileText } from "lucide-react";
import { useTranslation } from "@/lib/i18n";
import { buildAnalyticsData } from "./chartData";
import { ChartFrame } from "@/components/charts/shared/ChartFrame";
import { Chart01Burndown } from "@/components/charts/Chart01Burndown";
import { Chart02Burnup } from "@/components/charts/Chart02Burnup";
import { Chart03Velocity } from "@/components/charts/Chart03Velocity";
import { Chart04SprintReport } from "@/components/charts/Chart04SprintReport";
import { Chart05Cfd } from "@/components/charts/Chart05Cfd";
import { Chart06ControlChart } from "@/components/charts/Chart06ControlChart";
import { Chart07CycleTime } from "@/components/charts/Chart07CycleTime";
import { Chart08Throughput } from "@/components/charts/Chart08Throughput";
import { Chart09Forecast } from "@/components/charts/Chart09Forecast";
import { Chart10TimeTracking } from "@/components/charts/Chart10TimeTracking";
import { Chart11ResolutionTime } from "@/components/charts/Chart11ResolutionTime";
import { Chart12SlaCompliance } from "@/components/charts/Chart12SlaCompliance";
import { Chart13CreatedVsResolved } from "@/components/charts/Chart13CreatedVsResolved";
import { Chart14IssueTypeDist } from "@/components/charts/Chart14IssueTypeDist";
import { Chart15PriorityDist } from "@/components/charts/Chart15PriorityDist";

const ANALYTICS_TABS = [
  "概览", "燃尽图", "燃起图", "速度", "Sprint 报告", "累计流图", "控制图",
  "周期时间", "吞吐量", "预测", "时间跟踪", "解决时间", "SLA 合规",
  "创建vs解决", "问题分布", "优先级分布", "运维", "审计日志",
] as const;
type AnalyticsTab = (typeof ANALYTICS_TABS)[number];

const IPA_PHASES = [
  { id: "P2", label: "需求定义", done: 2, total: 20 },
  { id: "P3", label: "基本设计", done: 1, total: 21 },
  { id: "P4", label: "详细设计", done: 2, total: 20 },
  { id: "P5", label: "实现", done: 2, total: 15 },
  { id: "P6.1", label: "单元测试", done: 2, total: 11 },
  { id: "P6.2", label: "集成测试", done: 1, total: 12 },
] as const;

export default function AnalyticsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<AnalyticsTab>("概览");

  const workItems = useStore((s) => s.workItems);
  const agents = useStore((s) => s.agentSessions);
  const sprints = useStore((s) => s.sprints);

  const charts = useMemo(() => buildAnalyticsData(sprints, workItems), [sprints, workItems]);

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

  const preview = (
    <p className="mb-2 flex items-center gap-1.5 rounded-lg border border-warn/40 bg-warn/10 px-3 py-1.5 text-[10px] text-warn">
      <AlertTriangle size={11} />
      预览数据 — 图表基于本地 store 聚合；宿主 BI/数据库投影接入后切换为授权事实。
    </p>
  );

  const renderChart = (tab: AnalyticsTab) => {
    switch (tab) {
      case "燃尽图": return <ChartFrame title="燃尽图" description="剩余故事点 vs 理想线" chartId="C01_BURNDOWN">{preview}<Chart01Burndown data={charts.burndown} /></ChartFrame>;
      case "燃起图": return <ChartFrame title="燃起图" description="累计完成 vs 范围" chartId="C02_BURNUP">{preview}<Chart02Burnup data={charts.burnup} /></ChartFrame>;
      case "速度": return <ChartFrame title="速度" description="每冲刺完成故事点" chartId="C03_VELOCITY">{preview}<Chart03Velocity data={charts.velocity} /></ChartFrame>;
      case "Sprint 报告": return <ChartFrame title="Sprint 报告" description="完成 / 顺延 / 未完成" chartId="C04_SPRINT_REPORT">{preview}<Chart04SprintReport data={charts.sprintReport} /></ChartFrame>;
      case "累计流图": return <ChartFrame title="累计流图 (CFD)" description="各状态任务随时间累计" chartId="C05_CFD">{preview}<Chart05Cfd data={charts.cfd} /></ChartFrame>;
      case "控制图": return <ChartFrame title="控制图" description="周期时间稳定性" chartId="C06_CONTROL">{preview}<Chart06ControlChart data={charts.control} /></ChartFrame>;
      case "周期时间": return <ChartFrame title="周期时间" description="任务从开始到完成的时长分布" chartId="C07_CYCLE_TIME">{preview}<Chart07CycleTime data={charts.cycleTime} /></ChartFrame>;
      case "吞吐量": return <ChartFrame title="吞吐量" description="每周完成任务数" chartId="C08_THROUGHPUT">{preview}<Chart08Throughput data={charts.throughput} /></ChartFrame>;
      case "预测": return <ChartFrame title="预测" description="按速度外推完成日期" chartId="C09_FORECAST">{preview}<Chart09Forecast data={charts.forecast} /></ChartFrame>;
      case "时间跟踪": return <ChartFrame title="时间跟踪" description="预估 vs 实际工时" chartId="C10_TIME_TRACKING">{preview}<Chart10TimeTracking data={charts.timeTracking} /></ChartFrame>;
      case "解决时间": return <ChartFrame title="解决时间" description="按优先级分组的平均解决时长" chartId="C11_RESOLUTION">{preview}<Chart11ResolutionTime data={charts.resolution} /></ChartFrame>;
      case "SLA 合规": return <ChartFrame title="SLA 合规" description="各优先级 SLA 达成率" chartId="C12_SLA">{preview}<Chart12SlaCompliance data={charts.sla} /></ChartFrame>;
      case "创建vs解决": return <ChartFrame title="创建 vs 解决" description="backlog 净变化" chartId="C13_CVR">{preview}<Chart13CreatedVsResolved data={charts.cvr} /></ChartFrame>;
      case "问题分布": return <ChartFrame title="问题类型分布" description="task / bug / feature 占比" chartId="C14_ISSUE_DIST">{preview}<Chart14IssueTypeDist data={charts.issueDist} /></ChartFrame>;
      case "优先级分布": return <ChartFrame title="优先级分布" description="critical / high / medium / low" chartId="C15_PRIORITY_DIST">{preview}<Chart15PriorityDist data={charts.priorityDist} /></ChartFrame>;
      default: return null;
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg">
      <header className="border-b border-line bg-bg-card px-4 py-3 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <h1 className="text-base font-bold text-ink">分析</h1>
          <span className="text-[10px] px-1.5 py-0.5 rounded border border-line text-ink-dim">全局</span>
        </div>
        <div className="flex items-center gap-2">
          <select className="text-xs px-2 py-1.5 rounded border border-line bg-bg-soft text-ink" aria-label="项目">
            <option>Physis / GVPE</option>
            <option>StarGate Dashboard</option>
            <option>Mobile Companion</option>
          </select>
          <select className="text-xs px-2 py-1.5 rounded border border-line bg-bg-soft text-ink" aria-label="时间范围">
            <option>近 14 天</option>
            <option>近 30 天</option>
            <option>本季度</option>
          </select>
        </div>
      </header>

      <nav
        data-testid="analytics-tabs"
        className="flex items-center gap-0.5 overflow-x-auto border-b border-line bg-bg-card px-3 py-1.5"
        aria-label="分析视图"
      >
        {ANALYTICS_TABS.map((tab) => (
          <button
            key={tab}
            type="button"
            data-testid={`analytics-tab-${tab}`}
            onClick={() => setActiveTab(tab)}
            aria-pressed={activeTab === tab}
            className={`shrink-0 rounded-lg px-2.5 py-1 text-[11px] font-semibold transition-colors whitespace-nowrap ${
              activeTab === tab ? "bg-accent text-bg" : "text-ink-mute hover:text-ink hover:bg-bg-soft"
            }`}
          >
            {tab}
          </button>
        ))}
      </nav>

      <main className="flex-1 overflow-auto bg-bg p-4">
        {activeTab === "概览" && (
          <div className="max-w-7xl mx-auto space-y-4">
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

            <section data-testid="analytics-ipa-phases" className="bg-bg-card rounded-xl border border-line p-4">
              <div className="flex items-center justify-between mb-3">
                <h2 className="text-sm font-bold text-ink">IPA 阶段进度</h2>
                <span className="text-[10px] px-1.5 py-0.5 rounded border border-line text-ink-dim">Branch: feat/relation-bfs</span>
              </div>
              <div className="space-y-2">
                {IPA_PHASES.map((phase) => {
                  const pct = phase.total > 0 ? Math.round((phase.done / phase.total) * 100) : 0;
                  return (
                    <div key={phase.id} className="flex items-center gap-3">
                      <span className="text-[10px] font-mono text-ink-dim w-10 text-right">{phase.id}</span>
                      <span className="text-xs text-ink-dim w-24">{phase.label}</span>
                      <div className="flex-1 h-3 bg-bg-soft rounded-full overflow-hidden">
                        <div className="h-full bg-accent rounded-full transition-all duration-500" style={{ width: `${pct}%` }} />
                      </div>
                      <span className="text-[10px] font-mono text-ink-dim w-20 text-right">
                        {phase.done} / {phase.total} ({pct}%)
                      </span>
                    </div>
                  );
                })}
              </div>
            </section>

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

        {activeTab !== "概览" && activeTab !== "运维" && activeTab !== "审计日志" && (
          <div className="max-w-7xl mx-auto" data-testid={`analytics-chart-${activeTab}`}>
            {renderChart(activeTab)}
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
