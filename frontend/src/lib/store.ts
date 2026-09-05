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
  RefactorRound, RefactorCard, RefactorColumn, RefactorBoardConfig, RefactorStatus,
  Uuid,
} from "@/types/ids";
import type { AgentGameState, PerkId } from "@/lib/agent-game/types";
import {
  createInitialGameState,
  computeClaim,
  applyClaim,
  applyPerkChoice,
  applyCostSpend,
  applyDeath,
  applyRevive,
  applyRestart,
} from "@/lib/agent-game/leveling";
import type { GameMap, MapCell } from "@/lib/agent-game/mapgen";
import { generateMap } from "@/lib/agent-game/mapgen";
import { moveAgent, type MoveResult } from "@/lib/agent-game/movement";

/** claimReward 戻り値 (per 拍板) */
export type ClaimResult =
  | { ok: true; xp: number; coins: number; leveledUp: boolean; levelsGained: number; isMaxLevel: boolean }
  | { ok: false; reason: "work_item_not_found" | "already_claimed" | "not_done" | "agent_dead" };

/** spendCost 戻り値 (per 拍板) */
export type SpendResult =
  | { ok: true; died: boolean; hpAfter: number; triggerCostRatio: number }
  | { ok: false; reason: "agent_not_found" };

/** reviveAgent 戻り値 */
export type ReviveResult = { ok: true } | { ok: false; reason: string };
import { TODO_FALLBACK_STATUS, isFallbackStatus } from "@/components/board/constants";
import {
  REFACTOR_DEFAULT_BATCH_SIZE,
  isRefactorFallbackStatus,
  makeDefaultRefactorColumns,
  sortRefactorColumns,
} from "./board-refactor-constants";
import { REFACTOR_FALLBACK_STATUS } from "@/types/ids";

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

  // ── Refactor Sweep (per 2026-09-02 10:41 JST 拍板, docs/frontend/design/refactor-sweep-design.md) ──
  /** 全局所有 refactor rounds (active + 历史) */
  refactorRounds: RefactorRound[];
  /** per-project 重构看板配置 (列定义 + batch_size) */
  refactorBoardConfigs: Record<Uuid, RefactorBoardConfig>;

  // ── Agent Game (per 2026-09-05 11:42 JST 拍板, 拟人化游戏化) ──
  /** per-agent 游戏化状态 (level / xp / coins / hp / perks) */
  agentGameStates: Record<Uuid, AgentGameState>;
  /** per-agent Roguelike map (per 2026-09-05 12:23 JST 拍板, ask_8a60a3bc...) */
  agentMaps: Record<Uuid, GameMap>;
  /** per-agent 当前在 map 上的位置 (per 拍板 #3, 4-邻接移动) */
  agentPositions: Record<Uuid, { x: number; y: number }>;

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

  // 新增 work-item (per 2026-08-31 11:56 JST Ulysses 拍板: Kanban 列内 + Add task 按钮)
  // 输入: 部分 WorkItem 字段 (id / key / created_at / updated_at 由 store 生成)
  //   - 必须带: tenant_id / project_id / title / status / reporter_id (board / project 上下文由调用方填)
  //   - 可选带: description / kind / priority / assignee_id / labels / story_points / worktree_id
  // 行为:
  //   1. push 到 workItems
  //   2. 把它加到 board.columns[status === status] 列的 work_item_ids 末尾
  //   3. status 不在列里 -> 自动 addBoardColumn 走兜底 (保证可见)
  addWorkItem: (input: Omit<WorkItem, "id" | "key" | "created_at" | "updated_at" | "description" | "labels" | "worktree_id"> & {
    /** 可选注入 client id (回填后端响应); 不传由 store 生成 */
    id?: string;
    /** 可选注入 key (回填后端响应); 不传由 store 生成 (PHYSIS-N) */
    key?: string;
    /** description 可选, 缺省空串 */
    description?: string;
    /** labels 可选, 缺省空数组 */
    labels?: string[];
    /** worktree 关联 (per 2026-08-31 12:07 JST Kanban Drawer 拍板) */
    worktree_id?: string;
  }) => string; // 返回新生成的 work-item id

  // 原地更新 workItem 单字段 (per 2026-08-31 12:07 JST Kanban Drawer 拍板)
  //   - 只改指定字段, 其他字段不变
  //   - 改 status 走 reconcileBoard 同步 board.columns
  //   - updated_at 自动刷
  //   - 字段白名单: title / description / status / priority / kind / assignee_id / labels / worktree_id
  updateWorkItemField: <K extends "title" | "description" | "status" | "priority" | "kind" | "assignee_id" | "labels" | "worktree_id">(
    id: string,
    field: K,
    value: WorkItem[K],
  ) => void;

  // 删除 work-item (per 2026-08-31 12:07 JST Kanban Drawer 拍板, Drawer 加删除按钮)
  removeWorkItem: (id: string) => void;

  // ── Refactor Sweep 9 个 action (per 2026-09-02 10:41 JST 拍板) ──
  // 跟 Kanban addBoardColumn/removeBoardColumn/renameBoardColumn/reorderBoardColumns 1:1 对齐
  // 命名沿用既有 verb+noun 风格 (add/remove/rename/reorder/move/...)
  // 行为约束:
  //   - 兜底 status ("todo") 不可删
  //   - 删非兜底列: 列里卡 refactor_status 归 fallback, 数据零丢失
  //   - close round 后只读, 不能改 cards
  //   - 列操作只动 refactorBoardConfigs, 不动 cards
  /** 给项目取/初始化 RefactorBoardConfig (没有则用默认 5 列 + batch_size=5) */
  ensureRefactorBoardConfig: (projectId: Uuid) => RefactorBoardConfig;
  /** 在末尾追加新列, status 不能跟现有重复 */
  addRefactorColumn: (projectId: Uuid, status: RefactorStatus, name?: string) => void;
  /** 删列; 兜底列拒绝; 列里卡全部归 fallback */
  removeRefactorColumn: (projectId: Uuid, status: RefactorStatus) => void;
  /** 改列名 (status 标识不变, 仅 name) */
  renameRefactorColumn: (projectId: Uuid, status: RefactorStatus, newName: string) => void;
  /** 拖动列重排 (fromIdx, toIdx) */
  reorderRefactorColumns: (projectId: Uuid, fromIdx: number, toIdx: number) => void;
  /** 重置为默认 5 列 + batch_size (用户主动 "重置" 按钮) */
  resetRefactorColumns: (projectId: Uuid) => void;
  /** 改 batch_size (UI 顶部设置) */
  setRefactorBatchSize: (projectId: Uuid, size: number) => void;
  /** 开启一轮新 round (默认 round_number = max+1, 卡初始化为 todo) */
  openRefactorRound: (projectId: Uuid, opts?: { notes?: string; includeWorkItemIds?: Uuid[] }) => Uuid;
  /** 关闭当前 round (closed_at = now, 只读) */
  closeRefactorRound: (roundId: Uuid) => void;
  /** 开启下一轮 (上一轮 done → round + 1, 全卡 reset todo, 上一轮 closed) */
  startNextRefactorRound: (projectId: Uuid) => Uuid | null;
  /** 移动单张卡到新 refactor_status (写 history, 跟 Kanban onTransition 行为一致) */
  moveRefactorCard: (roundId: Uuid, workItemId: Uuid, toStatus: RefactorStatus) => void;
  /** 加卡到当前 round (用于"添加任务"按钮) */
  addRefactorCard: (roundId: Uuid, workItemId: Uuid) => void;
  /** 从 round 移除卡 (用户撤回) */
  removeRefactorCard: (roundId: Uuid, workItemId: Uuid) => void;
  /**
   * Merge 单张卡 (per 2026-09-02 10:50 JST 拍板)
   *   - 校验: 仅 refactor_status === "done" 的卡可 merge
   *   - 已 merged (merged_at 存在) 直接幂等返回
   *   - 副作用:
   *       1. WorkItem.worktree_id 存在 -> 改 Worktree.status = "merged"
   *       2. Worktree.pr_id 存在 -> 改 PullRequest.status = "merged" + merged_at
   *       3. RefactorCard.merged_at / merged_worktree_id / merged_pr_id 写入
   *   - 历史 round 不允许 (closed round 的卡只读)
   */
  mergeRefactorCard: (roundId: Uuid, workItemId: Uuid) => "ok" | "not_found" | "not_done" | "already_merged" | "closed_round" | "worktree_terminal" | "pr_terminal";

  // Canvas mutations(无限画布,frontend-canvas-design.md §2)
  addCanvasElement: (element: CanvasElement) => void;
  moveCanvasElement: (id: string, x: number, y: number) => void;
  deleteCanvasElement: (id: string) => void;
  addCanvasConnector: (connector: CanvasConnector) => void;
  setCanvasViewport: (canvasId: string, x: number, y: number, zoom: number) => void;

  // Agent Game (per 2026-09-05 11:42 JST 拍板, 拟人化游戏化)
  /** 初始化/重置某 agent 的 game state (Lv 1, HP 满, perks 清零) */
  initAgentGame: (agentId: Uuid, budgetUsd: number) => void;
  /** 领取 work-item 完成奖励 (xp + coins), 内部触发升级和死亡检测 */
  claimReward: (agentId: Uuid, workItemId: Uuid) => ClaimResult;
  /** 选 perk (5 选 1, 升级时调) */
  choosePerk: (agentId: Uuid, perkId: PerkId) => void;
  /** 消耗 cost (HP 扣血, 可能触发死亡) */
  spendCost: (agentId: Uuid, costDelta: number) => SpendResult;
  /** 复活 (扣 50 金币, 重置 Lv 1) */
  reviveAgent: (agentId: Uuid) => { ok: boolean; reason?: string };
  /** 重开 (不扣币) */
  restartAgent: (agentId: Uuid) => void;
  /** Roguelike: 生成 map (per 拍板 #2) */
  generateAgentMap: (agentId: Uuid, width: number, height: number, seed: number) => GameMap;
  /** Roguelike: 移动到 (x, y), 内部 cost + cell 效果 + 死亡检测 */
  moveAgentOnMap: (agentId: Uuid, targetPos: { x: number; y: number }) => MoveResult;
  /** Roguelike: 重置 map + 位置 (新一局) */
  resetAgentMap: (agentId: Uuid) => void;
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

  // Refactor Sweep (per 2026-09-02 10:41 JST 拍板)
  //   - 初始空数组, 由 ensureRefactorBoardConfig / openRefactorRound 首次调用时 lazy init
  //   - per-project RefactorBoardConfig 同样 lazy init (第一次访问项目时)
  refactorRounds: [] as RefactorRound[],
  refactorBoardConfigs: {} as Record<Uuid, RefactorBoardConfig>,

  // Agent Game (per 2026-09-05 11:42 JST 拍板)
  //   - per-agent 状态, 首次 claimReward / initAgentGame 时 lazy init
  agentGameStates: {} as Record<Uuid, AgentGameState>,
  // Roguelike map (per 2026-09-05 12:23 JST 拍板)
  //   - per-agent, 首次 generateAgentMap 时 lazy init
  agentMaps: {} as Record<Uuid, GameMap>,
  agentPositions: {} as Record<Uuid, { x: number; y: number }>,

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
  // 新增 work-item (per 2026-08-31 11:56 JST Ulysses 拍板)
  // 上下文: 客户端 zustand store 还没接后端, key/id 由 store 本地生成.
  //   - id: crypto.randomUUID() 或基于 seed wi 计数 fallback (SSR-safe)
  //   - key: PHYSIS-N+ (按当前项目现有 max N+1 推; 不同项目共用 store 暂按 tenant 简化 PHYSIS)
  addWorkItem: (input) => {
    const newId = input.id
      ?? (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
        ? crypto.randomUUID()
        : `wi-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`);
    const nowIso = new Date().toISOString();
    // key 推断: 同 project / kind 默认 "PHYSIS-N" 格式; 已存在 +1
    let newKey = input.key;
    if (!newKey) {
      const sameProject = useStore.getState().workItems.filter(
        (w) => w.project_id === input.project_id,
      );
      const maxN = sameProject.reduce((acc, w) => {
        const m = /-(\d+)$/.exec(w.key);
        return m ? Math.max(acc, Number(m[1])) : acc;
      }, 0);
      newKey = `PHYSIS-${maxN + 1}`;
    }
    set((s: StoreState) => {
      const newWi: WorkItem = {
        id: newId,
        key: newKey!,
        tenant_id: input.tenant_id,
        project_id: input.project_id,
        title: input.title,
        description: input.description ?? "",
        kind: input.kind ?? "task",
        status: input.status,
        priority: input.priority ?? "p2",
        assignee_id: input.assignee_id,
        reporter_id: input.reporter_id,
        story_points: input.story_points,
        labels: input.labels ?? [],
        sprint_id: input.sprint_id,
        workflow_id: input.workflow_id ?? "wf-default",
        due_date: input.due_date,
        worktree_id: input.worktree_id,
        created_at: nowIso,
        updated_at: nowIso,
      };
      // status 列不在 board.columns -> 走 addBoardColumn 等价行为,
      // 兜底保底: 直接 reconcile 把它加到对应 status 列, 缺列就自动加
      const hasCol = s.board.columns.some((c) => c.status === newWi.status);
      const newColumns = hasCol
        ? s.board.columns.map((c) =>
            c.status === newWi.status
              ? { ...c, work_item_ids: [...c.work_item_ids, newWi.id] }
              : c,
          )
        : [
            ...s.board.columns,
            { status: newWi.status, work_item_ids: [newWi.id] },
          ];
      return {
        workItems: [...s.workItems, newWi],
        board: { ...s.board, columns: newColumns },
      };
    });
    return newId;
  },
  // 原地更新 workItem 单字段 (per 2026-08-31 12:07 JST Kanban Drawer 拍板)
  //   - title / description / priority / kind / assignee_id / labels / worktree_id: 直接 set
  //   - status: 走 reconcileBoard 同步 board.columns (跟 transitionWorkItem 一致)
  updateWorkItemField: (id, field, value) =>
    set((s: StoreState) => {
      const idx = s.workItems.findIndex((w) => w.id === id);
      if (idx === -1) return s;
      const nowIso = new Date().toISOString();
      const newWorkItems = [...s.workItems];
      newWorkItems[idx] = { ...newWorkItems[idx], [field]: value, updated_at: nowIso };
      if (field === "status") {
        // 走 reconcile 让 board.columns 跟 workItems.status 同步
        return {
          workItems: newWorkItems,
          board: { ...s.board, columns: reconcileBoard(newWorkItems, s.board.columns) },
        };
      }
      return { workItems: newWorkItems };
    }),
  // 删除 work-item (per 2026-08-31 12:07 JST Kanban Drawer 拍板, Drawer 加删除按钮)
  //   - 从 workItems 移除
  //   - 从 board.columns[].work_item_ids 全部列移除 (reconcileBoard 自然过滤)
  removeWorkItem: (id) =>
    set((s: StoreState) => {
      const newWorkItems = s.workItems.filter((w) => w.id !== id);
      if (newWorkItems.length === s.workItems.length) return s; // 不存在
      return {
        workItems: newWorkItems,
        board: { ...s.board, columns: reconcileBoard(newWorkItems, s.board.columns) },
      };
    }),
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

  // ===================================================================
  // Agent Game 7 个 action (per 2026-09-05 11:42 JST 拍板, 拟人化游戏化)
  //   - lazy init: 首次访问某 agent 时自动建 game state
  //   - claimReward 内部触发 applyClaim / applyDeath (cost 触发死亡)
  //   - choosePerk 调 applyPerkChoice 累积
  //   - 复活 / 重开 走 守门 #9 v9 派生规, 不静默 commit
  // ===================================================================

  initAgentGame: (agentId, budgetUsd) =>
    set((s: StoreState) => ({
      agentGameStates: {
        ...s.agentGameStates,
        [agentId]: createInitialGameState(agentId, budgetUsd),
      },
    })),

  claimReward: (agentId, workItemId) => {
    const s = useStore.getState();
    const gs = s.agentGameStates[agentId] ?? createInitialGameState(agentId, 0);
    const wi = s.workItems.find((w) => w.id === workItemId);
    if (!wi) return { ok: false, reason: "work_item_not_found" } as const;
    if (gs.lastClaimAt[workItemId]) return { ok: false, reason: "already_claimed" } as const;
    if (wi.status !== "done") return { ok: false, reason: "not_done" } as const;
    if (!gs.alive) return { ok: false, reason: "agent_dead" } as const;
    const reward = computeClaim(wi, gs.perks, Math.random());
    const r = applyClaim(gs, reward.xp, reward.coins, Math.random());
    const finalState = {
      ...r.state,
      lastClaimAt: { ...gs.lastClaimAt, [workItemId]: new Date().toISOString() },
    };
    useStore.setState({
      agentGameStates: { ...s.agentGameStates, [agentId]: finalState },
    });
    return {
      ok: true as const,
      xp: reward.xp,
      coins: reward.coins,
      leveledUp: r.leveledUp,
      levelsGained: r.levelsGained,
      isMaxLevel: r.isMaxLevel,
    };
  },

  choosePerk: (agentId, perkId) =>
    set((s: StoreState) => {
      const gs = s.agentGameStates[agentId];
      if (!gs || !gs.alive) return s;
      return {
        agentGameStates: {
          ...s.agentGameStates,
          [agentId]: applyPerkChoice(gs, perkId),
        },
      };
    }),

  spendCost: (agentId, costDelta) => {
    const s = useStore.getState();
    const gs = s.agentGameStates[agentId] ?? createInitialGameState(agentId, 0);
    const agent = s.agentSessions.find((a) => a.id === agentId);
    if (!agent) return { ok: false, died: false, reason: "agent_not_found" } as const;
    const r = applyCostSpend(gs, costDelta, agent.cost_summary.budget_usd);
    let finalState = r.state;
    if (r.died) {
      finalState = applyDeath(r.state);
    }
    useStore.setState({
      agentGameStates: { ...s.agentGameStates, [agentId]: finalState },
    });
    return {
      ok: true as const,
      died: r.died,
      hpAfter: finalState.hp,
      triggerCostRatio: r.triggerCostRatio,
    };
  },

  reviveAgent: (agentId) => {
    const s = useStore.getState();
    const gs = s.agentGameStates[agentId];
    if (!gs) return { ok: false, reason: "not_initialized" };
    const r = applyRevive(gs);
    if (!r.ok) return { ok: false, reason: r.reason ?? "unknown" };
    useStore.setState({
      agentGameStates: { ...s.agentGameStates, [agentId]: r.state },
    });
    return { ok: true };
  },

  restartAgent: (agentId) =>
    set((s: StoreState) => {
      const gs = s.agentGameStates[agentId];
      if (!gs) return s;
      return {
        agentGameStates: {
          ...s.agentGameStates,
          [agentId]: applyRestart(gs),
        },
      };
    }),

  // ===================================================================
  // Roguelike map 3 个 action (per 2026-09-05 12:23 JST 拍板)
  //   - generateAgentMap: 程序生成 map, 设置 position = start
  //   - moveAgentOnMap: 4-邻接 step, 触发 cost + cell 效果
  //   - resetAgentMap: 重开一局 (新 map + 重置 position)
  // ===================================================================

  generateAgentMap: (agentId, width, height, seed) => {
    const s = useStore.getState();
    const wis = s.workItems.filter((w) => w.status !== "done");
    const map = generateMap({ width, height, seed, workItems: wis });
    useStore.setState({
      agentMaps: { ...s.agentMaps, [agentId]: map },
      agentPositions: { ...s.agentPositions, [agentId]: map.startPos },
    });
    return map;
  },

  moveAgentOnMap: (agentId, targetPos) => {
    const s = useStore.getState();
    const map = s.agentMaps[agentId] ?? null;
    const pos = s.agentPositions[agentId] ?? null;
    const gs = s.agentGameStates[agentId] ?? null;
    const agent = s.agentSessions.find((a) => a.id === agentId);
    if (!gs || !agent) {
      return { ok: false as const, reason: "no_map" as const };
    }
    const r = moveAgent(gs, map, pos, targetPos, agent.cost_summary.budget_usd);
    if (!r.ok) return r;
    // 应用 state 更新
    let newState = { ...gs, hp: r.hpAfter, coins: r.coinsAfter };
    if (r.died) {
      newState = applyDeath(newState);
    }
    useStore.setState({
      agentGameStates: { ...s.agentGameStates, [agentId]: newState },
      agentPositions: { ...s.agentPositions, [agentId]: targetPos },
    });
    return r;
  },

  resetAgentMap: (agentId) =>
    set((s: StoreState) => {
      const oldMap = s.agentMaps[agentId];
      if (!oldMap) return s;
      // 用同尺寸 + 不同 seed 重生 (Date.now 取 seed)
      const newMap = generateMap({
        width: oldMap.width,
        height: oldMap.height,
        seed: Date.now() % 0x7fffffff,
        workItems: s.workItems.filter((w) => w.status !== "done"),
      });
      return {
        agentMaps: { ...s.agentMaps, [agentId]: newMap },
        agentPositions: { ...s.agentPositions, [agentId]: newMap.startPos },
      };
    }),

  // ===================================================================
  // Refactor Sweep 9 个 action (per 2026-09-02 10:41 JST 拍板)
  //   跟 Kanban addBoardColumn/removeBoardColumn/renameBoardColumn/reorderBoardColumns 1:1 对齐
  //   兜底 status ("todo") 不可删, 删其他列时卡归 fallback, 数据零丢失
  // ===================================================================

  /** 给项目取/初始化 RefactorBoardConfig (没有则用默认 5 列 + batch_size=5) */
  ensureRefactorBoardConfig: (projectId) => {
    const existing = useStore.getState().refactorBoardConfigs[projectId];
    if (existing) return existing;
    const fresh: RefactorBoardConfig = {
      project_id: projectId,
      columns: makeDefaultRefactorColumns(),
      fallback_status: REFACTOR_FALLBACK_STATUS,
      batch_size: REFACTOR_DEFAULT_BATCH_SIZE,
      updated_at: new Date().toISOString(),
    };
    set((s: StoreState) => ({
      refactorBoardConfigs: { ...s.refactorBoardConfigs, [projectId]: fresh },
    }));
    return fresh;
  },

  /** 在末尾追加新列, status 不能跟现有重复 */
  addRefactorColumn: (projectId, status, name) =>
    set((s: StoreState) => {
      const cfg = s.refactorBoardConfigs[projectId] ?? {
        project_id: projectId,
        columns: makeDefaultRefactorColumns(),
        fallback_status: REFACTOR_FALLBACK_STATUS,
        batch_size: REFACTOR_DEFAULT_BATCH_SIZE,
        updated_at: new Date().toISOString(),
      };
      // 防重: 已存在该 status 跳过
      if (cfg.columns.some((c) => c.status === status)) return s;
      const newCol: RefactorColumn = {
        status,
        name,
        position: cfg.columns.length,
      };
      return {
        refactorBoardConfigs: {
          ...s.refactorBoardConfigs,
          [projectId]: {
            ...cfg,
            columns: sortRefactorColumns([...cfg.columns, newCol]),
            updated_at: new Date().toISOString(),
          },
        },
      };
    }),

  /** 删列; 兜底列拒绝; 列里卡全部归 fallback */
  removeRefactorColumn: (projectId, status) =>
    set((s: StoreState) => {
      const cfg = s.refactorBoardConfigs[projectId];
      if (!cfg) return s;
      if (isRefactorFallbackStatus(status)) {
        // eslint-disable-next-line no-console
        console.warn(`[store] removeRefactorColumn: refused to delete fallback status "${status}"`);
        return s;
      }
      if (!cfg.columns.some((c) => c.status === status)) return s;
      // 删列: 该 status 的卡 refactor_status 归 fallback, 历史 round 同样处理
      const fallback = REFACTOR_FALLBACK_STATUS;
      const newRounds = s.refactorRounds.map((r) => {
        if (r.project_id !== projectId) return r;
        if (r.closed_at) return r; // 已关闭 round 只读
        return {
          ...r,
          cards: r.cards.map((c) =>
            c.refactor_status === status
              ? {
                  ...c,
                  refactor_status: fallback,
                  moved_at: new Date().toISOString(),
                  history: [...c.history, { status: fallback, at: new Date().toISOString() }],
                }
              : c
          ),
        };
      });
      return {
        refactorBoardConfigs: {
          ...s.refactorBoardConfigs,
          [projectId]: {
            ...cfg,
            columns: sortRefactorColumns(cfg.columns.filter((c) => c.status !== status)),
            updated_at: new Date().toISOString(),
          },
        },
        refactorRounds: newRounds,
      };
    }),

  /** 改列名 (status 标识不变, 仅 name) */
  renameRefactorColumn: (projectId, status, newName) =>
    set((s: StoreState) => {
      const cfg = s.refactorBoardConfigs[projectId];
      if (!cfg) return s;
      return {
        refactorBoardConfigs: {
          ...s.refactorBoardConfigs,
          [projectId]: {
            ...cfg,
            columns: cfg.columns.map((c) =>
              c.status === status ? { ...c, name: newName.trim() || undefined } : c
            ),
            updated_at: new Date().toISOString(),
          },
        },
      };
    }),

  /** 拖动列重排 (fromIdx, toIdx) */
  reorderRefactorColumns: (projectId, fromIdx, toIdx) =>
    set((s: StoreState) => {
      const cfg = s.refactorBoardConfigs[projectId];
      if (!cfg) return s;
      const cols = sortRefactorColumns(cfg.columns);
      if (fromIdx < 0 || fromIdx >= cols.length || toIdx < 0 || toIdx >= cols.length) return s;
      const [moved] = cols.splice(fromIdx, 1);
      cols.splice(toIdx, 0, moved);
      return {
        refactorBoardConfigs: {
          ...s.refactorBoardConfigs,
          [projectId]: {
            ...cfg,
            columns: cols.map((c, i) => ({ ...c, position: i })),
            updated_at: new Date().toISOString(),
          },
        },
      };
    }),

  /** 重置为默认 5 列 + batch_size (用户主动 "重置" 按钮) */
  resetRefactorColumns: (projectId) =>
    set((s: StoreState) => {
      const cfg = s.refactorBoardConfigs[projectId];
      const baseColumns = makeDefaultRefactorColumns();
      // 重置时, 已存在 round 内卡 refactor_status 若引用了已删 status, 归 fallback
      const fallback = REFACTOR_FALLBACK_STATUS;
      const newStatuses = new Set(baseColumns.map((c) => c.status));
      const newRounds = s.refactorRounds.map((r) => {
        if (r.project_id !== projectId) return r;
        if (r.closed_at) return r;
        return {
          ...r,
          cards: r.cards.map((c) =>
            newStatuses.has(c.refactor_status)
              ? c
              : {
                  ...c,
                  refactor_status: fallback,
                  moved_at: new Date().toISOString(),
                  history: [...c.history, { status: fallback, at: new Date().toISOString() }],
                }
          ),
        };
      });
      return {
        refactorBoardConfigs: {
          ...s.refactorBoardConfigs,
          [projectId]: {
            project_id: projectId,
            columns: baseColumns,
            fallback_status: REFACTOR_FALLBACK_STATUS,
            batch_size: cfg?.batch_size ?? REFACTOR_DEFAULT_BATCH_SIZE,
            updated_at: new Date().toISOString(),
          },
        },
        refactorRounds: newRounds,
      };
    }),

  /** 改 batch_size (UI 顶部设置) */
  setRefactorBatchSize: (projectId, size) =>
    set((s: StoreState) => {
      const cfg = s.refactorBoardConfigs[projectId];
      const safeSize = Math.max(1, Math.min(50, Math.floor(size)));
      return {
        refactorBoardConfigs: {
          ...s.refactorBoardConfigs,
          [projectId]: {
            project_id: projectId,
            columns: cfg?.columns ?? makeDefaultRefactorColumns(),
            fallback_status: REFACTOR_FALLBACK_STATUS,
            batch_size: safeSize,
            updated_at: new Date().toISOString(),
          },
        },
      };
    }),

  /** 开启一轮新 round (默认 round_number = max+1, 卡初始化为 todo) */
  openRefactorRound: (projectId, opts) => {
    const now = new Date().toISOString();
    const tenantId = useStore.getState().tenants[0]?.id ?? "";
    const existingRounds = useStore.getState().refactorRounds.filter((r) => r.project_id === projectId);
    const maxRound = existingRounds.reduce((m, r) => Math.max(m, r.round_number), 0);
    const newRoundNumber = maxRound + 1;
    const newId = (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : `rr-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`);

    // 决定入选卡: 显式传 includeWorkItemIds 用之, 否则 = 当前 project 下 status=done 的 workItems
    let targetWIs: WorkItem[];
    if (opts?.includeWorkItemIds && opts.includeWorkItemIds.length > 0) {
      const idSet = new Set(opts.includeWorkItemIds);
      targetWIs = useStore.getState().workItems.filter((w) => idSet.has(w.id) && w.project_id === projectId);
    } else {
      targetWIs = useStore.getState().workItems.filter(
        (w) => w.project_id === projectId && w.status === "done"
      );
    }
    // 排除已在 active round 里的卡 (避免重复入 round)
    const activeRound = existingRounds.find((r) => !r.closed_at);
    const activeIds = new Set(activeRound?.cards.map((c) => c.work_item_id) ?? []);
    targetWIs = targetWIs.filter((w) => !activeIds.has(w.id));

    const cards: RefactorCard[] = targetWIs.map((w) => ({
      work_item_id: w.id,
      work_item_key: w.key,
      work_item_title: w.title,
      priority: w.priority,
      kind: w.kind,
      refactor_status: REFACTOR_FALLBACK_STATUS,
      entered_at: now,
      moved_at: now,
      history: [{ status: REFACTOR_FALLBACK_STATUS, at: now }],
      round_number: newRoundNumber,
    }));

    const newRound: RefactorRound = {
      id: newId,
      tenant_id: tenantId,
      project_id: projectId,
      round_number: newRoundNumber,
      notes: opts?.notes,
      started_at: now,
      cards,
    };

    set((s: StoreState) => ({ refactorRounds: [...s.refactorRounds, newRound] }));
    return newId;
  },

  /** 关闭当前 round (closed_at = now, 只读) */
  closeRefactorRound: (roundId) =>
    set((s: StoreState) => ({
      refactorRounds: s.refactorRounds.map((r) =>
        r.id === roundId && !r.closed_at
          ? { ...r, closed_at: new Date().toISOString() }
          : r
      ),
    })),

  /** 开启下一轮 (上一轮 active 全卡 done → round + 1, 全卡 reset todo, 上一轮 closed) */
  startNextRefactorRound: (projectId) => {
    const state = useStore.getState();
    const projectRounds = state.refactorRounds.filter((r) => r.project_id === projectId);
    const active = projectRounds.find((r) => !r.closed_at);
    if (!active) return null;
    // 校验: 所有卡都 done 才允许
    const allDone = active.cards.length === 0 || active.cards.every((c) => c.refactor_status === "done");
    if (!allDone) return null;
    // 关闭当前 round
    useStore.getState().closeRefactorRound(active.id);
    // 开新 round
    return useStore.getState().openRefactorRound(projectId);
  },

  /** 移动单张卡到新 refactor_status (写 history) */
  moveRefactorCard: (roundId, workItemId, toStatus) =>
    set((s: StoreState) => {
      const now = new Date().toISOString();
      return {
        refactorRounds: s.refactorRounds.map((r) => {
          if (r.id !== roundId || r.closed_at) return r;
          return {
            ...r,
            cards: r.cards.map((c) =>
              c.work_item_id === workItemId
                ? {
                    ...c,
                    refactor_status: toStatus,
                    moved_at: now,
                    history: [...c.history, { status: toStatus, at: now }],
                  }
                : c
            ),
          };
        }),
      };
    }),

  /** 加卡到当前 round (用于"添加任务"按钮) */
  addRefactorCard: (roundId, workItemId) =>
    set((s: StoreState) => {
      const round = s.refactorRounds.find((r) => r.id === roundId);
      if (!round || round.closed_at) return s;
      if (round.cards.some((c) => c.work_item_id === workItemId)) return s; // 防重
      const wi = s.workItems.find((w) => w.id === workItemId);
      if (!wi) return s;
      const now = new Date().toISOString();
      const newCard: RefactorCard = {
        work_item_id: wi.id,
        work_item_key: wi.key,
        work_item_title: wi.title,
        priority: wi.priority,
        kind: wi.kind,
        refactor_status: REFACTOR_FALLBACK_STATUS,
        entered_at: now,
        moved_at: now,
        history: [{ status: REFACTOR_FALLBACK_STATUS, at: now }],
        round_number: round.round_number,
      };
      return {
        refactorRounds: s.refactorRounds.map((r) =>
          r.id === roundId ? { ...r, cards: [...r.cards, newCard] } : r
        ),
      };
    }),

  /** 从 round 移除卡 (用户撤回) */
  removeRefactorCard: (roundId, workItemId) =>
    set((s: StoreState) => ({
      refactorRounds: s.refactorRounds.map((r) => {
        if (r.id !== roundId || r.closed_at) return r;
        return { ...r, cards: r.cards.filter((c) => c.work_item_id !== workItemId) };
      }),
    })),

  /**
   * Merge 单张卡 (per 2026-09-02 10:50 JST 拍板 + 10:56 JST 补缺口 #6 失败回滚)
   *   - 校验: 仅 refactor_status === "done" 的卡可 merge
   *   - 已 merged 幂等返回 "already_merged"
   *   - 副作用 (per 缺口 #6: 原子化, 全部或全不):
   *       1. WorkItem.worktree_id 存在 -> 改 Worktree.status = "merged"
   *          (校验 worktree 不在终态 merged/closed/abandoned/archived/reverted, 否则返回 "worktree_terminal")
   *       2. Worktree.pr_id 存在 -> 改 PullRequest.status = "merged" + merged_at
   *          (校验 pr 不在终态 merged/closed, 否则返回 "pr_terminal")
   *       3. RefactorCard.merged_at / merged_worktree_id / merged_pr_id 写入
   *   - 历史 round 不允许 (closed round 的卡只读)
   *   - 所有副作用在单个 set 块, zustand 保证原子性; 但 worktree/PR 终态校验在 set 前做,
   *     提前拒绝避免脏写
   *
   *   返回值: "ok" | "not_found" | "not_done" | "already_merged" | "closed_round"
   *           | "worktree_terminal" | "pr_terminal"
   */
  mergeRefactorCard: (roundId, workItemId) => {
    const state = useStore.getState();
    const round = state.refactorRounds.find((r) => r.id === roundId);
    if (!round) return "not_found";
    if (round.closed_at) return "closed_round";
    const card = round.cards.find((c) => c.work_item_id === workItemId);
    if (!card) return "not_found";
    if (card.refactor_status !== "done") return "not_done";
    if (card.merged_at) return "already_merged";

    // 终态校验 (per 缺口 #6: 提前拒绝, 避免脏写)
    const wi = state.workItems.find((w) => w.id === workItemId);
    const worktreeId = wi?.worktree_id;
    const worktree = worktreeId
      ? state.worktrees.find((wt) => wt.id === worktreeId)
      : undefined;
    const TERMINAL_WT: ReadonlyArray<WorktreeStatus> = ["merged", "closed", "abandoned", "archived", "reverted"];
    if (worktree && TERMINAL_WT.includes(worktree.status)) {
      return "worktree_terminal";
    }
    const prId = worktree?.pr_id;
    const pr = prId
      ? state.pullRequests.find((p) => p.id === prId)
      : undefined;
    const TERMINAL_PR: ReadonlyArray<PullRequestStatus> = ["merged", "closed"];
    if (pr && TERMINAL_PR.includes(pr.status)) {
      return "pr_terminal";
    }

    const now = new Date().toISOString();
    // 单 set 块原子写 (zustand 事务性, 任一字段错不会半写)
    set((s: StoreState) => {
      const newWorktrees = worktree
        ? s.worktrees.map((wt) =>
            wt.id === worktree.id
              ? { ...wt, status: "merged" as WorktreeStatus, last_event_at: now, lock_version: wt.lock_version + 1 }
              : wt
          )
        : s.worktrees;
      const newPRs = pr
        ? s.pullRequests.map((p) =>
            p.id === pr.id
              ? { ...p, status: "merged" as PullRequestStatus, merged_at: now }
              : p
          )
        : s.pullRequests;
      const newRounds = s.refactorRounds.map((r) => {
        if (r.id !== roundId) return r;
        return {
          ...r,
          cards: r.cards.map((c) =>
            c.work_item_id === workItemId
              ? {
                  ...c,
                  merged_at: now,
                  merged_worktree_id: worktree?.id,
                  merged_pr_id: pr?.id,
                }
              : c
          ),
        };
      });
      return {
        worktrees: newWorktrees,
        pullRequests: newPRs,
        refactorRounds: newRounds,
      };
    });
    return "ok";
  },
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
        // Refactor Sweep: 老 localStorage 数据可能没有 refactorRounds / refactorBoardConfigs,
        // 补默认 (lazy init, 仅当字段缺失时填)
        const cur = useStore.getState();
        if (!Array.isArray(cur.refactorRounds)) {
          useStore.setState({ refactorRounds: [] });
        }
        if (cur.refactorBoardConfigs == null || typeof cur.refactorBoardConfigs !== "object") {
          useStore.setState({ refactorBoardConfigs: {} });
        }
      },
    }
  )
);

// 显式暴露 persist 工具 (供 Provider / 测试用)
export const persistApi = typeof window !== "undefined" ? useStore.persist : null;
