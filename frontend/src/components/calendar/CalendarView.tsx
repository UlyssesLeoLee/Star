/*
@cypher schema=1 source_sha256=f23c7a8092e7e76d07535a994ec80e76520f63970d33776644db07f3e76d1b1c
MERGE (self:File {path:"frontend/src/components/calendar/CalendarView.tsx"})
MERGE (v:Symbol {id:"frontend/src/components/calendar/CalendarView.tsx::CalendarView",kind:"component"})
MERGE (header:Symbol {id:"frontend/src/components/calendar/CalendarHeader.tsx::CalendarHeader",kind:"component"})
MERGE (month:Symbol {id:"frontend/src/components/calendar/MonthView.tsx::MonthView",kind:"component"})
MERGE (week:Symbol {id:"frontend/src/components/calendar/WeekView.tsx::WeekView",kind:"component"})
MERGE (self)-[:DEFINES]->(v)
MERGE (self)-[:IMPORTS]->(header)
MERGE (self)-[:IMPORTS]->(month)
MERGE (self)-[:IMPORTS]->(week)
@endcypher
*/
"use client";

// =====================================================================
// CalendarView — 日历容器 (月/周视图 + 游标 + 事件移动)
// per 2026-11-14 用户发令: 老版本日历移植合并 —
//   从 ProjectsClient.tsx 的 Calendar 段提取为独立组件,
//   kanban 日历 tab 与 projects 页共用 (合并去重)
// =====================================================================

import { useCallback, useMemo, useState } from "react";
import { CalendarHeader } from "./CalendarHeader";
import { CalendarLegend } from "./CalendarLegend";
import { MonthView } from "./MonthView";
import { WeekView } from "./WeekView";
import { buildEvents } from "./events";
import type { Sprint, Milestone, WorkItem } from "@/types/ids";

export interface CalendarViewProps {
  sprints: Sprint[];
  milestones: Milestone[];
  workItems: WorkItem[];
  /** 事件移动回调 (可选; 无则只读) */
  onEventMove?: (eventId: string, newDate: string) => void;
  /** 受控视图 (可选; 不传则组件内部自治, 供 FilterBar 月/周子标签控制) */
  view?: "month" | "week";
  onViewChange?: (v: "month" | "week") => void;
}

export function CalendarView({ sprints, milestones, workItems, onEventMove, view, onViewChange }: CalendarViewProps) {
  const [internalView, setInternalView] = useState<"month" | "week">("month");
  const calendarView = view ?? internalView;
  const setCalendarView = useCallback((v: "month" | "week") => {
    setInternalView(v);
    onViewChange?.(v);
  }, [onViewChange]);
  const [calendarCursor, setCalendarCursor] = useState<{ year: number; month: number }>(() => {
    const now = new Date();
    return { year: now.getFullYear(), month: now.getMonth() };
  });
  const [weekStart, setWeekStart] = useState<Date>(() => {
    const w = new Date();
    w.setHours(0, 0, 0, 0);
    return w;
  });

  const events = useMemo(
    () => buildEvents(sprints, milestones, workItems),
    [sprints, milestones, workItems],
  );

  const handleEventMove = useCallback((eventId: string, newDate: string) => {
    onEventMove?.(eventId, newDate);
  }, [onEventMove]);

  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";

  return (
    <div data-testid="calendar-view" className="space-y-2">
      <CalendarHeader
        year={calendarCursor.year}
        month={calendarCursor.month}
        weekStart={weekStart}
        view={calendarView}
        onPrev={() => {
          if (calendarView === "month") {
            const d = new Date(calendarCursor.year, calendarCursor.month - 1, 1);
            setCalendarCursor({ year: d.getFullYear(), month: d.getMonth() });
          } else {
            const d = new Date(weekStart);
            d.setDate(d.getDate() - 7);
            setWeekStart(d);
          }
        }}
        onNext={() => {
          if (calendarView === "month") {
            const d = new Date(calendarCursor.year, calendarCursor.month + 1, 1);
            setCalendarCursor({ year: d.getFullYear(), month: d.getMonth() });
          } else {
            const d = new Date(weekStart);
            d.setDate(d.getDate() + 7);
            setWeekStart(d);
          }
        }}
        onToday={() => {
          const now = new Date();
          setCalendarCursor({ year: now.getFullYear(), month: now.getMonth() });
          const w = new Date(now);
          w.setHours(0, 0, 0, 0);
          setWeekStart(w);
        }}
        onViewChange={setCalendarView}
        userTimezone={timezone}
      />
      {calendarView === "month" ? (
        <MonthView
          year={calendarCursor.year}
          month={calendarCursor.month}
          events={events}
          onEventMove={handleEventMove}
          onMonthChange={(y, m) => setCalendarCursor({ year: y, month: m })}
        />
      ) : (
        <WeekView
          startDate={weekStart}
          events={events}
          onEventMove={handleEventMove}
          userTimezone={timezone}
        />
      )}
      <CalendarLegend />
    </div>
  );
}
