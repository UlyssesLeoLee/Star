"use client";

import { useMemo } from "react";
import { useRouter } from "next/navigation";
import { clsx } from "clsx";
import type { CalendarEvent, EventMoveHandler } from "./types";
import {
  buildMonthGrid,
  groupEventsByDay,
  localDateKey,
  sprintDays,
} from "./events";

const DOW = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

const COLOR_BG: Record<string, string> = {
  ok:     "bg-ok/15 border-ok/40 text-ok",
  info:   "bg-info/15 border-info/40 text-info",
  warn:   "bg-warn/15 border-warn/40 text-warn",
  err:    "bg-err/15 border-err/40 text-err",
  accent: "bg-accent/15 border-accent/40 text-accent",
  mute:   "bg-ink-mute/10 border-ink-mute/30 text-ink-dim",
};

export function MonthView({
  year,
  month,          // 0-indexed
  events,
  onEventMove,
  onMonthChange,
  today = new Date(),
}: {
  year: number;
  month: number;
  events: CalendarEvent[];
  onEventMove: EventMoveHandler;
  onMonthChange?: (year: number, month: number) => void;
  today?: Date;
}) {
  const router = useRouter();
  const cells = useMemo(() => buildMonthGrid(year, month), [year, month]);
  // 单点 events (milestone / work_item) 直接桶聚合
  const singleByDay = useMemo(() => groupEventsByDay(events), [events]);
  // sprint 跨多天, 单独展开
  const sprintByDay = useMemo(() => {
    const m = new Map<string, CalendarEvent[]>();
    for (const e of events) {
      if (e.kind !== "sprint") continue;
      for (const k of sprintDays(e)) {
        const arr = m.get(k) ?? [];
        arr.push(e);
        m.set(k, arr);
      }
    }
    return m;
  }, [events]);

  const todayKey = localDateKey(today);

  return (
    <div
      data-testid="month-view"
      data-year={year}
      data-month={month}
      className="card p-0 overflow-hidden"
    >
      {/* 周几 header */}
      <div className="grid grid-cols-7 border-b border-line bg-bg-soft/40">
        {DOW.map((d) => (
          <div
            key={d}
            className="px-2 py-1.5 text-[10px] font-medium uppercase tracking-wider text-ink-dim text-center border-r border-line last:border-r-0"
          >
            {d}
          </div>
        ))}
      </div>

      {/* 7x6 网格 */}
      <div
        role="grid"
        aria-label={`Calendar ${year}-${month + 1}`}
        data-testid="month-grid"
        className="grid grid-cols-7 grid-rows-6"
      >
        {cells.map(({ date, inMonth }, idx) => {
          const key = localDateKey(date);
          const isToday = key === todayKey;
          const dayItems = [
            ...(singleByDay.get(key) ?? []),
            ...(sprintByDay.get(key) ?? []),
          ];
          // 去重 (sprint 可能和 singleByDay 重复)
          const unique = Array.from(new Map(dayItems.map((e) => [e.id, e])).values());
          // 优先显示 P0/P1, 截 3
          const sorted = [...unique].sort((a, b) => {
            const order: Record<string, number> = { err: 0, warn: 1, accent: 2, info: 3, ok: 4, mute: 5 };
            return (order[a.color] ?? 9) - (order[b.color] ?? 9);
          });
          const visible = sorted.slice(0, 3);
          const more = sorted.length - visible.length;
          const dayNum = date.getDate();

          return (
            <div
              key={idx}
              role="gridcell"
              data-testid="month-cell"
              data-date={key}
              data-in-month={inMonth ? "1" : "0"}
              data-count={unique.length}
              onDragOver={(e) => {
                if (e.dataTransfer.types.includes("text/plain")) {
                  e.preventDefault();
                  e.dataTransfer.dropEffect = "move";
                }
              }}
              onDrop={(e) => {
                e.preventDefault();
                const id = e.dataTransfer.getData("text/plain");
                if (id) onEventMove(id, key);
              }}
              className={clsx(
                "relative border-b border-r border-line last:border-r-0 min-h-[6.5rem] p-1.5 text-left",
                !inMonth && "bg-bg-soft/30 text-ink-mute",
                inMonth && "bg-bg-card",
                isToday && "ring-1 ring-accent ring-inset",
              )}
            >
              <div className="flex items-center justify-between mb-1">
                <span
                  className={clsx(
                    "text-xs font-mono transition-colors",
                    isToday && "text-accent font-bold px-1.5 py-0.5 rounded bg-accent/15 border border-accent/40 shadow-[0_0_8px_rgba(0,240,255,0.4)]",
                    !inMonth && "opacity-40",
                  )}
                >
                  {dayNum}
                </span>
                {unique.length > 0 && (
                  <span
                    data-testid="day-count"
                    className="text-[9px] font-mono text-ink-mute"
                    title={`${unique.length} event(s)`}
                  >
                    {unique.length}
                  </span>
                )}
              </div>

              <div className="space-y-0.5">
                {visible.map((e) => (
                  <button
                    type="button"
                    key={`${key}-${e.id}`}
                    data-testid="day-event"
                    data-event-id={e.id}
                    data-event-kind={e.kind}
                    draggable
                    onDragStart={(ev) => {
                      ev.dataTransfer.setData("text/plain", e.id);
                      ev.dataTransfer.effectAllowed = "move";
                    }}
                    onClick={() => {
                      if (e.href) router.push(e.href);
                    }}
                    className={clsx(
                      "block w-full text-left truncate rounded border px-1.5 py-0.5 text-[10px] font-mono",
                      COLOR_BG[e.color] ?? COLOR_BG.mute,
                    )}
                    title={e.title}
                  >
                    {e.title}
                  </button>
                ))}
                {more > 0 && (
                  <div className="text-[9px] text-ink-mute pl-1">+{more} more</div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
