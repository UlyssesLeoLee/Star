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
//
// 兜底列保护 (per 2026-08-31 11:24 JST Ulysses 拍板):
//   - 兜底列 (todo) 不可删, ✕ 按钮置灰 + tooltip
//   - store removeBoardColumn 二次拒绝兜底 status
//   - 删其他列时, 列里 wi 状态统一归 todo, 保证数据零丢失
// =====================================================================

import { useState, useCallback, useMemo } from "react";
import { clsx } from "clsx";
import { KanbanCard } from "./KanbanCard";
import { StatusPill } from "@/components/StatusPill";
import { AlertTriangle, Plus, SlidersHorizontal } from "lucide-react";
import type { Board, WorkItem, WorkItemStatus, Identity } from "@/types/ids";
import { KANBAN_COLUMNS } from "@/mocks/data";
import { isFallbackStatus } from "./constants";

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
  /** 拖动列重排 (fromIdx, toIdx), per 2026-08-29 19:09 JST 补 reorderBoardColumns UI */
  onReorderColumns?: (fromIdx: number, toIdx: number) => void;
  // 列内新增任务卡 (per 2026-08-31 11:56 JST Ulysses 拍板: Jira 风格 + Add task)
  /** 列内点击 + Add task, 提交 title 后调 onAddWorkItem(status, title) */
  onAddWorkItem?: (status: WorkItemStatus, title: string) => void;
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
  onReorderColumns,
  onAddWorkItem,
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
  // 列内 + Add task (per 2026-08-31 11:56 JST Ulysses 拍板)
  // 每列独立 editingCard 状态, 点击 + Add task 展开 input, Enter 提交, Esc 取消
  const [editingCard, setEditingCard] = useState<WorkItemStatus | null>(null);
  const [editingCardTitle, setEditingCardTitle] = useState<string>("");
  const cardInputRef = useState<{ current: HTMLInputElement | null }>({ current: null })[0];
  const startAddCard = (status: WorkItemStatus) => {
    setEditingCard(status);
    setEditingCardTitle("");
    // auto-focus 在 next tick (input 已 mount)
    requestAnimationFrame(() => cardInputRef.current?.focus());
  };
  const commitAddCard = (status: WorkItemStatus) => {
    if (editingCard !== status) return;
    const trimmed = editingCardTitle.trim();
    setEditingCard(null);
    setEditingCardTitle("");
    if (trimmed) onAddWorkItem?.(status, trimmed);
  };
  const cancelAddCard = () => {
    setEditingCard(null);
    setEditingCardTitle("");
  };
  // 列拖动重排 (per 2026-08-29 19:09 JST)
  // 用 HTML5 native drag: 拖到目标列 drop 时 reorder
  const [draggingColIdx, setDraggingColIdx] = useState<number | null>(null);
  const [dropTargetColIdx, setDropTargetColIdx] = useState<number | null>(null);
  const handleColDragStart = useCallback((e: React.DragEvent<HTMLDivElement>, idx: number) => {
    if (!onReorderColumns) return;
    e.dataTransfer.setData("text/col-idx", String(idx));
    e.dataTransfer.effectAllowed = "move";
    setDraggingColIdx(idx);
  }, [onReorderColumns]);
  const handleColDragOver = useCallback((e: React.DragEvent<HTMLDivElement>, idx: number) => {
    if (!onReorderColumns) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    setDropTargetColIdx(idx);
  }, [onReorderColumns]);
  const handleColDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>, idx: number) => {
    const related = e.relatedTarget as Node | null;
    if (related && (e.currentTarget as Node).contains(related)) return;
    setDropTargetColIdx((cur) => (cur === idx ? null : cur));
  }, []);
  const handleColDrop = useCallback((e: React.DragEvent<HTMLDivElement>, toIdx: number) => {
    e.preventDefault();
    setDropTargetColIdx(null);
    const fromIdxStr = e.dataTransfer.getData("text/col-idx");
    if (!fromIdxStr) return;
    const fromIdx = Number(fromIdxStr);
    if (Number.isNaN(fromIdx) || fromIdx === toIdx) {
      setDraggingColIdx(null);
      return;
    }
    onReorderColumns?.(fromIdx, toIdx);
    setDraggingColIdx(null);
  }, [onReorderColumns]);
  const handleColDragEnd = useCallback(() => {
    setDraggingColIdx(null);
    setDropTargetColIdx(null);
  }, []);

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

  const handleAddColumn = () => {
    // 找未在现有 columns 的 status 作为新列 (todo/in_progress/review/done/blocked/wontfix 轮询)
    const candidates: WorkItemStatus[] = [
      "todo", "in_progress", "review", "done", "blocked", "wontfix",
    ];
    const used = new Set(board.columns.map((c) => c.status));
    const next = candidates.find((s) => !used.has(s)) ?? "blocked";
    onAddColumn?.(next);
  };

  return (
    <div className="space-y-3">
      {/* === Kanban Board Toolbar Control Bar === */}
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-2 text-xs font-mono text-ink-mute">
          <span className="font-bold text-ink-dim uppercase tracking-wider">
            COLUMNS // {board.columns.length} ACTIVE
          </span>
          <span className="text-[10px] opacity-60 hidden sm:inline">
            · 拖动 ⋮⋮ 重排列，点击列名重命名
          </span>
        </div>

        {onAddColumn && (
          <button
            type="button"
            data-testid="kanban-add-column"
            onClick={handleAddColumn}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-line hover:border-accent bg-bg-soft/70 hover:bg-accent/10 text-xs font-mono text-ink-dim hover:text-accent transition-all duration-150 shadow-sm group font-semibold"
            title="添加新看板列"
          >
            <Plus size={13} className="text-accent group-hover:rotate-90 transition-transform duration-200" />
            <span>+ Add Column</span>
          </button>
        )}
      </div>

      {/* === Kanban Columns Grid === */}
      <div
        data-testid="kanban-board"
        className="grid gap-3"
        // grid-cols-1 mobile, 2 col tablet, 列数 dynamic 桌面 (per 2026-08-29 18:52 JST)
        // minmax(260px, 1fr) per 2026-08-29 19:35 JST scope-ui-only 候选第 3 项 (Board 列宽):
        // 4 列 (260×4=1040) 1280 屏 fit, 5+ 列 → 父 main overflow-x-auto 横向滚动
        style={{ gridTemplateColumns: `repeat(${board.columns.length}, minmax(260px, 1fr))` }}
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

          const colIdx = board.columns.findIndex((c) => c.status === col.status);
          const isColDragging = draggingColIdx === colIdx;
          const isColDropTarget = dropTargetColIdx === colIdx && draggingColIdx !== null && draggingColIdx !== colIdx;
          return (
            <div
              key={col.status}
              data-testid={`kanban-column-${col.status}`}
              data-status={col.status}
              data-col-idx={colIdx}
              // 拖动高亮: card drop zone (红/绿) + col drop zone (蓝) 区分
              onDragOver={(e) => {
                const types = e.dataTransfer?.types ? Array.from(e.dataTransfer.types) : [];
                // card drop (text/issue-id) -> 红/绿环
                if (types.includes("text/issue-id") || types.length === 0) {
                  handleDragOver(e, col.status);
                }
                // col drop (text/col-idx) -> 蓝边
                if (types.includes("text/col-idx")) {
                  e.preventDefault();
                  if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
                  if (draggingColIdx !== colIdx) setDropTargetColIdx(colIdx);
                }
              }}
              onDragLeave={(e) => {
                handleDragLeave(e, col.status);
                handleColDragLeave(e, colIdx);
              }}
              onDrop={(e) => {
                const types = e.dataTransfer?.types ? Array.from(e.dataTransfer.types) : [];
                // 路由: col 拖到 col vs card 拖到 col
                if (types.includes("text/col-idx")) {
                  handleColDrop(e, colIdx);
                } else {
                  handleDrop(e, col.status);
                }
              }}
              className={clsx(
                "card min-h-[200px] transition-colors",
                overWip && "border-warn/60",
                isDropTarget && "ring-2 ring-accent bg-accent/10",
                // 列重排 drop 高亮 (per 2026-08-29 19:09 JST)
                isColDragging && "opacity-50",
                isColDropTarget && "ring-2 ring-cyan-400 bg-cyan-500/10",
              )}
            >
              {/* 列拖动手柄 (per 2026-08-29 19:09 JST, 把整列设为 draggable) */}
              {onReorderColumns && (
                <div
                  draggable
                  onDragStart={(e) => handleColDragStart(e, board.columns.findIndex((c) => c.status === col.status))}
                  onDragEnd={handleColDragEnd}
                  data-testid={`kanban-column-drag-handle-${col.status}`}
                  className="text-ink-mute hover:text-accent cursor-grab active:cursor-grabbing text-xs select-none"
                  title="拖动重排列"
                  aria-label={`重排列 ${col.name ?? col.status}`}
                >
                  ⋮⋮
                </div>
              )}
              <div className="flex items-center justify-between mb-3 gap-1 flex-1 min-w-0">
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
                  {onRemoveColumn && (() => {
                    // per 2026-08-31 11:24 JST 拍板: 兜底列 (todo) 不可删,
                    // 按钮置灰 + tooltip 解释, store 也会二次拒绝
                    const isFallback = isFallbackStatus(col.status);
                    return (
                      <button
                        type="button"
                        data-testid={`kanban-column-remove-${col.status}`}
                        onClick={() => {
                          if (isFallback) return; // 兜底列 click 直接吞掉, 兜底保护
                          onRemoveColumn(col.status);
                        }}
                        disabled={isFallback}
                        aria-label={isFallback ? `兜底列 ${col.name ?? col.status} 不可删除` : `删除列 ${col.name ?? col.status}`}
                        className={
                          isFallback
                            ? "text-ink-mute/40 cursor-not-allowed text-xs leading-none px-1"
                            : "text-ink-mute hover:text-err transition-colors text-xs leading-none px-1"
                        }
                        title={isFallback ? "兜底列不可删除 — 删除时其他列的任务会回到此列, 是数据兜底" : "删除列"}
                      >
                        ✕
                      </button>
                    );
                  })()}
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
              {/* + Add task (per 2026-08-31 11:56 JST Ulysses 拍板, Jira 风格) */}
              {onAddWorkItem && (
                editingCard === col.status ? (
                  <div
                    data-testid={`kanban-column-add-card-form-${col.status}`}
                    className="mt-2 rounded-lg border border-accent/60 bg-bg-card p-2 shadow-sm space-y-1.5"
                  >
                    <input
                      ref={(el) => { cardInputRef.current = el; }}
                      data-testid={`kanban-column-add-card-input-${col.status}`}
                      value={editingCardTitle}
                      onChange={(e) => setEditingCardTitle(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          e.preventDefault();
                          commitAddCard(col.status);
                        } else if (e.key === "Escape") {
                          e.preventDefault();
                          cancelAddCard();
                        }
                      }}
                      placeholder="输入任务标题, Enter 提交"
                      className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink placeholder:text-ink-mute outline-none focus:border-accent"
                    />
                    <div className="flex items-center gap-1.5">
                      <button
                        type="button"
                        data-testid={`kanban-column-add-card-submit-${col.status}`}
                        onClick={() => commitAddCard(col.status)}
                        disabled={!editingCardTitle.trim()}
                        className="px-2.5 py-1 text-[11px] font-semibold rounded bg-accent text-bg hover:bg-accent/90 disabled:bg-ink-mute/40 disabled:cursor-not-allowed transition-colors"
                      >
                        添加
                      </button>
                      <button
                        type="button"
                        data-testid={`kanban-column-add-card-cancel-${col.status}`}
                        onClick={cancelAddCard}
                        className="px-2 py-1 text-[11px] text-ink-mute hover:text-ink rounded hover:bg-bg-soft transition-colors"
                      >
                        取消
                      </button>
                    </div>
                  </div>
                ) : (
                  <button
                    type="button"
                    data-testid={`kanban-column-add-card-${col.status}`}
                    onClick={() => startAddCard(col.status)}
                    className="mt-1 w-full inline-flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-lg border border-dashed border-line/60 hover:border-accent/60 bg-transparent hover:bg-accent/5 text-[11px] text-ink-mute hover:text-accent transition-all duration-150 group"
                    title="新增任务卡"
                  >
                    <Plus size={12} className="group-hover:rotate-90 transition-transform duration-200" />
                    <span>Add task</span>
                  </button>
                )
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export { KANBAN_COLUMNS };
