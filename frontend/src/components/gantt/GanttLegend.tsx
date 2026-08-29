"use client";

// =====================================================================
// GanttLegend — 颜色图例 (per W2 任务 §1)
// - work-item status: todo / in_progress / review / done / blocked
// - sprint 状态: active / completed / planned
// - 关键路径标识 (red)
// =====================================================================

export function GanttLegend() {
  const items: Array<{ label: string; color: string; border?: string; dashed?: boolean; type: "square" | "line" }> = [
    { label: "todo", color: "#6e7681", type: "square" },
    { label: "in_progress", color: "#2f81f7", type: "square" },
    { label: "review", color: "#d29922", type: "square" },
    { label: "done", color: "#3fb950", type: "square" },
    { label: "blocked", color: "#f85149", type: "square" },
    { label: "critical path", color: "#f85149", border: "1px solid #ff6b6b", type: "square" },
    // task link 颜色 (per MS Project, 2026-08-29 17:33 JST)
    { label: "blocks", color: "#f85149", type: "line" },
    { label: "duplicates", color: "#d29922", type: "line" },
    { label: "relates_to", color: "#6e7681", dashed: true, type: "line" },
  ];

  return (
    <div data-testid="gantt-legend" className="flex items-center gap-2 text-[9px] text-ink-mute font-mono">
      {items.map((it) => (
        <span key={it.label} className="inline-flex items-center gap-1" data-legend-item={it.label}>
          {it.type === "line" ? (
            // task link 颜色: 用 line (高 1px, 宽 10px), dashed 用 stroke-dasharray
            <svg width="14" height="6" className="inline-block">
              <line
                x1="0"
                y1="3"
                x2="14"
                y2="3"
                stroke={it.color}
                strokeWidth="1.5"
                strokeDasharray={it.dashed ? "3 2" : undefined}
              />
            </svg>
          ) : (
            <span
              className="inline-block w-2.5 h-2.5 rounded-sm"
              style={{ backgroundColor: it.color, border: it.border }}
            />
          )}
          {it.label}
        </span>
      ))}
    </div>
  );
}
