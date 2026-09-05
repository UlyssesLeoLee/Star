"use client";

import { useMemo, useState } from "react";
import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Tabs } from "@/components/Tabs";
import { MonthView } from "@/components/calendar/MonthView";
import { WeekView } from "@/components/calendar/WeekView";
import { CalendarHeader } from "@/components/calendar/CalendarHeader";
import { CalendarLegend } from "@/components/calendar/CalendarLegend";
import { buildEvents } from "@/components/calendar/events";
import {
  Calendar,
  Flag,
  Target,
  CalendarRange,
} from "lucide-react";
import { useTranslation } from "@/lib/i18n";

export default function PlanningPage() {
  const { t } = useTranslation();
  const sprints = useStore((s) => s.sprints);
  const milestones = useStore((s) => s.milestones);
  const workItems = useStore((s) => s.workItems);
  const transitionMilestone = useStore((s) => s.transitionMilestone);

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

  const [tab, setTab] = useState<string>("sprints");
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

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/planning'].title}
        subtitle="Sprint 冲刺管理、排期日历与项目里程碑规划（甘特图与燃尽图已归入图表中心）"
        icon={<Calendar className="text-accent" size={20} />}
        track="E"
        count={`${sprints.length} sprints / ${milestones.length} milestones / ${workItems.length} work-items`}
      />

      <Tabs
        active={tab}
        onChange={setTab}
        items={[
          { id: "sprints",    label: "Sprints 冲刺",    icon: <Flag size={12} />,          badge: sprints.length },
          { id: "calendar",   label: "Calendar 排期日历", icon: <CalendarRange size={12} />,  badge: events.length },
          { id: "milestones", label: "Milestones 里程碑", icon: <Target size={12} />,         badge: milestones.length },
          { id: "overview",   label: "Overview 概览",    icon: <Target size={12} />,         badge: `${sprints.length + milestones.length}` },
        ]}
      />

      {tab === "milestones" && (
        <div data-testid="tab-milestones" className="card">
          <SectionTitle><Target size={11} className="inline mr-1 text-accent" /> Project Milestones</SectionTitle>
          <div className="space-y-3">
            {milestones.map((m) => (
              <div key={m.id} className="p-3 rounded border border-line bg-bg-soft/50 hover:border-accent/40 transition-colors">
                <div className="flex items-center justify-between mb-1.5">
                  <div className="text-sm font-medium">{m.name}</div>
                  <span className="text-xs text-ink-dim font-mono">due {new Date(m.due_date).toLocaleDateString()}</span>
                </div>
                <div className="h-2 rounded bg-bg-soft overflow-hidden mb-1.5">
                  <div className="h-full bg-accent shadow-[0_0_8px_rgba(0,240,255,0.6)]" style={{ width: `${m.progress * 100}%` }} />
                </div>
                <div className="flex items-center justify-between text-xs text-ink-dim font-mono">
                  <span>{m.work_item_ids.length} work items linked</span>
                  <span>{(m.progress * 100).toFixed(0)}% completed</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === "overview" && (
        <div data-testid="tab-overview" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div className="card">
              <Stat label="Total Sprints" value={sprints.length} tone="info" />
            </div>
            <div className="card">
              <Stat label="Total Milestones" value={milestones.length} tone="ok" />
            </div>
            <div className="card">
              <Stat label="Scheduled Items" value={events.length} />
            </div>
          </div>
          <div className="card">
            <SectionTitle>Quick Milestones Summary</SectionTitle>
            <div className="space-y-2 mt-2">
              {milestones.slice(0, 3).map((m) => (
                <div key={m.id} className="flex items-center justify-between p-2 rounded border border-line bg-bg-soft/30 text-xs">
                  <span className="font-medium">{m.name}</span>
                  <span className="text-ink-dim font-mono">{(m.progress * 100).toFixed(0)}%</span>
                </div>
              ))}
            </div>
          </div>
        </div>
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
