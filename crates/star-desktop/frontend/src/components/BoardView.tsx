// crates/star-desktop/frontend/src/components/BoardView.tsx
// =====================================================================
// Star Desktop — BoardView 集成组件 (PR-252 follow-up of PR #251)
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5 阶段 5 (per PR #244 §3.5)
//
// 集成:
//   1. useTauriWorkItems (PR #250) — fetch work items via Tauri IPC
//   2. KanbanBoard (PR #249) — render 6 列 × W/T/M swimlane
//   3. useLayoutEngine (PR #251) — pickAlgorithm for WIP analysis
//   4. useQueryEngine (PR #251) — DSL parsing for filter
//   5. onTransition handler — drop zone state migration
// 0 依赖 + 0 改 frontend/src/components/board/
// =====================================================================

import { useState, useMemo, useCallback } from "react";
import { KanbanBoard } from "./KanbanBoard";
import { useTauriWorkItems } from "../hooks/useTauriWorkItems";
import { useLayoutEngine, type ViewMode } from "../hooks/useLayoutEngine";
import { useQueryEngine } from "../hooks/useQueryEngine";
import type { WorkItem, WorkItemStatus } from "../types/ids";

export interface BoardViewProps {
  /** Initial view mode (per layout-engine §3.1) */
  initialViewMode?: ViewMode;
  /** Optional initial DSL filter (per query-engine) */
  initialDsl?: string;
  /** Optional refetch handler (e.g., from Tauri event listener) */
  onExternalRefetch?: () => void;
}

/**
 * BoardView — 集成看板视图
 *
 * State machine:
 *   IPC: workItems (loading, error, refetch)
 *   WASM: layoutReady (false = JS fallback), queryReady (false = JS fallback)
 *   Local: viewMode, dslFilter, workItemsOverride (for transitions)
 */
export function BoardView({
  initialViewMode = "tree",
  initialDsl = "",
  onExternalRefetch,
}: BoardViewProps) {
  // 1. Tauri IPC
  const { workItems, loading, error, refetch } = useTauriWorkItems();
  const [workItemsOverride, setWorkItemsOverride] = useState<WorkItem[] | null>(null);

  // 2. WASM hooks (per PR #251)
  const layout = useLayoutEngine();
  const query = useQueryEngine();

  // 3. Local state
  const [viewMode, setViewMode] = useState<ViewMode>(initialViewMode);
  const [dslFilter, setDslFilter] = useState<string>(initialDsl);

  // 4. Filtered work items via DSL (per useQueryEngine.parseKeywords)
  const filteredItems = useMemo(() => {
    const items = workItemsOverride ?? workItems;
    const keywords = dslFilter ? query.parseKeywords(dslFilter) : [];
    if (keywords.length === 0) return items;
    // Simple filter: include if any keyword matches kind (show/agent/behind/ahead/health/modified)
    // 完整实现留 PR-252+ 接 WASM
    return items.filter((item) => {
      if (keywords.includes(item.kind as never)) return true;
      if (keywords.includes("show") && item.status !== "done") return true;
      return false;
    });
  }, [workItems, workItemsOverride, dslFilter, query]);

  // 5. Algorithm selection via WASM hook
  const algorithm = layout.pickAlgorithm(viewMode, filteredItems.length);

  // 6. Transition handler (drop zone)
  const handleTransition = useCallback(
    (workItemId: string, toStatus: WorkItemStatus) => {
      setWorkItemsOverride((current) => {
        const base = current ?? workItems;
        return base.map((item) =>
          item.id === workItemId ? { ...item, status: toStatus } : item,
        );
      });
    },
    [workItems],
  );

  // 7. Refresh handler
  const handleRefresh = useCallback(async () => {
    setWorkItemsOverride(null);
    await refetch();
    onExternalRefetch?.();
  }, [refetch, onExternalRefetch]);

  // 8. Error / Loading states
  if (loading && workItemsOverride === null) {
    return (
      <div className="p-4 text-gray-500" data-testid="board-view-loading">
        Loading work items via Tauri IPC...
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-red-600" data-testid="board-view-error">
        Error: {error.message}
        <button onClick={handleRefresh} className="ml-2 underline">Retry</button>
      </div>
    );
  }

  return (
    <div className="board-view p-4" data-testid="board-view">
      {/* Toolbar */}
      <div className="flex items-center gap-3 mb-3 p-2 bg-gray-50 rounded border">
        <select
          value={viewMode}
          onChange={(e) => setViewMode(e.target.value as ViewMode)}
          className="border rounded px-2 py-1 text-sm"
          data-testid="view-mode-select"
          aria-label="View mode"
        >
          <option value="tree">tree</option>
          <option value="dependency">dependency</option>
          <option value="risk">risk</option>
          <option value="agent">agent</option>
          <option value="history">history</option>
        </select>

        <input
          type="text"
          value={dslFilter}
          onChange={(e) => setDslFilter(e.target.value)}
          placeholder="DSL filter (e.g., 'show ready', 'behind >5')"
          className="border rounded px-2 py-1 text-sm flex-1"
          data-testid="dsl-filter-input"
          aria-label="DSL filter"
        />

        <button
          onClick={handleRefresh}
          className="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
          data-testid="refresh-button"
        >
          Refresh
        </button>

        <span className="text-xs text-gray-500 ml-auto" data-testid="board-meta">
          algo: {algorithm} | {filteredItems.length} items | WASM: layout={layout.ready ? 'Y' : 'N'}/query={query.ready ? 'Y' : 'N'}
        </span>
      </div>

      {/* KanbanBoard (per PR #249) */}
      <KanbanBoard
        workItems={filteredItems}
        onTransition={handleTransition}
        swimlaneEnabled
      />
    </div>
  );
}
