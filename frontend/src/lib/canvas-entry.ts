/*
@cypher schema=1 source_sha256=3a455f05c3311ee78ff6649dffb196508dc9fdcc4a3ffd211e3fd4656c799c69
MERGE (self:File {path:"frontend/src/lib/canvas-entry.ts"})
MERGE (resolveCanvasId:Symbol {id:"frontend/src/lib/canvas-entry.ts::resolveCanvasId",kind:"function"})
MERGE (createCanvasLocal:Symbol {id:"frontend/src/lib/canvas-entry.ts::createCanvasLocal",kind:"function"})
MERGE (store:File {path:"frontend/src/lib/store.ts"})
MERGE (self)-[:DEFINES]->(resolveCanvasId)
MERGE (self)-[:DEFINES]->(createCanvasLocal)
MERGE (self)-[:IMPORTS]->(store)
@endcypher
*/
// =====================================================================
// /canvas 入口的上下文 → 画布解析 + 本地新建画布
// (per 2026-11-16 用户发令 10 条第 4 条: 取消固定跳 canvas-001,
//  按当前 Worktree / Lane 上下文进入正确画布)
//
// 关联模型 (types/ids.ts Canvas):
//   ref_kind = "worktree" + ref_id = worktree id → 该 worktree 的关联画布
//   ref_kind = "project"  + ref_id = project id  → 该 project 的画布,
//       用作 Lane 的默认画布 (Lane 类型无直接 canvas 字段)
//
// 解析级联 (先命中先返回; 全 miss 返回 null → 入口页渲染
// 未接入状态 + 新建画布按钮):
//   1. selectedWorktreeId → 该 worktree 的关联画布
//   2. selectedLaneId     → Lane 默认画布 (所属 project 的画布)
// =====================================================================

import { useStore } from "@/lib/store";
import type { Canvas, Lane, Worktree } from "@/types/ids";

export interface CanvasResolutionContext {
  worktreeId?: string;
  laneId?: string;
}

/** 按当前 Worktree / Lane 上下文解析应进入的画布; 无匹配返回 null。 */
export function resolveCanvasId(
  ctx: CanvasResolutionContext,
  worktrees: Worktree[],
  lanes: Lane[],
  canvases: Canvas[],
): Canvas | null {
  // 1) Worktree 关联画布
  if (ctx.worktreeId) {
    const worktree = worktrees.find((w) => w.id === ctx.worktreeId);
    if (worktree) {
      const wtCanvas = canvases.find(
        (c) => c.ref_kind === "worktree" && c.ref_id === worktree.id,
      );
      if (wtCanvas) return wtCanvas;
    }
  }
  // 2) Lane 默认画布 (所属 project 的画布)
  if (ctx.laneId) {
    const lane = lanes.find((l) => l.id === ctx.laneId);
    if (lane) {
      const laneCanvas = canvases.find(
        (c) => c.ref_kind === "project" && c.ref_id === lane.project_id,
      );
      if (laneCanvas) return laneCanvas;
    }
  }
  return null;
}

export interface CreateCanvasInput {
  /** 可选注入 id ([id] 页补齐缺失画布时传路由 id); 不传由本地生成 */
  id?: string;
  title?: string;
  ref_kind?: Canvas["ref_kind"];
  ref_id?: string;
}

/**
 * 本地新建画布 (画布后端 API 未接入前的本地持久化通道):
 * 写入 zustand store (persist → localStorage), 不依赖 seed 兜底。
 * id 已存在时幂等返回现有画布。
 */
export function createCanvasLocal(input: CreateCanvasInput): Canvas {
  const s = useStore.getState();
  const id = input.id ?? newLocalCanvasId();
  const existing = s.canvases.find((c) => c.id === id);
  if (existing) return existing;

  const now = new Date().toISOString();
  const creatorId = s.identities[0]?.id ?? "usr-001";
  const canvas: Canvas = {
    id,
    tenant_id: s.tenantId ?? s.tenants[0]?.id ?? "tenant-physis-corp",
    workspace_id: s.workspaces[0]?.id ?? "ws-001",
    title: input.title?.trim() || "未命名画布",
    ref_kind: input.ref_kind,
    ref_id: input.ref_id,
    viewport: { x: 0, y: 0, zoom: 1 },
    frames: [],
    creator_id: creatorId,
    collaborator_ids: [creatorId],
    created_at: now,
    updated_at: now,
  };
  useStore.setState((cur) => ({ canvases: [...cur.canvases, canvas] }));
  return canvas;
}

function newLocalCanvasId(): string {
  const gen =
    typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
      ? () => crypto.randomUUID()
      : () => `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  return `canvas-${gen()}`;
}
