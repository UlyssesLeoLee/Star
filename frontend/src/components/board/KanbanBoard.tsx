/*
@cypher schema=1 source_sha256=467a25e615a5611c8ba54e5e3afb4ae8de7d2c79a1c35266bcad956f18333abb
MERGE (self:File {path:"frontend/src/components/board/KanbanBoard.tsx"})
MERGE (board:Symbol {id:"frontend/src/components/board/KanbanBoard.tsx::KanbanBoard",kind:"component"})
MERGE (statusNoticeFile:File {path:"frontend/src/components/status/StatusNotice.tsx"})
MERGE (statusNotice:Symbol {id:"frontend/src/components/status/StatusNotice.tsx::StatusNotice",kind:"component"})
MERGE (self)-[:DEFINES]->(board)
MERGE (self)-[:IMPORTS]->(statusNoticeFile)
MERGE (board)-[:CALLS]->(statusNotice)
@endcypher
*/
"use client";

// =====================================================================
// KanbanBoard — Kanban 看板列容器 (per dynamic-interaction-design.md §3.4)
// =====================================================================
// 职责:
//   1. 渲染 board.columns 提供的状态列; 默认列顺序由 KANBAN_COLUMNS 管理
//   2. 列 onDragOver + onDrop 接收卡片拖动, 触发 onTransition(id, toStatus)
//   3. drop zone 高亮 (蓝边 2px + bg-accent/10) when dropTarget === status
//   4. WIP limit 超限显示 (per seed.ts wip_limit 字段)
//   5. 管理 dragging 状态, 让被拖卡片 opacity 0.5
//   6. 无授权数据诚实态 (per 2026-11-16 清 seed/mock 显示入口): unavailableState
//      非空时不渲染列/卡片/拖动/创建按钮, 改渲染 StatusNotice (未授权/未接入/空数据)
//      + 预览模式诚实提示; 本地 seed 任务卡不作为任务事实展示
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
import { StatusNotice } from "@/components/status/StatusNotice";
import { Tooltip } from "@/components/ui/tooltip";
import { GasParticlesHint } from "@/components/effects/GasParticlesHint";
import { AlertTriangle, Plus, SlidersHorizontal } from "lucide-react";
import type { Board, WorkItem, WorkItemStatus, Identity, WtmCategory } from "@/types/ids";
import { WT_M_FROM_KIND, WT_M_LABELS } from "@/types/ids";
import { KANBAN_COLUMNS } from "@/mocks/data";
import { isFallbackStatus } from "./constants";
import { useTranslation, interpolate, useStatusLabel } from "@/lib/i18n";

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
  // 列内新增任务卡 (per 2026-08-31 11:56 JST Ulysses 拍板: 列内快捷新增)
  // 2026-08-31 12:07 JST 二拍: +Add task 按钮点击触发父组件推 Drawer (mode="new", status 锁定为列)
  //   - 入参只传 status, title 由 Drawer 内输入
  onRequestNewWorkItem?: (status: WorkItemStatus) => void;
  // KanbanCard 点击 -> 父组件推 Drawer (mode="view", workItemId)
  // 2026-08-31 12:07 JST 拍板: 跟 Drawer view 模式联动
  onWorkItemClick?: (workItem: WorkItem) => void;
  // 高亮"下一步状态列" (per 2026-09-05 拍板, 场景 2: 当前用户最近 transition 卡片的下一状态列)
  // 父组件计算后传入, 该列右上角飘气态粒子. undefined = 不高亮.
  nextStepStatus?: WorkItemStatus | null;
  // 架构查看器 (per ADR-0041) → 父组件弹 ArchGraphModal
  // 透传到 KanbanCard 的 🕸 Arch icon 按钮
  onArchClick?: (workItem: WorkItem) => void;
  /** 无授权数据诚实态 (per 2026-11-16 清 seed/mock 显示入口):
   *  非空时不渲染本地 seed 任务卡 (不当真任务), 改渲染 StatusNotice +
   *  预览模式诚实提示; undefined = 沿用旧行为 (测试/其他调用方)。 */
  unavailableState?: KanbanUnavailableState | null;
}

/** 看板无授权数据的诚实态三选一 (per 2026-11-16 清 seed/mock 显示入口):
 *  - unauthorized: Group API 会话 401/失效 (宿主已装配但未授权)
 *  - disconnected: 宿主数据通道未装配 (网页预览模式)
 *  - empty: 宿主已接入但无授权任务数据 */
export type KanbanUnavailableState = "empty" | "unauthorized" | "disconnected";

const UNAVAILABLE_COPY: Record<KanbanUnavailableState, { title: string; hint: string }> = {
  unauthorized: {
    title: "未授权 — 看板数据不可用",
    hint: "当前会话未获授权读取服务端任务数据。请在宿主会话中重新授权后刷新, 本地 seed 数据不作为任务事实展示。",
  },
  disconnected: {
    title: "未接入 — 看板数据通道不可用",
    hint: "当前会话未装配宿主数据通道 (网页预览模式)。本地 seed 数据不作为任务事实展示, 宿主会话接入后自动切换为授权投影。",
  },
  empty: {
    title: "暂无任务数据",
    hint: "当前没有可显示的授权任务数据。请调整筛选条件, 或等待宿主会话接入后由授权投影填充。",
  },
};

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
  onRequestNewWorkItem,
  onWorkItemClick,
  nextStepStatus,
  onArchClick,
  unavailableState,
}: KanbanBoardProps) {
  const { t, tx } = useTranslation();
  const fallbackLabel = useStatusLabel("workItem", "todo");
  void fallbackLabel; // 防止未使用警告
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

  // 无授权数据诚实态 (per 2026-11-16 清 seed/mock 显示入口):
  // 未授权/未接入/空数据时不渲染本地 seed 任务卡 (不当真任务),
  // 改渲染 StatusNotice + 预览模式诚实提示; 宿主会话接入后自动切换授权投影。
  if (unavailableState) {
    const copy = UNAVAILABLE_COPY[unavailableState];
    return (
      <div data-testid="kanban-unavailable" className="space-y-3">
        <StatusNotice state={unavailableState} title={copy.title} hint={copy.hint} />
        <div
          data-testid="kanban-preview-hint"
          className="flex items-center gap-2 rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[11px] text-warn"
        >
          <span className="size-1.5 rounded-full bg-warn" />
          预览数据模式 — 当前看板展示本地 seed 数据, 非授权服务端任务事实。宿主会话接入后自动切换为授权投影。
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {/* === Kanban Board Toolbar Control Bar === */}
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-2 text-xs font-mono text-ink-mute">
          <span className="font-bold text-ink-dim uppercase tracking-wider">
            {interpolate(t.board.columnsActive, { count: board.columns.length })}
          </span>
          <span className="text-[10px] opacity-60 hidden sm:inline">
            {t.board.columnReorderHint}
          </span>
        </div>

        {onAddColumn && (
          <button
            type="button"
            data-testid="kanban-add-column"
            onClick={handleAddColumn}
            className="btn text-xs gap-1.5"
            title={t.board.addColumnTitle}
          >
            <Plus size={13} className="text-accent group-hover:rotate-90 transition-transform duration-200" />
            <span>{t.board.addColumn}</span>
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
              style={{ transformStyle: "preserve-3d" }}
              className={clsx(
                "card card-compact !p-3.5 min-h-[200px] transition-colors relative",
                overWip && "border-warn/60",
                isDropTarget && "ring-2 ring-accent bg-accent/10",
                // 列重排 drop 高亮 (per 2026-08-29 19:09 JST)
                isColDragging && "opacity-50",
                isColDropTarget && "ring-2 ring-accent bg-accent/10",
              )}
            >
              {/* 气态粒子提示 (per 2026-09-05 拍板, 场景 2: 下一步状态列) */}
              {nextStepStatus === col.status && (
                <GasParticlesHint
                  variant="swirl"
                  color="info"
                  density={0.5}
                  width={140}
                  height={80}
                  offsetX={-10}
                  offsetY={-10}
                />
              )}
              {/* 列拖动手柄 (per 2026-08-29 19:09 JST, 把整列设为 draggable) */}
              {onReorderColumns && (
                <div
                  draggable
                  onDragStart={(e) => handleColDragStart(e, board.columns.findIndex((c) => c.status === col.status))}
                  onDragEnd={handleColDragEnd}
                  data-testid={`kanban-column-drag-handle-${col.status}`}
                  className="text-ink-mute hover:text-accent cursor-grab active:cursor-grabbing text-xs select-none"
                  title={t.board.dragToReorder}
                  aria-label={tx(t.board.reorderColumn, { name: col.name ?? col.status })}
                >
                  ⋮⋮
                </div>
              )}
              <div
                style={{ transform: "translateZ(4px)" }}
                className="flex items-center justify-between mb-3 gap-1 flex-1 min-w-0"
              >
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
                    className="text-xs font-mono uppercase tracking-wider font-bold bg-bg-card border border-accent/60 rounded px-1.5 py-0.5 text-ink outline-none w-full"
                  />
                ) : (
                  <button
                    data-testid={`kanban-column-name-${col.status}`}
                    type="button"
                    onClick={() => startEdit(col.status, col.name ?? col.status)}
                    disabled={!onRenameColumn}
                    className="text-xs font-mono font-bold uppercase tracking-wider text-ink hover:text-accent transition-colors text-left truncate"
                    title={t.board.clickToRename}
                  >
                    {col.name ?? col.status}
                  </button>
                )}
                <div className="flex items-center gap-1 shrink-0">
                  <span className="text-xs text-ink-dim font-mono font-bold px-1.5 py-0.2 bg-black/10 dark:bg-black/40 border border-black/20">
                    {cards.length}
                    {col.wip_limit !== undefined && col.wip_limit < 99 && ` / ${col.wip_limit}`}
                  </span>
                  {onRemoveColumn && (() => {
                    // per 2026-08-31 11:24 JST 拍板: 兜底列 (todo) 不可删,
                    // 按钮置灰 + 漫画气泡 tooltip 解释, store 也会二次拒绝
                    const isFallback = isFallbackStatus(col.status);
                    const removeBtn = (
                      <button
                        type="button"
                        data-testid={`kanban-column-remove-${col.status}`}
                        onClick={() => {
                          if (isFallback) return; // 兜底列 click 直接吞掉, 兜底保护
                          onRemoveColumn(col.status);
                        }}
                        disabled={isFallback}
                        aria-label={isFallback
                          ? tx(t.board.fallbackColumnNotRemovable, { name: col.name ?? col.status })
                          : `${t.board.removeColumn} ${col.name ?? col.status}`}
                        className={
                          isFallback
                            ? "text-ink-mute/40 cursor-not-allowed text-xs leading-none px-1"
                            : "text-ink-mute hover:text-err transition-colors text-xs leading-none px-1"
                        }
                      >
                        ✕
                      </button>
                    );
                    // 兜底列: 长说明走漫画气泡; 可删列: 短提示
                    const tipText = isFallback
                      ? t.board.fallbackColumnProtected
                      : tx(t.board.reorderColumn, { name: col.name ?? col.status });
                    return (
                      <Tooltip content={tipText} side="top" delayShow={150}>
                        {removeBtn}
                      </Tooltip>
                    );
                  })()}
                </div>
              </div>
              {overWip && (
                <div className="mb-2 text-[10px] text-warn flex items-center gap-1">
                  <AlertTriangle size={10} /> {t.board.wipExceeded}
                </div>
              )}
              <div className="space-y-2">
              {cards.length === 0 && (
                <div className="text-[10px] text-ink-mute italic text-center py-4">
                  {t.board.dragCardsHere}
                </div>
              )}
              {/* === W/T/M 三類横展開 swimlane (per AGENTS.md §4 #13 + docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md) ===
                  - 把 cards 按 w_t_m 分 3 个 swimlane (W = 业务 / T = 执行 / M = 主数据)
                  - 每个 swimlane 头部小 label (W / T / M + count), 卡片按 swimlane 分组渲染
                  - 0 卡片的 swimlane 自动隐藏 (避免视觉噪声)
                  - 兜底 (uncategorized) 合并到 M swimlane 底部, 标"未分類"
                  - 拖动/drop 行为不变 (per测试 ✓): 卡片仍在 col.work_item_ids 数组里, drop 到 col 仍触发 onTransition
              */}
              {(() => {
                const wtm = (w: WorkItem): WtmCategory => w.w_t_m ?? WT_M_FROM_KIND[w.kind] ?? "uncategorized";
                // 顺序: W → T → M, uncategorized 合并到 M 末尾
                const order: WtmCategory[] = ["W", "T", "M"];
                const groups = order.map((cat) => ({
                  cat,
                  items: cards.filter((w) => wtm(w) === cat),
                })).filter((group) => group.items.length > 0);
                const uncat = cards.filter((w) => wtm(w) === "uncategorized");
                return (
                  <>
                    {groups.map((g) => (
                      <div key={g.cat} data-testid={`kanban-swimlane-${g.cat}-${col.status}`}>
                        {g.items.length > 0 && (
                          <div className="flex items-center gap-1.5 mb-1 px-0.5 select-none">
                            <span className={clsx(
                              "text-[9px] font-mono font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border",
                              g.cat === "W" && "border-accent/40 bg-accent/10 text-accent",
                              g.cat === "T" && "border-warn/40 bg-warn/10 text-warn",
                              g.cat === "M" && "border-ok/40 bg-ok/10 text-ok",
                            )}>
                              {g.cat} · {WT_M_LABELS[g.cat].zh}
                            </span>
                            <span className="text-[9px] text-ink-mute font-mono">{g.items.length}</span>
                            <span className="flex-1 border-t border-line/40" />
                          </div>
                        )}
                        <div className="space-y-2">
                          {g.items.map((w) => (
                            <KanbanCard
                              key={w.id}
                              workItem={w}
                              assignee={w.assignee_id ? identityMap[w.assignee_id] : undefined}
                              isDragging={effectiveDraggingId === w.id}
                              onDragStart={handleCardDragStart}
                              onDragEnd={handleCardDragEnd}
                              onClick={onWorkItemClick}
                              onArchClick={onArchClick}
                            />
                          ))}
                        </div>
                      </div>
                    ))}
                    {uncat.length > 0 && (
                      <div data-testid={`kanban-swimlane-uncategorized-${col.status}`}>
                        <div className="flex items-center gap-1.5 mb-1 px-0.5 select-none">
                          <span className="text-[9px] font-mono font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border border-ink-mute/30 bg-bg-soft text-ink-mute">
                            未分類 · {uncat.length}
                          </span>
                          <span className="flex-1 border-t border-line/40" />
                        </div>
                        <div className="space-y-2">
                          {uncat.map((w) => (
                            <KanbanCard
                              key={w.id}
                              workItem={w}
                              assignee={w.assignee_id ? identityMap[w.assignee_id] : undefined}
                              isDragging={effectiveDraggingId === w.id}
                              onDragStart={handleCardDragStart}
                              onDragEnd={handleCardDragEnd}
                              onClick={onWorkItemClick}
                              onArchClick={onArchClick}
                            />
                          ))}
                        </div>
                      </div>
                    )}
                  </>
                );
              })()}
              </div>
              {/* + Add task (per 2026-08-31 12:07 JST 二拍, Drawer 替代 inline 编辑)
                  - 点击 → 父组件 onRequestNewWorkItem(status) 推 WorkItemDetailDrawer
                  - 替代 11:56 JST 一拍的 inline input (per 拍板反转) */}
              {onRequestNewWorkItem && (
                <button
                  type="button"
                  data-testid={`kanban-column-add-card-${col.status}`}
                  onClick={() => onRequestNewWorkItem(col.status)}
                  style={{ transform: "translateZ(2px)" }}
                  className="btn-primary cel-btn-3d mt-1.5 w-full inline-flex items-center justify-center gap-1.5 !min-h-[34px] !px-2.5 !py-1.5 text-[11px] font-extrabold group"
                  title={t.board.addTaskTitle}
                >
                  <Plus size={12} className="group-hover:rotate-90 transition-transform duration-200" />
                  <span>{t.board.addTask}</span>
                </button>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export { KANBAN_COLUMNS };