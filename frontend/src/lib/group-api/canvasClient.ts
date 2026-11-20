// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/group-api/canvasClient.ts"})
MERGE (client:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient",kind:"class"})
MERGE (load:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient.loadCanvas",kind:"method"})
MERGE (createCanvas:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient.createCanvas",kind:"method"})
MERGE (createElement:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient.createElement",kind:"method"})
MERGE (move:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient.updateElementPosition",kind:"method"})
MERGE (deleteElement:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient.deleteElement",kind:"method"})
MERGE (subscribeWake:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient.subscribeWake",kind:"method"})
MERGE (makeSubscriber:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient.makeWakeSubscriber",kind:"method"})
MERGE (conflictError:Type {id:"frontend/src/lib/group-api/canvasClient.ts::CanvasConflictError",kind:"class"})
MERGE (readOnlyError:Type {id:"frontend/src/lib/group-api/canvasClient.ts::CanvasReadOnlyError",kind:"class"})
MERGE (disconnectedError:Type {id:"frontend/src/lib/group-api/canvasClient.ts::CanvasDisconnectedError",kind:"class"})
MERGE (draftStore:Type {id:"frontend/src/lib/group-api/canvasClient.ts::CanvasDraftStore",kind:"class"})
MERGE (snapshot:Type {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasSnapshot",kind:"interface"})
MERGE (permission:Type {id:"frontend/src/lib/group-api/canvasClient.ts::CanvasPermissionSnapshot",kind:"interface"})
MERGE (document:Type {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasDocument",kind:"interface"})
MERGE (element:Type {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasElement",kind:"interface"})
MERGE (apiClient:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient",kind:"class"})
MERGE (wakeSubscriber:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::CanvasOutboxWakeSubscriber",kind:"class"})
MERGE (self)-[:DEFINES]->(client)
MERGE (self)-[:DEFINES]->(conflictError)
MERGE (self)-[:DEFINES]->(readOnlyError)
MERGE (self)-[:DEFINES]->(disconnectedError)
MERGE (self)-[:DEFINES]->(draftStore)
MERGE (self)-[:DEFINES]->(snapshot)
MERGE (self)-[:DEFINES]->(permission)
MERGE (self)-[:DEFINES]->(document)
MERGE (self)-[:DEFINES]->(element)
MERGE (client)-[:CALLS]->(apiClient)
MERGE (makeSubscriber)-[:CREATES]->(wakeSubscriber)
MERGE (subscribeWake)-[:CALLS]->(:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.subscribeWorktreeCanvasWake",kind:"method"})
@endcypher
*/
// =====================================================================
// 统一 Canvas Client (per TASK-p1-canvas-data):
// 独立 /canvas/[id] 入口接 Group Canvas 授权真源。
// - 复用 worktreeGroupApi.ts (只读复用, 不改):
//   WorktreeGroupApiClient (授权 fetch + Idempotency-Key 写入),
//   CanvasOutboxWakeSubscriber / BrowserCanvasEventCursorStore
//   (SSE wake + 断线重连 + cursor catch-up)
// - 服务端真源: GET /api/v1/worktrees/{id}/canvas 返回画布 + 元素 +
//   权限快照 (can_write), 移动与新增共用同一授权判定
// - 错误分层: 409 冲突 (current/submitted 双方可见) / 403 只读 / 断线
// - 未确认草稿 localStorage 持久化 (CanvasDraftStore), 冲突不静默覆盖
// =====================================================================

import type { CanvasConnector, CanvasElement, CanvasFrame, CanvasViewport } from "@/types/ids";
import {
  CanvasOutboxWakeSubscriber,
  GroupApiError,
  WorktreeGroupApiClient,
  type CanvasOutboxCatchUpOptions,
  type WorktreeCanvasWakeEvent,
} from "@/lib/group/worktreeGroupApi";

export interface CanvasPermissionSnapshot {
  canWrite: boolean;
  canCreateCanvas: boolean;
  roles: string[];
  scopes: string[];
}

export interface GroupCanvasElement {
  id: string;
  canvas_id: string;
  kind: CanvasElement["kind"];
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  z_index: number;
  content: CanvasElement["content"];
  locked: boolean;
  hidden: boolean;
  version: number;
  entity_ref?: { ref_type: string; ref_id: string; worktree_id: string };
}

export interface GroupCanvasDocument {
  canvasId: string;
  title: string;
  version: number;
  viewport: CanvasViewport;
  frames: CanvasFrame[];
  connectors: CanvasConnector[];
}

export interface GroupCanvasSnapshot {
  worktreeId: string;
  projectId: string;
  canvas: GroupCanvasDocument | null;
  elements: GroupCanvasElement[];
  permission: CanvasPermissionSnapshot;
}

/** 新增元素输入 (CanvasView.onCreateElement 回调形状)。 */
export interface CanvasElementCreateInput {
  kind: CanvasElement["kind"];
  x: number;
  y: number;
  width: number;
  height: number;
  z_index: number;
  content: CanvasElement["content"];
}

/** 版本冲突: 双方内容可见, 不静默覆盖 (current=服务端最新, submitted=本地草稿)。 */
export class CanvasConflictError extends Error {
  readonly status = 409;
  constructor(
    readonly code: string,
    message: string,
    readonly current?: unknown,
    readonly submitted?: unknown,
  ) {
    super(message);
    this.name = "CanvasConflictError";
  }
}

/** 只读/未授权: 按权限决定写按钮禁用。 */
export class CanvasReadOnlyError extends Error {
  readonly status = 403;
  constructor(message: string) {
    super(message);
    this.name = "CanvasReadOnlyError";
  }
}

/** 断线: 网络失败或服务不可达, 触发重连与草稿保留。 */
export class CanvasDisconnectedError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CanvasDisconnectedError";
  }
}

// =====================================================================
// 未确认草稿持久化 (localStorage draft key, per TASK 交付 8)
// =====================================================================

export interface CanvasPendingDraft {
  kind: "element-create" | "element-move" | "element-delete";
  elementId?: string;
  payload: unknown;
  savedAt: string;
}

export const CanvasDraftStore = {
  storageKey(worktreeId: string, canvasId: string): string {
    return `star:canvas-draft:v1:${encodeURIComponent(worktreeId)}:${encodeURIComponent(canvasId)}`;
  },

  save(worktreeId: string, canvasId: string, draft: CanvasPendingDraft): void {
    try {
      globalThis.localStorage?.setItem(
        this.storageKey(worktreeId, canvasId),
        JSON.stringify({ ...draft, savedAt: new Date().toISOString() }),
      );
    } catch {
      // Storage 不可用时草稿保留在内存 (页面状态), 不阻塞写入。
    }
  },

  load(worktreeId: string, canvasId: string): CanvasPendingDraft | null {
    try {
      const raw = globalThis.localStorage?.getItem(this.storageKey(worktreeId, canvasId));
      if (!raw) return null;
      const parsed = JSON.parse(raw) as Partial<CanvasPendingDraft>;
      if (
        parsed.kind !== "element-create" &&
        parsed.kind !== "element-move" &&
        parsed.kind !== "element-delete"
      ) {
        return null;
      }
      return parsed as CanvasPendingDraft;
    } catch {
      return null;
    }
  },

  clear(worktreeId: string, canvasId: string): void {
    try {
      globalThis.localStorage?.removeItem(this.storageKey(worktreeId, canvasId));
    } catch {
      // 忽略
    }
  },
};

// =====================================================================
// GroupCanvasClient — 统一画布读写客户端
// =====================================================================

export class GroupCanvasClient {
  constructor(
    private readonly api: WorktreeGroupApiClient,
    readonly worktreeId: string,
  ) {}

  /** 读取服务端授权画布快照 (画布 + 元素 + 权限), 两入口同一资源。
   * 复用 WorktreeGroupApiClient 公开方法 (其 request 为 private):
   * group-context (granted_scopes/actor_roles) + canvases 列表 + elements。
   * 权限快照为 UI 层判定; 服务端在每个写请求上仍会复验 (403 → CanvasReadOnlyError)。 */
  async loadCanvas(signal?: AbortSignal): Promise<GroupCanvasSnapshot> {
    const [context, canvasesEnvelope] = await Promise.all([
      this.guard(
        this.api.getGroupContext<RawGroupContextEnvelope>(this.worktreeId, signal),
      ),
      this.guard(
        this.api.listCanvases<RawCanvasListEnvelope>(this.worktreeId, signal),
      ),
    ]);
    const worktreeId = requiredString(context.group_context?.worktree_id, "group_context.worktree_id");
    if (worktreeId !== this.worktreeId) {
      throw new Error("Group API 返回了当前 Worktree 之外的画布投影。");
    }
    if (canvasesEnvelope.worktree_id !== this.worktreeId) {
      throw new Error("Group API 返回了当前 Worktree 之外的画布列表。");
    }
    const rows = Array.isArray(canvasesEnvelope.canvases) ? canvasesEnvelope.canvases : [];
    const canvas = rows.length > 0 ? mapCanvasListRow(rows[0], this.worktreeId) : null;
    const elements = canvas
      ? await this.guard(
          this.api
            .listCanvasElements<RawElementListEnvelope>(this.worktreeId, canvas.canvasId, signal)
            .then((envelope) => mapElementListEnvelope(envelope, canvas.canvasId, this.worktreeId)),
        )
      : [];
    return {
      worktreeId,
      projectId: requiredString(context.group_context?.project_id, "group_context.project_id"),
      canvas,
      elements,
      permission: mapPermission(context),
    };
  }

  /** 该 worktree 尚无画布时创建 (仅 can_create_canvas)。 */
  async createCanvas(title: string, idempotencyKey: string): Promise<{ canvasId: string }> {
    const raw = await this.guard(
      this.api.createCanvas<{ canvas?: { canvas_id?: unknown } }>(
        this.worktreeId,
        { title },
        idempotencyKey,
      ),
    );
    const canvasId = requiredString(raw.canvas?.canvas_id, "canvas.canvas_id");
    return { canvasId };
  }

  /** 新增元素 (幂等键防双击重复创建)。 */
  async createElement(
    canvasId: string,
    body: CanvasElementCreateInput,
    idempotencyKey: string,
  ): Promise<{ id: string }> {
    const raw = await this.guard(
      this.api.createCanvasElement<{ element?: { element_id?: unknown } }>(
        this.worktreeId,
        canvasId,
        {
          ...body,
          rotation: 0,
          locked: false,
          hidden: false,
        },
        idempotencyKey,
      ),
    );
    return { id: requiredString(raw.element?.element_id, "element.element_id") };
  }

  /** 移动元素 (Position 模式, expected_version 乐观校验)。 */
  async updateElementPosition(
    canvasId: string,
    elementId: string,
    position: { x: number; y: number; expectedVersion: number },
    idempotencyKey: string,
  ): Promise<void> {
    await this.guard(
      this.api.updateCanvasElement(
        this.worktreeId,
        canvasId,
        elementId,
        {
          expected_version: position.expectedVersion,
          update_mode: "position",
          x: position.x,
          y: position.y,
        },
        idempotencyKey,
      ),
    );
  }

  /** 删除元素 (expected_version 乐观校验)。 */
  async deleteElement(
    canvasId: string,
    elementId: string,
    expectedVersion: number,
    idempotencyKey: string,
  ): Promise<void> {
    await this.guard(
      this.api.deleteCanvasElement(
        this.worktreeId,
        canvasId,
        elementId,
        { expected_version: expectedVersion },
        idempotencyKey,
      ),
    );
  }

  /** SSE wake 订阅 (canvas-wake / resync_required / error), 复用 worktreeGroupApi 多路复用。 */
  subscribeWake(listener: (event: WorktreeCanvasWakeEvent) => void): () => void {
    return this.api.subscribeWorktreeCanvasWake(this.worktreeId, listener);
  }

  /** 断线恢复基建: wake + cursor catch-up + 自动重连 (复用 CanvasOutboxWakeSubscriber)。 */
  makeWakeSubscriber(
    refreshProjection: (signal: AbortSignal) => Promise<void>,
    options?: CanvasOutboxCatchUpOptions,
  ): CanvasOutboxWakeSubscriber {
    return new CanvasOutboxWakeSubscriber(this.api, this.worktreeId, refreshProjection, options);
  }

  /** 错误分层: 409 冲突 / 403 只读 / 断线。 */
  private async guard<T>(promise: Promise<T>): Promise<T> {
    try {
      return await promise;
    } catch (error) {
      if (error instanceof GroupApiError) {
        if (error.status === 409) {
          throw new CanvasConflictError(
            error.code,
            "画布已被他人更新 — 服务端最新与本地草稿均已保留, 请选择保留哪一方。",
          );
        }
        if (error.status === 403 || error.status === 401) {
          throw new CanvasReadOnlyError(error.message);
        }
        throw error;
      }
      if (
        error instanceof DOMException &&
        (error.name === "AbortError" || error.name === "TimeoutError")
      ) {
        throw error;
      }
      // 网络层失败 (浏览器 fetch 失败为 TypeError) → 断线; 其余 (响应形状错误等) 原样抛出
      if (error instanceof TypeError) {
        throw new CanvasDisconnectedError(
          error instanceof Error ? error.message : "Canvas 连接中断, 正在重试。",
        );
      }
      throw error;
    }
  }
}

// =====================================================================
// 服务端响应映射与校验 (严格模式, 拒绝畸形响应)
// =====================================================================

interface RawGroupContextEnvelope {
  group_context?: {
    worktree_id?: unknown;
    project_id?: unknown;
    actor_roles?: unknown;
    granted_scopes?: unknown;
  };
}

interface RawCanvasListEnvelope {
  worktree_id?: unknown;
  project_id?: unknown;
  canvases?: unknown;
}

interface RawElementListEnvelope {
  worktree_id?: unknown;
  canvas_id?: unknown;
  elements?: unknown;
}

const WRITER_ROLES = new Set(["tenant_admin", "project_admin", "developer", "agent"]);

/** UI 层权限快照: scope 含 canvas:write 且 project 角色为 writer。
 * 服务端在每个写请求上仍会按 scope + project/run 角色复验。 */
function mapPermission(context: RawGroupContextEnvelope): CanvasPermissionSnapshot {
  const groupContext = context.group_context ?? {};
  const scopes = Array.isArray(groupContext.granted_scopes)
    ? groupContext.granted_scopes.filter((scope): scope is string => typeof scope === "string")
    : [];
  const roles = Array.isArray(groupContext.actor_roles)
    ? groupContext.actor_roles.filter((role): role is string => typeof role === "string")
    : [];
  const canWrite = scopes.includes("canvas:write") && roles.every((role) => WRITER_ROLES.has(role));
  return {
    canWrite,
    canCreateCanvas: canWrite,
    roles,
    scopes,
  };
}

function mapCanvasListRow(raw: unknown, expectedWorktreeId: string): GroupCanvasDocument {
  void expectedWorktreeId;
  const record = asRecord(raw, "canvas row");
  const document = asRecord(record.document, "canvas.document");
  const viewport = asRecord(document.viewport, "canvas.document.viewport");
  return {
    canvasId: requiredString(record.canvas_id, "canvas.canvas_id"),
    title: requiredString(record.title, "canvas.title"),
    version: finiteNumber(record.version, "canvas.version"),
    viewport: {
      x: finiteNumber(viewport.x, "canvas.viewport.x"),
      y: finiteNumber(viewport.y, "canvas.viewport.y"),
      zoom: finiteNumber(viewport.zoom, "canvas.viewport.zoom"),
    },
    frames: Array.isArray(document.frames) ? (document.frames as CanvasFrame[]) : [],
    connectors: Array.isArray(document.connectors)
      ? (document.connectors as CanvasConnector[])
      : [],
  };
}

function mapElementListEnvelope(
  raw: RawElementListEnvelope,
  canvasId: string,
  expectedWorktreeId: string,
): GroupCanvasElement[] {
  if (raw.worktree_id !== expectedWorktreeId) {
    throw new Error("Canvas API 返回了当前 Worktree 之外的元素。");
  }
  if (raw.canvas_id !== canvasId) {
    throw new Error("Canvas API 返回了其他画布的元素。");
  }
  return Array.isArray(raw.elements)
    ? raw.elements.map((item) => mapCanvasElement(item, canvasId, expectedWorktreeId))
    : [];
}

function mapCanvasElement(
  raw: unknown,
  canvasId: string,
  expectedWorktreeId: string,
): GroupCanvasElement {
  const record = asRecord(raw, "element");
  const entityRefRaw =
    record.entity_ref == null ? undefined : asRecord(record.entity_ref, "element.entity_ref");
  if (entityRefRaw && entityRefRaw.worktree_id !== expectedWorktreeId) {
    throw new Error("Canvas EntityRef 超出当前 Worktree 范围。");
  }
  return {
    id: requiredString(record.element_id, "element.element_id"),
    canvas_id: canvasId,
    kind: requiredString(record.kind, "element.kind") as CanvasElement["kind"],
    x: finiteNumber(record.x, "element.x"),
    y: finiteNumber(record.y, "element.y"),
    width: finiteNumber(record.width, "element.width"),
    height: finiteNumber(record.height, "element.height"),
    rotation: finiteNumber(record.rotation, "element.rotation"),
    z_index: finiteNumber(record.z_index, "element.z_index"),
    content: asRecord(record.content, "element.content") as CanvasElement["content"],
    locked: record.locked === true,
    hidden: record.hidden === true,
    version: finiteNumber(record.version, "element.version"),
    entity_ref: entityRefRaw
      ? {
          ref_type: requiredString(entityRefRaw.ref_type, "element.entity_ref.ref_type"),
          ref_id: requiredString(entityRefRaw.ref_id, "element.entity_ref.ref_id"),
          worktree_id: requiredString(entityRefRaw.worktree_id, "element.entity_ref.worktree_id"),
        }
      : undefined,
  };
}

function asRecord(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`无效的 Canvas API 响应: ${label}。`);
  }
  return value as Record<string, unknown>;
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`无效的 Canvas API 响应: ${label}。`);
  }
  return value;
}

function finiteNumber(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`无效的 Canvas API 响应: ${label}。`);
  }
  return value;
}
