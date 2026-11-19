/*
@cypher schema=1 source_sha256=f6db98c4313b4243ae0177af771bc8a1c063fc7bcd4427c344379a1b4ffa22e5
MERGE (self:File {path:"frontend/src/app/canvas/[id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasDetailPage",kind:"component"})
MERGE (canvasView:Symbol {id:"frontend/src/components/CanvasView.tsx::CanvasView",kind:"component"})
MERGE (relationshipView:Symbol {id:"frontend/src/app/(app)/agent-relationships/view/RelationshipView.tsx::RelationshipView",kind:"component"})
MERGE (agentViewPage:Symbol {id:"frontend/src/components/agent-view/AgentViewPage.tsx::AgentViewPage",kind:"component"})
MERGE (scopeSelector:Symbol {id:"frontend/src/components/canvas/CanvasScopeSelector.tsx::CanvasScopeSelector",kind:"component"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (createCanvasLocal:Symbol {id:"frontend/src/lib/canvas-entry.ts::createCanvasLocal",kind:"function"})
MERGE (groupClient:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient",kind:"class"})
MERGE (resolveSource:Symbol {id:"frontend/src/lib/canvas-entry.ts::resolveCanvasSource",kind:"function"})
MERGE (draftStore:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::CanvasDraftStore",kind:"class"})
MERGE (wakeSubscriber:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::CanvasOutboxWakeSubscriber",kind:"class"})
MERGE (groupApiHook:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupApi",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(canvasView)
MERGE (self)-[:IMPORTS]->(relationshipView)
MERGE (self)-[:IMPORTS]->(agentViewPage)
MERGE (self)-[:IMPORTS]->(scopeSelector)
MERGE (self)-[:IMPORTS]->(groupClient)
MERGE (self)-[:IMPORTS]->(resolveSource)
MERGE (self)-[:IMPORTS]->(draftStore)
MERGE (self)-[:IMPORTS]->(wakeSubscriber)
MERGE (self)-[:IMPORTS]->(groupApiHook)
MERGE (page)-[:CALLS]->(navStore)
MERGE (page)-[:CALLS]->(createCanvasLocal)
@endcypher
*/
"use client";

// =====================================================================
// /canvas/[id] — 无限画布统一页面 (per 2026-11-14 用户发令:
//   "无限画布类全部归纳进 canvas, 不再需要点击后三选一")
//   3 标签 = 全部无限画布类:
//     1. 画布 (CanvasView 白板: 帧/端点/连接线/画笔)
//     2. Agent 画布 (AgentCanvasView 拓扑)
//     3. 关系图 (RelationshipView)
//   非无限画布类 (IPA/低代码流程) 移出为单独页面 (/analytics, /workflow)
// 2026-11-15 (per TASK-ulys-canvas-ctx): Canvas 作为 Lane 应用入口,
//   通过 ?worktree=<id> 深链携带当前 Worktree 上下文, 顶部渲染上下文条.
// 2026-11-15 (per TASK-ulys-canvas-fine): 关系图 tab RelationshipView
//   tenantId 去固定值, 接 store.projects[0].tenant_id.
// 2026-11-16 (per Canvas 范围选择器需求, 10 条第 4 条): 顶部三级范围选择器
//   (Project → Lane → Worktree, 读写 navStore) + BI/Plugins/Workflow 入口
//   (点击读 navStore 范围传后端做权限复验, 接入点见下方注释).
// 2026-11-16 (per 10 条第 4 条): 画布按路由 id 取用 (不再 fallback
//   seed.canvases[0]); 找不到画布时渲染未接入状态 + 新建画布按钮.
// 2026-11-20 (per TASK-p1-canvas-data): 独立入口接 Group Canvas 授权真源 —
//   group 会话 + worktree 上下文 → 服务端画布 (group-context + canvases +
//   elements), 权限快照决定只读/可写按钮; 移动与新增走同一 canWrite 判定
//   (修复「live 模式能移动不能新增」: 消除 移动有回调/新增无回调 的分裂);
//   SSE wake + cursor catch-up 断线恢复 (复用 CanvasOutboxWakeSubscriber) +
//   断线横幅; 409 冲突 → 双方内容对话框 (不静默覆盖); 无会话 → 本地降级标注。
// =====================================================================

import { Suspense, use, useCallback, useEffect, useMemo, useState } from "react";
import { GitBranch, Network, Plus } from "lucide-react";
import { useStore } from "@/lib/store";
import { CanvasView } from "@/components/CanvasView";
import { RelationshipView } from "@/app/(app)/agent-relationships/view/RelationshipView";
import { AgentViewPage } from "@/components/agent-view/AgentViewPage";
import { WorkflowBuilder } from "@/components/agent-workflow/WorkflowBuilder";
import { GitCanvas } from "../components/GitCanvas";
import { CanvasScopeSelector, type CanvasScope } from "@/components/canvas/CanvasScopeSelector";
import { useNavStore } from "@/lib/nav/navStore";
import { createCanvasLocal, resolveCanvasSource } from "@/lib/canvas-entry";
import { useWorktreeGroupApi, useWorktreeGroupSessionState } from "@/lib/group/groupProjection";
import {
  CanvasConflictError,
  CanvasDraftStore,
  CanvasDisconnectedError,
  CanvasReadOnlyError,
  GroupCanvasClient,
  type CanvasPendingDraft,
  type GroupCanvasSnapshot,
} from "@/lib/group-api/canvasClient";
import { GroupApiError } from "@/lib/group/worktreeGroupApi";
import type { CanvasElementKind, CanvasEntityRef } from "@/types/ids";

const CANVAS_SUB_TABS = ["画布", "Agent 画布", "关系图", "流程", "Git 画布"] as const;

// =====================================================================
// 本页新文案 (i18n key 待父代理合并时补 dictionary/zh-CN/en/ja;
// 此处按共享约束用局部三语常量, 不碰共享 dictionary)
// =====================================================================
const UI_TEXT = {
  zh: {
    localBadge: "本地画布 — 未连接服务端",
    groupBadge: (worktreeId: string) => `Group Canvas 授权数据源 · worktree=${worktreeId}`,
    readOnlyHint: "当前会话为只读: 创建/修改/删除按钮已禁用",
    reconnecting: "连接中断, 重试中…",
    conflictTitle: "画布已被他人更新",
    conflictBody: "服务端最新与你的本地草稿均已保留, 请选择保留哪一方 (自动合并后续版本提供)。",
    keepServer: "使用服务端版本 (放弃我的草稿)",
    keepBoth: "保留双方草稿 (我的草稿留在本地, 可稍后恢复)",
    noServerCanvas: "该 Worktree 尚无服务端画布",
    createServerCanvas: "创建服务端画布",
    creatingCanvas: "创建中…",
    loadError: (message: string) => `画布加载失败: ${message}`,
    retry: "重试",
    loading: "加载 Group Canvas 授权数据…",
  },
  en: {
    localBadge: "Local canvas — not connected to server",
    groupBadge: (worktreeId: string) => `Group Canvas authorized source · worktree=${worktreeId}`,
    readOnlyHint: "Read-only session: create / edit / delete buttons are disabled",
    reconnecting: "Connection lost, retrying…",
    conflictTitle: "Canvas was updated elsewhere",
    conflictBody: "Server latest and your local draft are both kept. Choose which side to retain.",
    keepServer: "Use server version (discard my draft)",
    keepBoth: "Keep both drafts (my draft stays local for later recovery)",
    noServerCanvas: "This Worktree has no server canvas yet",
    createServerCanvas: "Create server canvas",
    creatingCanvas: "Creating…",
    loadError: (message: string) => `Canvas load failed: ${message}`,
    retry: "Retry",
    loading: "Loading Group Canvas data…",
  },
  ja: {
    localBadge: "ローカルキャンバス — サーバー未接続",
    groupBadge: (worktreeId: string) => `Group Canvas 承認データソース · worktree=${worktreeId}`,
    readOnlyHint: "読み取り専用: 作成/編集/削除ボタンは無効です",
    reconnecting: "接続が切れました, 再試行中…",
    conflictTitle: "キャンバスが他で更新されました",
    conflictBody: "サーバー最新とローカル下書きの両方を保持しています。どちらを残すか選択してください。",
    keepServer: "サーバー版を使う (下書きを破棄)",
    keepBoth: "両方の下書きを保持 (ローカルに残し後で復元)",
    noServerCanvas: "この Worktree にはまだサーバーキャンバスがありません",
    createServerCanvas: "サーバーキャンバスを作成",
    creatingCanvas: "作成中…",
    loadError: (message: string) => `キャンバス読み込み失敗: ${message}`,
    retry: "再試行",
    loading: "Group Canvas データを読み込み中…",
  },
} as const;

type GroupCanvasViewState =
  | { mode: "loading" }
  | { mode: "live"; snapshot: GroupCanvasSnapshot }
  | { mode: "error"; message: string };

interface CanvasConflictView {
  code: string;
  draft: CanvasPendingDraft;
  latest: GroupCanvasSnapshot | null;
}

export default function CanvasDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = use(params);
  const [activeTab, setActiveTab] = useState<typeof CANVAS_SUB_TABS[number]>("画布");

  // ?tab= 深链 (per 2026-11-15 Agent 中心并入: /agent-view → ?tab=agent)
  useEffect(() => {
    if (typeof window === "undefined") return;
    const tab = new URLSearchParams(window.location.search).get("tab");
    const map: Record<string, typeof CANVAS_SUB_TABS[number]> = {
      board: "画布",
      agent: "Agent 画布",
      relation: "关系图",
      flow: "流程",
      git: "Git 画布",
    };
    if (tab && map[tab]) setActiveTab(map[tab]);
  }, []);
  const canvases = useStore((s) => s.canvases);
  const canvasElements = useStore((s) => s.canvasElements);
  const canvasConnectors = useStore((s) => s.canvasConnectors);
  const workItems = useStore((s) => s.workItems);
  const worktrees = useStore((s) => s.worktrees);
  const projects = useStore((s) => s.projects);
  const lanes = useStore((s) => s.lanes);
  // 画布按路由 id 取用 (per 2026-11-16 10 条第 4 条: 不再 fallback seed.canvases[0]);
  // 找不到 → 画布 tab 渲染未接入状态 + 新建画布按钮 (本地创建, 写 store)
  const canvas = canvases.find((c) => c.id === id);

  // ===================================================================
  // Group 会话与 worktree 上下文解析 (per TASK-p1-canvas-data)
  // ===================================================================
  const groupApi = useWorktreeGroupApi();
  const groupSessionState = useWorktreeGroupSessionState();
  const groupSessionReady = Boolean(groupApi) && groupSessionState.status === "ready";
  const selectedWorktreeId = useNavStore((s) => s.selectedWorktreeId);
  const selectedLaneId = useNavStore((s) => s.selectedLaneId);
  const [deepLinkWorktreeId, setDeepLinkWorktreeId] = useState<string | null>(null);

  // 关系图 tenant 动态化 (per TASK-ulys-canvas-fine):
  //   RelationshipView.tenantId 语义 = RLS 13 類 tenant id (非 project id,
  //   per RelationshipViewProps: "Optional tenant id (RLS 13 類)");
  //   接 store.projects[0].tenant_id (Project.tenant_id 13 類必带 per REQ-SEC-001)。
  //   无 project 时为 undefined → RelationshipView 按未过滤加载 (同 AgentViewTab)。
  const relationTenantId = projects[0]?.tenant_id;

  // ===================================================================
  // Canvas 范围 (Project → Lane → Worktree, per 2026-11-16 范围选择器需求):
  //   - 顶部 CanvasScopeSelector 读写 navStore (selectedProjectId /
  //     selectedLaneId / selectedWorktreeId), onChange 触发画布重新加载
  //   - ?worktree=<id> 深链 (per TASK-ulys-canvas-ctx) → 同步进 navStore
  //     (client-only 解析, 不用 useSearchParams — dev 下与页面级 Suspense
  //     组合会产生双份 DOM, 同 kanban ?view= 模式), 数据源 useStore worktrees
  // ===================================================================
  const [canvasReloadKey, setCanvasReloadKey] = useState(0);
  const [scopeSyncError, setScopeSyncError] = useState<string | null>(null);
  const [scopeNotice, setScopeNotice] = useState<string | null>(null);
  useEffect(() => {
    if (typeof window === "undefined") return;
    const wt = new URLSearchParams(window.location.search).get("worktree");
    if (!wt) return;
    const match = useStore.getState().worktrees.find((w) => w.id === wt);
    if (!match) {
      setScopeSyncError(wt);
      return;
    }
    setDeepLinkWorktreeId(wt);
    const nav = useNavStore.getState();
    nav.setSelectedProjectId(match.project_id);
    nav.setLaneContext(match.lane_id ?? "", match.id, match.branch);
  }, []);
  const handleScopeChange = useCallback((scope: CanvasScope) => {
    setScopeSyncError(null);
    // 画布数据按新范围重新加载 (group 模式: 按 scope.worktreeId 重拉授权画布)
    setCanvasReloadKey((k) => k + 1);
    setScopeNotice(
      scope.worktreeId
        ? `范围已切换: project=${scope.projectId} lane=${scope.laneId} worktree=${scope.worktreeId} (画布已重新加载)`
        : `范围已切换: project=${scope.projectId}${scope.laneId ? ` lane=${scope.laneId}` : ""} (尚未指定 Worktree)`,
    );
  }, []);

  // ===================================================================
  // 数据源解析 (per TASK-p1-canvas-data): group 会话 → 授权真源; 否则本地降级
  // ===================================================================
  const localCanvasWorktreeRef =
    canvas?.ref_kind === "worktree" && canvas.ref_id ? canvas.ref_id : null;
  const contextWorktreeId = deepLinkWorktreeId ?? selectedWorktreeId ?? localCanvasWorktreeRef;

  const source = useMemo(
    () =>
      resolveCanvasSource(
        { worktreeId: contextWorktreeId ?? undefined, laneId: selectedLaneId ?? undefined },
        { worktreeId: groupSessionReady ? contextWorktreeId : null },
        worktrees,
        lanes,
        canvases,
      ),
    [canvases, contextWorktreeId, groupSessionReady, lanes, selectedLaneId, worktrees],
  );

  // ===================================================================
  // Group 模式: 授权真源加载 + SSE 断线恢复 + 冲突可见
  // ===================================================================
  const [groupViewState, setGroupViewState] = useState<GroupCanvasViewState>({ mode: "loading" });
  const [connection, setConnection] = useState<"connected" | "reconnecting">("connected");
  const [conflict, setConflict] = useState<CanvasConflictView | null>(null);
  const [createServerCanvasPending, setCreateServerCanvasPending] = useState(false);

  useEffect(() => {
    if (source.mode !== "group" || !groupApi) {
      setGroupViewState({ mode: "loading" });
      return;
    }
    const worktreeId = source.worktreeId;
    const client = new GroupCanvasClient(groupApi, worktreeId);
    const controller = new AbortController();
    let active = true;

    const refresh = async (signal: AbortSignal) => {
      const nextSnapshot = await client.loadCanvas(signal);
      if (active && !signal.aborted) {
        setGroupViewState({ mode: "live", snapshot: nextSnapshot });
        setConnection("connected");
      }
    };

    setGroupViewState({ mode: "loading" });
    void refresh(controller.signal).catch((error: unknown) => {
      if (!active || controller.signal.aborted) return;
      setGroupViewState({ mode: "error", message: errorMessage(error) });
    });

    // 断线恢复: 复用 CanvasOutboxWakeSubscriber (SSE wake + cursor catch-up + 自动重连)
    const subscriber = client.makeWakeSubscriber((signal) => refresh(signal));
    const unsubscribe = subscriber.subscribe(controller.signal, (error: unknown) => {
      if (!active || controller.signal.aborted) return;
      if (error instanceof GroupApiError && (error.status === 401 || error.status === 403)) {
        setGroupViewState({ mode: "error", message: "Group 会话已失效, 请重新登录后继续。" });
        return;
      }
      if (error instanceof CanvasReadOnlyError) return; // 只读本身不是断线
      setConnection("reconnecting");
    });

    return () => {
      active = false;
      controller.abort();
      unsubscribe();
    };
  }, [source.mode, source.mode === "group" ? source.worktreeId : "", groupApi, canvasReloadKey]);

  const snapshot = groupViewState.mode === "live" ? groupViewState.snapshot : null;
  const serverCanvas = snapshot?.canvas ?? null;
  const canWrite = snapshot?.permission.canWrite ?? false;
  const canCreateCanvas = snapshot?.permission.canCreateCanvas ?? false;

  const refreshGroup = useCallback(() => setCanvasReloadKey((k) => k + 1), []);

  /** 冲突可见: 保存本地草稿, 拉取服务端最新, 双方同屏展示。 */
  const openConflict = useCallback(
    async (error: CanvasConflictError, draft: CanvasPendingDraft) => {
      if (!groupApi || source.mode !== "group") return;
      const client = new GroupCanvasClient(groupApi, source.worktreeId);
      const latest = await client.loadCanvas().catch(() => null);
      setConflict({ code: error.code, draft, latest });
    },
    [groupApi, source],
  );

  /** 新增元素 (与移动共用同一 canWrite 判定 + 同一授权写入路径)。 */
  const handleCreateElement = useCallback(
    async (input: {
      kind: CanvasElementKind;
      x: number;
      y: number;
      width: number;
      height: number;
      z_index: number;
      content: Record<string, unknown>;
    }): Promise<{ id?: string } | void> => {
      if (!groupApi || source.mode !== "group" || !serverCanvas || !canWrite) {
        throw new Error("当前画布不可写");
      }
      const client = new GroupCanvasClient(groupApi, source.worktreeId);
      const draft: CanvasPendingDraft = {
        kind: "element-create",
        payload: input,
        savedAt: new Date().toISOString(),
      };
      CanvasDraftStore.save(source.worktreeId, serverCanvas.canvasId, draft);
      try {
        const { id: elementId } = await client.createElement(
          serverCanvas.canvasId,
          input,
          crypto.randomUUID(),
        );
        CanvasDraftStore.clear(source.worktreeId, serverCanvas.canvasId);
        refreshGroup();
        return { id: elementId };
      } catch (error) {
        if (error instanceof CanvasConflictError) await openConflict(error, draft);
        throw error;
      }
    },
    [canWrite, groupApi, openConflict, refreshGroup, serverCanvas, source],
  );

  /** 移动元素: 乐观预览由 CanvasView 处理, 服务端确认走 PUT position。 */
  const handleElementPositionChange = useCallback(
    async (position: { elementId: string; x: number; y: number; expectedVersion: number }) => {
      if (!groupApi || source.mode !== "group" || !serverCanvas || !canWrite) {
        throw new Error("当前画布不可写");
      }
      const client = new GroupCanvasClient(groupApi, source.worktreeId);
      const draft: CanvasPendingDraft = {
        kind: "element-move",
        elementId: position.elementId,
        payload: position,
        savedAt: new Date().toISOString(),
      };
      CanvasDraftStore.save(source.worktreeId, serverCanvas.canvasId, draft);
      try {
        await client.updateElementPosition(
          serverCanvas.canvasId,
          position.elementId,
          position,
          crypto.randomUUID(),
        );
        CanvasDraftStore.clear(source.worktreeId, serverCanvas.canvasId);
        refreshGroup();
      } catch (error) {
        if (error instanceof CanvasConflictError) await openConflict(error, draft);
        throw error;
      }
    },
    [canWrite, groupApi, openConflict, refreshGroup, serverCanvas, source],
  );

  /** 删除元素: 同一授权写入路径。 */
  const handleDeleteElements = useCallback(
    async (elementIds: string[]) => {
      if (!groupApi || source.mode !== "group" || !serverCanvas || !canWrite) {
        throw new Error("当前画布不可写");
      }
      const client = new GroupCanvasClient(groupApi, source.worktreeId);
      for (const elementId of elementIds) {
        const element = snapshot?.elements.find((el) => el.id === elementId);
        const draft: CanvasPendingDraft = {
          kind: "element-delete",
          elementId,
          payload: { expectedVersion: element?.version ?? 0 },
          savedAt: new Date().toISOString(),
        };
        CanvasDraftStore.save(source.worktreeId, serverCanvas.canvasId, draft);
        try {
          await client.deleteElement(
            serverCanvas.canvasId,
            elementId,
            element?.version ?? 0,
            crypto.randomUUID(),
          );
          CanvasDraftStore.clear(source.worktreeId, serverCanvas.canvasId);
        } catch (error) {
          if (error instanceof CanvasConflictError) await openConflict(error, draft);
          throw error;
        }
      }
      refreshGroup();
    },
    [canWrite, groupApi, openConflict, refreshGroup, serverCanvas, snapshot, source],
  );

  /** 服务端画布创建 (仅 can_create_canvas)。 */
  const handleCreateServerCanvas = useCallback(async () => {
    if (!groupApi || source.mode !== "group" || !canCreateCanvas || createServerCanvasPending) return;
    setCreateServerCanvasPending(true);
    try {
      const client = new GroupCanvasClient(groupApi, source.worktreeId);
      await client.createCanvas("Worktree Canvas", crypto.randomUUID());
      refreshGroup();
    } catch {
      setScopeNotice("服务端画布创建失败, 请重试。");
    } finally {
      setCreateServerCanvasPending(false);
    }
  }, [canCreateCanvas, createServerCanvasPending, groupApi, refreshGroup, source]);

  /** 冲突对话框: 使用服务端版本 → 清草稿并刷新。 */
  const resolveConflictKeepServer = useCallback(() => {
    if (!conflict || source.mode !== "group" || !serverCanvas) return;
    CanvasDraftStore.clear(source.worktreeId, serverCanvas.canvasId);
    setConflict(null);
    refreshGroup();
  }, [conflict, refreshGroup, serverCanvas, source]);

  // ===================================================================
  // BI / Plugins / Workflow 入口 (per 2026-11-16 范围选择器需求):
  //   点击时读取 navStore 当前选中范围 → 传给后端做权限复验。
  //   ── 后端接入点 (授权投影接入后替换 TODO) ─────────────────────
  //     POST /api/v1/canvas/{canvas.id}/entries/{entry}/open
  //       body: { scope: { project_id, lane_id, worktree_id } }
  //     后端按 RLS (tenant/project/lane/worktree) 校验后返回授权 URL
  //     或数据; 未授权返回 403, 前端渲染拒绝态 (不打开入口)。
  //   ── 本地预览行为 (无后端) ─────────────────────────────────────
  //     workflow: 直接切"流程"tab; bi/plugins: 仅展示已读取的范围提示。
  // ===================================================================
  const openScopedEntry = useCallback(
    (entry: "bi" | "plugins" | "workflow") => {
      const nav = useNavStore.getState();
      const scope = {
        project_id: nav.selectedProjectId,
        lane_id: nav.selectedLaneId,
        worktree_id: nav.selectedWorktreeId,
      };
      // TODO(backend): await fetch(`/api/v1/canvas/${canvas?.id}/entries/${entry}/open`,
      //   { method: "POST", headers: { "Content-Type": "application/json" },
      //     body: JSON.stringify({ scope }) }) → 按授权结果跳转/渲染拒绝态
      setScopeNotice(
        `${entry.toUpperCase()} 入口 · 已按范围 project=${scope.project_id || "-"} lane=${scope.lane_id || "-"} worktree=${scope.worktree_id || "-"} 请求后端权限复验 (本地预览未接入后端)`,
      );
      if (entry === "workflow") setActiveTab("流程");
    },
    [canvas?.id],
  );

  // ===================================================================
  // 画布 tab: group 模式 (授权真源) / local 模式 (store 降级)
  // ===================================================================
  const renderWhiteboard = () => {
    if (source.mode === "group") {
      if (groupViewState.mode === "loading") {
        return (
          <div className="h-full grid place-items-center" data-testid="canvas-group-loading">
            <p className="text-sm text-ink-mute">{text.loading}</p>
          </div>
        );
      }
      if (groupViewState.mode === "error") {
        return (
          <div className="h-full flex flex-col items-center justify-center gap-3" data-testid="canvas-group-error">
            <p className="text-sm text-err" data-testid="canvas-group-error-hint">
              {text.loadError(groupViewState.message)}
            </p>
            <button
              type="button"
              data-testid="canvas-group-retry"
              onClick={refreshGroup}
              className="rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-xs font-semibold text-accent hover:bg-accent/20"
            >
              {text.retry}
            </button>
          </div>
        );
      }
      // live: 服务端画布
      if (!snapshot?.canvas) {
        return (
          <div className="h-full flex flex-col items-center justify-center gap-3" data-testid="canvas-group-no-canvas">
            <p className="text-sm text-ink-mute" data-testid="canvas-group-no-canvas-hint">
              {text.noServerCanvas}
            </p>
            {canCreateCanvas && (
              <button
                type="button"
                data-testid="canvas-group-create-canvas"
                disabled={createServerCanvasPending}
                onClick={() => void handleCreateServerCanvas()}
                className="flex items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-xs font-semibold text-accent hover:bg-accent/20 disabled:opacity-50"
              >
                <Plus size={14} /> {createServerCanvasPending ? text.creatingCanvas : text.createServerCanvas}
              </button>
            )}
          </div>
        );
      }
      const viewElements = snapshot.elements.map((el) => ({
        ...el,
        entity_ref: el.entity_ref
          ? ({
              ref_type: el.entity_ref.ref_type as CanvasEntityRef["ref_type"],
              ref_id: el.entity_ref.ref_id,
              worktree_id: el.entity_ref.worktree_id,
            } satisfies CanvasEntityRef)
          : undefined,
      }));
      return (
        <div className="relative h-full overflow-hidden" data-testid="canvas-tab-whiteboard">
          <CanvasView
            key={`group:${snapshot.canvas.canvasId}:${snapshot.canvas.version}`}
            canvas={{
              id: snapshot.canvas.canvasId,
              viewport: snapshot.canvas.viewport,
              frames: snapshot.canvas.frames,
            }}
            elements={viewElements}
            connectors={snapshot.canvas.connectors}
            readOnly={!canWrite}
            groupWorkItems={workItems.length > 0 ? workItems : undefined}
            groupWorktrees={worktrees.length > 0 ? worktrees : undefined}
            onCreateElement={canWrite ? handleCreateElement : undefined}
            onElementPositionChange={canWrite ? handleElementPositionChange : undefined}
            onDeleteElements={canWrite ? handleDeleteElements : undefined}
          />
          {!canWrite && (
            <p
              className="absolute bottom-10 left-3 z-20 rounded border border-warn/40 bg-warn/10 px-2 py-1 font-mono text-[10px] text-warn"
              data-testid="canvas-group-readonly-hint"
            >
              {text.readOnlyHint}
            </p>
          )}
          {conflict && (
            <div
              className="absolute inset-0 z-40 grid place-items-center bg-bg/70 p-6"
              data-testid="canvas-conflict-dialog"
              role="dialog"
              aria-label={text.conflictTitle}
            >
              <div className="w-full max-w-lg rounded-xl border border-warn/40 bg-bg-card p-4 shadow-soft">
                <h3 className="mb-2 text-sm font-semibold text-warn">{text.conflictTitle}</h3>
                <p className="mb-3 text-xs text-ink-mute">{text.conflictBody}</p>
                <div className="mb-3 grid grid-cols-2 gap-2 text-[11px]">
                  <div className="rounded border border-line p-2" data-testid="canvas-conflict-server">
                    <p className="mb-1 font-mono text-ink-mute">服务端最新</p>
                    <p className="break-all font-mono text-ink">
                      {conflict.code} · {conflict.latest ? `version=${conflict.latest.canvas?.version}` : "不可达"}
                    </p>
                  </div>
                  <div className="rounded border border-line p-2" data-testid="canvas-conflict-draft">
                    <p className="mb-1 font-mono text-ink-mute">本地草稿</p>
                    <p className="break-all font-mono text-ink">
                      {conflict.draft.kind} {conflict.draft.elementId ?? ""} · {conflict.draft.savedAt.slice(0, 19)}
                    </p>
                  </div>
                </div>
                <div className="flex justify-end gap-2">
                  <button
                    type="button"
                    data-testid="canvas-conflict-keep-server"
                    onClick={resolveConflictKeepServer}
                    className="rounded border border-line bg-bg-soft px-3 py-1.5 text-xs text-ink hover:border-accent hover:text-accent"
                  >
                    {text.keepServer}
                  </button>
                  <button
                    type="button"
                    data-testid="canvas-conflict-keep-both"
                    onClick={() => setConflict(null)}
                    className="rounded border border-accent/40 bg-accent/10 px-3 py-1.5 text-xs font-semibold text-accent hover:bg-accent/20"
                  >
                    {text.keepBoth}
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      );
    }
    // local 模式 (无 group 会话): store 通道降级, 明确标注
    return (
      <div className="relative h-full overflow-hidden" data-testid="canvas-tab-whiteboard">
        {canvas ? (
          <>
            <CanvasView
              canvas={canvas}
              elements={canvasElements}
              connectors={canvasConnectors}
              readOnly={false}
              groupWorkItems={workItems}
              groupWorktrees={worktrees}
              onCreateElement={async (input) => {
                const el = {
                  id: `el-${globalThis.crypto.randomUUID()}`,
                  canvas_id: canvas.id,
                  kind: input.kind,
                  x: input.x,
                  y: input.y,
                  width: input.width,
                  height: input.height,
                  rotation: 0,
                  z_index: input.z_index,
                  content: input.content,
                  locked: false,
                  hidden: false,
                  created_by: "usr-001",
                  created_at: new Date().toISOString(),
                  updated_at: new Date().toISOString(),
                };
                useStore.getState().addCanvasElement(el);
                return { id: el.id };
              }}
            />
            <span
              className="absolute bottom-3 left-3 z-20 rounded border border-warn/40 bg-warn/10 px-1.5 py-0.5 font-mono text-[9px] text-warn"
              data-testid="canvas-local-mode-badge"
            >
              {text.localBadge}
            </span>
          </>
        ) : (
          <div
            className="h-full flex flex-col items-center justify-center gap-3"
            data-testid="canvas-unbound"
          >
            <p className="text-sm text-ink-mute" data-testid="canvas-unbound-hint">
              画布未接入 — 找不到 {id} 对应的画布 (不再回退 seed 默认画布)
            </p>
            <button
              type="button"
              data-testid="canvas-unbound-create"
              onClick={() =>
                createCanvasLocal({ id, title: `画布 ${id}`, ref_kind: "free" })
              }
              className="flex items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-xs font-semibold text-accent transition-colors hover:bg-accent/20"
            >
              <Plus size={14} /> 新建画布
            </button>
          </div>
        )}
      </div>
    );
  };

  const renderCanvasContent = () => {
    switch (activeTab) {
      case "画布":
        // 无限画布核心: group 模式接授权真源, local 模式 store 通道降级
        return renderWhiteboard();
      case "Agent 画布":
        return (
          <div className="h-full overflow-hidden" data-testid="canvas-tab-agent-canvas">
            <AgentViewPage variant="canvas-only" />
          </div>
        );
      case "关系图":
        return (
          <div className="h-full overflow-y-auto p-4" data-testid="canvas-tab-relation">
            <div className="mb-3 flex items-center justify-end">
              <a
                href="/canvas/worktree-graph"
                data-testid="canvas-relation-open-wtg"
                className="flex items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/10 px-2.5 py-1.5 text-[11px] font-semibold text-accent hover:bg-accent/20 transition-colors"
              >
                <Network size={12} /> Worktree 图 · 5 视图
              </a>
            </div>
            <RelationshipView tenantId={relationTenantId} />
          </div>
        );
      case "流程":
        return (
          <div className="h-full overflow-y-auto" data-testid="canvas-tab-flow">
            <WorkflowBuilder />
          </div>
        );
      case "Git 画布":
        return (
          <div className="h-full overflow-hidden" data-testid="canvas-tab-git">
            <GitCanvas />
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg" data-testid="canvas-detail-page">
      {/* Canvas 范围条 (per 2026-11-16: 三级范围选择器 + BI/Plugins/Workflow 权限复验入口) */}
      <div
        className="flex flex-wrap items-center gap-2 border-b border-line bg-bg-card px-3 py-1.5"
        data-testid="canvas-worktree-context"
      >
        <GitBranch size={13} className="shrink-0 text-accent" />
        <CanvasScopeSelector onScopeChange={handleScopeChange} />
        {source.mode === "group" ? (
          <span
            data-testid="canvas-worktree-data-scope"
            className="rounded border border-accent/40 bg-accent/10 px-1.5 py-0.5 font-mono text-[9px] text-accent"
          >
            {text.groupBadge(source.worktreeId)}
          </span>
        ) : (
          <span
            data-testid="canvas-worktree-data-scope"
            className="rounded border border-warn/40 bg-warn/10 px-1.5 py-0.5 font-mono text-[9px] text-warn"
          >
            {text.localBadge}
          </span>
        )}
        {source.mode === "group" && connection === "reconnecting" && (
          <span className="text-xs text-warn" data-testid="canvas-connection-reconnecting">
            {text.reconnecting}
          </span>
        )}
        {source.mode === "group" && snapshot?.permission.canWrite === false && (
          <span className="text-[10px] text-ink-mute" data-testid="canvas-group-readonly-bar">
            {text.readOnlyHint}
          </span>
        )}
        {scopeSyncError && (
          <span className="text-xs text-warn" data-testid="canvas-worktree-context-missing">
            Worktree 上下文未找到 / 未授权读取: {scopeSyncError}
          </span>
        )}
        <span className="ml-auto flex items-center gap-1">
          <button
            type="button"
            data-testid="canvas-entry-bi"
            onClick={() => openScopedEntry("bi")}
            className="rounded border border-line bg-bg-soft px-2 py-0.5 text-[11px] text-ink transition-colors hover:border-accent hover:text-accent"
          >
            BI
          </button>
          <button
            type="button"
            data-testid="canvas-entry-plugins"
            onClick={() => openScopedEntry("plugins")}
            className="rounded border border-line bg-bg-soft px-2 py-0.5 text-[11px] text-ink transition-colors hover:border-accent hover:text-accent"
          >
            Plugins
          </button>
          <button
            type="button"
            data-testid="canvas-entry-workflow"
            onClick={() => openScopedEntry("workflow")}
            className="rounded border border-line bg-bg-soft px-2 py-0.5 text-[11px] text-ink transition-colors hover:border-accent hover:text-accent"
          >
            Workflow
          </button>
        </span>
      </div>
      {scopeNotice && (
        <div
          data-testid="canvas-entry-scope-notice"
          className="border-b border-line bg-bg-soft px-3 py-1 font-mono text-[10px] text-ink-mute"
        >
          {scopeNotice}
        </div>
      )}

      {/* 无限画布标签切换 (无三选一, 直接呈现) */}
      <div className="flex items-center gap-1 border-b border-line bg-bg-card px-3" data-testid="canvas-merged-tabs">
        {CANVAS_SUB_TABS.map((tab) => (
          <button
            key={tab}
            type="button"
            onClick={() => setActiveTab(tab)}
            data-testid={`canvas-merged-tab-${tab}`}
            className={`px-3 py-2 text-sm border-b-2 -mb-px transition-colors ${
              activeTab === tab
                ? "border-accent text-accent font-semibold"
                : "border-transparent text-ink-dim hover:text-ink"
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      <main className="flex-1 overflow-hidden" key={canvasReloadKey}>
        <Suspense fallback={null}>{renderCanvasContent()}</Suspense>
      </main>
    </div>
  );
}

function errorMessage(error: unknown): string {
  if (error instanceof CanvasReadOnlyError) return "当前会话对画布只读。";
  if (error instanceof CanvasDisconnectedError) return "无法连接 Group Canvas 服务, 请检查网络。";
  return error instanceof Error ? error.message : "Group Canvas 请求失败。";
}
