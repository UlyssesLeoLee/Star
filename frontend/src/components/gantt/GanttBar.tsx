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
import toast from "react-hot-toast";
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
  /**
   * 拖动冲突检测 (per MS Project task link constraint, 2026-08-29 18:48 JST)
   * 返回 string 错误信息 = 冲突, 阻止 onDragEnd 触发, bar 红色 flash 1.5s
   * 返回 null = 无冲突, 正常 onDragEnd
   */
  onCheckConflict?: (newStart: string, newEnd: string) => string | null;
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
    onCheckConflict,
  } = props;

  const start = parseISO(startDate);
  const end = parseISO(endDate);
  const chartStart = parseISO(dateRangeStart);
  const dayCount = Math.max(1, differenceInDays(end, start) + 1);
  const daysFromChartStart = differenceInDays(start, chartStart);
  const baseLeft = Math.max(0, daysFromChartStart) * pxPerDay;
  const baseWidth = Math.max(8, dayCount * pxPerDay);

  const [dragDelta, setDragDelta] = useState(0);
  const [resizeWidth, setResizeWidth] = useState(0);
  // 冲突状态: 拖动冲突时显示红色 flash 1.5s, 阻止 onDragEnd 写入 (per 2026-08-29 19:14 JST 接入)
  const [conflictMsg, setConflictMsg] = useState<string | null>(null);
  // dragMode 决定本次拖动语义: "move" 整体平移 / "resize-left" 左把手改 start / "resize-right" 右把手改 end
  const dragModeRef = useRef<"move" | "resize-left" | "resize-right">("move");
  const isDraggingRef = useRef(false);
  const startXRef = useRef(0);
  const lastDeltaRef = useRef(0);

  // milestone 用菱形 (CSS clip-path) — 视觉区别
  // 提前到 useCallback handleMouseDown 之前声明, 避免 TS2448 used-before-declaration
  // (per 2026-08-29 19:27 JST 修)
  const isMilestone = variant === "milestone";

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (!onDragEnd) {
        // 不可拖, 只触发 click
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      // 从 target 的 data-mode 读 drag 模式 (per 2026-08-29 17:33 JST MS Project 风格 resize)
      const target = e.currentTarget as HTMLElement;
      const mode = (target.dataset.mode as "move" | "resize-left" | "resize-right") ?? "move";
      dragModeRef.current = mode;
      isDraggingRef.current = true;
      startXRef.current = e.clientX;
      lastDeltaRef.current = 0;
      setDragDelta(0);
      setResizeWidth(0);

      const onMove = (ev: MouseEvent) => {
        if (!isDraggingRef.current) return;
        const delta = ev.clientX - startXRef.current;
        lastDeltaRef.current = delta;
        if (mode === "move") {
          setDragDelta(delta);
        } else {
          // resize 只动 width (左把手 width=baseWidth+delta, 右把手 width=baseWidth+delta, 起点不动或不动)
          setResizeWidth(delta);
        }
      };
      const onUp = () => {
        if (!isDraggingRef.current) return;
        isDraggingRef.current = false;
        const delta = lastDeltaRef.current;
        const deltaDays = Math.round(delta / pxPerDay);
        if (deltaDays !== 0) {
          let newStart = start;
          let newEnd = end;
          if (mode === "move") {
            newStart = addDays(start, deltaDays);
            newEnd = addDays(end, deltaDays);
          } else if (mode === "resize-left") {
            // milestone 用 due_date 一天, 不支持拉长 (start == end)
            if (!isMilestone) {
              newStart = addDays(start, deltaDays);
              // 最小 1 天 (newStart 不能超过 newEnd)
              if (newStart >= end) {
                newStart = addDays(end, -1);
              }
            }
          } else if (mode === "resize-right") {
            if (!isMilestone) {
              newEnd = addDays(end, deltaDays);
              if (newEnd <= start) {
                newEnd = addDays(start, 1);
              }
            }
          }
          // 冲突检查 (per 2026-08-29 19:14 JST): 拖动前先看 predecessor.end 是否冲突
          if (onCheckConflict) {
            const msg = onCheckConflict(
              format(newStart, "yyyy-MM-dd"),
              format(addDays(newEnd, 1), "yyyy-MM-dd"),
            );
            if (msg) {
              // 冲突: bar 红色 flash 1.5s (即时视觉反馈给拖拽者) +
              //       toast.error (顶部 right, 全局可见, 文案详细)
              // per 2026-08-29 19:24 JST react-hot-toast 接入
              setConflictMsg(msg);
              setTimeout(() => setConflictMsg(null), 1500);
              toast.error(`⚠ 调度冲突 — ${msg}`, {
                duration: 4500,
                id: `gantt-conflict-${item.id}-${Date.now()}`,
              });
            } else {
              onDragEnd?.(
                format(newStart, "yyyy-MM-dd"),
                // GanttBar endDate 是 exclusive, 加 1 天转回 inclusive
                format(addDays(newEnd, 1), "yyyy-MM-dd"),
              );
            }
          } else {
            // 无 conflict check, 兼容旧行为
            onDragEnd?.(
              format(newStart, "yyyy-MM-dd"),
              format(addDays(newEnd, 1), "yyyy-MM-dd"),
            );
          }
        }
        setDragDelta(0);
        setResizeWidth(0);
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
      };
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [onDragEnd, start, end, pxPerDay, isMilestone],
  );

  // milestone 用菱形 (CSS clip-path) — 视觉区别
  // (isMilestone 已在 useCallback 之前声明, 见上行)

  const bg = isCritical ? "#f85149" : (STATUS_COLOR[item.status] ?? "#6e7681");
  const text = STATUS_TEXT[item.status] ?? String(item.status);

  const isDragging = isDraggingRef.current || dragDelta !== 0 || resizeWidth !== 0;
  const displayLeft = baseLeft + dragDelta;
  const displayWidth = isMilestone
    ? Math.max(18, baseWidth)
    : Math.max(8, baseWidth + resizeWidth);

  const height = isMilestone ? 18 : variant === "work-item" ? 10 : 24;
  const top = isMilestone ? 8 : variant === "work-item" ? 4 : 6;
  const width = displayWidth;

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
    // 冲突 flash (per 2026-08-29 19:14 JST): 红色 box-shadow + 0.7 opacity 1.5s
    boxShadow: conflictMsg
      ? "0 0 16px rgba(255, 51, 102, 1), 0 0 0 2px #ff3366"
      : isCritical
        ? "0 0 10px rgba(255, 51, 102, 0.8), 0 0 0 1px #ff3366"
        : "0 1px 4px rgba(0, 0, 0, 0.25)",
    opacity: conflictMsg ? 0.7 : 1,
    transform: isMilestone ? "rotate(45deg)" : undefined,
    transformOrigin: "center",
  };

  // resize handle 宽度: 6px (per MS Project 风格, 鼠标 hover 时变蓝色提示可拖)
  const HANDLE_W = 6;
  // milestone 不渲染 handle (它只有 1 天, 不能 resize)
  const showHandles = !isMilestone && onDragEnd;

  return (
    <div
      data-testid="gantt-bar"
      data-bar-id={item.id}
      data-bar-status={item.status}
      data-bar-variant={variant}
      data-bar-critical={isCritical ? "true" : "false"}
      data-bar-conflict={conflictMsg ? "true" : "false"}
      title={
        conflictMsg
          ? `⚠ ${conflictMsg}`
          : `${item.label} — ${text}${isCritical ? " (critical path)" : ""} (拖动移动 / 两端把手拉长缩短)`
      }
      style={style}
      onMouseDown={(e) => {
        // 把 target.dataset.mode 注入到 e.currentTarget, 让 handleMouseDown 读到
        // 实际 handle 元素的 dataset.mode = resize-left/right, move 区域 = move
        handleMouseDown(e);
      }}
      onClick={(e) => {
        // 拖动结束的 click 不触发
        if (lastDeltaRef.current !== 0) {
          e.preventDefault();
          return;
        }
        onClick?.();
      }}
    >
      {/* 左 resize handle: 改 start_date (move 区域外, e.stopPropagation 避免冒泡到 move) */}
      {showHandles && (
        <div
          data-mode="resize-left"
          data-resize-handle="left"
          aria-label="拉长缩短起点"
          onMouseDown={(e) => {
            e.stopPropagation();
            handleMouseDown(e);
          }}
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            width: HANDLE_W,
            height: "100%",
            cursor: "ew-resize",
            background: "transparent",
            // hover 时显示蓝色把手 (per MS Project 风格)
          }}
          className="gantt-bar-handle gantt-bar-handle-left"
        />
      )}
      {/* 中间 move 区域: 整体平移 (data-mode="move", 但 div 本身已带 onMouseDown) */}
      {/* 右 resize handle: 改 end_date */}
      {showHandles && (
        <div
          data-mode="resize-right"
          data-resize-handle="right"
          aria-label="拉长缩短终点"
          onMouseDown={(e) => {
            e.stopPropagation();
            handleMouseDown(e);
          }}
          style={{
            position: "absolute",
            top: 0,
            right: 0,
            width: HANDLE_W,
            height: "100%",
            cursor: "ew-resize",
            background: "transparent",
          }}
          className="gantt-bar-handle gantt-bar-handle-right"
        />
      )}
      <span style={isMilestone ? { display: "block", transform: "rotate(-45deg)" } : undefined}>
        {item.label}
      </span>
    </div>
  );
}
