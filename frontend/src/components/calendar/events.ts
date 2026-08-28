// =====================================================================
// 把 store 里的 3 类源 (sprint / milestone / work-item) 转成统一 CalendarEvent
// per dynamic-interaction-design.md §5.2 验证: 月/周/拖动共用同一 source
// =====================================================================

import type { Sprint, Milestone, WorkItem } from "@/types/ids";
import type { CalendarEvent } from "./types";

export function buildEvents(
  sprints: Sprint[],
  milestones: Milestone[],
  workItems: WorkItem[],
): CalendarEvent[] {
  const events: CalendarEvent[] = [];

  for (const s of sprints) {
    events.push({
      id: s.id,
      kind: "sprint",
      title: s.name,
      start_date: s.start_date,
      end_date: s.end_date,
      color: s.status === "active" ? "ok" : s.status === "planned" ? "info" : "mute",
      status: s.status,
      badge: s.status === "active" ? "ACTIVE" : s.status.toUpperCase(),
    });
  }
  for (const m of milestones) {
    events.push({
      id: m.id,
      kind: "milestone",
      title: m.name,
      start_date: m.due_date,
      color: m.progress >= 0.8 ? "ok" : m.progress >= 0.4 ? "info" : "warn",
      badge: `${Math.round(m.progress * 100)}%`,
      href: `/work-item?highlight=${m.work_item_ids[0] ?? ""}`,
    });
  }
  for (const w of workItems) {
    if (!w.due_date) continue;
    events.push({
      id: w.id,
      kind: "work_item",
      title: `${w.key} · ${w.title}`,
      start_date: w.due_date,
      color: w.priority === "p0" ? "err" : w.priority === "p1" ? "warn" : w.priority === "p2" ? "info" : "mute",
      priority: w.priority,
      status: w.status,
      badge: w.priority.toUpperCase(),
      href: `/work-item/${w.id}`,
    });
  }
  return events;
}

// 按本地日期 (yyyy-MM-dd) 桶聚合 events
// 用本地时间不是 UTC, 因为日历显示的是 user 视角
export function groupEventsByDay(
  events: CalendarEvent[],
): Map<string, CalendarEvent[]> {
  const map = new Map<string, CalendarEvent[]>();
  for (const e of events) {
    const d = new Date(e.start_date);
    const key = localDateKey(d);
    const arr = map.get(key) ?? [];
    arr.push(e);
    map.set(key, arr);
  }
  return map;
}

export function localDateKey(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

// sprint 跨多天, 返回它覆盖的所有 yyyy-MM-dd (含 start, 含 end)
export function sprintDays(e: CalendarEvent): string[] {
  if (e.kind !== "sprint" || !e.end_date) return [localDateKey(new Date(e.start_date))];
  const out: string[] = [];
  const cur = new Date(e.start_date);
  cur.setHours(0, 0, 0, 0);
  const end = new Date(e.end_date);
  end.setHours(0, 0, 0, 0);
  while (cur <= end) {
    out.push(localDateKey(cur));
    cur.setDate(cur.getDate() + 1);
  }
  return out;
}

// 月视图 7x6 网格的 42 天 (含跨月灰显)
// 返回 { date, inMonth } 数组
export function buildMonthGrid(year: number, month: number): { date: Date; inMonth: boolean }[] {
  // month 是 0-indexed (0=Jan)
  const first = new Date(year, month, 1);
  // 找到 grid 的第一天: 退到当周的 Sunday (用 0=Sunday 习惯)
  const startDow = first.getDay();
  const gridStart = new Date(year, month, 1 - startDow);
  const cells: { date: Date; inMonth: boolean }[] = [];
  for (let i = 0; i < 42; i++) {
    const d = new Date(gridStart);
    d.setDate(gridStart.getDate() + i);
    cells.push({ date: d, inMonth: d.getMonth() === month });
  }
  return cells;
}

// 周视图 7 天, 从 startDate 所在周的 Sunday 开始
export function buildWeekGrid(startDate: Date): Date[] {
  const start = new Date(startDate);
  start.setHours(0, 0, 0, 0);
  const dow = start.getDay();
  const gridStart = new Date(start);
  gridStart.setDate(start.getDate() - dow);
  const days: Date[] = [];
  for (let i = 0; i < 7; i++) {
    const d = new Date(gridStart);
    d.setDate(gridStart.getDate() + i);
    days.push(d);
  }
  return days;
}
