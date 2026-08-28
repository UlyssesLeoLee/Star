"use client";

import { clsx } from "clsx";

const SWATCH: Record<string, string> = {
  ok:    "bg-ok/30 border-ok/50",
  info:  "bg-info/30 border-info/50",
  warn:  "bg-warn/30 border-warn/50",
  err:   "bg-err/30 border-err/50",
  accent:"bg-accent/30 border-accent/50",
  mute:  "bg-ink-mute/20 border-ink-mute/40",
};

const ITEMS = [
  { kind: "Sprint",    color: "ok",     hint: "active sprint" },
  { kind: "Sprint",    color: "info",   hint: "planned sprint" },
  { kind: "Milestone", color: "accent", hint: "milestone due" },
  { kind: "P0 item",   color: "err",    hint: "high priority" },
  { kind: "P1 item",   color: "warn",   hint: "medium" },
  { kind: "P2/P3",     color: "info",   hint: "low" },
] as const;

export function CalendarLegend() {
  return (
    <div
      data-testid="calendar-legend"
      className="flex items-center gap-3 flex-wrap text-[10px] text-ink-mute font-mono"
    >
      <span className="uppercase tracking-wider text-ink-dim">Legend:</span>
      {ITEMS.map((it, i) => (
        <span key={i} className="flex items-center gap-1.5">
          <span
            aria-hidden
            className={clsx("inline-block w-3 h-2 rounded-sm border", SWATCH[it.color])}
          />
          <span>{it.kind}</span>
          <span className="text-ink-mute/70">· {it.hint}</span>
        </span>
      ))}
    </div>
  );
}
