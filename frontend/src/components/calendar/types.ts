// =====================================================================
// Calendar 共享类型 (per dynamic-interaction-design.md §5.2)
//
// 3 种 event 源:
//   - sprint     : from sprints (start_date → end_date 区间)
//   - milestone  : from milestones (due_date 单点)
//   - work_item  : from workItems (due_date 单点, W3 加的字段)
//
// CalendarEvent 是统一渲染单位, 供 MonthView / WeekView 共享.
// =====================================================================

export type CalendarEventKind = "sprint" | "milestone" | "work_item";

export interface CalendarEvent {
  id: string;                       // 原始 entity id (e.g. "ms-001", "wi-007", "spr-001")
  kind: CalendarEventKind;
  title: string;                    // 显示名
  start_date: string;               // ISO 8601, 必有 (单点 = due_date)
  end_date?: string;                // ISO 8601, sprint 才有
  // 详情链接 (work-item onClick → /work-item/{id})
  href?: string;
  // 视觉
  color: "info" | "ok" | "warn" | "err" | "accent" | "mute";
  // 优先级 / 状态
  priority?: string;
  status?: string;
  // 用于 legend
  badge?: string;                   // 显示在卡片右上角 (e.g. "P0", "MVP")
}

export type CalendarView = "month" | "week";

// 拖动回调: eventId → 新日期 (ISO date 字符串, 不带时间)
export type EventMoveHandler = (eventId: string, newDate: string) => void;
