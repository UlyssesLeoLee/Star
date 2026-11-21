/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts"})
MERGE (columns:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::UNIFIED_KANBAN_COLUMNS",kind:"constant"})
MERGE (columnStatus:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::UNIFIED_COLUMN_STATUS",kind:"constant"})
MERGE (columnLabels:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::UNIFIED_COLUMN_LABELS",kind:"constant"})
MERGE (itemType:Type {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::UnifiedWorkItem",kind:"interface"})
MERGE (conflictType:Type {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::UnifiedWorkItemConflictError",kind:"class"})
MERGE (listFn:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::listUnifiedWorkItems",kind:"function"})
MERGE (createFn:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::createUnifiedWorkItem",kind:"function"})
MERGE (moveFn:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::moveUnifiedWorkItem",kind:"function"})
MERGE (transitionFn:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::transitionUnifiedWorkItem",kind:"function"})
MERGE (self)-[:DEFINES]->(columns)
MERGE (self)-[:DEFINES]->(columnStatus)
MERGE (self)-[:DEFINES]->(columnLabels)
MERGE (self)-[:DEFINES]->(itemType)
MERGE (self)-[:DEFINES]->(conflictType)
MERGE (self)-[:DEFINES]->(listFn)
MERGE (self)-[:DEFINES]->(createFn)
MERGE (self)-[:DEFINES]->(moveFn)
MERGE (self)-[:DEFINES]->(transitionFn)
MERGE (listFn)-[:CALLS]->(:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.listWorkItems",kind:"method"})
MERGE (createFn)-[:CALLS]->(:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.createLegacyRunTaskCard",kind:"method"})
MERGE (moveFn)-[:CALLS]->(:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.transitionWorkItem",kind:"method"})
MERGE (transitionFn)-[:CALLS]->(:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.transitionWorkItem",kind:"method"})
@endcypher
*/
"use client";

// =====================================================================
// unifiedWorkItems — Backlog/Sprint/Kanban 三入口的统一服务端数据通道
// =====================================================================
// 只读复用共享模块 (worktreeGroupApi / runDirectoryApi / groupProjection),
// 不在共享文件上新增任何方法:
// - 读: WorktreeGroupApiClient.listWorkItems (worktree 范围, PostgreSQL 真源)
// - 建: RunDirectoryApiClient.createLegacyRunTaskCard (POST worktree work-items, 挂 branch/worktree)
// - 移/转: WorktreeGroupApiClient.transitionWorkItem (POST /lifecycle,
//   乐观锁 expected_version; 与服务端 /move 路由共用同一 RunTaskCardCommandPort 写路径)
// 列↔状态映射是服务端 group_api/work_items.rs `unified_column_status` 的镜像
// (读侧与 groupProjection.mapWorkItems 同表), 三入口共用同一张表。

import type { GroupWorkItemView } from "@/lib/group/groupProjection";
import { GroupApiError, WorktreeGroupApiClient } from "@/lib/group/worktreeGroupApi";
import {
  RunDirectoryApiClient,
  RunDirectoryError,
  type RunTaskCard,
  type RunTaskCardPriority,
  type RunTaskCardTargetStatus,
  type RunContextEnvelope,
} from "@/lib/run/runDirectoryApi";
import type { WorkItemKind, WorkItemPriority } from "@/types/ids";

/** groupProjection 同款 kind 归一 (未知 item_type 兜底为 task). */
const KNOWN_KINDS = ["story", "task", "bug", "spike", "epic"] as const;

function normalizeKind(itemType: string): WorkItemKind {
  return (KNOWN_KINDS as readonly string[]).includes(itemType)
    ? (itemType as WorkItemKind)
    : "task";
}

function normalizePriority(priority: string): WorkItemPriority {
  return priority === "urgent" ? "p0" : priority === "high" ? "p1" : priority === "low" ? "p3" : "p2";
}

/** 前端 p0-p3 → 服务端 low/medium/high/urgent (创建/写请求用). */
function normalizeRunPriority(priority: string): RunTaskCardPriority {
  return priority === "p0" ? "urgent" : priority === "p1" ? "high" : priority === "p3" ? "low" : "medium";
}

/** 服务端 low/medium/high/urgent → 前端 store 词汇 p0-p3 (store 只读缓存水合用). */
export function toStorePriority(priority: RunTaskCardPriority): WorkItemPriority {
  return normalizePriority(priority);
}

/** 统一列闭集 (镜像服务端 closed set; 服务端无 "review" 列语义 — 评审走 review 流程). */
export const UNIFIED_KANBAN_COLUMNS = [
  "todo",
  "in_progress",
  "blocked",
  "done",
  "wontfix",
] as const;
export type UnifiedKanbanColumn = (typeof UNIFIED_KANBAN_COLUMNS)[number];

/** 列 → 六态生命周期状态的写侧单一映射 (与服务端 unified_column_status 一致). */
export const UNIFIED_COLUMN_STATUS: Record<UnifiedKanbanColumn, RunTaskCardTargetStatus> = {
  todo: "pending",
  in_progress: "in_progress",
  blocked: "failed",
  done: "completed",
  wontfix: "cancelled",
};

/** 读侧映射: 生命周期状态 → 统一列 (与 groupProjection.mapWorkItems 同表).
 *  export: LANE Task Cards 面板的全局任务状态筛选复用同一张表, 不另造映射。 */
export const LIFECYCLE_COLUMN: Record<string, UnifiedKanbanColumn> = {
  pending: "todo",
  claimed: "in_progress",
  in_progress: "in_progress",
  completed: "done",
  failed: "blocked",
  cancelled: "wontfix",
};

/** 本地三语文案 (共享 i18n dictionary 不在本 lane manifest; 父代理合并时可迁入). */
export const UNIFIED_COLUMN_LABELS: Record<
  UnifiedKanbanColumn,
  { "zh-CN": string; en: string; ja: string }
> = {
  todo: { "zh-CN": "待办", en: "Todo", ja: "未着手" },
  in_progress: { "zh-CN": "进行中", en: "In Progress", ja: "進行中" },
  blocked: { "zh-CN": "阻塞", en: "Blocked", ja: "ブロック" },
  done: { "zh-CN": "完成", en: "Done", ja: "完了" },
  wontfix: { "zh-CN": "归档", en: "Won't Fix", ja: "却下" },
};

/** 三入口统一的任务卡视图 (ID/状态/版本 = 服务端真源, 不做本地二次推断). */
export interface UnifiedWorkItem {
  id: string;
  title: string;
  kind: string;
  description: string;
  priority: RunTaskCardPriority;
  labels: string[];
  column: UnifiedKanbanColumn;
  lifecycleStatus: RunTaskCardTargetStatus;
  lifecycleVersion: number;
  reviewState: string;
  activeWorktreeId: string | null;
  reporterId: string;
  worktreeId: string;
  // per 2026-11-21: branch 隔离 — 服务端真源 branch_id (UUID) + 分支名
  branchId: string | null;
  branchName: string | null;
}

/** 服务端写冲突/越权的明确反馈载体 (409/403/4xx 由服务端稳定错误码驱动). */
export class UnifiedWorkItemConflictError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = "UnifiedWorkItemConflictError";
  }

  /** 乐观锁版本冲突 (409 version_conflict 等). */
  isConflict(): boolean {
    return this.status === 409;
  }

  /** 越权 (403 insufficient_scope / forbidden). */
  isForbidden(): boolean {
    return this.status === 403;
  }
}

function newCorrelationId(): string {
  return typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : `corr-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

function columnFor(status: string): UnifiedKanbanColumn {
  const column = LIFECYCLE_COLUMN[status];
  if (!column) {
    throw new Error(`Group API returned an unsupported WorkItem lifecycle status: ${status}`);
  }
  return column;
}

/** 把 GroupWorkItemView 投影归一为统一视图 (读侧, 数据来自服务端 list). */
export function normalizeGroupWorkItems(
  items: GroupWorkItemView[],
  worktreeId: string,
): UnifiedWorkItem[] {
  return items.map((item) => ({
    id: item.id,
    title: item.title,
    kind: item.kind,
    description: item.description,
    priority: normalizeRunPriority(item.priority),
    labels: item.labels,
    column: columnFor(item.lifecycle_status),
    lifecycleStatus: item.lifecycle_status,
    lifecycleVersion: item.lifecycle_version,
    reviewState: item.review_state,
    activeWorktreeId: item.active_worktree_id,
    reporterId: item.reporter_id,
    worktreeId,
    // GroupWorkItemView 投影暂未含 branch 字段 — 诚实置 null (未绑定, 全局可见)
    branchId: null,
    branchName: null,
  }));
}

/** 把 RunTaskCard (run 范围) 归一为统一视图. */
export function normalizeRunTaskCards(cards: RunTaskCard[], worktreeId: string): UnifiedWorkItem[] {
  return cards.map((card) => ({
    id: card.work_item_id,
    title: card.title,
    kind: card.item_type,
    description: card.description,
    priority: card.priority as RunTaskCardPriority,
    labels: card.labels,
    column: columnFor(card.lifecycle.status),
    lifecycleStatus: card.lifecycle.status as RunTaskCardTargetStatus,
    lifecycleVersion: card.lifecycle.version,
    reviewState: card.lifecycle.review_state,
    activeWorktreeId: card.lifecycle.active_worktree_id,
    reporterId: "",
    worktreeId,
    branchId: card.branch_id,
    branchName: card.branch_name,
  }));
}

/** 把共享 client 抛出的 GroupApiError / RunDirectoryError 归一为明确冲突类型. */
export function toUnifiedError(error: unknown): UnifiedWorkItemConflictError {
  if (error instanceof GroupApiError || error instanceof RunDirectoryError) {
    return new UnifiedWorkItemConflictError(error.status, error.code, error.message);
  }
  if (error instanceof UnifiedWorkItemConflictError) return error;
  return new UnifiedWorkItemConflictError(0, "unexpected_error", error instanceof Error ? error.message : "Group API request failed.");
}

/** worktree 范围的任务卡列表 (服务端 PostgreSQL 真源, 每次进入页面重新 fetch). */
export async function listUnifiedWorkItems(
  client: WorktreeGroupApiClient,
  worktreeId: string,
  signal?: AbortSignal,
): Promise<UnifiedWorkItem[]> {
  // 服务端 group_api/work_items.rs `project_work_item` 输出的是原始行
  // (work_item_id + lifecycle{...}), 不是 GroupWorkItemView 形状;
  // 这里做与 groupProjection.mapWorkItems 同表的归一, 不改共享文件。
  const response = await client.listWorkItems<{
    worktree_id: string;
    project_id: string;
    work_items: Array<Record<string, unknown>>;
  }>(worktreeId, signal);
  if (response.worktree_id !== worktreeId) {
    throw new Error("Group API returned a projection outside the active Worktree.");
  }
  const views: GroupWorkItemView[] = response.work_items.map((raw) => {
    const lifecycle = (raw.lifecycle ?? {}) as Record<string, unknown>;
    const status = String(lifecycle.status ?? "");
    const version = Number(lifecycle.version);
    if (!status || !Number.isSafeInteger(version) || version < 1) {
      throw new Error("Group API returned an unsupported WorkItem lifecycle projection.");
    }
    const id = String(raw.work_item_id ?? raw.id ?? "");
    if (!id) throw new Error("Group API returned a WorkItem without an id.");
    return {
      id,
      project_id: response.project_id,
      worktree_id: worktreeId,
      key: id,
      title: String(raw.title ?? ""),
      description: typeof raw.description === "string" ? raw.description : "",
      kind: normalizeKind(String(raw.item_type ?? "task")),
      status: LIFECYCLE_COLUMN[status] === "done" ? "done"
        : LIFECYCLE_COLUMN[status] === "blocked" ? "blocked"
        : LIFECYCLE_COLUMN[status] === "wontfix" ? "wontfix"
        : LIFECYCLE_COLUMN[status] === "in_progress" ? "in_progress" : "todo",
      priority: normalizePriority(String(raw.priority ?? "medium")),
      lifecycle_status: status as GroupWorkItemView["lifecycle_status"],
      lifecycle_version: version,
      review_state: typeof lifecycle.review_state === "string" ? lifecycle.review_state : "none",
      active_worktree_id: typeof lifecycle.active_worktree_id === "string" ? lifecycle.active_worktree_id : null,
      labels: Array.isArray(raw.labels) ? raw.labels.filter((l): l is string => typeof l === "string") : [],
      reporter_id: String(raw.reporter_user_id ?? ""),
    };
  });
  return normalizeGroupWorkItems(views, worktreeId);
}

/** 创建任务卡, 挂当前 worktree (branch) — 与 sprint/backlog 入口同一服务端写入路径. */
export async function createUnifiedWorkItem(
  runClient: RunDirectoryApiClient,
  runContext: RunContextEnvelope,
  worktreeId: string,
  input: { title: string; description: string; priority: RunTaskCardPriority; idempotencyKey: string },
  signal?: AbortSignal,
): Promise<UnifiedWorkItem> {
  const card = await runClient.createLegacyRunTaskCard(
    runContext,
    {
      title: input.title,
      description: input.description,
      priority: input.priority,
      correlation_id: newCorrelationId(),
    },
    input.idempotencyKey,
    worktreeId,
    signal,
  );
  return normalizeRunTaskCards([card], worktreeId)[0];
}

/** 移动任务卡到统一列 (kanban/sprint 两入口共用; 同列移动为视图重排, 不产生服务端写). */
export async function moveUnifiedWorkItem(
  client: WorktreeGroupApiClient,
  worktreeId: string,
  item: UnifiedWorkItem,
  targetColumn: UnifiedKanbanColumn,
  idempotencyKey: string,
): Promise<UnifiedWorkItem> {
  const targetStatus = UNIFIED_COLUMN_STATUS[targetColumn];
  if (targetStatus === item.lifecycleStatus) {
    // 同列内重排: 服务端无列内排序, 不产生写; 返回当前服务端快照
    return item;
  }
  return transitionUnifiedWorkItem(client, worktreeId, item, targetStatus, idempotencyKey);
}

/** 状态转换 (六态生命周期, 乐观锁 expected_version; 冲突由服务端 409 明确反馈). */
export async function transitionUnifiedWorkItem(
  client: WorktreeGroupApiClient,
  worktreeId: string,
  item: UnifiedWorkItem,
  targetStatus: RunTaskCardTargetStatus,
  idempotencyKey: string,
): Promise<UnifiedWorkItem> {
  const snapshot = await client.transitionWorkItem<{
    work_item_id: string;
    title: string;
    item_type: string;
    description: string;
    priority: string;
    labels: string[];
    reporter_user_id: string;
    lifecycle: {
      status: string;
      review_state: string;
      active_worktree_id: string | null;
      version: number;
      updated_at: string;
    };
    metadata_updated_at: string;
  }>(
    worktreeId,
    item.id,
    {
      target_status: targetStatus,
      expected_version: item.lifecycleVersion,
      correlation_id: newCorrelationId(),
      reason: null,
    },
    idempotencyKey,
  );
  const updated = normalizeGroupWorkItems(
    [
      {
        id: snapshot.work_item_id,
        project_id: "",
        worktree_id: worktreeId,
        key: snapshot.work_item_id,
        title: snapshot.title,
        description: snapshot.description,
        kind: normalizeKind(snapshot.item_type),
        status: columnFor(snapshot.lifecycle.status),
        priority: normalizePriority(snapshot.priority),
        lifecycle_status: snapshot.lifecycle.status as GroupWorkItemView["lifecycle_status"],
        lifecycle_version: snapshot.lifecycle.version,
        review_state: snapshot.lifecycle.review_state,
        active_worktree_id: snapshot.lifecycle.active_worktree_id,
        labels: snapshot.labels,
        reporter_id: snapshot.reporter_user_id,
      },
    ],
    worktreeId,
  );
  return updated[0];
}
