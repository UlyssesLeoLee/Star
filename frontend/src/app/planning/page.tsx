"use client";

import { useMemo, useState } from "react";
import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { GanttChart } from "@/components/gantt";
import { Tabs } from "@/components/Tabs";
import { MonthView } from "@/components/calendar/MonthView";
import { WeekView } from "@/components/calendar/WeekView";
import { CalendarHeader } from "@/components/calendar/CalendarHeader";
import { CalendarLegend } from "@/components/calendar/CalendarLegend";
import { buildEvents } from "@/components/calendar/events";
import {
  Calendar,
  TrendingDown,
  Flag,
  Target,
  SquareChartGantt,
  CalendarRange,
} from "lucide-react";
import { addDays, format, parseISO, differenceInDays } from "date-fns";

export default function PlanningPage() {
  const sprints = useStore((s) => s.sprints);
  const milestones = useStore((s) => s.milestones);
  const burndown = useStore((s) => s.burndownSeries);
  const workItems = useStore((s) => s.workItems);
  const transitionMilestone = useStore((s) => s.transitionMilestone);
  const transitionSprint = useStore((s) => s.transitionSprint);

  // W3 calendar 拖动 → 更新 work-item due_date (per dynamic-interaction-design.md §5.3)
  // W5 store 还没实装 updateWorkItemDueDate / transitionWorkItemSprint,
  // 用 useStore.setState 直接改, 与 BoardPage (W1) 风格一致
  // (per W3 守门: 不重写 store, 仅在调用方补偿; U3 接手时 store 升级后会替换)
  const updateWorkItemDueDate = (workItemId: string, isoDueDate: string) => {
    useStore.setState((s) => ({
      workItems: s.workItems.map((w) =>
        w.id === workItemId
          ? { ...w, due_date: isoDueDate, updated_at: new Date().toISOString() }
          : w,
      ),
    }));
  };
  const transitionWorkItemSprint = (workItemId: string, newSprintId: string) => {
    useStore.setState((s) => ({
      workItems: s.workItems.map((w) =>
        w.id === workItemId
          ? { ...w, sprint_id: newSprintId, updated_at: new Date().toISOString() }
          : w,
      ),
    }));
  };

  const [tab, setTab] = useState<string>("gantt");
  const [view, setView] = useState<"month" | "week">("month");
  const [cursor, setCursor] = useState<{ year: number; month: number }>(() => {
    const now = new Date();
    return { year: now.getFullYear(), month: now.getMonth() };
  });
  const [weekStart, setWeekStart] = useState<Date>(() => {
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    return now;
  });

  const events = useMemo(
    () => buildEvents(sprints, milestones, workItems),
    [sprints, milestones, workItems],
  );

  const handleEventMove = (eventId: string, newDate: string) => {
    const ev = events.find((e) => e.id === eventId);
    if (!ev) return;
    const iso = `${newDate}T00:00:00.000Z`;
    if (ev.kind === "milestone") {
      transitionMilestone(eventId, iso);
    } else if (ev.kind === "work_item") {
      updateWorkItemDueDate(eventId, iso);
    }
  };

  const handleMonthChange = (year: number, month: number) =>
    setCursor({ year, month });

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
    transitionWorkItemSprint(workItemId, newSprintId);
  };

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Planning"
        subtitle={`Sprint / Milestone / Burndown + Gantt 时间轴 (W2) + Calendar 月/周 (W3) 四件套。Calendar 拖 work-item/milestone 改 due_date, Gantt 拖动改 milestone due_date / sprint 起止 / work-item 跨 sprint。`}
        icon={<Calendar className="text-accent" size={20} />}
        track="E"
        count={`${sprints.length} sprints / ${milestones.length} milestones / ${workItems.length} work-items`}
      />

      <Tabs
        active={tab}
        onChange={setTab}
        items={[
          { id: "overview",  label: "Overview",  icon: <Target size={12} />,          badge: `${sprints.length + milestones.length}` },
          { id: "burndown",  label: "Burndown",  icon: <TrendingDown size={12} />,   badge: burndown.length },
          { id: "gantt",     label: "Gantt",     icon: <SquareChartGantt size={12} />, badge: milestones.length },
          { id: "calendar",  label: "Calendar",  icon: <CalendarRange size={12} />,   badge: events.length },
          { id: "sprints",   label: "Sprints",   icon: <Flag size={12} />,             badge: sprints.length },
        ]}
      />

      {tab === "overview" && (
        <div data-testid="tab-overview">
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
                <line x1="30" y1="20" x2="30"  y2="170" stroke="#21262d" />
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
        </div>
      )}

      {tab === "burndown" && (
        <div data-testid="tab-burndown" className="card">
          <SectionTitle><TrendingDown size={11} className="inline mr-1" /> Sprint burndown (14 days)</SectionTitle>
          <svg viewBox="0 0 500 200" className="w-full h-64">
            <defs>
              <linearGradient id="burn-fill-2" x1="0" x2="0" y1="0" y2="1">
                <stop offset="0%" stopColor="#2f81f7" stopOpacity="0.3" />
                <stop offset="100%" stopColor="#2f81f7" stopOpacity="0" />
              </linearGradient>
            </defs>
            <line x1="30" y1="170" x2="490" y2="170" stroke="#21262d" />
            <line x1="30" y1="20" x2="30"  y2="170" stroke="#21262d" />
            <line x1="30" y1="30" x2="490" y2="155" stroke="#6e7681" strokeDasharray="3,3" />
            <path
              d={`M 30 ${170 - (burndown[0].remaining_points / maxRemaining) * 140} ` +
                 burndown.map((b, i) => `L ${30 + (i * 460 / (burndown.length - 1))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ") +
                 ` L 490 170 L 30 170 Z`}
              fill="url(#burn-fill-2)"
            />
            <path
              d={`M 30 ${170 - (burndown[0].remaining_points / maxRemaining) * 140} ` +
                 burndown.map((b, i) => `L ${30 + (i * 460 / (burndown.length - 1))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ")}
              fill="none"
              stroke="#2f81f7"
              strokeWidth="2"
            />
          </svg>
        </div>
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
        <div data-testid="tab-calendar" className="space-y-3">
          <CalendarHeader
            year={cursor.year}
            month={cursor.month}
            weekStart={weekStart}
            view={view}
            onPrev={() => {
              if (view === "month") {
                const d = new Date(cursor.year, cursor.month - 1, 1);
                setCursor({ year: d.getFullYear(), month: d.getMonth() });
              } else {
                const d = new Date(weekStart);
                d.setDate(d.getDate() - 7);
                setWeekStart(d);
              }
            }}
            onNext={() => {
              if (view === "month") {
                const d = new Date(cursor.year, cursor.month + 1, 1);
                setCursor({ year: d.getFullYear(), month: d.getMonth() });
              } else {
                const d = new Date(weekStart);
                d.setDate(d.getDate() + 7);
                setWeekStart(d);
              }
            }}
            onToday={() => {
              const now = new Date();
              setCursor({ year: now.getFullYear(), month: now.getMonth() });
              const w = new Date(now);
              w.setHours(0, 0, 0, 0);
              setWeekStart(w);
            }}
            onViewChange={setView}
            userTimezone={Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"}
          />

          {view === "month" ? (
            <MonthView
              year={cursor.year}
              month={cursor.month}
              events={events}
              onEventMove={handleEventMove}
              onMonthChange={handleMonthChange}
            />
          ) : (
            <WeekView
              startDate={weekStart}
              events={events}
              onEventMove={handleEventMove}
              userTimezone={Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"}
            />
          )}

          <CalendarLegend />
        </div>
      )}

      {tab === "sprints" && (
        <div data-testid="tab-sprints">
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
        </div>
      )}
    </div>
  );
}
