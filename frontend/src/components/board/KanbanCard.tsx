"use client";

// =====================================================================
// KanbanCard — 可拖动卡片 (per dynamic-interaction-design.md §3.4)
// =====================================================================
// 职责:
//   1. 渲染 work-item 关键信息 (key/title/kind/priority/SP/assignee)
//   2. HTML5 native drag, 拖动时降低 opacity
//   3. onClick → 跳 /work-item/{id} (per §7 跨模块联动)
//   4. onDragStart 通过 dataTransfer 传递 issue id
//
// 设计:
//   - 用 HTML5 native (per §2.4), 不引 dnd-kit / react-dnd
//   - StatusPill 复用现有 components/StatusPill.tsx
//   - 拖动中 opacity-50 + 蓝边 (drag-over 高亮由父 KanbanBoard 控制)
// =====================================================================

import { useRouter } from "next/navigation";
import { clsx } from "clsx";
import { StatusPill } from "@/components/StatusPill";
import { Flag, User } from "lucide-react";
import type { WorkItem, Identity } from "@/types/ids";

export interface KanbanCardProps {
  workItem: WorkItem;
  /** 父组件传 onDragStart 钩子, 卡片触发时设置 dataTransfer */
  onDragStart?: (e: React.DragEvent<HTMLDivElement>, workItem: WorkItem) => void;
  /** 父组件传 onDragEnd, 用来清 isDragging 状态 */
  onDragEnd?: (e: React.DragEvent<HTMLDivElement>) => void;
  /** 当前是否处于拖动态 (用于 opacity 0.5) */
  isDragging?: boolean;
  /** 解析后的 assignee Identity (供卡片显示 display_name) */
  assignee?: Identity | undefined;
  /** 点击跳详情, 默认跳 /work-item/{id} */
  onClick?: (workItem: WorkItem) => void;
}

const PRIORITY_COLOR: Record<WorkItem["priority"], string> = {
  p0: "border-l-err",
  p1: "border-l-warn",
  p2: "border-l-info",
  p3: "border-l-ink-mute",
};

export function KanbanCard({
  workItem,
  onDragStart,
  onDragEnd,
  isDragging,
  assignee,
  onClick,
}: KanbanCardProps) {
  const router = useRouter();
  const pColor = PRIORITY_COLOR[workItem.priority] ?? "border-l-ink-mute";

  const handleDragStart = (e: React.DragEvent<HTMLDivElement>) => {
    // HTML5 native drag API: dataTransfer 传 issue id
    e.dataTransfer.setData("text/issue-id", workItem.id);
    e.dataTransfer.effectAllowed = "move";
    onDragStart?.(e, workItem);
  };

  const handleClick = () => {
    if (onClick) {
      onClick(workItem);
    } else {
      // 默认行为: 跳 work-item 详情页 (per §7 跨模块联动)
      router.push(`/work-item/${workItem.id}`);
    }
  };

  return (
    <div
      role="article"
      data-testid={`kanban-card-${workItem.id}`}
      data-issue-id={workItem.id}
      draggable
      onDragStart={handleDragStart}
      onDragEnd={onDragEnd}
      onClick={handleClick}
      className={clsx(
        "p-2 rounded border border-line border-l-2 bg-bg-soft/60",
        "hover:bg-bg-soft cursor-pointer select-none",
        "transition-colors transition-opacity",
        pColor,
        isDragging && "opacity-50 ring-2 ring-accent",
      )}
    >
      {/* Row 1: key + story_points */}
      <div className="flex items-center justify-between mb-1">
        <span className="font-mono text-[10px] text-info">{workItem.key}</span>
        <span className="font-mono text-[10px] text-ink-mute">
          {workItem.story_points ?? "—"}sp
        </span>
      </div>

      {/* Row 2: title */}
      <div className="text-xs line-clamp-2 mb-1.5">{workItem.title}</div>

      {/* Row 3: kind + status pills */}
      <div className="flex flex-wrap items-center gap-1 mb-1">
        <StatusPill value={workItem.kind} size="xs" />
        <StatusPill value={workItem.status} size="xs" />
      </div>

      {/* Row 4: priority + assignee */}
      <div className="flex items-center justify-between text-[10px] text-ink-mute">
        <span className={clsx(
          "font-mono flex items-center gap-0.5",
          workItem.priority === "p0" && "text-err",
          workItem.priority === "p1" && "text-warn",
          workItem.priority === "p2" && "text-info",
          workItem.priority === "p3" && "text-ink-dim",
        )}>
          <Flag size={9} />
          {workItem.priority.toUpperCase()}
        </span>
        {assignee && (
          <span className="flex items-center gap-0.5 truncate max-w-[80px]" title={assignee.display_name}>
            <User size={9} />
            <span className="truncate">{assignee.display_name}</span>
          </span>
        )}
      </div>

      {/* Row 5: labels (前 2 个) */}
      {workItem.labels.length > 0 && (
        <div className="mt-1 flex flex-wrap gap-1">
          {workItem.labels.slice(0, 2).map((l) => (
            <span key={l} className="text-[9px] text-ink-mute">#{l}</span>
          ))}
        </div>
      )}
    </div>
  );
}
