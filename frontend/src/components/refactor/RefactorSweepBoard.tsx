"use client";

// =====================================================================
// RefactorSweepBoard — 重构专项 5 列看板 (per 2026-09-02 10:41 JST 拍板)
// =====================================================================
// 形态 1:1 镜像 KanbanBoard, 但操作对象是 RefactorRound.cards + RefactorBoardConfig.columns
// 行为对齐 (per 2026-09-02 拍板):
//   1. 默认 5 列: todo / doing / testing / review / done
//      (testing 在 doing 和 review 中间, per 拍板)
//   2. 拖动卡到列触发 moveRefactorCard (写 history)
//   3. 列可增 (含自定义 status) / 删 (兜底 todo 拒绝) / 重命名 (inline) / 重排 (drag handle)
//   4. 跟 KanbanBoard 行为 1:1 (inline edit / drop 蓝边 / fallback tooltip / 拖动手柄 ⋮⋮)
// =====================================================================

import { useState, useCallback, useMemo, useRef, useEffect } from "react";
import { clsx } from "clsx";
import { Plus, X, GripVertical, SlidersHorizontal } from "lucide-react";
import { useTranslation, useStatusLabel, interpolate } from "@/lib/i18n";
import { isRefactorFallbackStatus } from "@/lib/board-refactor-constants";
import type {
  RefactorCard as RefactorCardData,
  RefactorColumn,
  RefactorRound,
  RefactorStatus,
  RefactorBoardConfig,
} from "@/types/ids";
import { RefactorCard } from "./RefactorCard";

export interface RefactorSweepBoardProps {
  round: RefactorRound;
  config: RefactorBoardConfig;
  // 列 CRUD 回调 (父组件接 store action)
  onMoveCard: (workItemId: string, toStatus: RefactorStatus) => void;
  onAddColumn: (status: RefactorStatus, name?: string) => void;
  onRemoveColumn: (status: RefactorStatus) => void;
  onRenameColumn: (status: RefactorStatus, newName: string) => void;
  onReorderColumns: (fromIdx: number, toIdx: number) => void;
  onCardClick?: (card: RefactorCardData) => void;
  /** 是否禁用交互 (e.g. round.closed_at) */
  readOnly?: boolean;
  /**
   * Merge 回调 (per 2026-09-02 10:50 JST 拍板)
   *   - 父组件接 store.mergeRefactorCard, 触发 worktree→merged + PR→merged
   *   - 仅 done 列的卡显示 Merge 按钮
   */
  onMergeCard?: (workItemId: string) => void;
  /** 给定 work_item_id 判断是否有关联 worktree (用于按钮 title hint 区分) */
  hasWorktree?: (workItemId: string) => boolean;
}

export function RefactorSweepBoard({
  round, config,
  onMoveCard, onAddColumn, onRemoveColumn, onRenameColumn, onReorderColumns,
  onCardClick, readOnly = false, onMergeCard, hasWorktree,
}: RefactorSweepBoardProps) {
  const { t, tx } = useTranslation();
  const sortedColumns = useMemo(
    () => [...config.columns].sort((a, b) => a.position - b.position),
    [config.columns],
  );

  // 拖动卡到列
  const [dropTarget, setDropTarget] = useState<RefactorStatus | null>(null);
  const [draggingCardId, setDraggingCardId] = useState<string | null>(null);
  const handleCardDragStart = useCallback(
    (e: React.DragEvent<HTMLDivElement>, card: RefactorCardData) => {
      if (readOnly) {
        e.preventDefault();
        return;
      }
      setDraggingCardId(card.work_item_id);
    },
    [readOnly],
  );
  const handleCardDragEnd = useCallback(() => setDraggingCardId(null), []);

  // 列拖动重排
  const [draggingColIdx, setDraggingColIdx] = useState<number | null>(null);
  const [dropTargetColIdx, setDropTargetColIdx] = useState<number | null>(null);
  const handleColDragStart = useCallback((e: React.DragEvent<HTMLDivElement>, idx: number) => {
    e.dataTransfer.setData("text/refactor-col-idx", String(idx));
    e.dataTransfer.effectAllowed = "move";
    setDraggingColIdx(idx);
  }, []);
  const handleColDragOver = useCallback((e: React.DragEvent<HTMLDivElement>, idx: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    setDropTargetColIdx(idx);
  }, []);
  const handleColDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>, idx: number) => {
    const related = e.relatedTarget as Node | null;
    if (related && (e.currentTarget as Node).contains(related)) return;
    setDropTargetColIdx((cur) => (cur === idx ? null : cur));
  }, []);
  const handleColDrop = useCallback((e: React.DragEvent<HTMLDivElement>, toIdx: number) => {
    e.preventDefault();
    const fromIdxStr = e.dataTransfer.getData("text/refactor-col-idx");
    if (!fromIdxStr) return;
    const fromIdx = Number(fromIdxStr);
    if (Number.isNaN(fromIdx) || fromIdx === toIdx) {
      setDraggingColIdx(null);
      setDropTargetColIdx(null);
      return;
    }
    onReorderColumns(fromIdx, toIdx);
    setDraggingColIdx(null);
    setDropTargetColIdx(null);
  }, [onReorderColumns]);
  const handleColDragEnd = useCallback(() => {
    setDraggingColIdx(null);
    setDropTargetColIdx(null);
  }, []);

  // 列 drop 接收卡
  const handleColumnDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    if (readOnly) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  }, [readOnly]);
  const handleColumnDrop = useCallback((e: React.DragEvent<HTMLDivElement>, status: RefactorStatus) => {
    if (readOnly) return;
    e.preventDefault();
    const cardId = e.dataTransfer.getData("text/refactor-card-id");
    if (!cardId) return;
    onMoveCard(cardId, status);
    setDropTarget(null);
    setDraggingCardId(null);
  }, [onMoveCard, readOnly]);

  // 列名 inline 编辑
  const [editingCol, setEditingCol] = useState<RefactorStatus | null>(null);
  const [editingName, setEditingName] = useState<string>("");
  const startEditName = (col: RefactorColumn, currentDisplayName: string) => {
    setEditingCol(col.status);
    setEditingName(currentDisplayName);
  };
  const commitEditName = (status: RefactorStatus, originalDisplayName: string) => {
    if (editingCol !== status) return;
    const trimmed = editingName.trim();
    if (trimmed && trimmed !== originalDisplayName) {
      onRenameColumn(status, trimmed);
    }
    setEditingCol(null);
  };

  // add column prompt (status + name)
  const handleAddColumn = useCallback(() => {
    if (typeof window === "undefined") return;
    const status = window.prompt(tx(t.refactor.addColumnTitle, {}))?.trim();
    if (!status) return;
    if (sortedColumns.some((c) => c.status === status)) {
      window.alert(`status "${status}" 已存在`);
      return;
    }
    onAddColumn(status);
  }, [onAddColumn, sortedColumns, t, tx]);

  // 自动聚焦 inline edit input
  const editInputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (editingCol && editInputRef.current) {
      editInputRef.current.focus();
      editInputRef.current.select();
    }
  }, [editingCol]);

  return (
    <div
      data-testid="refactor-sweep-board"
      className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-3"
    >
      {sortedColumns.map((col, idx) => {
        const cards = round.cards.filter((c) => c.refactor_status === col.status);
        const isFallback = isRefactorFallbackStatus(col.status);
        const displayName = col.name ?? useStatusLabel("refactor", col.status);
        const overWip = col.wip_limit !== undefined && cards.length > col.wip_limit;
        const isDropTarget = dropTarget === col.status;
        const isColDropTarget = dropTargetColIdx === idx && draggingColIdx !== null;
        return (
          <div
            key={col.status}
            data-testid={`refactor-col-${col.status}`}
            data-col-idx={idx}
            draggable={draggingColIdx === null && !readOnly}
            onDragStart={(e) => handleColDragStart(e, idx)}
            onDragOver={(e) => {
              handleColDragOver(e, idx);
              handleColumnDragOver(e);
            }}
            onDragLeave={(e) => handleColDragLeave(e, idx)}
            onDrop={(e) => {
              handleColDrop(e, idx);
              handleColumnDrop(e, col.status);
            }}
            onDragEnd={handleColDragEnd}
            className={clsx(
              "card flex flex-col min-h-[280px] transition-all duration-200",
              isDropTarget && "ring-2 ring-accent bg-accent/5",
              isColDropTarget && "ring-2 ring-warn",
              draggingColIdx === idx && "opacity-50",
            )}
          >
            {/* 列头: 拖手柄 + 名称 (inline edit) + 计数 + ✕ */}
            <div className="flex items-center justify-between mb-2 pb-1.5 border-b border-line">
              <div className="flex items-center gap-1.5 min-w-0 flex-1">
                {!readOnly && (
                  <GripVertical
                    size={11}
                    className="text-ink-mute opacity-50 hover:opacity-100 shrink-0 cursor-grab"
                    aria-label={t.refactor.dragToReorder}
                  />
                )}
                {editingCol === col.status ? (
                  <input
                    ref={editInputRef}
                    value={editingName}
                    onChange={(e) => setEditingName(e.target.value)}
                    onBlur={() => commitEditName(col.status, displayName)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") commitEditName(col.status, displayName);
                      else if (e.key === "Escape") setEditingCol(null);
                    }}
                    className="text-[11px] font-mono font-bold bg-bg-card border border-accent/60 rounded px-1 py-0.5 w-full"
                  />
                ) : (
                  <button
                    type="button"
                    onClick={() => !readOnly && startEditName(col, displayName)}
                    className={clsx(
                      "text-[11px] font-mono font-bold uppercase tracking-wider truncate text-left",
                      "hover:text-accent transition-colors",
                      readOnly && "cursor-default",
                    )}
                    title={!readOnly ? t.refactor.renameColumn : undefined}
                    data-testid={`refactor-col-name-${col.status}`}
                  >
                    {displayName}
                  </button>
                )}
                {isFallback && (
                  <span
                    title={t.refactor.fallbackNotRemovable.replace("{name}", displayName)}
                    className="text-[8px] font-mono px-1 py-0 rounded border border-ink-mute/40 bg-bg-soft text-ink-mute shrink-0"
                  >
                    FB
                  </span>
                )}
              </div>
              <div className="flex items-center gap-1.5 shrink-0">
                <span
                  className={clsx(
                    "text-[10px] font-mono px-1.5 py-0.2 rounded border",
                    overWip
                      ? "border-warn/40 bg-warn/10 text-warn"
                      : "border-line bg-bg-soft text-ink-mute",
                  )}
                  title={overWip ? t.refactor.wipExceeded : undefined}
                >
                  {cards.length}
                </span>
                {!isFallback && !readOnly && (
                  <button
                    type="button"
                    onClick={() => onRemoveColumn(col.status)}
                    title={interpolate(t.refactor.fallbackProtected, { name: displayName })}
                    data-testid={`refactor-col-remove-${col.status}`}
                    className="p-0.5 rounded text-ink-mute hover:text-err hover:bg-err/10 transition-colors"
                  >
                    <X size={11} />
                  </button>
                )}
              </div>
            </div>

            {/* 卡片区 */}
            <div
              className="flex-1 space-y-1.5 min-h-[60px]"
              onDragOver={(e) => handleColumnDragOver(e)}
              onDrop={(e) => handleColumnDrop(e, col.status)}
            >
              {cards.length === 0 ? (
                <div className="h-full min-h-[60px] flex items-center justify-center text-[10px] font-mono text-ink-mute/60 border border-dashed border-line/40 rounded">
                  {t.refactor.dropCardHere}
                </div>
              ) : (
                cards.map((c) => (
                  <RefactorCard
                    key={c.work_item_id}
                    card={c}
                    isDragging={draggingCardId === c.work_item_id}
                    onDragStart={handleCardDragStart}
                    onDragEnd={handleCardDragEnd}
                    onClick={onCardClick}
                    onMerge={onMergeCard ? () => onMergeCard(c.work_item_id) : undefined}
                    hasWorktree={hasWorktree?.(c.work_item_id)}
                    readOnly={readOnly}
                  />
                ))
              )}
            </div>
          </div>
        );
      })}

      {/* + Add column 按钮 (放末尾) */}
      {!readOnly && (
        <button
          type="button"
          onClick={handleAddColumn}
          data-testid="refactor-add-column"
          className="card border-dashed border-line hover:border-accent/60 hover:bg-accent/5 flex flex-col items-center justify-center min-h-[280px] text-ink-mute hover:text-accent transition-colors"
        >
          <Plus size={18} className="mb-1.5" />
          <span className="text-[10px] font-mono font-bold uppercase tracking-wider">
            {t.refactor.addColumn}
          </span>
          <span className="text-[9px] font-mono text-ink-mute/60 mt-1 text-center px-2">
            {t.refactor.dragToReorder}
          </span>
        </button>
      )}
    </div>
  );
}
