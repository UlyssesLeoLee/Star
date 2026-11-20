/*
@cypher schema=1 source_sha256=2fdcb51f80b0e5fcf741ed2087b07b9e9cdce91510e94a381d901a6fc3591bf0
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

import { useEffect, useState, useMemo } from "react";
import { useStore } from "@/lib/store";
import { CheckCircle2, TrendingUp, Cpu, AlertTriangle, FileText, ListChecks, RefreshCw } from "lucide-react";
import { useTranslation } from "@/lib/i18n";
import { buildAnalyticsData } from "./chartData";
import {
  useBiAnalyticsScope,
  type BiAnalyticsScope,
  type BiTaskEventItem,
  type BiTaskSummary,
} from "./serverAnalytics";
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
import { GanttView } from "@/components/gantt/GanttView";
import { Suspense } from "react";

const ANALYTICS_TABS = [
  "概览", "燃尽图", "燃起图", "速度", "Sprint 报告", "累计流图", "控制图",
  "周期时间", "吞吐量", "预测", "时间跟踪", "解决时间", "SLA 合规",
  "创建vs解决", "问题分布", "优先级分布", "甘特图", "运维", "审计日志",
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

// =====================================================================
// 服务端 BI 指标区 (per P1 BI 接通任务): 状态分布 + 完成量, 每张卡标注
// 数据来源与 as_of; 可展开对回 task-events 源记录列表 (同一查询真源)。
// 无授权会话 / 无 run 上下文时诚实空态, 不使用本地估算冒充服务端 BI。
// =====================================================================
const MAX_RECONCILE_PAGES = 200;
const RECONCILE_PAGE_SIZE = 50;

function scopeStatusLabel(scope: BiAnalyticsScope): { text: string; tone: "ok" | "warn" | "err" } {
  switch (scope.kind) {
    case "ready":
      return { text: `服务端 BI 已接通 · Run ${scope.runId.slice(0, 8)}…`, tone: "ok" };
    case "no-session":
      return { text: "服务端 BI 未接通 — 无宿主认证会话 (数字不可与授权数据核对)", tone: "warn" };
    case "invalid-run":
      return { text: "服务端 BI 未接通 — URL 中 engineering_run_id 格式无效", tone: "err" };
    case "no-run":
      return { text: "服务端 BI 未接通 — 无 engineering run 上下文 (缺 ?engineering_run_id=)", tone: "warn" };
  }
}

function ServerBiSection({ scope }: { scope: BiAnalyticsScope }) {
  const [summary, setSummary] = useState<BiTaskSummary | null>(null);
  const [summaryError, setSummaryError] = useState("");
  const [loading, setLoading] = useState(false);
  const [reconcileOpen, setReconcileOpen] = useState(false);
  const [events, setEvents] = useState<BiTaskEventItem[]>([]);
  const [eventsLoading, setEventsLoading] = useState(false);
  const [eventsError, setEventsError] = useState("");
  const [eventsExhausted, setEventsExhausted] = useState(false);

  const runId = scope.kind === "ready" ? scope.runId : null;
  const client = scope.kind === "ready" ? scope.client : null;

  useEffect(() => {
    let active = true;
    setSummary(null);
    setSummaryError("");
    setEvents([]);
    setEventsExhausted(false);
    if (!runId || !client) {
      setLoading(false);
      return () => { active = false; };
    }
    setLoading(true);
    client.getTaskSummary(runId).then((result) => {
      if (!active) return;
      setSummary(result);
    }).catch((cause: unknown) => {
      if (active) setSummaryError(cause instanceof Error ? cause.message : "服务端汇总读取失败。");
    }).finally(() => {
      if (active) setLoading(false);
    });
    return () => { active = false; };
  }, [runId, client]);

  async function toggleReconcile() {
    if (!runId || !client) return;
    if (reconcileOpen) {
      setReconcileOpen(false);
      return;
    }
    setReconcileOpen(true);
    if (events.length > 0 && eventsExhausted) return;
    setEventsLoading(true);
    setEventsError("");
    setEvents([]);
    let cursor: { before_occurred_at: string; before_event_id: string } | null = null;
    const collected: BiTaskEventItem[] = [];
    let exhausted = false;
    try {
      for (let page = 0; page < MAX_RECONCILE_PAGES; page += 1) {
        const result = await client.listTaskEvents(runId, {
          limit: RECONCILE_PAGE_SIZE,
          ...(cursor ? cursor : {}),
        });
        collected.push(...result.items);
        if (!result.has_more) {
          exhausted = true;
          break;
        }
        if (!result.next_cursor) break;
        cursor = result.next_cursor;
      }
    } catch (cause) {
      setEventsError(cause instanceof Error ? cause.message : "源记录对回失败。");
    } finally {
      setEventsLoading(false);
    }
    setEvents(collected);
    setEventsExhausted(exhausted);
  }

  if (!runId || !client) {
    const status = scopeStatusLabel(scope);
    return (
      <section data-testid="analytics-server-bi" className="rounded-xl border border-warn/40 bg-warn/10 p-4">
        <div className="flex items-start gap-3">
          <AlertTriangle size={15} className="mt-0.5 shrink-0 text-warn" />
          <div>
            <h2 className="text-sm font-bold text-ink">服务端指标</h2>
            <p className="mt-1 text-xs text-warn" data-testid="analytics-server-bi-empty">{status.text}</p>
            <p className="mt-2 text-[11px] text-ink-dim">
              服务端 BI 走 <code className="font-mono">/api/v1/engineering-runs/&#123;run&#125;/bi/task-events</code> 授权查询
              (状态分布 / 完成量均从同一源记录聚合, 可对回核对)。当前页面无 engineering run 上下文, 不展示任何伪装的 BI 数字。
              下方图表全部为本地估算, 已在每张图上标注。
            </p>
          </div>
        </div>
      </section>
    );
  }

  const status = scopeStatusLabel(scope);
  const distTotal = summary
    ? summary.status_distribution.reduce((sum, bucket) => sum + bucket.count, 0)
    : 0;
  const reconcileCount = events.length;
  const consistent = summary !== null
    && eventsExhausted
    && reconcileCount === summary.total_count;
  const latestEventByWorkItem = new Map<string, BiTaskEventItem>();
  for (const item of events) {
    if (!latestEventByWorkItem.has(item.work_item_id)) latestEventByWorkItem.set(item.work_item_id, item);
  }

  return (
    <section data-testid="analytics-server-bi" className="rounded-xl border border-line bg-bg-card p-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 className="flex items-center gap-2 text-sm font-bold text-ink">
            服务端指标
            <span className={`rounded border px-1.5 py-0.5 text-[10px] font-semibold ${status.tone === "ok" ? "border-ok/40 bg-ok/10 text-ok" : status.tone === "warn" ? "border-warn/40 bg-warn/10 text-warn" : "border-err/40 bg-err/10 text-err"}`}>
              {status.text}
            </span>
          </h2>
          <p className="mt-1 text-[11px] text-ink-dim">
            数据来源: <code className="font-mono">bi/task-summary</code> (与 task-events 同一授权查询真源)
            {summary && <> · 聚合时间 as_of {new Date(summary.as_of).toLocaleString()}</>}
            {summary && <> · 覆盖 {summary.source_event_range.earliest_occurred_at ?? "—"} ~ {summary.source_event_range.latest_occurred_at ?? "—"}</>}
          </p>
        </div>
        <button
          type="button"
          data-testid="analytics-bi-reconcile-toggle"
          onClick={toggleReconcile}
          disabled={eventsLoading}
          className="btn-secondary inline-flex items-center gap-1.5 text-xs"
        >
          <ListChecks size={13} />
          {reconcileOpen ? "收起源记录对回" : "对回 task-events 源记录"}
        </button>
      </div>

      {loading && <p className="mt-3 text-xs text-ink-mute">正在读取服务端汇总…</p>}
      {summaryError && <p role="alert" data-testid="analytics-bi-summary-error" className="mt-3 rounded border border-red-500/40 bg-red-500/5 px-3 py-2 text-xs text-red-600">服务端汇总读取失败：{summaryError}</p>}

      {summary && (
        <>
          <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <div data-testid="analytics-bi-total" className="rounded-lg bg-bg-soft p-3">
              <span className="text-[11px] text-ink-dim">事件总数 (total_count)</span>
              <p className="mt-1 text-2xl font-bold font-mono text-ink">{summary.total_count.toLocaleString()}</p>
              <p className="text-[10px] text-ink-mute">与 task-events 同范围全量条数一致</p>
            </div>
            <div data-testid="analytics-bi-completed" className="rounded-lg bg-bg-soft p-3">
              <span className="text-[11px] text-ink-dim">完成量 (completed_count)</span>
              <p className="mt-1 text-2xl font-bold font-mono text-ink">{summary.completed_count.toLocaleString()}</p>
              <p className="text-[10px] text-ink-mute">最新状态为 completed 的任务数</p>
            </div>
            <div data-testid="analytics-bi-work-items" className="rounded-lg bg-bg-soft p-3">
              <span className="text-[11px] text-ink-dim">任务数 (work_item_count)</span>
              <p className="mt-1 text-2xl font-bold font-mono text-ink">{summary.work_item_count.toLocaleString()}</p>
              <p className="text-[10px] text-ink-mute">状态分布合计 = {distTotal.toLocaleString()}</p>
            </div>
            <div className="rounded-lg bg-bg-soft p-3">
              <span className="text-[11px] text-ink-dim">覆盖说明</span>
              <p className="mt-1 text-[11px] text-ink-mute leading-snug">{summary.coverage.basis}</p>
            </div>
          </div>

          <div className="mt-3" data-testid="analytics-bi-status-dist">
            <span className="text-[11px] font-semibold text-ink-dim">任务状态分布 (每任务取最新事件状态)</span>
            {summary.status_distribution.length === 0 ? (
              <p className="mt-1 text-xs text-ink-mute">该范围暂无任务事件。</p>
            ) : (
              <ul className="mt-2 flex flex-wrap gap-2">
                {summary.status_distribution.map((bucket) => (
                  <li key={bucket.status} className="flex items-center gap-1.5 rounded border border-line px-2 py-1 text-[11px]">
                    <span className="font-mono text-ink">{bucket.status}</span>
                    <span className="font-mono text-accent">{bucket.count}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </>
      )}

      {reconcileOpen && (
        <div className="mt-3 rounded-lg border border-line" data-testid="analytics-bi-reconcile">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-line bg-bg-soft/60 px-3 py-2">
            <span className="text-[11px] font-semibold text-ink">对回 task-events 源记录 (每任务标记最新事件)</span>
            {eventsExhausted && summary && (
              <span
                data-testid="analytics-bi-reconcile-verdict"
                className={`rounded border px-1.5 py-0.5 text-[10px] font-semibold ${consistent ? "border-ok/40 bg-ok/10 text-ok" : "border-err/40 bg-err/10 text-err"}`}
              >
                {consistent
                  ? `一致 — 源记录 ${reconcileCount.toLocaleString()} 条 = total_count`
                  : `不一致 — 源记录 ${reconcileCount.toLocaleString()} 条 ≠ total_count ${summary.total_count.toLocaleString()}`}
              </span>
            )}
            {!eventsExhausted && !eventsLoading && !eventsError && (
              <span className="text-[10px] text-warn">分页未走完 (上限 {MAX_RECONCILE_PAGES} 页), 未做全量核对</span>
            )}
          </div>
          {eventsLoading && <p className="px-3 py-2 text-xs text-ink-mute"><RefreshCw size={12} className="mr-1 inline animate-spin" />正在对回源记录…</p>}
          {eventsError && <p role="alert" className="px-3 py-2 text-xs text-red-600">源记录读取失败：{eventsError}</p>}
          {!eventsLoading && !eventsError && events.length === 0 && (
            <p className="px-3 py-2 text-xs text-ink-mute">该范围没有可对回的 task-events 源记录。</p>
          )}
          {events.length > 0 && (
            <div className="max-h-72 overflow-y-auto">
              <table className="w-full text-left text-[11px]" aria-label="task-events 源记录对回列表">
                <thead className="sticky top-0 bg-bg-card">
                  <tr className="border-b border-line text-ink-mute">
                    <th className="px-3 py-1.5">发生时间</th>
                    <th className="px-3 py-1.5">work_item</th>
                    <th className="px-3 py-1.5">事件类型</th>
                    <th className="px-3 py-1.5">lifecycle_status</th>
                    <th className="px-3 py-1.5">变更</th>
                  </tr>
                </thead>
                <tbody>
                  {events.map((item) => {
                    const isLatest = latestEventByWorkItem.get(item.work_item_id)?.event_id === item.event_id;
                    return (
                      <tr key={item.event_id} className="border-b border-line/50 text-ink-dim">
                        <td className="px-3 py-1.5 font-mono">{new Date(item.occurred_at).toLocaleString()}</td>
                        <td className="px-3 py-1.5 font-mono">{item.work_item_id.slice(0, 8)}…</td>
                        <td className="px-3 py-1.5 font-mono">{item.event_type}</td>
                        <td className="px-3 py-1.5 font-mono">
                          {item.lifecycle_status ?? "—"}
                          {isLatest && <span className="ml-1 rounded border border-accent/40 bg-accent/10 px-1 text-[9px] text-accent">最新</span>}
                        </td>
                        <td className="px-3 py-1.5 font-mono">
                          {item.from_status || item.to_status ? `${item.from_status ?? "—"} → ${item.to_status ?? "—"}` : "—"}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </section>
  );
}

export default function AnalyticsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<AnalyticsTab>("概览");
  const biScope = useBiAnalyticsScope();
  const biStatus = scopeStatusLabel(biScope);

  const workItems = useStore((s) => s.workItems);
  const agents = useStore((s) => s.agentSessions);
  const sprints = useStore((s) => s.sprints);
  const board = useStore((s) => s.board);
  const identities = useStore((s) => s.identities);

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
      本地估算 — 基于当前会话数据 (store 聚合), 未接服务端 BI, 不可与授权数据核对；服务端指标见「概览」顶部。
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
          <span
            data-testid="analytics-data-source-status"
            className={`text-[10px] px-1.5 py-0.5 rounded border ${
              biStatus.tone === "ok" ? "border-ok/40 bg-ok/10 text-ok"
                : biStatus.tone === "warn" ? "border-warn/40 bg-warn/10 text-warn"
                : "border-err/40 bg-err/10 text-err"
            }`}
          >
            {biStatus.text}
          </span>
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
            <ServerBiSection scope={biScope} />

            <div
              data-testid="analytics-local-estimate-strip"
              className="flex items-center gap-1.5 rounded-lg border border-warn/40 bg-warn/10 px-3 py-1.5 text-[10px] text-warn"
            >
              <AlertTriangle size={11} />
              以下 KPI / 图表为本地估算 — 基于当前会话数据 (store 聚合), 非服务端 BI, 不可与授权数据核对。
            </div>

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

        {activeTab === "甘特图" && (
          <div className="max-w-7xl mx-auto" data-testid="analytics-chart-甘特图">
            <ChartFrame title="甘特图" description="冲刺 / 里程碑排期与任务时间轴" chartId="C16_GANTT">
              {preview}
              <Suspense fallback={<div className="flex h-full items-center justify-center text-ink-dim">Loading Gantt...</div>}>
                <GanttView board={board} workItems={workItems} identities={identities} />
              </Suspense>
            </ChartFrame>
          </div>
        )}

        {activeTab !== "概览" && activeTab !== "运维" && activeTab !== "审计日志" && activeTab !== "甘特图" && (
          <div className="max-w-7xl mx-auto" data-testid={`analytics-chart-${activeTab}`}>
            {renderChart(activeTab)}
          </div>
        )}

        {activeTab === "运维" && (
          <div className="max-w-7xl mx-auto space-y-4">
            <section className="bg-bg-card rounded-xl border border-line p-4">
              <h2 className="text-sm font-bold text-ink mb-3 flex items-center gap-2">运维指标 (对接 MetricsTab / LogAITab)
                <span data-testid="analytics-ops-demo-badge" className="text-[10px] px-1.5 py-0.5 rounded border border-warn/40 bg-warn/10 text-warn">演示数据 — 未接入服务端指标</span>
              </h2>
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
              <h2 className="text-sm font-bold text-ink mb-3 flex items-center gap-2">审计日志
                <span data-testid="analytics-audit-demo-badge" className="text-[10px] px-1.5 py-0.5 rounded border border-warn/40 bg-warn/10 text-warn">演示数据 — 未接入服务端审计日志</span>
              </h2>
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
