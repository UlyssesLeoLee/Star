"use client";

// =====================================================================
// GanttBar — 单条 (sprint / milestone / work-item)
//
// 拖动 = mousedown/mousemove/mouseup (custom drag, 不用 HTML5 draggable)
// 因为 HTML5 native drag 的 ghost image 限制难以做到 "拖动时实时改 style.left"
// (per W2 任务 §1 GanttBar "拖动时实时改 `style.left/style.width`")
//
// 颜色按 status (per W2 任务):
//   todo        -> 灰 #6e7681
//   in_progress -> 蓝 #2f81f7
//   done        -> 绿 #3fb950
//   blocked     -> 红 #f85149
//   review      -> 黄 #d29922
//   active      -> 蓝 (sprint active 复用)
//   completed   -> 绿 (sprint completed 复用)
//   planned     -> 灰
//   cancelled   -> 黑
// =====================================================================

import { useState, useRef, useCallback } from "react";
import { addDays, differenceInDays, format, parseISO } from "date-fns";
import type { WorkItemStatus, SprintStatus } from "@/types/ids";

export type GanttBarStatus =
  | WorkItemStatus
  | SprintStatus
  | "active" /* sprint 状态 alias, 与 in_progress 同色 */;

export interface GanttBarItem {
  id: string;
  label: string;
  status: GanttBarStatus;
}

export interface GanttBarProps {
  item: GanttBarItem;
  startDate: string;
  endDate: string;
  dateRangeStart: string;
  pxPerDay: number;
  variant?: "sprint" | "milestone" | "work-item";
  isCritical?: boolean;
  onClick?: () => void;
  /** (newStart ISO, newEnd ISO) — parent 决定是否接受 */
  onDragEnd?: (newStart: string, newEnd: string) => void;
}

const STATUS_COLOR: Record<string, string> = {
  todo: "#6e7681",
  in_progress: "#2f81f7",
  done: "#3fb950",
  blocked: "#f85149",
  review: "#d29922",
  wontfix: "#30363d",
  active: "#2f81f7",
  completed: "#3fb950",
  planned: "#6e7681",
  cancelled: "#30363d",
};

const STATUS_TEXT: Record<string, string> = {
  todo: "todo",
  in_progress: "in progress",
  done: "done",
  blocked: "blocked",
  review: "review",
  wontfix: "wontfix",
  active: "active",
  completed: "completed",
  planned: "planned",
  cancelled: "cancelled",
};

export function GanttBar(props: GanttBarProps) {
  const {
    item,
    startDate,
    endDate,
    dateRangeStart,
    pxPerDay,
    variant = "sprint",
    isCritical = false,
    onClick,
    onDragEnd,
  } = props;

  const start = parseISO(startDate);
  const end = parseISO(endDate);
  const chartStart = parseISO(dateRangeStart);
  const dayCount = Math.max(1, differenceInDays(end, start) + 1);
  const daysFromChartStart = differenceInDays(start, chartStart);
  const baseLeft = Math.max(0, daysFromChartStart) * pxPerDay;
  const baseWidth = Math.max(8, dayCount * pxPerDay);

  const [dragDelta, setDragDelta] = useState(0);
  const isDraggingRef = useRef(false);
  const startXRef = useRef(0);
  const lastDeltaRef = useRef(0);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (!onDragEnd) {
        // 不可拖, 只触发 click
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      isDraggingRef.current = true;
      startXRef.current = e.clientX;
      lastDeltaRef.current = 0;
      setDragDelta(0);

      const onMove = (ev: MouseEvent) => {
        if (!isDraggingRef.current) return;
        const delta = ev.clientX - startXRef.current;
        lastDeltaRef.current = delta;
        setDragDelta(delta);
      };
      const onUp = () => {
        if (!isDraggingRef.current) return;
        isDraggingRef.current = false;
        const delta = lastDeltaRef.current;
        const deltaDays = Math.round(delta / pxPerDay);
        if (deltaDays !== 0) {
          const newStart = format(addDays(start, deltaDays), "yyyy-MM-dd");
          const newEnd = format(addDays(end, deltaDays), "yyyy-MM-dd");
          onDragEnd?.(newStart, newEnd);
        }
        setDragDelta(0);
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
      };
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [onDragEnd, start, end, pxPerDay],
  );

  const bg = isCritical ? "#f85149" : (STATUS_COLOR[item.status] ?? "#6e7681");
  const text = STATUS_TEXT[item.status] ?? String(item.status);

  const isDragging = isDraggingRef.current || dragDelta !== 0;
  const displayLeft = baseLeft + dragDelta;

  // milestone 用菱形 (CSS clip-path) — 视觉区别
  const isMilestone = variant === "milestone";
  const height = isMilestone ? 18 : variant === "work-item" ? 10 : 24;
  const top = isMilestone ? 8 : variant === "work-item" ? 4 : 6;
  const width = isMilestone ? Math.max(18, baseWidth) : baseWidth;

  const style: React.CSSProperties = {
    position: "absolute",
    top,
    left: displayLeft,
    width,
    height,
    backgroundColor: bg,
    borderRadius: isMilestone ? 0 : 4,
    color: "white",
    fontSize: isMilestone ? 9 : variant === "work-item" ? 8 : 10,
    lineHeight: `${height}px`,
    padding: isMilestone ? "0 2px" : "0 4px",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
    userSelect: "none",
    cursor: onDragEnd ? (isDragging ? "grabbing" : "grab") : "pointer",
    transition: isDragging ? "none" : "left 0.15s, width 0.15s",
    boxShadow: isCritical ? "0 0 0 1px #ff6b6b" : undefined,
    transform: isMilestone ? "rotate(45deg)" : undefined,
    transformOrigin: "center",
  };

  return (
    <div
      data-testid="gantt-bar"
      data-bar-id={item.id}
      data-bar-status={item.status}
      data-bar-variant={variant}
      data-bar-critical={isCritical ? "true" : "false"}
      title={`${item.label} — ${text}${isCritical ? " (critical path)" : ""}`}
      style={style}
      onMouseDown={handleMouseDown}
      onClick={(e) => {
        // 拖动结束的 click 不触发
        if (lastDeltaRef.current !== 0) {
          e.preventDefault();
          return;
        }
        onClick?.();
      }}
    >
      <span style={isMilestone ? { display: "block", transform: "rotate(-45deg)" } : undefined}>
        {item.label}
      </span>
    </div>
  );
}
