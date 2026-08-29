"use client";

// =====================================================================
// /analytics — Dashboard KPI + 图表中心 (Burndown, Gantt, Cost, Velocity, Leaderboard)
// =====================================================================
// 日漫风格 Charisma 科技美学 + 低认知负荷选项卡架构
// =====================================================================

import { useEffect, useMemo, useState } from "react";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { Tabs } from "@/components/Tabs";
import { GanttChart } from "@/components/gantt";
import { useStore } from "@/lib/store";
import {
  BarChart3,
  TrendingDown,
  TrendingUp,
  SquareChartGantt,
  Activity,
  Trophy,
  DollarSign,
  Cpu,
} from "lucide-react";
import { addDays, format, parseISO, differenceInDays } from "date-fns";
import { MOCK_KPI_FALLBACK, COST_SERIES_FALLBACK } from "@/mocks/data";
import type { KpiCard, CostPoint } from "@/mocks/schemas/analytics";

function MiniLineChart({ data }: { data: ReadonlyArray<CostPoint> }) {
  const W = 500;
  const H = 160;
  const PAD = 16;
  const max = Math.max(...data.map((d) => d.usd), 1);
  const stepX = (W - PAD * 2) / Math.max(data.length - 1, 1);
  const points = data.map((d, i) => {
    const x = PAD + i * stepX;
    const y = H - PAD - (d.usd / max) * (H - PAD * 2);
    return `${x},${y}`;
  });
  const linePath = `M ${points.join(" L ")}`;
  return (
    <svg
      data-testid="cost-trend-chart"
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-48"
      role="img"
      aria-label="Cost trend (mock)"
    >
      <defs>
        <linearGradient id="cost-fill-anime" x1="0" x2="0" y1="0" y2="1">
          <stop offset="0%" stopColor="#00f0ff" stopOpacity="0.35" />
          <stop offset="100%" stopColor="#8b5cf6" stopOpacity="0" />
        </linearGradient>
      </defs>
      <line x1={PAD} y1={H - PAD} x2={W - PAD} y2={H - PAD} stroke="var(--color-border)" strokeWidth="1" />
      <line x1={PAD} y1={PAD} x2={PAD} y2={H - PAD} stroke="var(--color-border)" strokeWidth="1" />
      <path
        d={`${linePath} L ${PAD + (data.length - 1) * stepX},${H - PAD} L ${PAD},${H - PAD} Z`}
        fill="url(#cost-fill-anime)"
      />
      <path
        d={linePath}
        fill="none"
        stroke="#00f0ff"
        strokeWidth={2}
        className="drop-shadow-[0_0_8px_rgba(0,240,255,0.6)]"
      />
      {data.map((d, i) => {
        const x = PAD + i * stepX;
        const y = H - PAD - (d.usd / max) * (H - PAD * 2);
        return (
          <circle
            key={d.day}
            cx={x}
            cy={y}
            r={3.5}
            className="fill-accent stroke-bg-card stroke-2 drop-shadow-[0_0_6px_rgba(0,240,255,0.8)]"
          />
        );
      })}
    </svg>
  );
}

export default function AnalyticsPage() {
  const [activeTab, setActiveTab] = useState<string>("burndown");
  const [kpi, setKpi] = useState<ReadonlyArray<KpiCard>>(MOCK_KPI_FALLBACK);
  const [costSeries, setCostSeries] = useState<ReadonlyArray<CostPoint>>(COST_SERIES_FALLBACK);

  // Store data for Gantt & Burndown
  const sprints = useStore((s) => s.sprints);
  const milestones = useStore((s) => s.milestones);
  const workItems = useStore((s) => s.workItems);
  const burndown = useStore((s) => s.burndownSeries);
  const transitionMilestone = useStore((s) => s.transitionMilestone);
  const transitionSprint = useStore((s) => s.transitionSprint);

  useEffect(() => {
    fetch("/api/analytics/kpi")
      .then((r) => r.json())
      .then((data: ReadonlyArray<KpiCard>) => setKpi(data))
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标) */
      });
  }, []);

  useEffect(() => {
    fetch("/api/analytics/cost")
      .then((r) => r.json())
      .then((data: ReadonlyArray<CostPoint>) => setCostSeries(data))
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标) */
      });
  }, []);

  const maxRemaining = Math.max(...burndown.map((b) => b.remaining_points), 1);

  const dateRange = useMemo(() => {
    const all = [
      ...sprints.flatMap((s) => [parseISO(s.start_date), parseISO(s.end_date)]),
      ...milestones.map((m) => parseISO(m.due_date)),
    ];
    if (all.length === 0) {
      const today = new Date();
      return { start: format(today, "yyyy-MM-dd"), end: format(addDays(today, 60), "yyyy-MM-dd") };
    }
    const min = all.reduce((a, b) => (a < b ? a : b));
    const max = all.reduce((a, b) => (a > b ? a : b));
    const start = addDays(min, -7);
    const end = addDays(max, 7);
    if (differenceInDays(end, start) > 180) {
      return { start: format(start, "yyyy-MM-dd"), end: format(addDays(start, 180), "yyyy-MM-dd") };
    }
    return { start: format(start, "yyyy-MM-dd"), end: format(end, "yyyy-MM-dd") };
  }, [sprints, milestones]);

  const handleMilestoneUpdate = (id: string, newDueDate: string) => {
    transitionMilestone(id, newDueDate);
  };
  const handleSprintUpdate = (id: string, newStart: string, newEnd: string) => {
    transitionSprint(id, newStart, newEnd);
  };
  const handleWorkItemMove = (workItemId: string, newSprintId: string) => {
    useStore.setState((s) => ({
      workItems: s.workItems.map((w) =>
        w.id === workItemId
          ? { ...w, sprint_id: newSprintId, updated_at: new Date().toISOString() }
          : w,
      ),
    }));
  };

  return (
    <div className="max-w-7xl mx-auto" data-testid="analytics-page">
      <PageHeader
        title="Analytics"
        subtitle="日漫科技分析中心: 甘特图 / 燃尽图 / 成本趋势 / 速率 / 排行榜"
        icon={<BarChart3 className="text-accent" size={20} />}
        count={`${kpi.length} KPIs`}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        {kpi.map((k) => (
          <Stat key={k.label} label={k.label} value={k.value} hint={k.hint} tone={k.tone} />
        ))}
      </div>

      <Tabs
        active={activeTab}
        onChange={setActiveTab}
        items={[
          { id: "burndown", label: "Burndown 燃尽图", icon: <TrendingDown size={12} />, badge: `${burndown.length}d` },
          { id: "gantt", label: "Gantt 甘特图", icon: <SquareChartGantt size={12} />, badge: milestones.length },
          { id: "cost", label: "Cost 成本趋势", icon: <DollarSign size={12} />, badge: `${costSeries.length}d` },
          { id: "velocity", label: "Velocity 速率", icon: <Activity size={12} /> },
          { id: "leaderboard", label: "Leaderboard 排行榜", icon: <Trophy size={12} /> },
        ]}
      />

      {activeTab === "burndown" && (
        <div data-testid="tab-burndown" className="card relative overflow-hidden">
          <div className="flex items-center justify-between mb-2">
            <SectionTitle><TrendingDown size={11} className="inline mr-1 text-accent" /> Sprint Burndown Chart (14 days)</SectionTitle>
            <span className="text-xs text-ink-dim font-mono flex items-center gap-1.5">
              <Cpu size={11} className="text-accent" />
              <span>MAX: <strong className="text-accent font-semibold">{maxRemaining} SP</strong></span>
            </span>
          </div>
          <svg viewBox="0 0 500 200" className="w-full h-64">
            <defs>
              <linearGradient id="burn-fill-analytics" x1="0" x2="0" y1="0" y2="1">
                <stop offset="0%" stopColor="#00f0ff" stopOpacity="0.35" />
                <stop offset="100%" stopColor="#8b5cf6" stopOpacity="0" />
              </linearGradient>
            </defs>
            <line x1="30" y1="170" x2="490" y2="170" stroke="var(--color-border)" />
            <line x1="30" y1="20" x2="30"  y2="170" stroke="var(--color-border)" />
            <line x1="30" y1="30" x2="490" y2="155" stroke="#6e7681" strokeDasharray="4,4" />
            <path
              d={`M 30 ${170 - (burndown[0]?.remaining_points || 0) / maxRemaining * 140} ` +
                 burndown.map((b, i) => `L ${30 + (i * 460 / (Math.max(burndown.length - 1, 1)))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ") +
                 ` L 490 170 L 30 170 Z`}
              fill="url(#burn-fill-analytics)"
            />
            <path
              d={`M 30 ${170 - (burndown[0]?.remaining_points || 0) / maxRemaining * 140} ` +
                 burndown.map((b, i) => `L ${30 + (i * 460 / (Math.max(burndown.length - 1, 1)))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ")}
              fill="none"
              stroke="#00f0ff"
              strokeWidth="2.5"
              className="drop-shadow-[0_0_8px_rgba(0,240,255,0.6)]"
            />
            <text x="5" y="30" fontSize="9" fill="var(--color-text-dim)" fontFamily="monospace">{maxRemaining}</text>
            <text x="5" y="170" fontSize="9" fill="var(--color-text-dim)" fontFamily="monospace">0</text>
            <text x="240" y="190" fontSize="9" fill="var(--color-text-dim)" textAnchor="middle" fontFamily="monospace">// SPRINT TIMELINE (14 DAYS) //</text>
          </svg>
          <div className="flex items-center gap-4 text-xs text-ink-dim mt-2 pt-2 border-t border-line">
            <span className="flex items-center gap-1.5"><span className="w-3 h-0.5 bg-info shadow-[0_0_6px_rgba(0,240,255,0.8)]" /> 实际剩余点数 (Actual Remaining)</span>
            <span className="flex items-center gap-1.5"><span className="w-3 h-0.5 border-t border-dashed border-ink-mute" /> 理想燃尽斜率 (Ideal Guideline)</span>
          </div>
        </div>
      )}

      {activeTab === "gantt" && (
        <div data-testid="tab-gantt" className="space-y-3">
          <GanttChart
            sprints={sprints}
            milestones={milestones}
            workItems={workItems}
            dateRange={dateRange}
            onMilestoneUpdate={handleMilestoneUpdate}
            onSprintUpdate={handleSprintUpdate}
            onWorkItemMove={handleWorkItemMove}
          />
        </div>
      )}

      {activeTab === "cost" && (
        <div className="card mb-3" data-testid="cost-trend-card">
          <div className="flex items-center justify-between mb-2">
            <SectionTitle>Daily Cost Trend (7-day API & Model Spend)</SectionTitle>
            <TrendingUp size={14} className="text-ok drop-shadow-[0_0_6px_rgba(16,185,129,0.5)]" />
          </div>
          <MiniLineChart data={costSeries} />
          <div className="flex justify-between mt-2 font-mono text-xs text-ink-dim">
            {costSeries.map((d) => (
              <span key={d.day}>{d.day} (${d.usd})</span>
            ))}
          </div>
        </div>
      )}

      {activeTab === "velocity" && (
        <div className="card text-center py-12" data-testid="tab-velocity">
          <Activity size={36} className="mx-auto text-accent mb-3 drop-shadow-[0_0_12px_rgba(0,240,255,0.6)]" />
          <div className="text-base font-semibold text-ink">Team Velocity Tracking</div>
          <div className="text-xs text-ink-dim mt-1 max-w-md mx-auto">
            历史冲刺速率分析与预测模型正在接入中。当前平均团队速率约为 <strong className="text-accent">38 SP / Sprint</strong>。
          </div>
        </div>
      )}

      {activeTab === "leaderboard" && (
        <div className="card text-center py-12" data-testid="tab-leaderboard">
          <Trophy size={36} className="mx-auto text-accent mb-3 drop-shadow-[0_0_12px_rgba(0,240,255,0.6)]" />
          <div className="text-base font-semibold text-ink">Agent & Contributor Leaderboard</div>
          <div className="text-xs text-ink-dim mt-1 max-w-md mx-auto">
            Agent 贡献排行、工作项吞吐量与成本效益评分榜单正在接入中 (Phase I+ 缺口)。
          </div>
        </div>
      )}

      <div className="card text-xs text-ink-dim mt-4">
        <SectionTitle>Data Source & Known Gaps (缺标比错标安全)</SectionTitle>
        <ul className="space-y-1 list-disc pl-4 text-ink-dim">
          <li>Gantt / Burndown 已迁移至本视图，由全局 store 统一驱动。</li>
          <li>Cost 真实数据: <span className="font-mono text-ink-mute">/api/analytics/cost</span> 待接入真实流。</li>
          <li>Token 真实数据: <span className="font-mono text-ink-mute">/api/analytics/tokens</span> 待接入。</li>
        </ul>
      </div>
    </div>
  );
}
