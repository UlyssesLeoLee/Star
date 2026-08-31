// =====================================================================
// 全局 in-memory store + 操作 API + persist
// =====================================================================
// 增强 (per docs/frontend/design/dynamic-interaction-design.md §8.3 / §9):
//   1. zustand/middleware persist 包装,localStorage 键 "star-store:v1"
//   2. SSR-safe storage (server 端 no-op)
//   3. partialize 排除 canvasElements (大字段,每次拖动会重写)
//   4. 新增 action: applyRemoteChange / transitionMilestone / transitionSprint
//   5. 保持向后兼容: useStore 导出形态不变,所有 useStore((s) => s.xxx) 零改动
// =====================================================================
"use client";

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import * as seed from "./seed";
import type {
  Worktree, WorktreeStatus,
  AgentSession, AgentStatus,
  Feedback, FeedbackStatus,
  PullRequest, PullRequestStatus,
  WorkItem, WorkItemStatus,
  ChangeSet, ChangeSetStatus,
  Notification, NotificationStatus,
  Canvas, CanvasElement, CanvasConnector,
  Board,
} from "@/types/ids";
import { TODO_FALLBACK_STATUS, isFallbackStatus } from "@/components/board/constants";

// =====================================================================
// Board reconcile 工具 — 让 board.columns[].work_item_ids 始终是
//   workItems[].status 的派生视图 (per 2026-08-31 11:24 JST Ulysses 拍板)
// =====================================================================
// 规则:
//   1. 对每个 status, board.columns 里该 status 列的 work_item_ids 包含
//      且仅包含 workItems 里 status 等于它的 wi.id
//   2. 保留列在 board.columns 里的顺序 (不重排)
//   3. 不存在的 status (workItems 有但 board.columns 没有) — 不自动加列;
//      由 KanbanBoard 端"列 = 状态映射"的渲染决定是否显示游离 wi
//      (这里只保证 *已存在列* 的 work_item_ids 跟 workItems.status 一致)
//   4. 兜底列缺失时也不自动补 — 因为兜底列在 addBoardColumn/seed 阶段保证存在
// =====================================================================
const reconcileBoard = (
  workItems: WorkItem[],
  columns: Board["columns"],
): Board["columns"] =>
  columns.map((col) => {
    // 该列 status 期望的 wi 集合 (按 workItems 顺序稳定输出, 避免抖动)
    const expectedIds = workItems
      .filter((w) => w.status === col.status)
      .map((w) => w.id);
    // 现状: 列里有的 id, 但 workItems 已不匹配 status (漂移) — 移除
    const cleaned = col.work_item_ids.filter((id) => {
      const w = workItems.find((x) => x.id === id);
      return w?.status === col.status;
    });
    // 缺的: workItems 期望但列里没有 — 补回
    const haveSet = new Set(cleaned);
    for (const id of expectedIds) {
      if (!haveSet.has(id)) {
        cleaned.push(id);
        haveSet.add(id);
      }
    }
    return { ...col, work_item_ids: cleaned };
  });

// =====================================================================
// SSR-safe localStorage 包装
// =====================================================================
// Next.js App Router 在 server 端会先 import 这个模块生成静态 HTML。
// 期间 localStorage 不存在,需 no-op。客户端 hydration 后自动用真实 localStorage。
// =====================================================================
const ssrSafeStorage = createJSONStorage(() => ({
  getItem: (name: string): string | null => {
    if (typeof window === "undefined") return null;
    try {
      return window.localStorage.getItem(name);
    } catch {
      return null;
    }
  },
  setItem: (name: string, value: string): void => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem(name, value);
    } catch {
      // quota exceeded or private mode — silent
    }
  },
  removeItem: (name: string): void => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.removeItem(name);
    } catch {
      // silent
    }
  },
}));

// =====================================================================
// Store 状态 + 变更 action
// =====================================================================
interface StoreState {
  // read accessors (immutable from outside)
  tenants: typeof seed.tenants;
  projects: typeof seed.projects;
  identities: typeof seed.identities;
  workspaces: typeof seed.workspaces;
  workItems: WorkItem[];
  comments: typeof seed.comments;
  permissionSchemes: typeof seed.permissionSchemes;
  permissionRules: typeof seed.permissionRules;
  workflows: typeof seed.workflows;
  changeSets: ChangeSet[];
  worktrees: Worktree[];
  agentSessions: AgentSession[];
  feedbacks: Feedback[];
  contextPackets: typeof seed.contextPackets;
  contextDecisions: typeof seed.contextDecisions;
  validationCases: typeof seed.validationCases;
  localRuntimes: typeof seed.localRuntimes;
  repositories: typeof seed.repositories;
  pullRequests: PullRequest[];
  notifications: Notification[];
  searchHits: typeof seed.searchHits;
  savedSearches: typeof seed.savedSearches;
  integrations: typeof seed.integrations;
  presenceCursors: typeof seed.presenceCursors;
  whiteboards: typeof seed.whiteboards;
  canvases: Canvas[];
  canvasElements: CanvasElement[];   // 故意大字段 — 不持久化
  canvasConnectors: CanvasConnector[];
  sprints: typeof seed.sprints;
  milestones: typeof seed.milestones;
  burndownSeries: typeof seed.burndownSeries;
  board: typeof seed.board;
  relations: typeof seed.relations;
  auditEvents: typeof seed.auditEvents;
  automationRules: typeof seed.automationRules;

  // 5 状态机迁移 (保留 B.2.5 已实装的 6 个)
  transitionWorktree: (id: string, to: WorktreeStatus) => void;
  transitionAgent:    (id: string, to: AgentStatus) => void;
  transitionFeedback: (id: string, to: FeedbackStatus) => void;
  transitionPR:       (id: string, to: PullRequestStatus) => void;
  transitionWorkItem: (id: string, to: WorkItemStatus) => void;
  transitionChangeSet:(id: string, to: ChangeSetStatus) => void;
  markNotificationRead: (id: string) => void;

  // 跨模块联动 (per §7.2 / §10) — W5 新增
  transitionMilestone: (id: string, newDueDate: string) => void;  // ISO8601
  transitionSprint:    (id: string, newStart: string, newEnd: string) => void;

  // Board 列编辑 (per 2026-08-29 18:52 JST 拍板: 列可改 + 增加减少; 2026-08-31 11:24 JST
  // 强化: workItems.status 为主源, board.columns[].work_item_ids 派生; todo 兜底不可删)
  addBoardColumn:    (status: WorkItemStatus) => void;     // 在末尾追加新列, 并回填 workItems.status 匹配的 wi
  removeBoardColumn: (status: WorkItemStatus) => void;     // 删列; 兜底列拒绝; 列里 wi 全部归 todo (兜底)
  renameBoardColumn: (status: WorkItemStatus, newName: string) => void;  // 改 name
  reorderBoardColumns: (fromIdx: number, toIdx: number) => void;  // 拖动列重排 (后续)

  // 多人协同 (per §8.3) — W5 新增
  // 接受 boardSync 推送的 partial snapshot,执行 last-write-wins 覆盖
  applyRemoteChange: (snapshot: Partial<Pick<StoreState,
    "board" | "workItems" | "milestones" | "sprints" | "notifications"
  >>) => void;

  // Canvas mutations(无限画布,frontend-canvas-design.md §2)
  addCanvasElement: (element: CanvasElement) => void;
  moveCanvasElement: (id: string, x: number, y: number) => void;
  deleteCanvasElement: (id: string) => void;
  addCanvasConnector: (connector: CanvasConnector) => void;
  setCanvasViewport: (canvasId: string, x: number, y: number, zoom: number) => void;
}

// =====================================================================
// 初始 state factory — 每次 store create 都重置成 seed
// (per zustand persist 模式: 把 create((set) => state) 抽出来)
// =====================================================================
const initialState = (set: any): StoreState => ({
  tenants: seed.tenants,
  projects: seed.projects,
  identities: seed.identities,
  workspaces: seed.workspaces,
  workItems: seed.workItems,
  comments: seed.comments,
  permissionSchemes: seed.permissionSchemes,
  permissionRules: seed.permissionRules,
  workflows: seed.workflows,
  changeSets: seed.changeSets,
  worktrees: seed.worktrees,
  agentSessions: seed.agentSessions,
  feedbacks: seed.feedbacks,
  contextPackets: seed.contextPackets,
  contextDecisions: seed.contextDecisions,
  validationCases: seed.validationCases,
  localRuntimes: seed.localRuntimes,
  repositories: seed.repositories,
  pullRequests: seed.pullRequests,
  notifications: seed.notifications,
  searchHits: seed.searchHits,
  savedSearches: seed.savedSearches,
  integrations: seed.integrations,
  presenceCursors: seed.presenceCursors,
  whiteboards: seed.whiteboards,
  canvases: seed.canvases,
  canvasElements: seed.canvasElements,
  canvasConnectors: seed.canvasConnectors,
  sprints: seed.sprints,
  milestones: seed.milestones,
  burndownSeries: seed.burndownSeries,
  board: seed.board,
  relations: seed.relations,
  auditEvents: seed.auditEvents,
  automationRules: seed.automationRules,

  // 6 状态机 (B.2.5 已有)
  transitionWorktree: (id, to) =>
    set((s: StoreState) => ({
      worktrees: s.worktrees.map((w) => w.id === id ? { ...w, status: to, last_event_at: new Date().toISOString(), lock_version: w.lock_version + 1 } : w),
    })),
  transitionAgent: (id, to) =>
    set((s: StoreState) => ({
      agentSessions: s.agentSessions.map((a) => a.id === id ? { ...a, status: to, ended_at: ["completed","failed","cancelled"].includes(to) ? new Date().toISOString() : a.ended_at } : a),
    })),
  transitionFeedback: (id, to) =>
    set((s: StoreState) => ({
      feedbacks: s.feedbacks.map((f) => f.id === id ? { ...f, status: to, answered_at: to === "resolved" || to === "wontfix" ? new Date().toISOString() : f.answered_at } : f),
    })),
  transitionPR: (id, to) =>
    set((s: StoreState) => ({
      pullRequests: s.pullRequests.map((p) => p.id === id ? { ...p, status: to, merged_at: to === "merged" ? new Date().toISOString() : p.merged_at } : p),
    })),
  transitionWorkItem: (id, to) =>
    set((s: StoreState) => {
      const newWorkItems = s.workItems.map((w) =>
        w.id === id ? { ...w, status: to, updated_at: new Date().toISOString() } : w
      );
      // per 2026-08-31 11:24 JST Ulysses 拍板: workItems.status 是主源,
      // 改 status 后 board.columns[].work_item_ids 必须 reconcile 跟随,
      // 否则 board 视图会跟 workItems 漂移 (drop 到不存在的列会丢卡)
      return {
        workItems: newWorkItems,
        board: { ...s.board, columns: reconcileBoard(newWorkItems, s.board.columns) },
      };
    }),
  transitionChangeSet: (id, to) =>
    set((s: StoreState) => ({
      changeSets: s.changeSets.map((c) => c.id === id ? { ...c, status: to } : c),
    })),
  markNotificationRead: (id) =>
    set((s: StoreState) => ({
      notifications: s.notifications.map((n) => n.id === id ? { ...n, status: "read" as NotificationStatus } : n),
    })),

  // W5 新增 — 跨模块联动
  transitionMilestone: (id, newDueDate) =>
    set((s: StoreState) => ({
      milestones: s.milestones.map((m) => m.id === id ? { ...m, due_date: newDueDate } : m),
    })),
  transitionSprint: (id, newStart, newEnd) =>
    set((s: StoreState) => ({
      sprints: s.sprints.map((sp) => sp.id === id ? { ...sp, start_date: newStart, end_date: newEnd } : sp),
    })),

  // Board 列编辑 (per 2026-08-29 18:52 JST 拍板; 2026-08-31 11:24 JST 强化)
  addBoardColumn: (status) =>
    set((s: StoreState) => {
      // 防重: 已存在该 status 跳过
      if (s.board.columns.some((c) => c.status === status)) return s;
      // 回填: 把 workItems 里 status 匹配的 wi 拉进新列 (per 11:24 拍板: 双向回填)
      const newCol: Board["columns"][number] = {
        status,
        work_item_ids: s.workItems.filter((w) => w.status === status).map((w) => w.id),
      };
      return {
        board: {
          ...s.board,
          columns: [...s.board.columns, newCol],
        },
      };
    }),
  removeBoardColumn: (status) =>
    set((s: StoreState) => {
      const col = s.board.columns.find((c) => c.status === status);
      if (!col) return s;
      // 兜底列不可删 (per 11:24 拍板: todo 列是数据兜底, 不允许被删)
      if (isFallbackStatus(status)) {
        // eslint-disable-next-line no-console
        console.warn(`[store] removeBoardColumn: refused to delete fallback column "${status}"`);
        return s;
      }
      // 删非兜底列: 列里 wi 状态全部改回 todo (兜底), 让 reconcile 把它放回 todo 列
      const idsInCol = new Set(col.work_item_ids);
      const newWorkItems = s.workItems.map((w) =>
        idsInCol.has(w.id) ? { ...w, status: TODO_FALLBACK_STATUS } : w
      );
      // 删列后 reconcile 一次, 确保 todo 列的 work_item_ids 包含刚改回 todo 的 wi
      const newColumns = reconcileBoard(
        newWorkItems,
        s.board.columns.filter((c) => c.status !== status),
      );
      return {
        board: { ...s.board, columns: newColumns },
        workItems: newWorkItems,
      };
    }),
  renameBoardColumn: (status, newName) =>
    set((s: StoreState) => ({
      board: {
        ...s.board,
        columns: s.board.columns.map((c) =>
          c.status === status ? { ...c, name: newName } : c
        ),
      },
    })),
  reorderBoardColumns: (fromIdx, toIdx) =>
    set((s: StoreState) => {
      const cols = [...s.board.columns];
      if (fromIdx < 0 || fromIdx >= cols.length || toIdx < 0 || toIdx >= cols.length) return s;
      const [moved] = cols.splice(fromIdx, 1);
      cols.splice(toIdx, 0, moved);
      return { board: { ...s.board, columns: cols } };
    }),

  // W5 新增 — 多人协同: last-write-wins 覆盖本地
  applyRemoteChange: (snapshot) =>
    set((s: StoreState) => ({ ...s, ...snapshot })),

  // Canvas mutations
  addCanvasElement: (element) =>
    set((s: StoreState) => ({ canvasElements: [...s.canvasElements, element] })),
  moveCanvasElement: (id, x, y) =>
    set((s: StoreState) => ({
      canvasElements: s.canvasElements.map((e) => e.id === id ? { ...e, x, y, updated_at: new Date().toISOString() } : e),
    })),
  deleteCanvasElement: (id) =>
    set((s: StoreState) => ({
      canvasElements: s.canvasElements.filter((e) => e.id !== id),
      canvasConnectors: s.canvasConnectors.filter((c) => c.from_element_id !== id && c.to_element_id !== id),
    })),
  addCanvasConnector: (connector) =>
    set((s: StoreState) => ({ canvasConnectors: [...s.canvasConnectors, connector] })),
  setCanvasViewport: (canvasId, x, y, zoom) =>
    set((s: StoreState) => ({
      canvases: s.canvases.map((c) => c.id === canvasId ? { ...c, viewport: { x, y, zoom } } : c),
    })),
});

// =====================================================================
// 创建带 persist 包装的 store
// =====================================================================
export const useStore = create<StoreState>()(
  persist(
    (set) => initialState(set),
    {
      name: "star-store:v1",
      storage: ssrSafeStorage,
      // 大字段 canvasElements 每次拖动都改,持久化会引发频繁 localStorage write
      partialize: (state) => {
        const { canvasElements, ...rest } = state;
        return rest as StoreState;
      },
      // 持久化版本号 — 升 schema 时改 version + 加 migrate
      version: 1,
      // hydrate 完成后跑一次 board reconcile (per 2026-08-31 11:24 JST 拍板):
      //   老 localStorage 数据 / seed 跟 workItems.status 错位 / 11:24 前删除列的脏数据
      //   都能在启动时一次性修复, 避免每个 action 各自去 catch 漂移
      onRehydrateStorage: () => (state) => {
        if (!state) return;
        const reconciledCols = reconcileBoard(state.workItems, state.board.columns);
        // 仅在确实漂移时才 set, 避免无谓的 persist write
        const drifted = reconciledCols.some((c, i) => {
          const orig = state.board.columns[i];
          if (!orig) return true;
          if (c.status !== orig.status) return true;
          if (c.work_item_ids.length !== orig.work_item_ids.length) return true;
          for (let k = 0; k < c.work_item_ids.length; k++) {
            if (c.work_item_ids[k] !== orig.work_item_ids[k]) return true;
          }
          return false;
        });
        if (drifted) {
          useStore.setState({ board: { ...state.board, columns: reconciledCols } });
        }
      },
    }
  )
);

// 显式暴露 persist 工具 (供 Provider / 测试用)
export const persistApi = typeof window !== "undefined" ? useStore.persist : null;
