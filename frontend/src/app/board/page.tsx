/*
@cypher schema=1 source_sha256=976cf72fc9c26344fec78df192851e91bed4497d8b621092c04923d92893f9a5
MERGE (self:File {path:"frontend/src/app/board/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/board/page.tsx::BoardPage",kind:"component"})
MERGE (groupProjection:File {path:"frontend/src/lib/group/groupProjection.tsx"})
MERGE (groupApiHook:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupApi",kind:"function"})
MERGE (sessionStateHook:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupSessionState",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(groupProjection)
MERGE (page)-[:CALLS]->(groupApiHook)
MERGE (page)-[:CALLS]->(sessionStateHook)
@endcypher
*/
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
import { SquareKanban as KanbanIcon, RefreshCw, X } from "lucide-react";
import type { WorkItem, WorkItemStatus } from "@/types/ids";
import { useTranslation } from "@/lib/i18n";
import { useWorktreeGroupApi, useWorktreeGroupSessionState } from "@/lib/group/groupProjection";
import type { KanbanUnavailableState } from "@/components/board/KanbanBoard";

// 状态机下一步: todo→in_progress, in_progress→review, review→done; blocked/wontfix/done 终止
const NEXT_STATUS: Partial<Record<WorkItemStatus, WorkItemStatus>> = {
  todo: "in_progress",
  in_progress: "review",
  review: "done",
};

export default function BoardPage() {
  const { t } = useTranslation();
  const board = useStore((s) => s.board);
  const workItems = useStore((s) => s.workItems);
  const identities = useStore((s) => s.identities);
  const transitionWorkItem = useStore((s) => s.transitionWorkItem);

  // ---- 宿主数据通道信号 (per 2026-11-16 清 seed/mock 显示入口) ----
  // Group API 会话存在 = 宿主已接入; 网页预览模式为 null → 看板不渲染
  // 本地 seed 任务卡 (不当真任务), 改诚实未接入/未授权/空数据态;
  // 401 信号来自 useWorktreeGroupSessionState() (status === "invalidated"),
  // 判定顺序: unauthorized 先于 disconnected (401 时 provider 同时置空 client)。
  const groupApi = useWorktreeGroupApi();
  const groupSession = useWorktreeGroupSessionState();
  const unauthorized = groupSession.status === "invalidated";
  const hostConnected = groupApi !== null;

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
    // 未接入/未授权时不做假同步轮询 (per 2026-11-16: 本地 seed 不冒充他人会话改动)
    enabled: hostConnected && !unauthorized,
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

  // ---- 看板诚实态 (per 2026-11-16 清 seed/mock 显示入口):
  // 未授权 > 未接入 > 空数据 > 正常渲染; 本地 seed 任务卡不作为任务事实展示
  const unavailableState: KanbanUnavailableState | null = unauthorized
    ? "unauthorized"
    : !hostConnected
      ? "disconnected"
      : allIds.length === 0
        ? "empty"
        : null;

  // ---- 拖动 transition 处理 ----
  // per 2026-08-31 11:24 JST Ulysses 拍板: workItems.status 为主源;
  //   store.transitionWorkItem 内部已 reconcile board.columns[].work_item_ids,
  //   page 不再手工 setState 改 board (之前双写易漂移, 已被 store 接管)
  // (per W1 守门: page 只剩 audit log, store 是单一入口)
  const handleTransition = useCallback(
    (workItemId: string, toStatus: WorkItemStatus) => {
      // 1) 走 store 的状态机 (workItem.status + board reconcile 都在 store 完成)
      transitionWorkItem(workItemId, toStatus);

      // 2) audit log (per §3.3 数据流) — stub, 1s 后写 console + localStorage event
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

  // ---- 下一步状态列高亮 (per 2026-09-05 拍板, 场景 2: Kanban 下一步列)
  // 找优先级最高 (P0) + 状态非终止 的卡片的下一状态列; 找不到返 null.
  const nextStepStatus = useMemo<WorkItemStatus | null>(() => {
    const candidates = workItems
      .filter((w) => w.status !== "done" && w.status !== "wontfix" && w.status !== "blocked")
      .sort((a, b) => {
        const order = { p0: 0, p1: 1, p2: 2, p3: 3 };
        return order[a.priority] - order[b.priority];
      });
    if (candidates.length === 0) return null;
    const top = candidates[0];
    return NEXT_STATUS[top.status] ?? null;
  }, [workItems]);

  // E2E test hook: 暴露 transition 用于自动化
  useEffect(() => {
    if (typeof window !== "undefined") {
      (window as unknown as { __kanbanTransition?: typeof handleTransition }).__kanbanTransition = handleTransition;
    }
  }, [handleTransition]);

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/board'].title}
        subtitle="Kanban 视图 + 拖动 transition + 多人协同 (2s polling)。每列定义 status + 可选 wip_limit;超限时高亮。"
        icon={<KanbanIcon className="text-accent" size={20} />}
        track="E"
        count={
          unauthorized
            ? "未授权"
            : !hostConnected
              ? "预览模式 (seed)"
              : `${board.columns.reduce((s, c) => s + c.work_item_ids.length, 0)} cards`
        }
      />

      {/* 数据来源标识 (per 2026-11-16: 预览/seed 数据不得冒充正式管理视图, 诚实说明) */}
      <div
        data-testid="board-data-source"
        className="mb-3 flex items-center gap-2 rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[11px] text-warn"
      >
        <span className="size-1.5 rounded-full bg-warn" />
        预览数据模式 — 当前看板展示本地 seed 数据, 非授权服务端任务事实。宿主会话接入后自动切换为授权投影。
      </div>

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
        nextStepStatus={nextStepStatus}
        unavailableState={unavailableState}
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
