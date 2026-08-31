"use client";

// =====================================================================
// GanttLegend — 颜色图例 (per W2 任务 §1)
// - work-item status: todo / in_progress / review / done / blocked
// - sprint 状态: active / completed / planned
// - 关键路径标识 (red)
// =====================================================================

import { useStatusLabel } from "@/lib/i18n";

export function GanttLegend() {
  // data-legend-item 用 enum 原值 (测试/调试用, 跟语言无关)
  // 视觉 label 走 i18n 翻译
  const todoLabel = useStatusLabel("workItem", "todo");
  const inProgressLabel = useStatusLabel("workItem", "in_progress");
  const reviewLabel = useStatusLabel("workItem", "review");
  const doneLabel = useStatusLabel("workItem", "done");
  const blockedLabel = useStatusLabel("workItem", "blocked");
  // 关键路径 + task link 是技术术语, 留英文 (跨语言通用)
  const items: Array<{ key: string; label: string; color: string; border?: string; dashed?: boolean; type: "square" | "line" }> = [
    { key: "todo", label: todoLabel, color: "#6e7681", type: "square" },
    { key: "in_progress", label: inProgressLabel, color: "#2f81f7", type: "square" },
    { key: "review", label: reviewLabel, color: "#d29922", type: "square" },
    { key: "done", label: doneLabel, color: "#3fb950", type: "square" },
    { key: "blocked", label: blockedLabel, color: "#f85149", type: "square" },
    { key: "critical", label: "critical path", color: "#f85149", border: "1px solid #ff6b6b", type: "square" },
    // task link 颜色 (per MS Project, 2026-08-29 17:33 JST) - 关系 kind 是技术名词
    { key: "blocks", label: "blocks", color: "#f85149", type: "line" },
    { key: "duplicates", label: "duplicates", color: "#d29922", type: "line" },
    { key: "relates_to", label: "relates_to", color: "#6e7681", dashed: true, type: "line" },
  ];

  return (
    <div data-testid="gantt-legend" className="flex items-center gap-2 text-[9px] text-ink-mute font-mono">
      {items.map((it) => (
        <span key={it.key} className="inline-flex items-center gap-1" data-legend-item={it.key}>
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
