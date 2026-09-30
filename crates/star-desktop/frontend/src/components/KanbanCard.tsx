// crates/star-desktop/frontend/src/components/KanbanCard.tsx
// =====================================================================
// Star Desktop — KanbanCard 可拖动卡片 (PR-248 follow-up of PR #247)
// 复用 frontend/src/components/board/KanbanCard.tsx (187 LOC)
// 适配:
//   - 0 依赖 next/navigation (用 React Router navigate 替代)
//   - 0 依赖 lucide-react (用 inline emoji 替代 Flag/User/Network)
//   - 0 依赖 @/components/StatusPill (用 inline <span className="badge">)
//   - 0 依赖 @/lib/i18n (用 const PRIORITY_LABEL 字符串)
//   - 0 依赖 @/types/ids (改用相对路径 ../types/ids)
// =====================================================================

import { useNavigate } from "react-router-dom";
import { clsx } from "clsx";
import type { WorkItem, Identity } from "../types/ids";

export interface KanbanCardProps {
  workItem: WorkItem;
  onDragStart?: (e: React.DragEvent<HTMLDivElement>, workItem: WorkItem) => void;
  onDragEnd?: (e: React.DragEvent<HTMLDivElement>) => void;
  isDragging?: boolean;
  assignee?: Identity | undefined;
  onClick?: (workItem: WorkItem) => void;
  /**
   * 架构查看器触发 (per ADR-0041-arch-agent-graph-viewer v0.1)
   * 不传 = 按钮隐藏
   */
  onArchClick?: (workItem: WorkItem) => void;
}

// Priority color class (per frontend globals.css 自定义色板)
const PRIORITY_COLOR: Record<WorkItem["priority"], string> = {
  P0: "border-red-500",
  P1: "border-yellow-500",
  P2: "border-blue-500",
  P3: "border-gray-400",
};

const PRIORITY_LABEL: Record<WorkItem["priority"], string> = {
  P0: "P0",
  P1: "P1",
  P2: "P2",
  P3: "P3",
};

export function KanbanCard({
  workItem,
  onDragStart,
  onDragEnd,
  isDragging,
  assignee,
  onClick,
  onArchClick,
}: KanbanCardProps) {
  const navigate = useNavigate();
  const pColor = PRIORITY_COLOR[workItem.priority] ?? "border-gray-400";
  const priorityLabel = PRIORITY_LABEL[workItem.priority];

  const handleDragStart = (e: React.DragEvent<HTMLDivElement>) => {
    e.dataTransfer.setData("text/issue-id", workItem.id);
    e.dataTransfer.effectAllowed = "move";
    onDragStart?.(e, workItem);
  };

  const handleClick = () => {
    if (onClick) {
      onClick(workItem);
    } else {
      // 默认行为: react-router navigate to /work-item/{id}
      navigate(`/work-item/${workItem.id}`);
    }
  };

  const handleArchClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    e.preventDefault();
    onArchClick?.(workItem);
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
        "p-3 border-2 border-gray-800 border-l-4 bg-gray-50",
        "hover:bg-white hover:-translate-x-0.5 hover:-translate-y-0.5 cursor-pointer select-none",
        "transition-all duration-100 shadow-sm",
        pColor,
        isDragging && "opacity-50 ring-2 ring-blue-400",
      )}
    >
      {/* Row 1: key + story_points */}
      <div className="flex items-center justify-between mb-1.5">
        <span className="font-mono text-xs text-blue-600 font-bold tracking-tight flex items-center gap-1">
          <span className="text-[10px] text-gray-500 font-normal">//</span>
          {workItem.key}
        </span>
        {workItem.story_points !== undefined && (
          <span className="font-mono text-[11px] font-bold px-1.5 py-0.5 rounded border border-gray-300 bg-white text-gray-600">
            {workItem.story_points} SP
          </span>
        )}
      </div>

      {/* Row 2: title */}
      <div className="text-sm font-semibold text-gray-900 line-clamp-2 mb-2 leading-snug">{workItem.title}</div>

      {/* Row 3: kind + status pills (inline per PR-248 适配: 0 依赖 StatusPill) */}
      <div className="flex flex-wrap items-center gap-1 mb-2">
        <span className="inline-block text-[10px] font-mono px-1.5 py-0.5 rounded bg-blue-100 text-blue-800">
          {workItem.kind}
        </span>
        <span className="inline-block text-[10px] font-mono px-1.5 py-0.5 rounded bg-gray-200 text-gray-800">
          {workItem.status}
        </span>
      </div>

      {/* Row 4: priority + assignee + arch (per ADR-0041) */}
      <div className="flex items-center justify-between text-[10px] text-gray-500 pt-1 border-t border-gray-200">
        <span className={clsx(
          "font-mono flex items-center gap-1 font-bold",
          workItem.priority === "P0" && "text-red-600",
          workItem.priority === "P1" && "text-yellow-600",
          workItem.priority === "P2" && "text-blue-600",
          workItem.priority === "P3" && "text-gray-500",
        )}>
          <span aria-hidden="true">🚩</span>
          {priorityLabel}
        </span>
        <div className="flex items-center gap-1.5">
          {/* Arch icon button (per ADR-0041 §2.3.1, emoji 替代 lucide-react Network) */}
          {onArchClick && (
            <button
              type="button"
              data-testid={`kanban-card-arch-${workItem.id}`}
              onClick={handleArchClick}
              aria-label={`View architecture graph for ${workItem.key}`}
              title="View architecture context (cypher graph)"
              className="text-gray-500 hover:text-blue-600 transition-colors p-0.5 rounded hover:bg-blue-50"
            >
              <span aria-hidden="true">🕸</span>
            </button>
          )}
          {assignee && (
            <span className="flex items-center gap-1 truncate max-w-[90px] font-mono text-[9px] text-gray-500" title={assignee.display_name}>
              <span aria-hidden="true" className="text-blue-500">👤</span>
              <span className="truncate">{assignee.display_name}</span>
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
