"use client";

import { useMemo } from "react";
import { useRouter } from "next/navigation";
import { clsx } from "clsx";
import type { CalendarEvent, EventMoveHandler } from "./types";
import { buildWeekGrid, groupEventsByDay, localDateKey, sprintDays } from "./events";

const COLOR_BG: Record<string, string> = {
  ok:     "bg-ok/15 border-ok/40 text-ok",
  info:   "bg-info/15 border-info/40 text-info",
  warn:   "bg-warn/15 border-warn/40 text-warn",
  err:    "bg-err/15 border-err/40 text-err",
  accent: "bg-accent/15 border-accent/40 text-accent",
  mute:   "bg-ink-mute/10 border-ink-mute/30 text-ink-dim",
};

const COLOR_DOT: Record<string, string> = {
  ok: "bg-ok", info: "bg-info", warn: "bg-warn", err: "bg-err", accent: "bg-accent", mute: "bg-ink-mute",
};

const DOW_FULL = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

export function WeekView({
  startDate,
  events,
  onEventMove,
  userTimezone = "UTC",
  today = new Date(),
}: {
  startDate: Date;       // any date in the week; we snap to its Sunday
  events: CalendarEvent[];
  onEventMove: EventMoveHandler;
  userTimezone?: string;
  today?: Date;
}) {
  const router = useRouter();
  const days = useMemo(() => buildWeekGrid(startDate), [startDate]);
  const singleByDay = useMemo(() => groupEventsByDay(events), [events]);
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
      data-testid="week-view"
      data-start={localDateKey(days[0])}
      className="card p-0 overflow-hidden"
    >
      {/* 时区 banner */}
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-line bg-bg-soft/40 text-[10px] text-ink-mute font-mono">
        <span data-testid="week-tz">
          Timezone: <span className="text-ink-dim">UTC</span>
          {" · "}
          <span className="text-accent">{userTimezone}</span>
        </span>
        <span>7-day rolling · drag work-item to reschedule</span>
      </div>

      {/* 7 列 header + body */}
      <div className="grid grid-cols-7" data-testid="week-grid">
        {days.map((d) => {
          const key = localDateKey(d);
          const isToday = key === todayKey;
          const items = [
            ...(singleByDay.get(key) ?? []),
            ...(sprintByDay.get(key) ?? []),
          ];
          const unique = Array.from(new Map(items.map((e) => [e.id, e])).values());
          return (
            <div
              key={key}
              role="region"
              aria-label={`${DOW_FULL[d.getDay()]} ${d.getDate()}`}
              data-testid="week-day"
              data-date={key}
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
                "min-h-[20rem] border-r border-line last:border-r-0",
                isToday ? "bg-accent/5" : "bg-bg-card",
              )}
            >
              {/* day header */}
              <div
                className={clsx(
                  "px-2 py-1.5 border-b border-line text-center",
                  isToday ? "bg-accent/15" : "bg-bg-soft/40",
                )}
              >
                <div className="text-[10px] uppercase tracking-wider text-ink-dim">
                  {DOW_FULL[d.getDay()].slice(0, 3)}
                </div>
                <div
                  className={clsx(
                    "text-base font-mono",
                    isToday ? "text-accent font-bold" : "text-ink",
                  )}
                >
                  {d.getDate()}
                </div>
              </div>

              {/* work-item list (one per row) */}
              <div className="p-1.5 space-y-1">
                {unique.length === 0 && (
                  <div className="text-[10px] text-ink-mute/60 italic text-center py-3">
                    (empty)
                  </div>
                )}
                {unique.map((e) => (
                  <div
                    key={`${key}-${e.id}`}
                    data-testid="week-event"
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
                      "rounded border px-1.5 py-1 text-[10px] cursor-pointer hover:brightness-125 transition",
                      COLOR_BG[e.color] ?? COLOR_BG.mute,
                    )}
                    title={e.title}
                  >
                    <div className="flex items-center justify-between gap-1">
                      <span className="font-medium uppercase tracking-wider text-[9px] opacity-80">
                        {e.badge ?? e.kind}
                      </span>
                      <span className={clsx("w-1.5 h-1.5 rounded-full", COLOR_DOT[e.color] ?? "bg-ink-mute")} />
                    </div>
                    <div className="truncate font-mono text-[10px] mt-0.5">
                      {e.title}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
