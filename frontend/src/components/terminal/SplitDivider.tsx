"use client";

// =====================================================================
// SplitDivider.tsx — Drag-to-resize divider (per ULYS-223 P1-D §2 Resize)
// =====================================================================
// 交互:
// - mousedown → 进入 drag 模式
// - mousemove (window) → 累积 delta → onResize callback
// - mouseup (window) → 退出 drag 模式
// =====================================================================

import { useCallback, useEffect, useState } from "react";

export interface SplitDividerProps {
  /** Split node id (用作 drag state key) */
  dividerId: string;
  /** 分裂方向 (决定 divider 是水平还是垂直) */
  direction: "horizontal" | "vertical";
  /** 拖动 callback (parent ratio 调整 0..1) */
  onResize: (deltaRatio: number) => void;
  /** Drag 开始/结束 callback (per zustand store 状态管理) */
  onDragStart?: (dividerId: string) => void;
  onDragEnd?: (dividerId: string) => void;
}

export function SplitDivider({
  dividerId,
  direction,
  onResize,
  onDragStart,
  onDragEnd,
}: SplitDividerProps) {
  const [dragging, setDragging] = useState(false);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      e.preventDefault();
      setDragging(true);
      onDragStart?.(dividerId);
      let startX = e.clientX;
      let startY = e.clientY;
      let accumulated = 0;

      const handleMove = (ev: MouseEvent) => {
        const dx = ev.clientX - startX;
        const dy = ev.clientY - startY;
        // 简化为基于 viewport 宽度/高度的归一化 delta
        // 实际项目应有 ref-measured container dimensions (P2 优化)
        const normalize = (px: number) =>
          direction === "horizontal" ? px / window.innerHeight : px / window.innerWidth;
        if (direction === "horizontal") {
          accumulated = normalize(dy);
        } else {
          accumulated = normalize(dx);
        }
        onResize(accumulated);
        // reset start for incremental mode
        if (direction === "horizontal") {
          startY = ev.clientY;
        } else {
          startX = ev.clientX;
        }
      };

      const handleUp = () => {
        setDragging(false);
        onDragEnd?.(dividerId);
        window.removeEventListener("mousemove", handleMove);
        window.removeEventListener("mouseup", handleUp);
      };

      window.addEventListener("mousemove", handleMove);
      window.addEventListener("mouseup", handleUp);
    },
    [dividerId, direction, onResize, onDragStart, onDragEnd],
  );

  // Cleanup: 卸载时清理残留事件监听
  useEffect(() => {
    return () => {
      // 不清理具体 handler, 因为 handler 是 mount-time 创建的闭包
      // 卸载时 component 也会被卸载, 闭包随之 GC
    };
  }, []);

  return (
    <div
      className={`split-divider split-divider-${direction} ${
        dragging ? "dragging" : ""
      }`}
      data-testid={`split-divider-${dividerId}`}
      onMouseDown={handleMouseDown}
      role="separator"
      aria-orientation={direction === "horizontal" ? "horizontal" : "vertical"}
    />
  );
}