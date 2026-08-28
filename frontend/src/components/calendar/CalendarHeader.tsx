"use client";

import { ChevronLeft, ChevronRight, CalendarDays, Columns3 } from "lucide-react";
import { clsx } from "clsx";
import type { CalendarView } from "./types";

const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

export function CalendarHeader({
  year,
  month,                  // 0-indexed (0=Jan)
  weekStart,              // for week view: Date (Sunday of the week)
  view,
  onPrev,
  onNext,
  onToday,
  onViewChange,
  userTimezone,           // 字符串 e.g. "Asia/Tokyo", "UTC"
}: {
  year: number;
  month: number;
  weekStart: Date;
  view: CalendarView;
  onPrev: () => void;
  onNext: () => void;
  onToday: () => void;
  onViewChange: (v: CalendarView) => void;
  userTimezone: string;
}) {
  const label = view === "month"
    ? `${MONTH_NAMES[month]} ${year}`
    : `Week of ${MONTH_NAMES[weekStart.getMonth()]} ${weekStart.getDate()}, ${weekStart.getFullYear()}`;

  return (
    <div
      data-testid="calendar-header"
      className="flex items-center justify-between mb-3 gap-2 flex-wrap"
    >
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onPrev}
          aria-label="Previous"
          data-testid="cal-prev"
          className="btn px-2 py-1"
        >
          <ChevronLeft size={14} />
        </button>
        <button
          type="button"
          onClick={onToday}
          data-testid="cal-today"
          className="btn"
        >
          Today
        </button>
        <button
          type="button"
          onClick={onNext}
          aria-label="Next"
          data-testid="cal-next"
          className="btn px-2 py-1"
        >
          <ChevronRight size={14} />
        </button>
        <h2
          data-testid="cal-label"
          className="ml-2 text-base font-semibold text-ink font-mono"
        >
          {label}
        </h2>
      </div>

      <div className="flex items-center gap-2">
        <span
          data-testid="cal-tz"
          className="text-[10px] text-ink-mute font-mono"
          title={`Timezone: ${userTimezone}`}
        >
          UTC · {userTimezone}
        </span>
        <div
          role="tablist"
          aria-label="Calendar view"
          className="inline-flex rounded-md border border-line bg-bg-soft overflow-hidden"
        >
          <button
            type="button"
            role="tab"
            aria-selected={view === "month"}
            data-testid="cal-view-month"
            onClick={() => onViewChange("month")}
            className={clsx(
              "px-2.5 py-1 text-xs flex items-center gap-1",
              view === "month" ? "bg-accent/15 text-accent" : "text-ink-dim hover:text-ink",
            )}
          >
            <CalendarDays size={12} /> Month
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={view === "week"}
            data-testid="cal-view-week"
            onClick={() => onViewChange("week")}
            className={clsx(
              "px-2.5 py-1 text-xs flex items-center gap-1 border-l border-line",
              view === "week" ? "bg-accent/15 text-accent" : "text-ink-dim hover:text-ink",
            )}
          >
            <Columns3 size={12} /> Week
          </button>
        </div>
      </div>
    </div>
  );
}
