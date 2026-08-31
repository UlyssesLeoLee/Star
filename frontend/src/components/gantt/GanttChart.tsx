"use client";

// =====================================================================
// GanttChart — 主容器 (per W2 任务 §1 + design §4.3)
//
// props:
//   sprints, milestones, workItems — 数据
//   dateRange                       — { start, end } ISO 字符串
//   onMilestoneUpdate, onSprintUpdate, onWorkItemMove — 拖动回调
//
// 顶部缩放: week / month / quarter (CSS grid 列宽)
// 左侧 Y 轴: sprint 行 + milestone 行 (per design §4.2)
// X 轴: 日期 header (GanttHeader 渲染)
// 网格线: 浅色虚线 (1px border-line/30) — header 列分隔
// 关键路径 (critical path): progress < 50% 的 milestones 红色 (per design §4.2)
//
// 跨模块联动 (per W2 任务 §3):
//   - milestone onClick -> router.push("/work-item?milestone={id}")
//   - 拖动 milestone 改 due_date -> onMilestoneUpdate -> stub 1s 后 /api/audit
//   - 跨 sprint 拖 work-item -> onWorkItemMove
// =====================================================================

import { useMemo, useState, useCallback, useEffect } from "react";
import { useRouter } from "next/navigation";
import { addDays, differenceInDays, format, parseISO } from "date-fns";
import type { Sprint, Milestone, WorkItem, WorkItemStatus, SprintStatus } from "@/types/ids";
import { GanttBar, type GanttBarStatus } from "./GanttBar";
import { GanttHeader } from "./GanttHeader";
import { GanttLegend } from "./GanttLegend";

export type ZoomLevel = "week" | "month" | "quarter";

export interface GanttChartProps {
  sprints: Sprint[];
  milestones: Milestone[];
  workItems: WorkItem[];
  /** 任务依赖关系 (per MS Project task link), 用于渲染 SVG 箭头 + 检测拖动冲突 */
  relations?: import("@/types/ids").Relation[];
  dateRange: { start: string; end: string };
  onMilestoneUpdate?: (id: string, newDueDate: string) => void;
  onSprintUpdate?: (id: string, newStart: string, newEnd: string) => void;
  onWorkItemMove?: (workItemId: string, newSprintId: string) => void;
}

const PX_PER_DAY: Record<ZoomLevel, number> = {
  week: 60,    // 1 day = 60px
  month: 20,   // 1 day = 20px
  quarter: 8,  // 1 day = 8px
};

const ZOOM_ORDER: ZoomLevel[] = ["week", "month", "quarter"];

function sprintStatusToBarStatus(s: SprintStatus): GanttBarStatus {
  // sprint 状态映射 (sprint 用同一套颜色)
  return s;
}

function workItemStatusToBarStatus(s: WorkItemStatus): GanttBarStatus {
  return s;
}

export function GanttChart(props: GanttChartProps) {
  const {
    sprints, milestones, workItems, relations = [], dateRange,
    onMilestoneUpdate, onSprintUpdate, onWorkItemMove,
  } = props;

  const router = useRouter();
  // Default zoom "week" (60 px/day)
  const [zoom, setZoom] = useState<ZoomLevel>("week");
  useEffect(() => {
    if (typeof window === "undefined") return;
    const saved = window.localStorage.getItem("star.gantt.zoom");
    if (saved && (saved === "week" || saved === "month" || saved === "quarter")) {
      setZoom(saved as ZoomLevel);
    }
  }, []);
  useEffect(() => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem("star.gantt.zoom", zoom);
  }, [zoom]);
  const pxPerDay = PX_PER_DAY[zoom];

  const start = parseISO(dateRange.start);
  const end = parseISO(dateRange.end);
  const totalDays = Math.max(1, differenceInDays(end, start) + 1);
  const totalWidth = totalDays * pxPerDay;

  // critical path: progress < 50% 的 milestones (self define per design §4.2)
  const criticalIds = useMemo(() => {
    return new Set(milestones.filter((m) => m.progress < 0.5).map((m) => m.id));
  }, [milestones]);

  // 任务依赖 link 渲染 (per MS Project task link 设计, 2026-08-29 17:33 JST)
  // 计算每条 work_item->work_item blocks/relates_to/duplicates 关系的源点 / 终点坐标
  // 行高: sprint row 12px (h-12), milestone row 10px (h-10), 行间距 8px
  const ROW = { sprint: 48, ms: 40, gap: 8 } as const;
  const rowOffsets = useMemo(() => {
    // 按 id 找行, sprint 用 sprint_id, milestone 用 milestone row index
    // 但 workItem link 可能跨 sprint, 行高用 rowIndex 算
    const wiById = new Map(workItems.map((w) => [w.id, w]));
    const sprintById = new Map(sprints.map((s) => [s.id, s]));
    const sprintIdx = new Map(sprints.map((s, i) => [s.id, i]));
    const msIdx = new Map(milestones.map((m, i) => [m.id, i]));

    // 对每个 workItem, 它在 gantt 的 y 坐标 = sprint 行 (因为 wi 是 sprint 的子元素, 但实际
    // 渲染在 sprint 行内顶部, 这里用 sprint row top)
    // 但 wi 有 sprint_id 也有 start_date/end_date, 跨 sprint 拖动时用 sprint 索引
    const chartStart = parseISO(dateRange.start);

    const result: Record<string, { x: number; y: number; end_x: number; rowTop: number }> = {};
    for (const sp of sprints) {
      const i = sprintIdx.get(sp.id) ?? 0;
      const top = i * (ROW.sprint + ROW.gap);
      const spStart = parseISO(sp.start_date);
      const spEnd = parseISO(addDays(parseISO(sp.end_date), 1).toISOString());
      result[sp.id] = {
        x: Math.max(0, differenceInDays(spStart, chartStart)) * pxPerDay,
        y: top + ROW.sprint / 2,
        end_x: Math.max(0, differenceInDays(spEnd, chartStart)) * pxPerDay,
        rowTop: top,
      };
    }
    for (const ms of milestones) {
      const i = msIdx.get(ms.id) ?? 0;
      const top = sprints.length * (ROW.sprint + ROW.gap) + ROW.gap + i * (ROW.ms + ROW.gap);
      const msDate = parseISO(ms.due_date);
      result[ms.id] = {
        x: Math.max(0, differenceInDays(msDate, chartStart)) * pxPerDay,
        y: top + ROW.ms / 2,
        end_x: result[ms.id]?.x ?? 0,
        rowTop: top,
      };
    }
    return result;
  }, [workItems, sprints, milestones, dateRange.start, pxPerDay]);

  // SVG link 列表: 从 relations 过滤 work_item->work_item + work_item->milestone + milestone->work_item
  // 用 FS (Finish-to-Start) 风格箭头
  const links = useMemo(() => {
    const out: Array<{
      id: string;
      fromId: string;
      toId: string;
      kind: string;
      fromX: number;
      fromY: number;
      toX: number;
      toY: number;
    }> = [];
    for (const r of relations) {
      if (r.kind === "parent_of" || r.kind === "cloned_from") continue; // 父级关系, 不画箭头
      const from = rowOffsets[r.from_id];
      const to = rowOffsets[r.to_id];
      if (!from || !to) continue;
      out.push({
        id: r.id,
        fromId: r.from_id,
        toId: r.to_id,
        kind: r.kind,
        fromX: from.end_x,  // 箭头从 from.end 出发 (FS 风格)
        fromY: from.y,
        toX: to.x,         // 箭头指向 to.start
        toY: to.y,
      });
    }
    return out;
  }, [relations, rowOffsets]);

  // 拖动冲突检测 (per MS Project FS link): workItem 新 start 不能早于所有 predecessor.end
  // (per 2026-08-29 17:33 JST 增强, 避免 dependency 冲突)
  const checkWorkItemConflict = useCallback(
    (workItemId: string, newStart: string, newEnd: string): string | null => {
      // 找 workItem 的所有 predecessor
      const preds = relations.filter(
        (r) => r.to_id === workItemId && (r.kind === "blocks" || r.kind === "relates_to"),
      );
      if (preds.length === 0) return null;
      for (const p of preds) {
        // predecessor 是 work_item, 找它的 sprint (wi.start_date = sp.start_date, wi.end_date = sp.end_date)
        const wi = workItems.find((w) => w.id === p.from_id);
        if (!wi || !wi.sprint_id) continue;
        const sp = sprints.find((s) => s.id === wi.sprint_id);
        if (!sp) continue;
        if (newStart < sp.end_date) {
          return `依赖冲突: predecessor ${wi.key || wi.id} (${sp.name}) 结束于 ${sp.end_date}, 当前任务不能早于此`;
        }
      }
      return null;
    },
    [relations, workItems, sprints],
  );

  // work items 按 sprint_id 分组 (跨 sprint 拖动)
  const workItemsBySprint = useMemo(() => {
    const map: Record<string, WorkItem[]> = {};
    for (const wi of workItems) {
      if (wi.sprint_id) {
        (map[wi.sprint_id] ||= []).push(wi);
      }
    }
    return map;
  }, [workItems]);

  const handleSprintClick = useCallback(
    (sprintId: string) => {
      router.push(`/work-item?sprint=${sprintId}`);
    },
    [router],
  );

  const handleMilestoneClick = useCallback(
    (milestoneId: string) => {
      router.push(`/work-item?milestone=${milestoneId}`);
    },
    [router],
  );

  return (
    <div className="card overflow-hidden" data-testid="gantt-chart" data-zoom={zoom}>
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-3 gap-2 flex-wrap">
        <div className="flex items-center gap-2">
          <span className="text-[10px] uppercase tracking-wider text-ink-mute">Zoom</span>
          {ZOOM_ORDER.map((z) => (
            <button
              key={z}
              type="button"
              onClick={() => setZoom(z)}
              className={`px-2 py-1 text-[10px] font-mono rounded border transition-colors ${
                zoom === z
                  ? "border-accent bg-accent/10 text-accent"
                  : "border-line text-ink-dim hover:border-ink-dim"
              }`}
              data-zoom-button={z}
              data-active={zoom === z ? "true" : "false"}
            >
              {z}
            </button>
          ))}
          <span className="ml-2 text-[10px] text-ink-mute font-mono">
            {totalDays}d · {pxPerDay}px/day · {totalWidth}px
          </span>
          {links.length > 0 && (
            <span
              data-testid="gantt-link-count"
              className="ml-2 text-[10px] font-mono text-accent border border-accent/40 bg-accent/10 px-1.5 py-0.5 rounded"
              title={`任务依赖链接数: ${links.length} 条 (per MS Project task link)`}
            >
              🔗 {links.length} link{links.length !== 1 ? "s" : ""}
            </span>
          )}
        </div>
        <GanttLegend />
      </div>

      <div className="flex border border-line rounded overflow-hidden bg-bg-soft/30">
        {/* Y-axis (fixed 200px) */}
        <div
          className="w-[200px] flex-shrink-0 border-r border-line bg-bg-soft/50"
          data-testid="gantt-yaxis"
        >
          <div className="h-10 border-b border-line bg-bg-soft/80 flex items-center px-2 text-[10px] uppercase tracking-wider text-ink-mute">
            Sprints
          </div>
          {sprints.map((sp) => (
            <div
              key={`y-s-${sp.id}`}
              data-row-label-kind="sprint"
              data-row-label-id={sp.id}
              className="h-12 px-2 py-1 border-b border-line/50 flex flex-col justify-center cursor-pointer hover:bg-bg-soft/60"
              onClick={() => handleSprintClick(sp.id)}
            >
              <div className="text-[11px] font-medium text-ink truncate">{sp.name}</div>
              <div className="text-[9px] text-ink-mute truncate">{sp.status}</div>
            </div>
          ))}
          <div className="h-2 bg-bg-soft/80" />
          <div className="h-7 border-b border-line bg-bg-soft/80 flex items-center px-2 text-[10px] uppercase tracking-wider text-ink-mute">
            Milestones
          </div>
          {milestones.map((ms) => {
            const critical = criticalIds.has(ms.id);
            return (
              <div
                key={`y-m-${ms.id}`}
                data-row-label-kind="milestone"
                data-row-label-id={ms.id}
                data-row-critical={critical ? "true" : "false"}
                className="h-10 px-2 py-1 border-b border-line/50 flex flex-col justify-center cursor-pointer hover:bg-bg-soft/60"
                onClick={() => handleMilestoneClick(ms.id)}
              >
                <div className={`text-[11px] font-medium truncate ${critical ? "text-err" : "text-ink"}`}>
                  ◆ {ms.name}
                </div>
                <div className="text-[9px] text-ink-mute truncate">
                  {Math.round(ms.progress * 100)}% · due {ms.due_date.slice(5)}
                </div>
              </div>
            );
          })}
        </div>

        {/* Timeline area (scrollable) */}
        <div className="flex-1 overflow-x-auto" data-testid="gantt-timeline">
          <div style={{ width: totalWidth, minWidth: "100%" }}>
            <GanttHeader
              start={start}
              totalDays={totalDays}
              pxPerDay={pxPerDay}
              zoom={zoom}
            />

            {/* Sprint rows */}
            {sprints.map((sp) => {
              const wiList = workItemsBySprint[sp.id] ?? [];
              return (
                <div
                  key={`tl-s-${sp.id}`}
                  data-row-kind="sprint"
                  data-row-id={sp.id}
                  className="h-12 border-b border-line/50 relative bg-bg-card/30"
                >
                  <GanttBar
                    item={{
                      id: sp.id,
                      label: sp.name,
                      status: sprintStatusToBarStatus(sp.status),
                    }}
                    startDate={sp.start_date}
                    endDate={addDays(parseISO(sp.end_date), 1).toISOString()}
                    dateRangeStart={dateRange.start}
                    pxPerDay={pxPerDay}
                    variant="sprint"
                    onCheckConflict={undefined /* sprint 不检查 predecessor, 见 workItem 渲染分支 */}
                    onClick={() => handleSprintClick(sp.id)}
                    onDragEnd={(newStart, newEnd) => {
                      // newEnd 是 +1 day (GanttBar 内部 exclusive end), 转回 inclusive
                      // 用 date-fns format 保持 local date 一致 (避免 toISOString 时区漂移)
                      const inclusiveEnd = format(addDays(parseISO(newEnd), -1), "yyyy-MM-dd");
                      onSprintUpdate?.(sp.id, newStart, inclusiveEnd);
                    }}
                  />
                  {/* work items: 在 sprint 行底部小条 (供跨 sprint 拖) */}
                  {wiList.slice(0, 4).map((wi, i) => (
                    <div
                      key={`wi-${wi.id}`}
                      data-work-item-id={wi.id}
                      data-work-item-status={wi.status}
                      draggable
                      onDragStart={(e) => {
                        e.dataTransfer.setData("text/work-item-id", wi.id);
                        e.dataTransfer.effectAllowed = "move";
                      }}
                      className="absolute h-3 text-[8px] text-white px-1 cursor-grab rounded-sm select-none"
                      style={{
                        top: 32,
                        left: 6 + i * 90,
                        width: 84,
                        backgroundColor:
                          wi.status === "done" ? "#3fb950" :
                          wi.status === "in_progress" ? "#2f81f7" :
                          wi.status === "blocked" ? "#f85149" :
                          wi.status === "review" ? "#d29922" : "#6e7681",
                        lineHeight: "12px",
                      }}
                      title={`${wi.key} — ${wi.title} (拖到其他 sprint 改 sprint_id)`}
                    >
                      {wi.key}
                    </div>
                  ))}
                </div>
              );
            })}

            {/* Spacer between sprints and milestones */}
            <div className="h-2 bg-bg-soft/60" />

            {/* Milestone rows */}
            {milestones.map((ms) => {
              const critical = criticalIds.has(ms.id);
              return (
                <div
                  key={`tl-m-${ms.id}`}
                  data-row-kind="milestone"
                  data-row-id={ms.id}
                  data-row-critical={critical ? "true" : "false"}
                  className="h-10 border-b border-line/50 relative bg-bg-card/30"
                  onDragOver={(e) => {
                    e.preventDefault();
                    e.dataTransfer.dropEffect = "move";
                  }}
                  onDrop={(e) => {
                    e.preventDefault();
                    const wiId = e.dataTransfer.getData("text/work-item-id");
                    if (wiId && onWorkItemMove) {
                      // milestone 行接 work-item drop = 把 wi 关联到 milestone (TODO: 真实 attr)
                      onWorkItemMove(wiId, ms.id);
                    }
                  }}
                >
                  <GanttBar
                    item={{
                      id: ms.id,
                      label: ms.name,
                      status: critical ? "blocked" : "done",
                    }}
                    startDate={ms.due_date}
                    endDate={addDays(parseISO(ms.due_date), 1).toISOString()}
                    dateRangeStart={dateRange.start}
                    pxPerDay={pxPerDay}
                    variant="milestone"
                    isCritical={critical}
                    onClick={() => handleMilestoneClick(ms.id)}
                    onDragEnd={(newStart) => {
                      onMilestoneUpdate?.(ms.id, newStart);
                    }}
                  />
                </div>
              );
            })}

            {/* SVG link 渲染层 (per MS Project task link, 2026-08-29 17:33 JST) */}
            <svg
              data-testid="gantt-link-layer"
              width={totalWidth}
              height={(sprints.length * (ROW.sprint + ROW.gap)) + ROW.gap + (milestones.length * (ROW.ms + ROW.gap))}
              style={{ position: "absolute", top: 0, left: 0, pointerEvents: "none" }}
            >
              <defs>
                {/* 箭头 marker (per MS Project FS link) */}
                <marker
                  id="gantt-arrow-blocks"
                  viewBox="0 0 8 8"
                  refX="6"
                  refY="4"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path d="M 0 0 L 8 4 L 0 8 z" fill="#f85149" />
                </marker>
                <marker
                  id="gantt-arrow-relates"
                  viewBox="0 0 8 8"
                  refX="6"
                  refY="4"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path d="M 0 0 L 8 4 L 0 8 z" fill="#6e7681" />
                </marker>
                <marker
                  id="gantt-arrow-duplicates"
                  viewBox="0 0 8 8"
                  refX="6"
                  refY="4"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path d="M 0 0 L 8 4 L 0 8 z" fill="#d29922" />
                </marker>
              </defs>
              {links.map((l) => {
                // 绘制 L 型折线: 从 (fromX, fromY) 水平到中点, 然后垂直到 toY, 水平到 toX
                const midX = (l.fromX + l.toX) / 2;
                const color = l.kind === "blocks" ? "#f85149" :
                              l.kind === "duplicates" ? "#d29922" : "#6e7681";
                const marker = l.kind === "blocks" ? "url(#gantt-arrow-blocks)" :
                              l.kind === "duplicates" ? "url(#gantt-arrow-duplicates)" :
                              "url(#gantt-arrow-relates)";
                return (
                  <g key={l.id} data-link-id={l.id} data-link-kind={l.kind}>
                    <path
                      d={`M ${l.fromX} ${l.fromY} L ${midX} ${l.fromY} L ${midX} ${l.toY} L ${l.toX} ${l.toY}`}
                      fill="none"
                      stroke={color}
                      strokeWidth="1.5"
                      strokeDasharray={l.kind === "relates_to" ? "4 2" : "none"}
                      markerEnd={marker}
                    />
                  </g>
                );
              })}
            </svg>
          </div>
        </div>
      </div>
    </div>
  );
}

// (helper removed — 改用 date-fns format 避免 toISOString 时区漂移)
