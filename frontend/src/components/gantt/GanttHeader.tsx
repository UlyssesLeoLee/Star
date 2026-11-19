/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/gantt/GanttHeader.tsx"})
MERGE (props:Type {id:"frontend/src/components/gantt/GanttHeader.tsx::GanttHeaderProps",kind:"interface"})
MERGE (header:Symbol {id:"frontend/src/components/gantt/GanttHeader.tsx::GanttHeader",kind:"component"})
MERGE (self)-[:DEFINES]->(props)
MERGE (self)-[:DEFINES]->(header)
@endcypher
*/
"use client";

// =====================================================================
// GanttHeader — 日期 header (per W2 任务 §1)
// - 按缩放级别显示: week=日 / month=周 / quarter=月
// - 网格线: 浅色虚线 (1px border-line/30) 在 GanttChart 画, 这里只画 label + 列分隔
// =====================================================================

import { addDays, format } from "date-fns";
import type { ZoomLevel } from "./GanttChart";

export interface GanttHeaderProps {
  start: Date;
  totalDays: number;
  pxPerDay: number;
  zoom: ZoomLevel;
  /** 列头右键情景菜单 (per canvas-ux 任务 §E) — 由 GanttChart 分派 */
  onContextMenu?: (e: React.MouseEvent) => void;
}

export function GanttHeader({ start, totalDays, pxPerDay, zoom, onContextMenu }: GanttHeaderProps) {
  const totalWidth = totalDays * pxPerDay;
  const cells: Array<{ key: string; left: number; width: number; label: string; sub?: string }> = [];

  if (zoom === "week") {
    // 日级别: 每 1 天一格
    for (let i = 0; i < totalDays; i++) {
      const d = addDays(start, i);
      cells.push({
        key: `d-${i}`,
        left: i * pxPerDay,
        width: pxPerDay,
        label: format(d, "d"),
        sub: i % 7 === 0 ? format(d, "MMM") : undefined,
      });
    }
  } else if (zoom === "month") {
    // 周级别: 每 7 天一格
    let i = 0;
    while (i < totalDays) {
      const d = addDays(start, i);
      cells.push({
        key: `w-${i}`,
        left: i * pxPerDay,
        width: 7 * pxPerDay,
        label: format(d, "MMM d"),
      });
      i += 7;
    }
  } else {
    // quarter: 月级别: 每 30 天一格
    let i = 0;
    while (i < totalDays) {
      const d = addDays(start, i);
      const remaining = Math.min(30, totalDays - i);
      cells.push({
        key: `m-${i}`,
        left: i * pxPerDay,
        width: remaining * pxPerDay,
        label: format(d, "MMM yyyy"),
      });
      i += 30;
    }
  }

  return (
    <div
      data-testid="gantt-header"
      data-zoom={zoom}
      data-px-per-day={pxPerDay}
      className="h-10 border-b border-line relative bg-bg-soft/40 select-none"
      style={{ width: totalWidth }}
      onContextMenu={onContextMenu}
    >
      {cells.map((c) => (
        <div
          key={c.key}
          className="absolute top-0 bottom-0 border-r border-line/40 px-1 py-1 flex flex-col justify-center"
          style={{ left: c.left, width: c.width }}
        >
          <span className="text-[10px] font-mono text-ink-dim leading-none">{c.label}</span>
          {c.sub && <span className="text-[8px] text-ink-mute leading-none mt-0.5">{c.sub}</span>}
        </div>
      ))}
    </div>
  );
}
