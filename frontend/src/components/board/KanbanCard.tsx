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
import { useTranslation } from "@/lib/i18n";

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
  const { t } = useTranslation();
  const pColor = PRIORITY_COLOR[workItem.priority] ?? "border-l-ink-mute";
  // 优先级显示文本 (P0/P1/P2/P3 — 简写, 全语言通用)
  const PRIORITY_LABEL: Record<WorkItem["priority"], string> = {
    p0: t.workItem.priorityP0,
    p1: t.workItem.priorityP1,
    p2: t.workItem.priorityP2,
    p3: t.workItem.priorityP3,
  };
  const priorityLabel = PRIORITY_LABEL[workItem.priority];

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
        "p-3 border-2 border-black border-l-4 bg-[var(--cel-surface-sub,#151c2c)]",
        "hover:bg-[var(--cel-surface-stage,#1a2438)] hover:-translate-x-0.5 hover:-translate-y-0.5 cursor-pointer select-none",
        "transition-all duration-100 cel-shadow",
        pColor,
        isDragging && "opacity-50 ring-2 ring-[var(--cel-cyan,#00f0ff)] shadow-[0_0_15px_rgba(0,240,255,0.4)]",
      )}
    >
      {/* Row 1: key + story_points */}
      <div className="flex items-center justify-between mb-1">
        <span className="font-mono text-[10px] text-info font-medium tracking-tight flex items-center gap-1">
          <span className="text-[9px] text-ink-mute">//</span>
          {workItem.key}
        </span>
        {workItem.story_points !== undefined && (
          <span className="font-mono text-[9px] px-1 py-0.2 rounded border border-line/60 bg-bg-card text-ink-mute">
            {workItem.story_points} {t.workItem.storyPointsUnit}
          </span>
        )}
      </div>

      {/* Row 2: title */}
      <div className="text-xs font-medium text-ink line-clamp-2 mb-2 leading-snug">{workItem.title}</div>

      {/* Row 3: kind + status pills */}
      <div className="flex flex-wrap items-center gap-1 mb-2">
        <StatusPill value={workItem.kind} size="xs" translateAs="workItemKind" />
        <StatusPill value={workItem.status} size="xs" translateAs="workItem" />
      </div>

      {/* Row 4: priority + assignee */}
      <div className="flex items-center justify-between text-[10px] text-ink-mute pt-1 border-t border-line/40">
        <span className={clsx(
          "font-mono flex items-center gap-1 font-medium",
          workItem.priority === "p0" && "text-err drop-shadow-[0_0_6px_rgba(255,51,102,0.4)]",
          workItem.priority === "p1" && "text-warn",
          workItem.priority === "p2" && "text-info",
          workItem.priority === "p3" && "text-ink-dim",
        )}>
          <Flag size={9} />
          {priorityLabel}
        </span>
        {assignee && (
          <span className="flex items-center gap-1 truncate max-w-[90px] font-mono text-[9px] text-ink-dim" title={assignee.display_name}>
            <User size={9} className="text-accent" />
            <span className="truncate">{assignee.display_name}</span>
          </span>
        )}
      </div>

      {/* Row 5: labels (前 2 个) */}
      {workItem.labels.length > 0 && (
        <div className="mt-1.5 flex flex-wrap gap-1">
          {workItem.labels.slice(0, 2).map((l) => (
            <span key={l} className="text-[8px] font-mono text-ink-mute px-1 rounded bg-bg-card/90 border border-line/40">
              #{l}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
