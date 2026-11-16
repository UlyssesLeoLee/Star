/*
@cypher schema=1 source_sha256=6ede489826895062bb18855585f3b305027071479cf1188bcd3953fc522fffe3
MERGE (self:File {path:"frontend/src/lib/canvas-entry.test.ts"})
MERGE (resolveCanvasId:Symbol {id:"frontend/src/lib/canvas-entry.ts::resolveCanvasId",kind:"function"})
MERGE (createCanvasLocal:Symbol {id:"frontend/src/lib/canvas-entry.ts::createCanvasLocal",kind:"function"})
MERGE (self)-[:TESTS]->(resolveCanvasId)
MERGE (self)-[:TESTS]->(createCanvasLocal)
@endcypher
*/
// =====================================================================
// canvas-entry 单测: /canvas 入口的上下文 → 画布解析 + 本地新建画布
// (per 2026-11-16 用户发令 10 条第 4 条: 取消固定跳 canvas-001)
// =====================================================================

import { beforeEach, describe, expect, it } from "vitest";
import { createCanvasLocal, resolveCanvasId } from "./canvas-entry";
import { useStore } from "./store";
import type { Canvas, Lane, Worktree } from "@/types/ids";

const makeCanvas = (overrides: Partial<Canvas> & { id: string }): Canvas => ({
  id: overrides.id,
  tenant_id: "ten-acme",
  workspace_id: "ws-001",
  title: `canvas ${overrides.id}`,
  ref_kind: "free",
  viewport: { x: 0, y: 0, zoom: 1 },
  frames: [],
  creator_id: "usr-001",
  collaborator_ids: ["usr-001"],
  created_at: "2026-11-16T00:00:00.000Z",
  updated_at: "2026-11-16T00:00:00.000Z",
  ...overrides,
});

const makeWorktree = (id: string, project_id: string): Worktree => ({
  id,
  tenant_id: "ten-acme",
  project_id,
  name: `wt ${id}`,
  branch: "dev",
  base_branch: "main",
  status: "active",
  lock_version: 0,
  last_event_at: "2026-11-16T00:00:00.000Z",
  created_at: "2026-11-16T00:00:00.000Z",
});

const makeLane = (id: string, project_id: string): Lane => ({
  id,
  tenant_id: "ten-acme",
  project_id,
  name: `lane ${id}`,
  created_at: "2026-11-16T00:00:00.000Z",
});

describe("resolveCanvasId — Worktree / Lane 上下文 → 画布", () => {
  const worktrees = [makeWorktree("wt-001", "prj-physis")];
  const lanes = [makeLane("lane-001", "prj-physis")];

  it("Worktree 关联画布优先 (ref_kind=worktree + ref_id=worktree id)", () => {
    const canvases = [
      makeCanvas({ id: "canvas-wt", ref_kind: "worktree", ref_id: "wt-001" }),
      makeCanvas({ id: "canvas-proj", ref_kind: "project", ref_id: "prj-physis" }),
    ];
    const hit = resolveCanvasId({ worktreeId: "wt-001", laneId: "lane-001" }, worktrees, lanes, canvases);
    expect(hit?.id).toBe("canvas-wt");
  });

  it("Worktree 无画布时级联到 Lane 默认画布 (所属 project 的画布)", () => {
    const canvases = [makeCanvas({ id: "canvas-proj", ref_kind: "project", ref_id: "prj-physis" })];
    const hit = resolveCanvasId({ worktreeId: "wt-001", laneId: "lane-001" }, worktrees, lanes, canvases);
    expect(hit?.id).toBe("canvas-proj");
  });

  it("仅 Lane 上下文 → 返回 Lane 所属 project 的画布", () => {
    const canvases = [makeCanvas({ id: "canvas-proj", ref_kind: "project", ref_id: "prj-physis" })];
    const hit = resolveCanvasId({ laneId: "lane-001" }, worktrees, lanes, canvases);
    expect(hit?.id).toBe("canvas-proj");
  });

  it("free 画布不作为 Lane 默认画布", () => {
    const canvases = [makeCanvas({ id: "canvas-free", ref_kind: "free" })];
    expect(resolveCanvasId({ laneId: "lane-001" }, worktrees, lanes, canvases)).toBeNull();
  });

  it("无上下文 / 上下文无匹配 → null (入口页渲染未接入状态)", () => {
    const canvases = [makeCanvas({ id: "canvas-proj", ref_kind: "project", ref_id: "prj-physis" })];
    expect(resolveCanvasId({}, worktrees, lanes, canvases)).toBeNull();
    expect(resolveCanvasId({ worktreeId: "wt-999", laneId: "lane-999" }, worktrees, lanes, canvases)).toBeNull();
    expect(resolveCanvasId({ worktreeId: "wt-001" }, [], lanes, canvases)).toBeNull();
  });

  it("其他 project 的画布不会误配到当前 Lane", () => {
    const canvases = [makeCanvas({ id: "canvas-other", ref_kind: "project", ref_id: "prj-stargate" })];
    expect(resolveCanvasId({ laneId: "lane-001" }, worktrees, lanes, canvases)).toBeNull();
  });
});

describe("createCanvasLocal — 本地新建画布 (store 直写, 不依赖 seed)", () => {
  beforeEach(() => {
    useStore.setState({ canvases: [], workspaces: [], identities: [], tenants: [], tenantId: undefined });
  });

  it("注入 id 创建画布并写入 store", () => {
    const canvas = createCanvasLocal({ id: "canvas-new", title: "测试画布", ref_kind: "free" });
    expect(canvas.id).toBe("canvas-new");
    expect(canvas.title).toBe("测试画布");
    expect(useStore.getState().canvases.map((c) => c.id)).toContain("canvas-new");
  });

  it("不传 id 时生成 canvas- 前缀的唯一 id", () => {
    const a = createCanvasLocal({});
    const b = createCanvasLocal({});
    expect(a.id).toMatch(/^canvas-/);
    expect(a.id).not.toBe(b.id);
  });

  it("重复 id 幂等返回现有画布, 不重复插入", () => {
    const first = createCanvasLocal({ id: "canvas-dup" });
    const second = createCanvasLocal({ id: "canvas-dup", title: "覆盖标题" });
    expect(second).toBe(first);
    expect(useStore.getState().canvases.filter((c) => c.id === "canvas-dup").length).toBe(1);
  });

  it("worktree 绑定画布携带 ref_kind/ref_id", () => {
    const canvas = createCanvasLocal({ ref_kind: "worktree", ref_id: "wt-001", title: "wt 画布" });
    expect(canvas.ref_kind).toBe("worktree");
    expect(canvas.ref_id).toBe("wt-001");
  });
});
