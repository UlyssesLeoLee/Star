// crates/star-desktop/frontend/src/components/KanbanBoard.tsx
// =====================================================================
// Star Desktop — KanbanBoard 看板容器 (PR-249 follow-up of PR #247/#248)
// 复用 frontend/src/components/board/KanbanBoard.tsx (521 LOC V0.1)
// 适配 0 依赖:
//   - @/components/StatusPill → inline <span className="badge">
//   - @/components/ui/tooltip → <span title="...">
//   - @/components/effects/GasParticlesHint → 0 移除
//   - lucide-react AlertTriangle/Plus/SlidersHorizontal → inline emoji/inline button
//   - @/lib/i18n → const 字符串
//   - @/mocks/data KANBAN_COLUMNS → use DEFAULT_KANBAN_COLUMNS from constants.ts
//   - @/types/ids (Next.js alias) → ../types/ids
// =====================================================================

import { useState, useCallback, useMemo } from "react";
import { clsx } from "clsx";
import { KanbanCard } from "./KanbanCard";
import type { WorkItem, WorkItemStatus, Identity, Wtm } from "../types/ids";
import { DEFAULT_KANBAN_COLUMNS, isFallbackStatus } from "../constants";

export interface KanbanBoardProps {
  workItems: WorkItem[];
  /** WIP limit per column (override defaults) */
  wipLimitOverride?: Partial<Record<WorkItemStatus, number>>;
  /** Drop zone hook (per PR-248)
   * 父组件传 onTransition(id, toStatus) 触发状态迁移
   */
  onTransition?: (workItemId: string, toStatus: WorkItemStatus) => void;
  /** Optional assignee resolver (mock by default) */
  resolveAssignee?: (assigneeId: string) => Identity | undefined;
  /** W/T/M swimlane enabled (per PR-216 default true) */
  swimlaneEnabled?: boolean;
}

// WT_M_FROM_KIND 派生表 (per docs §3.4 W/T/M swimlane)
const WT_M_FROM_KIND: Record<string, Wtm> = {
  story: "W",
  epic: "W",
  task: "T",
  bug: "T",
  spike: "M",
};

const WT_M_LABELS: Record<Wtm, string> = {
  W: "Work",
  T: "Transaction",
  M: "Master",
};

export function KanbanBoard({
  workItems,
  wipLimitOverride,
  onTransition,
  resolveAssignee,
  swimlaneEnabled = true,
}: KanbanBoardProps) {
  const [dropTarget, setDropTarget] = useState<WorkItemStatus | null>(null);
  const [draggingId, setDraggingId] = useState<string | null>(null);

  // 1. Group workItems by (swimlane, status)
  const groupedByLane = useMemo(() => {
    const result: Record<Wtm, Record<WorkItemStatus, WorkItem[]>> = {
      W: { todo: [], in_progress: [], review: [], blocked: [], done: [], wontfix: [] },
      T: { todo: [], in_progress: [], review: [], blocked: [], done: [], wontfix: [] },
      M: { todo: [], in_progress: [], review: [], blocked: [], done: [], wontfix: [] },
    };
    for (const item of workItems) {
      const wtm = item.w_t_m || WT_M_FROM_KIND[item.kind] || "W";
      if (result[wtm] && result[wtm][item.status]) {
        result[wtm][item.status].push(item);
      }
    }
    return result;
  }, [workItems]);

  // 2. Drag handlers
  const handleDragStart = useCallback(
    (_e: React.DragEvent<HTMLDivElement>, item: WorkItem) => {
      setDraggingId(item.id);
    },
    [],
  );

  const handleDragEnd = useCallback(() => {
    setDraggingId(null);
    setDropTarget(null);
  }, []);

  const handleDragOver = useCallback(
    (e: React.DragEvent<HTMLDivElement>, status: WorkItemStatus) => {
      e.preventDefault();
      e.dataTransfer.dropEffect = "move";
      setDropTarget(status);
    },
    [],
  );

  const handleDragLeave = useCallback(() => {
    setDropTarget(null);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent<HTMLDivElement>, toStatus: WorkItemStatus) => {
      e.preventDefault();
      const id = e.dataTransfer.getData("text/issue-id");
      if (id) {
        onTransition?.(id, toStatus);
      }
      setDropTarget(null);
      setDraggingId(null);
    },
    [onTransition],
  );

  // 3. Resolve wip_limit
  const getWipLimit = (status: WorkItemStatus): number | undefined => {
    if (wipLimitOverride?.[status] !== undefined) return wipLimitOverride[status];
    return DEFAULT_KANBAN_COLUMNS.find((c) => c.id === status)?.wip_limit;
  };

  // 4. Render swimlanes
  const swimlanes: Wtm[] = swimlaneEnabled ? ["W", "T", "M"] : [];

  return (
    <div className="kanban-board flex flex-col gap-2 p-2" data-testid="kanban-board">
      {swimlaneEnabled && (
        <div className="swimlane-labels flex gap-2 mb-1">
          {swimlanes.map((wtm) => (
            <div key={wtm} className="flex-1 text-center font-semibold text-sm text-gray-700">
              {wtm} — {WT_M_LABELS[wtm]}
            </div>
          ))}
        </div>
      )}

      <div
        className={clsx(
          "kanban-grid gap-2",
          swimlaneEnabled ? "flex flex-col" : "grid",
        )}
      >
        {swimlanes.map((wtm) => (
          <div key={wtm} className="kanban-swimlane grid grid-cols-6 gap-2" data-swimlane={wtm}>
            {DEFAULT_KANBAN_COLUMNS.map((col) => {
              const items = groupedByLane[wtm][col.id];
              const wip = getWipLimit(col.id);
              const overLimit = wip !== undefined && items.length > wip;
              const isDropTarget = dropTarget === col.id;
              const isFallback = isFallbackStatus(col.id);

              return (
                <div
                  key={col.id}
                  data-testid={`kanban-column-${col.id}`}
                  data-status={col.id}
                  onDragOver={(e) => handleDragOver(e, col.id)}
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, col.id)}
                  className={clsx(
                    "kanban-column bg-white border-2 rounded p-2 min-h-[120px]",
                    isFallback ? "border-blue-400" : "border-gray-300",
                    isDropTarget && "border-blue-500 bg-blue-50",
                    overLimit && "border-red-500 bg-red-50",
                  )}
                >
                  {/* Column header */}
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-semibold text-sm text-gray-800">
                      {col.name}
                      {isFallback && (
                        <span title="兜底列不可删除" className="ml-1 text-xs text-blue-500">
                          (protected)
                        </span>
                      )}
                    </span>
                    <span className={clsx(
                      "text-xs font-mono px-1.5 rounded",
                      overLimit ? "text-red-700 bg-red-100" : "text-gray-600 bg-gray-100",
                    )}>
                      {items.length}
                      {wip !== undefined && ` / ${wip}`}
                    </span>
                  </div>

                  {/* WIP limit warning */}
                  {overLimit && (
                    <div className="mb-2 text-xs text-red-700 flex items-center gap-1">
                      <span aria-hidden="true">⚠</span>
                      Over WIP limit
                    </div>
                  )}

                  {/* Cards */}
                  <div className="kanban-cards flex flex-col gap-2">
                    {items.map((item) => (
                      <KanbanCard
                        key={item.id}
                        workItem={item}
                        isDragging={draggingId === item.id}
                        assignee={resolveAssignee?.(item.assignee?.id ?? "")}
                        onDragStart={handleDragStart}
                        onDragEnd={handleDragEnd}
                      />
                    ))}
                  </div>

                  {/* Empty state */}
                  {items.length === 0 && (
                    <div className="text-xs text-gray-400 text-center py-4">
                      No items
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        ))}
      </div>

      {/* No-swimlane fallback (single grid) */}
      {!swimlaneEnabled && (
        <div className="grid grid-cols-6 gap-2">
          {DEFAULT_KANBAN_COLUMNS.map((col) => {
            const items = groupedByLane["W"][col.id].concat(
              groupedByLane["T"][col.id],
              groupedByLane["M"][col.id],
            );
            return (
              <div key={col.id} data-testid={`kanban-column-${col.id}`}>
                <h3 className="text-sm font-semibold mb-2">{col.name}</h3>
                {items.map((item) => (
                  <KanbanCard key={item.id} workItem={item} />
                ))}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export { DEFAULT_KANBAN_COLUMNS };
