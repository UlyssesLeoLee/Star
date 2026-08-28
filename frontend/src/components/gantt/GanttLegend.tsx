"use client";

// =====================================================================
// GanttLegend — 颜色图例 (per W2 任务 §1)
// - work-item status: todo / in_progress / review / done / blocked
// - sprint 状态: active / completed / planned
// - 关键路径标识 (red)
// =====================================================================

export function GanttLegend() {
  const items: Array<{ label: string; color: string; border?: string }> = [
    { label: "todo", color: "#6e7681" },
    { label: "in_progress", color: "#2f81f7" },
    { label: "review", color: "#d29922" },
    { label: "done", color: "#3fb950" },
    { label: "blocked", color: "#f85149" },
    { label: "critical path", color: "#f85149", border: "1px solid #ff6b6b" },
  ];

  return (
    <div data-testid="gantt-legend" className="flex items-center gap-2 text-[9px] text-ink-mute font-mono">
      {items.map((it) => (
        <span key={it.label} className="inline-flex items-center gap-1">
          <span
            className="inline-block w-2.5 h-2.5 rounded-sm"
            style={{ backgroundColor: it.color, border: it.border }}
          />
          {it.label}
        </span>
      ))}
    </div>
  );
}
