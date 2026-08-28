"use client";

import { useState, useMemo } from "react";
import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Calendar, TrendingDown, Flag, Target, SquareChartGantt } from "lucide-react";
import { GanttChart } from "@/components/gantt";
import {
  transitionMilestone,
  transitionSprint,
  transitionWorkItemSprint,
} from "@/components/gantt";
import { addDays, format, parseISO, differenceInDays } from "date-fns";

type TabKey = "overview" | "gantt" | "calendar";

const TABS: Array<{ key: TabKey; label: string; icon: React.ReactNode }> = [
  { key: "overview", label: "Sprint / Milestone", icon: <Target size={11} /> },
  { key: "gantt", label: "Gantt", icon: <SquareChartGantt size={11} /> },
  { key: "calendar", label: "Calendar", icon: <Calendar size={11} /> },
];

export default function PlanningPage() {
  const sprints = useStore((s) => s.sprints);
  const milestones = useStore((s) => s.milestones);
  const burndown = useStore((s) => s.burndownSeries);
  const workItems = useStore((s) => s.workItems);

  const [tab, setTab] = useState<TabKey>("gantt");

  const maxRemaining = Math.max(...burndown.map((b) => b.remaining_points), 1);

  // Gantt dateRange: from earliest start - 7d to latest end + 7d, capped at 120d
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
    // cap at 180d for sanity
    if (differenceInDays(end, start) > 180) {
      return { start: format(start, "yyyy-MM-dd"), end: format(addDays(start, 180), "yyyy-MM-dd") };
    }
    return { start: format(start, "yyyy-MM-dd"), end: format(end, "yyyy-MM-dd") };
  }, [sprints, milestones]);

  // W2 stub handlers: console + audit mock (W5 替换为真实 store action)
  const handleMilestoneUpdate = (id: string, newDueDate: string) => {
    transitionMilestone(id, newDueDate);
  };
  const handleSprintUpdate = (id: string, newStart: string, newEnd: string) => {
    transitionSprint(id, newStart, newEnd);
  };
  const handleWorkItemMove = (workItemId: string, newSprintId: string) => {
    transitionWorkItemSprint(workItemId, newSprintId);
  };

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Planning"
        subtitle="Sprint / Milestone / Burndown 三件套 + Gantt 时间轴 (W2 实装)。每个 sprint 有 capacity / committed / completed 三维度。"
        icon={<Calendar className="text-accent" size={20} />}
        track="E"
        count={`${sprints.length} sprints / ${milestones.length} milestones`}
      />

      {/* Tabs (per W2 任务 §2) */}
      <div className="flex items-center gap-1 mb-4 border-b border-line">
        {TABS.map((t) => (
          <button
            key={t.key}
            type="button"
            onClick={() => setTab(t.key)}
            data-tab={t.key}
            data-active={tab === t.key ? "true" : "false"}
            className={`px-3 py-2 text-xs flex items-center gap-1.5 border-b-2 transition-colors ${
              tab === t.key
                ? "border-accent text-accent"
                : "border-transparent text-ink-dim hover:text-ink hover:border-line"
            }`}
          >
            {t.icon}
            {t.label}
            {t.key === "calendar" && (
              <span className="text-[9px] text-ink-mute ml-1 font-mono">(W3)</span>
            )}
          </button>
        ))}
      </div>

      {tab === "overview" && (
        <>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-3 mb-5">
            <div className="card">
              <SectionTitle><TrendingDown size={11} className="inline mr-1" /> Burndown</SectionTitle>
              <svg viewBox="0 0 500 200" className="w-full h-48">
                <defs>
                  <linearGradient id="burn-fill" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0%" stopColor="#2f81f7" stopOpacity="0.3" />
                    <stop offset="100%" stopColor="#2f81f7" stopOpacity="0" />
                  </linearGradient>
                </defs>
                <line x1="30" y1="170" x2="490" y2="170" stroke="#21262d" />
                <line x1="30" y1="20"  x2="30"  y2="170" stroke="#21262d" />
                <line x1="30" y1="30" x2="490" y2="155" stroke="#6e7681" strokeDasharray="3,3" />
                <path
                  d={`M 30 ${170 - (burndown[0].remaining_points / maxRemaining) * 140} ` +
                     burndown.map((b, i) => `L ${30 + (i * 460 / (burndown.length - 1))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ") +
                     ` L 490 170 L 30 170 Z`}
                  fill="url(#burn-fill)"
                />
                <path
                  d={`M 30 ${170 - (burndown[0].remaining_points / maxRemaining) * 140} ` +
                     burndown.map((b, i) => `L ${30 + (i * 460 / (burndown.length - 1))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ")}
                  fill="none"
                  stroke="#2f81f7"
                  strokeWidth="2"
                />
                <text x="5" y="30" fontSize="9" fill="#6e7681">{maxRemaining}</text>
                <text x="5" y="170" fontSize="9" fill="#6e7681">0</text>
                <text x="240" y="190" fontSize="9" fill="#6e7681" textAnchor="middle">days</text>
              </svg>
              <div className="flex items-center gap-3 text-[10px] text-ink-mute">
                <span className="flex items-center gap-1"><span className="w-3 h-0.5 bg-info" /> actual</span>
                <span className="flex items-center gap-1"><span className="w-3 h-0.5 bg-ink-mute" style={{borderTop:"1px dashed #6e7681"}} /> ideal</span>
              </div>
            </div>

            <div className="card">
              <SectionTitle><Flag size={11} className="inline mr-1" /> Milestones</SectionTitle>
              <div className="space-y-2">
                {milestones.map((m) => (
                  <div key={m.id} className="p-2 rounded border border-line bg-bg-soft/40">
                    <div className="flex items-center justify-between mb-1">
                      <div className="text-sm font-medium">{m.name}</div>
                      <span className="text-[10px] text-ink-mute font-mono">due {new Date(m.due_date).toLocaleDateString()}</span>
                    </div>
                    <div className="h-1.5 rounded bg-bg-soft overflow-hidden">
                      <div className="h-full bg-accent" style={{ width: `${m.progress * 100}%` }} />
                    </div>
                    <div className="mt-1 flex items-center justify-between text-[10px] text-ink-mute font-mono">
                      <span>{m.work_item_ids.length} work-items</span>
                      <span>{(m.progress * 100).toFixed(0)}% complete</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          <SectionTitle><Target size={11} className="inline mr-1" /> Sprints</SectionTitle>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {sprints.map((sp) => {
              const pct = sp.committed_points > 0 ? sp.completed_points / sp.committed_points : 0;
              return (
                <div key={sp.id} className="card">
                  <div className="flex items-center justify-between mb-2">
                    <div>
                      <div className="text-sm font-semibold">{sp.name}</div>
                      <div className="text-xs text-ink-dim mt-0.5">{sp.goal}</div>
                    </div>
                    <StatusPill value={sp.status} />
                  </div>
                  <div className="grid grid-cols-3 gap-2 mb-2">
                    <Stat label="Capacity" value={sp.capacity_points} />
                    <Stat label="Committed" value={sp.committed_points} tone="info" />
                    <Stat label="Completed" value={sp.completed_points} tone="ok" />
                  </div>
                  <div className="h-2 rounded bg-bg-soft overflow-hidden">
                    <div className="h-full bg-gradient-to-r from-info to-ok" style={{ width: `${pct * 100}%` }} />
                  </div>
                  <div className="mt-1 text-[10px] text-ink-mute font-mono">
                    {new Date(sp.start_date).toLocaleDateString()} → {new Date(sp.end_date).toLocaleDateString()}
                  </div>
                </div>
              );
            })}
          </div>
        </>
      )}

      {tab === "gantt" && (
        <GanttChart
          sprints={sprints}
          milestones={milestones}
          workItems={workItems}
          dateRange={dateRange}
          onMilestoneUpdate={handleMilestoneUpdate}
          onSprintUpdate={handleSprintUpdate}
          onWorkItemMove={handleWorkItemMove}
        />
      )}

      {tab === "calendar" && (
        <div className="card text-center text-ink-mute text-sm py-12" data-tab-placeholder="calendar">
          <Calendar size={28} className="mx-auto mb-2 text-ink-dim" />
          <div>Calendar view (W3 模块占位)</div>
          <div className="text-[10px] mt-1 font-mono">per dynamic-interaction-design.md §5</div>
        </div>
      )}
    </div>
  );
}
