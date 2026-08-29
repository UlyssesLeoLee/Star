"use client";

// =====================================================================
// KanbanBoard — Kanban 看板列容器 (per dynamic-interaction-design.md §3.4)
// =====================================================================
// 职责:
//   1. 渲染 4 列 (todo / in_progress / review / done) per seed.ts board 数据
//   2. 列 onDragOver + onDrop 接收卡片拖动, 触发 onTransition(id, toStatus)
//   3. drop zone 高亮 (蓝边 2px + bg-accent/10) when dropTarget === status
//   4. WIP limit 超限显示 (per seed.ts wip_limit 字段)
//   5. 管理 dragging 状态, 让被拖卡片 opacity 0.5
//
// 状态:
//   - dropTarget: 当前鼠标悬浮的列
//   - draggingId: 当前正在拖的卡片 id
//
// 已知缺口 (per 缺标比错标):
//   - 同列内排序 (intra-column reorder) 不支持 — Phase D.6+ 加
//   - 触屏拖动 (touch events) 未适配 — Phase Mobile 验证 (per §10.3 #5)
//   - ARIA live region (a11y) 未实现 (per §10.3 #6)
// =====================================================================

import { useState, useCallback, useMemo } from "react";
import { clsx } from "clsx";
import { KanbanCard } from "./KanbanCard";
import { StatusPill } from "@/components/StatusPill";
import { AlertTriangle } from "lucide-react";
import type { Board, WorkItem, WorkItemStatus, Identity } from "@/types/ids";
import { KANBAN_COLUMNS } from "@/mocks/data";

export interface KanbanBoardProps {
  board: Board;
  /** 父组件传 workItems 全集, 卡片取 workItem 通过 workItemIds 映射 */
  workItems: WorkItem[];
  /** 父组件传 onTransition, 卡片被 drop 到某列时调用 */
  onTransition: (workItemId: string, toStatus: WorkItemStatus) => void;
  /** identity 解析, 卡片显示 assignee display_name */
  identities: Identity[];
  /** 父组件传 filter, 让卡片过滤生效 (空数组 = 不显示) */
  filter?: (workItem: WorkItem) => boolean;
  /** 已拖动中卡片 id, 让该卡片 opacity 0.5 */
  draggingId?: string | null;
  /** 拖动开始回调, 父组件用来 setDraggingId */
  onDragStartCard?: (workItemId: string) => void;
  /** 拖动结束回调 */
  onDragEndCard?: () => void;
  // Board 列编辑 (per 2026-08-29 18:52 JST 拍板: 列可改 + 增加减少)
  /** 在末尾追加新列 (status) */
  onAddColumn?: (status: WorkItemStatus) => void;
  /** 删除列 (status) */
  onRemoveColumn?: (status: WorkItemStatus) => void;
  /** 重命名列 (status, newName) */
  onRenameColumn?: (status: WorkItemStatus, newName: string) => void;
}

const KANBAN_COLUMNS_LOCAL: ReadonlyArray<WorkItemStatus> = KANBAN_COLUMNS;

export function KanbanBoard({
  board,
  workItems,
  onTransition,
  identities,
  filter,
  draggingId,
  onDragStartCard,
  onDragEndCard,
  onAddColumn,
  onRemoveColumn,
  onRenameColumn,
}: KanbanBoardProps) {
  const [dropTarget, setDropTarget] = useState<WorkItemStatus | null>(null);
  // 内部拖动 id 兜底, 父组件没传时本地维护
  const [localDraggingId, setLocalDraggingId] = useState<string | null>(null);
  const effectiveDraggingId = draggingId ?? localDraggingId;
  // 列名 inline 编辑 (per 2026-08-29 18:52 JST 拍板)
  const [editingCol, setEditingCol] = useState<WorkItemStatus | null>(null);
  const [editingName, setEditingName] = useState<string>("");
  const startEdit = (status: WorkItemStatus, currentName: string) => {
    setEditingCol(status);
    setEditingName(currentName);
  };
  const commitEdit = (status: WorkItemStatus) => {
    if (editingCol !== status) return;
    const trimmed = editingName.trim();
    if (trimmed && trimmed !== (board.columns.find((c) => c.status === status)?.name ?? status)) {
      onRenameColumn?.(status, trimmed);
    }
    setEditingCol(null);
  };

  const workItemMap = useMemo(
    () => Object.fromEntries(workItems.map((w) => [w.id, w])),
    [workItems],
  );
  const identityMap = useMemo(
    () => Object.fromEntries(identities.map((u) => [u.id, u])),
    [identities],
  );

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>, status: WorkItemStatus) => {
    e.preventDefault(); // 必须, 否则 drop 不触发
    e.dataTransfer.dropEffect = "move";
    setDropTarget(status);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>, status: WorkItemStatus) => {
    // 防止 column 内部 child 触发的 dragleave 闪烁 — 仅在真正离开列时清状态
    const related = e.relatedTarget as Node | null;
    if (related && (e.currentTarget as Node).contains(related)) return;
    setDropTarget((cur) => (cur === status ? null : cur));
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent<HTMLDivElement>, toStatus: WorkItemStatus) => {
      e.preventDefault();
      setDropTarget(null);
      const issueId = e.dataTransfer.getData("text/issue-id");
      if (!issueId) return;
      onTransition(issueId, toStatus);
    },
    [onTransition],
  );

  const handleCardDragStart = useCallback(
    (e: React.DragEvent<HTMLDivElement>, workItem: WorkItem) => {
      // KanbanCard 内部已 setData, 这里通知父组件
      onDragStartCard?.(workItem.id);
      if (draggingId === undefined) {
        setLocalDraggingId(workItem.id);
      }
    },
    [draggingId, onDragStartCard],
  );

  const handleCardDragEnd = useCallback(() => {
    onDragEndCard?.();
    setLocalDraggingId(null);
    setDropTarget(null);
  }, [onDragEndCard]);

  return (
    <div
      data-testid="kanban-board"
      className="grid gap-3"
      // grid-cols-1 mobile, 2 col tablet, 列数 dynamic 桌面 (per 2026-08-29 18:52 JST)
      style={{ gridTemplateColumns: `repeat(${board.columns.length}, minmax(0, 1fr))` }}
    >
      {board.columns.map((col) => {
        const overWip =
          col.wip_limit !== undefined &&
          col.wip_limit < 99 &&
          col.work_item_ids.length > col.wip_limit;
        const isDropTarget = dropTarget === col.status;
        const cards = col.work_item_ids
          .map((id) => workItemMap[id])
          .filter((w): w is WorkItem => Boolean(w))
          .filter((w) => (filter ? filter(w) : true));

        return (
          <div
            key={col.status}
            data-testid={`kanban-column-${col.status}`}
            data-status={col.status}
            onDragOver={(e) => handleDragOver(e, col.status)}
            onDragLeave={(e) => handleDragLeave(e, col.status)}
            onDrop={(e) => handleDrop(e, col.status)}
            className={clsx(
              "card min-h-[200px] transition-colors",
              overWip && "border-warn/60",
              isDropTarget && "ring-2 ring-accent bg-accent/10",
            )}
          >
            <div className="flex items-center justify-between mb-3 gap-1">
              {editingCol === col.status && onRenameColumn ? (
                // Inline edit 模式 (per 2026-08-29 18:52 JST 拍板)
                <input
                  data-testid={`kanban-column-name-input-${col.status}`}
                  autoFocus
                  value={editingName}
                  onChange={(e) => setEditingName(e.target.value)}
                  onBlur={() => commitEdit(col.status)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") commitEdit(col.status);
                    else if (e.key === "Escape") setEditingCol(null);
                  }}
                  className="text-[11px] font-mono uppercase tracking-wider bg-bg-card border border-accent/60 rounded px-1.5 py-0.5 text-ink outline-none w-full"
                />
              ) : (
                <button
                  data-testid={`kanban-column-name-${col.status}`}
                  type="button"
                  onClick={() => startEdit(col.status, col.name ?? col.status)}
                  disabled={!onRenameColumn}
                  className="text-[11px] font-mono uppercase tracking-wider text-ink hover:text-accent transition-colors text-left truncate"
                  title="点击改列名"
                >
                  {col.name ?? col.status}
                </button>
              )}
              <div className="flex items-center gap-1 shrink-0">
                <span className="text-[10px] text-ink-mute font-mono">
                  {cards.length}
                  {col.wip_limit !== undefined && col.wip_limit < 99 && ` / ${col.wip_limit}`}
                </span>
                {onRemoveColumn && (
                  <button
                    type="button"
                    data-testid={`kanban-column-remove-${col.status}`}
                    onClick={() => onRemoveColumn(col.status)}
                    aria-label={`删除列 ${col.name ?? col.status}`}
                    className="text-ink-mute hover:text-err transition-colors text-xs leading-none px-1"
                    title="删除列"
                  >
                    ✕
                  </button>
                )}
              </div>
            </div>
            {overWip && (
              <div className="mb-2 text-[10px] text-warn flex items-center gap-1">
                <AlertTriangle size={10} /> WIP 超过限制
              </div>
            )}
            <div className="space-y-2">
              {cards.length === 0 && (
                <div className="text-[10px] text-ink-mute italic text-center py-4">
                  拖卡片到此
                </div>
              )}
              {cards.map((w) => (
                <KanbanCard
                  key={w.id}
                  workItem={w}
                  assignee={w.assignee_id ? identityMap[w.assignee_id] : undefined}
                  isDragging={effectiveDraggingId === w.id}
                  onDragStart={handleCardDragStart}
                  onDragEnd={handleCardDragEnd}
                />
              ))}
            </div>
          </div>
        );
      })}

      {/* Add column 按钮 (per 2026-08-29 18:52 JST 拍板) */}
      {onAddColumn && (
        <button
          type="button"
          data-testid="kanban-add-column"
          onClick={() => {
            // 找未在现有 columns 的 status 作为新列 (todo/in_progress/review/done/blocked/wontfix 轮询)
            const candidates: WorkItemStatus[] = [
              "todo", "in_progress", "review", "done", "blocked", "wontfix",
            ];
            const used = new Set(board.columns.map((c) => c.status));
            const next = candidates.find((s) => !used.has(s)) ?? "blocked";
            onAddColumn(next);
          }}
          className="card min-h-[200px] flex items-center justify-center text-ink-mute hover:text-accent hover:border-accent/40 transition-all duration-150 border-dashed cursor-pointer"
        >
          <span className="text-2xl leading-none">+</span>
          <span className="ml-2 text-sm">Add column</span>
        </button>
      )}
    </div>
  );
}

export { KANBAN_COLUMNS };
