"use client";

// =====================================================================
// Board Page — Kanban + 多人协同
// =====================================================================
// W1 负责 (per docs/frontend/design/dynamic-interaction-design.md §3):
//   - 用 <KanbanBoard> 替换 mock 渲染
//   - 订阅 useStore() (zustand), 调 transitionWorkItem (B.2.5 已实装)
//   - useBoardSync 2s 轮询 + 检测他人改动 → 本地 toast
//   - WIP limit 显示 (per seed.ts wip_limit 字段, 红色当超限)
//   - 跨模块联动: 卡片 onClick → /work-item/{id} (KanbanCard 内部实现)
//
// 不做 (per W1 守门):
//   - 不动 store.ts (W5 负责) — 直接 import useStore 即可
//   - 不引 dnd-kit / react-dnd (用 HTML5 native, per §2.4)
//   - 不改 design doc (read-only)
//   - 不动 layout.tsx (W5 负责 Toaster provider)
//
// 已知缺口 (per 缺标比错标):
//   - 不接 WebSocket, 用 2s 轮询 (per §10.3 #1)
//   - audit log 是 console.log stub, 真后端 D.6+ 接入 (per §3.3)
//   - 不接 react-hot-toast, 用内联 <SyncToast> (避免改 layout.tsx, W5 接管)
//   - filter 状态不持久化 (W5 加 zustand/persist 后再补)
// =====================================================================

import { useEffect, useState, useCallback, useMemo } from "react";
import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { KanbanBoard, KANBAN_COLUMNS } from "@/components/board/KanbanBoard";
import { KanbanFilters, type KanbanFiltersValue } from "@/components/board/KanbanFilters";
import { useBoardSync, type BoardSyncChange } from "@/hooks/useBoardSync";
import { Trello, RefreshCw, X } from "lucide-react";
import type { WorkItem, WorkItemStatus } from "@/types/ids";

export default function BoardPage() {
  const board = useStore((s) => s.board);
  const workItems = useStore((s) => s.workItems);
  const identities = useStore((s) => s.identities);
  const transitionWorkItem = useStore((s) => s.transitionWorkItem);

  // ---- filter state (本地上下文, 不持久化, W5 加 store persist 后再补) ----
  const [filter, setFilter] = useState<KanbanFiltersValue>({
    kind: "all",
    assignee_id: "all",
    priority: "all",
  });

  // ---- sync toast state (本地实现, 不引 react-hot-toast, W5 接管 layout) ----
  const [toastMsg, setToastMsg] = useState<string | null>(null);
  const showToast = useCallback((msg: string) => {
    setToastMsg(msg);
    // 4s 自动消失
    const t = setTimeout(() => setToastMsg(null), 4000);
    return () => clearTimeout(t);
  }, []);

  // ---- 多人协同: useBoardSync 2s 轮询, 检测到他人改动 → toast ----
  // (per §8.1 last-write-wins; W5 接管 Toaster 后, 这里改成 toast.success)
  useBoardSync({
    projectId: board.project_id,
    intervalMs: 2000,
    enabled: true,
    onRemoteChange: (changes: BoardSyncChange[]) => {
      const summary = changes
        .map((c) => `${c.work_item_id}: ${c.from_status}→${c.to_status} by ${c.changed_by}`)
        .join("; ");
      showToast(`Board updated in another session — ${summary}`);
    },
  });

  // ---- filter 谓词 ----
  const filterFn = useCallback(
    (w: WorkItem) =>
      (filter.kind === "all" || w.kind === filter.kind) &&
      (filter.assignee_id === "all" || w.assignee_id === filter.assignee_id) &&
      (filter.priority === "all" || w.priority === filter.priority),
    [filter],
  );

  // ---- 显示 work-item 数 (after filter) ----
  const allIds = useMemo(
    () => board.columns.flatMap((c) => c.work_item_ids),
    [board.columns],
  );
  const workItemById = useMemo(
    () => Object.fromEntries(workItems.map((w) => [w.id, w])),
    [workItems],
  );
  const filteredCount = useMemo(
    () => allIds.filter((id) => {
      const w = workItemById[id];
      return w ? filterFn(w) : false;
    }).length,
    [allIds, workItemById, filterFn],
  );

  // ---- 拖动 transition 处理 ----
  // 注: store.transitionWorkItem 只更新 workItems[i].status,
  //     不更新 board.columns[j].work_item_ids, 所以这里在 page 层同步两处
  // (per W1 守门: 不重写 store, 仅在调用方补偿)
  const handleTransition = useCallback(
    (workItemId: string, toStatus: WorkItemStatus) => {
      // 1) 走 store 的状态机 (INV-PM-01~05 校验由 store 后续实装承担)
      transitionWorkItem(workItemId, toStatus);

      // 2) 同步更新 board.columns, 移动 work_item_ids (last-write-wins, 拖动 = 一次 transition)
      useStore.setState((s) => {
        const fromCol = s.board.columns.find((c) => c.work_item_ids.includes(workItemId));
        const toCol = s.board.columns.find((c) => c.status === toStatus);
        if (!fromCol || !toCol) return s;
        if (fromCol.status === toCol.status) return s;
        return {
          board: {
            ...s.board,
            columns: s.board.columns.map((c) => {
              if (c.status === fromCol.status) {
                return { ...c, work_item_ids: c.work_item_ids.filter((id) => id !== workItemId) };
              }
              if (c.status === toCol.status) {
                return { ...c, work_item_ids: [...c.work_item_ids, workItemId] };
              }
              return c;
            }),
          },
        };
      });

      // 3) audit log (per §3.3 数据流) — stub, 1s 后写 console + localStorage event
      // 真实后端 D.6+ 接入, 这里保留 stub 给 E2E 验证
      setTimeout(() => {
        const auditEntry = {
          ts: new Date().toISOString(),
          action: "work_item.transition",
          work_item_id: workItemId,
          to_status: toStatus,
          actor: "usr-001", // 当前 user (Ulysses), 真实后端从 session 拿
        };
        // eslint-disable-next-line no-console
        console.log("[audit:stub]", JSON.stringify(auditEntry));
        try {
          const prev = JSON.parse(localStorage.getItem("star-audit-stub") || "[]");
          prev.push(auditEntry);
          localStorage.setItem("star-audit-stub", JSON.stringify(prev.slice(-50))); // 保留最近 50 条
        } catch {
          // localStorage 不可用 (SSR / 隐私模式) 静默
        }
      }, 1000);
    },
    [transitionWorkItem],
  );

  // E2E test hook: 暴露 transition 用于自动化
  useEffect(() => {
    if (typeof window !== "undefined") {
      (window as unknown as { __kanbanTransition?: typeof handleTransition }).__kanbanTransition = handleTransition;
    }
  }, [handleTransition]);

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Board"
        subtitle="Kanban 视图 + 拖动 transition + 多人协同 (2s polling)。每列定义 status + 可选 wip_limit;超限时高亮。"
        icon={<Trello className="text-accent" size={20} />}
        track="E"
        count={`${board.columns.reduce((s, c) => s + c.work_item_ids.length, 0)} cards`}
      />

      <KanbanFilters
        value={filter}
        onChange={setFilter}
        assignees={identities}
        total={allIds.length}
        shown={filteredCount}
      />

      <KanbanBoard
        board={board}
        workItems={workItems}
        identities={identities}
        onTransition={handleTransition}
        filter={filterFn}
      />

      {/* 本地 sync toast (per 已知缺口 #3: W5 接管 Toaster 前, 自实现) */}
      {toastMsg && (
        <div
          data-testid="kanban-sync-toast"
          className="fixed bottom-6 right-6 z-50 max-w-md card border-info/40 bg-info/10 flex items-start gap-2 shadow-lg"
          role="status"
        >
          <RefreshCw size={14} className="text-info shrink-0 mt-0.5" />
          <div className="text-xs flex-1">{toastMsg}</div>
          <button
            onClick={() => setToastMsg(null)}
            className="text-ink-mute hover:text-ink"
            aria-label="dismiss"
          >
            <X size={12} />
          </button>
        </div>
      )}

      {/* 状态机小贴士 (帮助用户理解列 = 状态) */}
      <div className="mt-4 text-[10px] text-ink-mute font-mono">
        列对应状态: {KANBAN_COLUMNS.join(" / ")} — 拖动卡片到不同列触发 transitionWorkItem (走状态机 INV-PM-01~05)
      </div>
    </div>
  );
}
