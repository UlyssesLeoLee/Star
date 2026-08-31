"use client";

import { clsx } from "clsx";
import { useTranslation, useStatusLabel } from "@/lib/i18n";

const SWATCH: Record<string, string> = {
  ok:    "bg-ok/30 border-ok/50",
  info:  "bg-info/30 border-info/50",
  warn:  "bg-warn/30 border-warn/50",
  err:   "bg-err/30 border-err/50",
  accent:"bg-accent/30 border-accent/50",
  mute:  "bg-ink-mute/20 border-ink-mute/40",
};

export function CalendarLegend() {
  const { t } = useTranslation();
  // Sprint / Milestone 是产品术语, 走 status 字典
  const sprintActiveLabel = useStatusLabel("sprint", "active");
  const sprintPlannedLabel = useStatusLabel("sprint", "planned");
  const ITEMS: Array<{ key: string; kind: string; color: string; hint: string }> = [
    { key: "sprint-active", kind: sprintActiveLabel, color: "ok", hint: t.calendar.legendActiveSprint },
    { key: "sprint-planned", kind: sprintPlannedLabel, color: "info", hint: t.calendar.legendPlannedSprint },
    { key: "milestone", kind: t.calendar.legendMilestoneKind, color: "accent", hint: t.calendar.legendMilestone },
    { key: "p0", kind: t.calendar.legendP0, color: "err", hint: t.calendar.legendP0Hint },
    { key: "p1", kind: t.calendar.legendP1, color: "warn", hint: t.calendar.legendP1Hint },
    { key: "p2p3", kind: t.calendar.legendP2P3, color: "info", hint: t.calendar.legendP2P3Hint },
  ];

  return (
    <div
      data-testid="calendar-legend"
      className="flex items-center gap-3 flex-wrap text-[10px] text-ink-mute font-mono"
    >
      <span className="uppercase tracking-wider text-ink-dim">{t.calendar.legendHeader}</span>
      {ITEMS.map((it) => (
        <span key={it.key} className="flex items-center gap-1.5">
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
