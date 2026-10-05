/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/components/AuthenticatedGroupWorkspace.tsx",type:"file",language:"tsx"}),
  (appId:Class {name:"GroupAppId",type:"class"}),
  (loadState:Class {name:"LoadState",type:"class"}),
  (canvasState:Class {name:"CanvasLoadState",type:"class"}),
  (canvasApi:Variable {name:"canvasApi",type:"variable",language:"typescript"}),(canvasWriteGranted:Variable {name:"canvasWriteGranted",type:"variable",language:"typescript"}),(workItemWriteGranted:Variable {name:"workItemWriteGranted",type:"variable",language:"typescript"}),(workItemReadGranted:Variable {name:"workItemReadGranted",type:"variable",language:"typescript"}),(canCreateCanvasTask:Variable {name:"canCreateCanvasTask",type:"variable",language:"typescript"}),(canvasTaskTitle:Variable {name:"canvasTaskTitle",type:"variable",language:"typescript"}),(canvasTaskBusy:Variable {name:"canvasTaskBusy",type:"variable",language:"typescript"}),(canvasTaskFormOpen:Variable {name:"canvasTaskFormOpen",type:"variable",language:"typescript"}),(viewportSaveTimer:Variable {name:"viewportSaveTimer",type:"variable",language:"typescript"}),(pendingViewport:Variable {name:"pendingViewport",type:"variable",language:"typescript"}),
  (projectWriterRole:Variable {name:"projectWriterRole",type:"variable",language:"typescript"}),
  (apps:Variable {name:"GROUP_APPS",type:"variable",language:"typescript"}),
  (isApp:Function {name:"isGroupAppId",type:"function",signature:"isGroupAppId(value: string | null): value is GroupAppId",visibility:"private",complexity:"simple"}),
  (viewElement:Function {name:"toCanvasElement",type:"function",signature:"toCanvasElement(canvasId: string, element: GroupCanvasElement): CanvasElement",visibility:"private",complexity:"simple"}),
  (errorCode:Function {name:"apiErrorMessage",type:"function",signature:"apiErrorMessage(error: unknown): string",visibility:"private",complexity:"simple"}),
  (appHref:Function {name:"appHref",type:"function",signature:"appHref(app: GroupAppId, extra?: Record<string, string>): string",visibility:"private",complexity:"simple"}),
  (component:Function {name:"AuthenticatedGroupWorkspace",type:"function",signature:"AuthenticatedGroupWorkspace(props: { worktreeId: string }): JSX.Element",visibility:"public",complexity:"complex"}),
  (contextEffect:Function {name:"loadGroupContext",type:"function",signature:"loadGroupContext(): () => void",visibility:"private",complexity:"moderate"}),
  (tasksEffect:Function {name:"loadWorkItems",type:"function",signature:"loadWorkItems(): () => void",visibility:"private",complexity:"moderate"}),
  (canvasEffect:Function {name:"loadCanvas",type:"function",signature:"loadCanvas(): () => void",visibility:"private",complexity:"complex"}),
  (saveViewport:Function {name:"saveCanvasViewport",type:"function",signature:"saveCanvasViewport(viewport: CanvasViewport): void",visibility:"private",complexity:"moderate"}),
  (createCanvasAction:Function {name:"createGroupCanvas",type:"function",signature:"createGroupCanvas(): Promise<void>",visibility:"private",complexity:"simple"}),
  (createCanvasTask:Function {name:"createCanvasWorkItem",type:"function",signature:"createCanvasWorkItem(): Promise<void>",visibility:"private",complexity:"moderate"}),
  (scopeChange:Function {name:"changeScope",type:"function",signature:"changeScope(scope: ChatScope): void",visibility:"private",complexity:"simple"}),
  (taskLink:Function {name:"taskHref",type:"function",signature:"taskHref(workItemId?: string): string",visibility:"private",complexity:"simple"}),
  (openTask:Function {name:"openWorkItem",type:"function",signature:"openWorkItem(workItemId: string, referencedWorktreeId?: string): void",visibility:"private",complexity:"moderate"}),
  (file)-[:CONTAINS]->(appId),(file)-[:CONTAINS]->(loadState),(file)-[:CONTAINS]->(canvasState),(file)-[:CONTAINS]->(canvasApi),(file)-[:CONTAINS]->(canvasWriteGranted),(file)-[:CONTAINS]->(workItemWriteGranted),(file)-[:CONTAINS]->(workItemReadGranted),(file)-[:CONTAINS]->(canCreateCanvasTask),(file)-[:CONTAINS]->(canvasTaskTitle),(file)-[:CONTAINS]->(canvasTaskBusy),(file)-[:CONTAINS]->(canvasTaskFormOpen),(file)-[:CONTAINS]->(viewportSaveTimer),(file)-[:CONTAINS]->(pendingViewport),(file)-[:CONTAINS]->(projectWriterRole),
  (file)-[:CONTAINS]->(apps),(file)-[:CONTAINS]->(isApp),(file)-[:CONTAINS]->(viewElement),(file)-[:CONTAINS]->(errorCode),
  (file)-[:CONTAINS]->(component),(component)-[:HAS_METHOD]->(contextEffect),(component)-[:HAS_METHOD]->(tasksEffect),(component)-[:CONTAINS]->(appHref),(component)-[:HAS_METHOD]->(appHref),
  (component)-[:HAS_METHOD]->(canvasEffect),(component)-[:HAS_METHOD]->(saveViewport),(component)-[:HAS_METHOD]->(createCanvasAction),(component)-[:HAS_METHOD]->(createCanvasTask),(component)-[:HAS_METHOD]->(scopeChange),
  (component)-[:HAS_METHOD]->(taskLink),(component)-[:HAS_METHOD]->(openTask),
  (component)-[:CALLS]->(isApp),(component)-[:CALLS]->(viewElement),(component)-[:CALLS]->(errorCode),
  (component)-[:USES]->(apps),(contextEffect)-[:CALLS]->(errorCode),(tasksEffect)-[:CALLS]->(errorCode),
  (canvasEffect)-[:CALLS]->(errorCode),(canvasEffect)-[:CALLS]->(viewElement),(saveViewport)-[:CALLS]->(canvasApi),(saveViewport)-[:USES]->(canvasWriteGranted),(saveViewport)-[:USES]->(viewportSaveTimer),(saveViewport)-[:USES]->(pendingViewport),(createCanvasAction)-[:CALLS]->(canvasApi),(createCanvasAction)-[:CALLS]->(appHref),(createCanvasAction)-[:CALLS]->(errorCode),(createCanvasTask)-[:CALLS]->(canvasApi),(createCanvasTask)-[:CALLS]->(viewElement),(createCanvasTask)-[:CALLS]->(errorCode),(createCanvasTask)-[:USES]->(canvasWriteGranted),(createCanvasTask)-[:USES]->(workItemWriteGranted),(createCanvasTask)-[:USES]->(workItemReadGranted),(createCanvasTask)-[:USES]->(canvasTaskTitle),
  (openTask)-[:CALLS]->(taskLink),(scopeChange)-[:USES]->(taskLink);
*/

"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import {
  Blocks,
  ClipboardList,
  GitBranch,
  LayoutGrid,
  Plug,
  SquareKanban,
  Workflow,
  type LucideIcon,
} from "lucide-react";

import { CanvasView, type CanvasViewCanvas, type CanvasViewWorkItemSummary, type CanvasViewWorktreeSummary } from "@/components/CanvasView";
import {
  GroupApiError,
  GroupContextApiClient,
  type GroupWorkItemProjection,
  type GroupWorktreeContextResponse,
} from "@/lib/group-api/group-context";
import {
  GroupCanvasApiClient,
  GroupCanvasApiError,
  type GroupCanvasElement,
  type GroupCanvasRow,
} from "@/lib/group-api/canvas";
import type { CanvasElement, CanvasViewport } from "@/types/ids";

type GroupAppId = "multica" | "jira" | "task-card" | "canvas" | "workflow" | "plugins";
type ChatScope = "WORKTREE" | "GLOBAL";

type LoadState<T> =
  | { status: "loading" }
  | { status: "ready"; data: T }
  | { status: "error"; message: string };

type CanvasLoadState =
  | { status: "idle" | "loading" }
  | { status: "ready"; canvas: GroupCanvasRow | null; elements: GroupCanvasElement[] }
  | { status: "error"; message: string };

const GROUP_APPS: Array<{ id: GroupAppId; label: string; icon: LucideIcon }> = [
  { id: "multica", label: "Multica", icon: Blocks },
  { id: "jira", label: "Jira 视图", icon: SquareKanban },
  { id: "task-card", label: "Task Cards", icon: ClipboardList },
  { id: "canvas", label: "Infinite Canvas", icon: LayoutGrid },
  { id: "workflow", label: "Workflow", icon: Workflow },
  { id: "plugins", label: "Plugins", icon: Plug },
];

function isGroupAppId(value: string | null): value is GroupAppId {
  return GROUP_APPS.some((app) => app.id === value);
}

function toCanvasElement(canvasId: string, element: GroupCanvasElement): CanvasElement {
  return {
    id: element.element_id,
    canvas_id: canvasId,
    kind: element.kind,
    x: element.x,
    y: element.y,
    width: element.width,
    height: element.height,
    rotation: element.rotation,
    z_index: element.z_index,
    content: element.content,
    locked: element.locked,
    hidden: element.hidden,
    created_by: element.created_by,
    created_at: element.created_at,
    updated_at: element.updated_at,
    ...(element.entity_ref ? { entity_ref: element.entity_ref } : {}),
  };
}

function apiErrorMessage(error: unknown): string {
  if (error instanceof GroupApiError || error instanceof GroupCanvasApiError) {
    if (error.status === 401) return "需要有效的登录会话；页面没有回退到本地 seed。";
    if (error.status === 403) return "当前身份没有读取此 Worktree 或应用的权限。";
    if (error.status === 404) return "Worktree 不存在，或当前身份无权访问。";
    return `Group API 请求失败：${error.code}`;
  }
  return "无法连接 Group API；请检查登录会话和服务状态。";
}

export function AuthenticatedGroupWorkspace({ worktreeId }: { worktreeId: string }) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const contextApi = useMemo(() => new GroupContextApiClient(), []);
  const canvasApi = useMemo(() => new GroupCanvasApiClient(), []);
  const [contextState, setContextState] = useState<LoadState<GroupWorktreeContextResponse>>({ status: "loading" });
  const grantedScopes = contextState.status === "ready" ? contextState.data.group_context.granted_scopes : [];
  const projectWriterRole = contextState.status === "ready" && contextState.data.group_context.actor_roles.some((role) => ["tenant_admin", "project_admin", "developer", "agent"].includes(role));
  const canvasWriteGranted = projectWriterRole && grantedScopes.includes("canvas:write");
  const workItemWriteGranted = grantedScopes.includes("work-item:write");
  const workItemReadGranted = grantedScopes.includes("work-item:read");
  const [taskState, setTaskState] = useState<LoadState<GroupWorkItemProjection[]>>({ status: "loading" });
  const [canvasState, setCanvasState] = useState<CanvasLoadState>({ status: "idle" });
  const [canvasLinkError, setCanvasLinkError] = useState<string | null>(null);
  const [canvasCreateBusy, setCanvasCreateBusy] = useState(false);
  const [canvasTaskTitle, setCanvasTaskTitle] = useState("");
  const [canvasTaskBusy, setCanvasTaskBusy] = useState(false);
  const [canvasTaskFormOpen, setCanvasTaskFormOpen] = useState(false);
  const canvasStateRef = useRef(canvasState);
  canvasStateRef.current = canvasState;
  const viewportSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingViewport = useRef<CanvasViewport | null>(null);
  const appParam = searchParams.get("app");
  const currentApp: GroupAppId = isGroupAppId(appParam) ? appParam : "multica";
  const scope: ChatScope = searchParams.get("scope") === "GLOBAL" ? "GLOBAL" : "WORKTREE";
  const requestedTaskId = searchParams.get("work_item_id");
  const activeCanvasId = searchParams.get("canvas_id");

  useEffect(function loadGroupContext() {
    const controller = new AbortController();
    setContextState({ status: "loading" });
    void contextApi.resolveWorktreeContext(worktreeId, controller.signal)
      .then((data) => setContextState({ status: "ready", data }))
      .catch((error: unknown) => {
        if (!controller.signal.aborted) setContextState({ status: "error", message: apiErrorMessage(error) });
      });
    return () => controller.abort();
  }, [contextApi, worktreeId]);

  useEffect(function loadWorkItems() {
    if (contextState.status !== "ready") return;
    const controller = new AbortController();
    setTaskState({ status: "loading" });
    void contextApi.listWorkItems(worktreeId, controller.signal)
      .then((response) => setTaskState({ status: "ready", data: response.work_items }))
      .catch((error: unknown) => {
        if (!controller.signal.aborted) setTaskState({ status: "error", message: apiErrorMessage(error) });
      });
    return () => controller.abort();
  }, [contextApi, contextState.status, worktreeId]);

  useEffect(function loadCanvas() {
    if (contextState.status !== "ready" || currentApp !== "canvas") return;
    const controller = new AbortController();
    setCanvasState({ status: "loading" });
    void canvasApi.listCanvases(worktreeId, controller.signal)
      .then(async (response) => {
        const canvas = activeCanvasId
          ? response.canvases.find((item) => item.canvas_id === activeCanvasId) ?? null
          : response.canvases[0] ?? null;
        if (!canvas) return { canvas: null, elements: [] as GroupCanvasElement[] };
        const elementResponse = await canvasApi.listElements(worktreeId, canvas.canvas_id, controller.signal);
        return { canvas, elements: elementResponse.elements };
      })
      .then((data) => {
        if (!controller.signal.aborted) setCanvasState({ status: "ready", ...data });
      })
      .catch((error: unknown) => {
        if (!controller.signal.aborted) setCanvasState({ status: "error", message: apiErrorMessage(error) });
      });
    return () => controller.abort();
  }, [activeCanvasId, canvasApi, contextState.status, currentApp, worktreeId]);

  useEffect(() => () => {
    if (viewportSaveTimer.current) clearTimeout(viewportSaveTimer.current);
  }, []);

  const saveCanvasViewport = useCallback((viewport: CanvasViewport) => {
    const current = canvasStateRef.current;
    if (
      current.status !== "ready" ||
      !current.canvas ||
      !canvasWriteGranted
    ) return;
    pendingViewport.current = viewport;
    if (viewportSaveTimer.current) clearTimeout(viewportSaveTimer.current);
    viewportSaveTimer.current = setTimeout(() => {
      const latest = canvasStateRef.current;
      const nextViewport = pendingViewport.current;
      if (latest.status !== "ready" || !latest.canvas || !nextViewport) return;
      const canvas = latest.canvas;
      if (
        canvas.document.viewport.x === nextViewport.x &&
        canvas.document.viewport.y === nextViewport.y &&
        canvas.document.viewport.zoom === nextViewport.zoom
      ) return;

      void canvasApi.updateDocument(worktreeId, canvas.canvas_id, {
        expected_version: canvas.document.version,
        viewport: nextViewport,
        frames: canvas.document.frames,
        connectors: canvas.document.connectors,
        correlation_id: crypto.randomUUID(),
      }, {
        idempotencyKey: crypto.randomUUID(),
      }).then((response) => {
        setCanvasState((previous) => {
          if (previous.status !== "ready" || previous.canvas?.canvas_id !== response.canvas_id) return previous;
          return {
            ...previous,
            canvas: { ...previous.canvas, document: response.document },
          };
        });
        setCanvasLinkError(null);
      }).catch((error: unknown) => {
        const message = error instanceof GroupCanvasApiError && error.code === "canvas_document_version_conflict"
          ? "Canvas 文档已被其他成员更新；请刷新后继续，未覆盖对方的版本。"
          : apiErrorMessage(error);
        setCanvasLinkError(message);
      });
    }, 450);
  }, [canvasApi, canvasWriteGranted, worktreeId]);

  const context = contextState.status === "ready" ? contextState.data : null;
  const worktree = context?.worktree;
  const canWriteCanvas = canvasWriteGranted;
  const canCreateCanvasTask = canvasWriteGranted && workItemWriteGranted && workItemReadGranted;
  const workItems = taskState.status === "ready" ? taskState.data : [];
  const selectedTask = requestedTaskId ? workItems.find((item) => item.work_item_id === requestedTaskId) : undefined;
  const taskLinkUnavailable = Boolean(requestedTaskId) && taskState.status === "ready" && selectedTask === undefined;
  const cliRequested = searchParams.get("cli") === "1" && selectedTask !== undefined;
  const basePath = `/worktree/${encodeURIComponent(worktreeId)}/group`;

  const appHref = useCallback((app: GroupAppId, extra: Record<string, string> = {}) => {
    const query = new URLSearchParams({ app, scope, ...extra });
    return `${basePath}?${query.toString()}`;
  }, [basePath, scope]);

  const taskHref = useCallback((workItemId?: string) => {
    const query = new URLSearchParams({ app: "task-card", scope });
    if (workItemId) query.set("work_item_id", workItemId);
    return `${basePath}?${query.toString()}`;
  }, [basePath, scope]);

  const changeScope = useCallback((nextScope: ChatScope) => {
    const query = new URLSearchParams(searchParams.toString());
    query.set("scope", nextScope);
    router.replace(`${basePath}?${query.toString()}`, { scroll: false });
  }, [basePath, router, searchParams]);

  const createGroupCanvas = useCallback(async function createGroupCanvas() {
    if (!worktree) return;
    setCanvasCreateBusy(true);
    setCanvasLinkError(null);
    try {
      const response = await canvasApi.createCanvas(worktreeId, `${worktree.name} Canvas`, {
        idempotencyKey: crypto.randomUUID(),
        correlationId: crypto.randomUUID(),
      });
      router.push(appHref("canvas", { canvas_id: response.canvas.canvas_id }));
    } catch (error) {
      setCanvasLinkError(apiErrorMessage(error));
    } finally {
      setCanvasCreateBusy(false);
    }
  }, [appHref, canvasApi, router, worktree, worktreeId]);

  const createCanvasWorkItem = useCallback(async function createCanvasWorkItem() {
    const current = canvasStateRef.current;
    const title = canvasTaskTitle.trim();
    if (
      current.status !== "ready" ||
      !current.canvas ||
      !title ||
      !canvasWriteGranted ||
      !workItemWriteGranted ||
      !workItemReadGranted
    ) return;

    const currentCanvasId = current.canvas.canvas_id;
    setCanvasTaskBusy(true);
    setCanvasLinkError(null);
    const correlationId = crypto.randomUUID();
    try {
      const response = await canvasApi.createWorkItem(worktreeId, currentCanvasId, {
        work_item: {
          item_type: "task",
          title,
          description: "",
          priority: "medium",
          labels: [],
        },
        x: current.canvas.document.viewport.x + 600 / current.canvas.document.viewport.zoom,
        y: current.canvas.document.viewport.y + 400 / current.canvas.document.viewport.zoom,
        width: 260,
        height: 120,
        z_index: current.elements.reduce((top, element) => Math.max(top, element.z_index), 0) + 1,
      }, {
        idempotencyKey: crypto.randomUUID(),
        correlationId,
      });
      void contextApi.listWorkItems(worktreeId)
        .then((workItemsResponse) => setTaskState({ status: "ready", data: workItemsResponse.work_items }))
        .catch((error: unknown) => setTaskState({ status: "error", message: apiErrorMessage(error) }));
      setCanvasState((previous) => {
        if (previous.status !== "ready" || previous.canvas?.canvas_id !== currentCanvasId) return previous;
        if (previous.elements.some((element) => element.element_id === response.element.element_id)) return previous;
        return { ...previous, elements: [...previous.elements, response.element] };
      });
      setCanvasTaskTitle("");
      setCanvasTaskFormOpen(false);
    } catch (error) {
      setCanvasLinkError(apiErrorMessage(error));
    } finally {
      setCanvasTaskBusy(false);
    }
  }, [canvasApi, canvasTaskTitle, canvasWriteGranted, contextApi, workItemReadGranted, workItemWriteGranted, worktreeId]);

  const openWorkItem = useCallback((workItemId: string, referencedWorktreeId?: string) => {
    const isAuthorizedHere = worktreeId === referencedWorktreeId &&
      workItems.some((item) => item.work_item_id === workItemId);
    if (!isAuthorizedHere) {
      setCanvasLinkError("此引用不属于当前授权 Worktree，已阻止跳转。");
      return;
    }
    setCanvasLinkError(null);
    router.push(taskHref(workItemId));
  }, [router, taskHref, workItems, worktreeId]);

  if (contextState.status === "loading") {
    return <main className="mx-auto w-full max-w-5xl p-6"><section className="card text-sm text-ink-dim">正在验证 Worktree Group 权限…</section></main>;
  }
  if (contextState.status === "error") {
    return (
      <main className="mx-auto w-full max-w-5xl p-6">
        <section className="card space-y-3" role="alert" data-testid="group-api-error">
          <h1 className="text-lg font-semibold">无法打开 Worktree Group</h1>
          <p className="text-sm text-warning">{contextState.message}</p>
          <p className="font-mono text-[10px] text-ink-mute">worktree_id: {worktreeId}</p>
          <Link href="/worktree" className="btn w-fit">返回 Worktree Index</Link>
        </section>
      </main>
    );
  }
  if (!context || !worktree) return null;

  const viewCanvas: CanvasViewCanvas = canvasState.status === "ready" && canvasState.canvas
    ? {
        viewport: canvasState.canvas.document.viewport,
        frames: canvasState.canvas.document.frames.map((frame) => ({
          ...frame,
          canvas_id: canvasState.canvas!.canvas_id,
        })),
      }
    : { viewport: { x: 0, y: 0, zoom: 1 }, frames: [] };
  const viewElements = canvasState.status === "ready" && canvasState.canvas
    ? canvasState.elements.map((element) => toCanvasElement(canvasState.canvas!.canvas_id, element))
    : [];
  const viewWorkItems: CanvasViewWorkItemSummary[] = workItems.map((item) => ({
    id: item.work_item_id,
    key: item.work_item_id.slice(0, 8),
    title: item.title,
    status: item.lifecycle.status,
  }));
  const viewWorktrees: CanvasViewWorktreeSummary[] = [{
    id: worktree.id,
    branch: worktree.branch,
    status: `${worktree.human_state} · ${worktree.machine_state}`,
  }];

  return (
    <main className="flex min-h-[calc(100vh-3rem)] flex-col pb-28 md:pb-20" data-testid="authenticated-group-workspace">
      <header className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-4 py-3">
        <div className="min-w-0">
          <h1 className="truncate text-lg font-semibold">{worktree.name}</h1>
          <p className="mt-1 flex flex-wrap gap-x-2 text-[10px] text-ink-mute">
            <span>{worktreeId}</span><span>·</span><span>{worktree.branch}</span>
            <span>· Project {worktree.project_id}</span>
            <span>· {worktree.human_state} / {worktree.machine_state}</span>
          </p>
        </div>
        <Link href={`/worktree?project_id=${encodeURIComponent(worktree.project_id)}`} className="btn text-xs">Project Worktree Index</Link>
      </header>

      <div className="grid min-h-0 flex-1 md:grid-cols-[13rem_minmax(0,1fr)]">
        <nav aria-label="Worktree Group 应用" className="border-b border-line p-2 md:border-b-0 md:border-r">
          <div className="px-2 py-2 text-[10px] font-semibold uppercase tracking-widest text-ink-mute">{worktree.name} · 同级应用</div>
          <div className="grid grid-cols-3 gap-1 md:grid-cols-1">
            {GROUP_APPS.map((app) => {
              const Icon = app.icon;
              return (
                <Link key={app.id} href={appHref(app.id)} aria-current={currentApp === app.id ? "page" : undefined}
                  className={`flex items-center gap-2 rounded-md px-2 py-2 text-xs ${currentApp === app.id ? "bg-accent/10 text-accent" : "text-ink-dim hover:bg-bg-soft"}`}>
                  <Icon size={14} />{app.label}
                </Link>
              );
            })}
          </div>
        </nav>

        <section className="min-w-0 p-4" aria-label={`${currentApp} 应用内容`}>
          {currentApp === "canvas" && (
            <div className="space-y-3">
              <div className="flex flex-wrap items-end justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Infinite Canvas</h2><p className="text-xs text-ink-mute">读写当前 Worktree 授权的画布文档；Task Card 通过 canonical WorkItem 命令创建。</p></div>
                <div className="flex flex-wrap items-center gap-2">
                  {canvasState.status === "ready" && canvasState.canvas && canCreateCanvasTask && <button type="button" className="btn text-xs" onClick={() => setCanvasTaskFormOpen((open) => !open)}>在画布创建 Task Card</button>}
                  <span className="rounded border border-accent/30 bg-accent/5 px-2 py-1 text-[10px] text-accent">GroupContext · {context.group_context.permission_snapshot_ref}</span>
                </div>
              </div>
              {canvasLinkError && <p role="alert" className="rounded border border-warning/30 bg-warning/5 p-3 text-xs text-warning">{canvasLinkError}</p>}
              {canvasTaskFormOpen && canvasState.status === "ready" && canvasState.canvas && (
                <form className="card flex flex-wrap items-center gap-2 p-3" onSubmit={(event) => { event.preventDefault(); void createCanvasWorkItem(); }}>
                  <label htmlFor="canvas-task-title" className="text-xs text-ink-dim">任务标题</label>
                  <input id="canvas-task-title" autoFocus maxLength={200} required value={canvasTaskTitle} onChange={(event) => setCanvasTaskTitle(event.target.value)} className="min-w-[220px] flex-1 rounded-md border border-line bg-bg-soft px-3 py-2 text-xs" placeholder="输入新的 Task Card 标题" />
                  <button type="submit" className="btn-primary text-xs" disabled={canvasTaskBusy || !canvasTaskTitle.trim()}>{canvasTaskBusy ? "正在创建…" : "创建并放入画布"}</button>
                  <button type="button" className="btn text-xs" disabled={canvasTaskBusy} onClick={() => setCanvasTaskFormOpen(false)}>取消</button>
                </form>
              )}
              {canvasState.status === "loading" && <div className="card text-xs text-ink-dim">正在读取当前 Worktree 的 Canvas…</div>}
              {canvasState.status === "error" && <div className="card text-xs text-warning" role="alert">{canvasState.message}</div>}
              {canvasState.status === "ready" && canvasState.canvas && (
                <div className="h-[min(68vh,760px)] overflow-hidden rounded-lg border border-line bg-bg-card" data-testid="group-canvas-api">
                  <div className="border-b border-line px-3 py-2 text-xs font-medium">{canvasState.canvas.title}</div>
                  <p className="border-b border-line px-3 py-2 text-[10px] text-ink-mute">文档版本 {canvasState.canvas.document.version} · viewport、frames 与画布连接线已由 Worktree 文档保存；任务关系仍由 Multica 事实源管理。</p>
                  <CanvasView
                    canvas={viewCanvas}
                    elements={viewElements}
                    connectors={canvasState.canvas.document.connectors}
                    workItemSummaries={viewWorkItems}
                    worktreeSummaries={viewWorktrees}
                    highlightElementId={searchParams.get("highlight") ?? undefined}
                    readOnly
                    onViewportChange={saveCanvasViewport}
                    onOpenWorkItem={openWorkItem}
                  />
                </div>
              )}
              {canvasState.status === "ready" && !canvasState.canvas && (
                <div className="card flex min-h-56 flex-col items-center justify-center gap-2 px-6 text-center">
                  <p className="text-sm text-ink-dim">当前 Worktree 没有已登记的 Group Canvas。</p>
                  <p className="max-w-xl text-xs text-ink-mute">没有读取全局 Project / Free Canvas，也没有回退到本地 seed。</p>
                  {canWriteCanvas && <button type="button" className="btn-primary mt-2 text-xs" disabled={canvasCreateBusy} onClick={() => void createGroupCanvas()}>{canvasCreateBusy ? "正在创建…" : "创建当前 Worktree Canvas"}</button>}
                </div>
              )}
            </div>
          )}

          {(currentApp === "multica" || currentApp === "jira" || currentApp === "task-card") && (
            <div className="space-y-4">
              <div><h2 className="text-lg font-semibold">{GROUP_APPS.find((app) => app.id === currentApp)?.label}</h2><p className="text-xs text-ink-mute">canonical WorkItem · 当前 Worktree 授权范围</p>
                {currentApp === "multica" && <p className="mt-1 text-[10px] text-warning">当前展示任务投影；Multica Agent 编排和运行状态 API 尚未接入。</p>}
                {currentApp === "jira" && <p className="mt-1 text-[10px] text-warning">当前展示 canonical WorkItem 投影；Jira issue adapter 与同步尚未接入。</p>}
              </div>
              {taskState.status === "loading" && <div className="card text-xs text-ink-dim">正在读取当前 Worktree 的 Task Card…</div>}
              {taskState.status === "error" && <div className="card text-xs text-warning" role="alert">{taskState.message}</div>}
              {taskLinkUnavailable && <div className="card text-xs text-warning" role="alert" data-testid="group-task-link-unavailable">此任务不存在，或不属于当前授权 Worktree。</div>}
              {taskState.status === "ready" && currentApp === "task-card" && selectedTask && (
                <article className="card space-y-3" data-testid="group-task-card-detail">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div><h3 className="text-base font-semibold">{selectedTask.title}</h3><p className="font-mono text-[10px] text-ink-mute">{selectedTask.work_item_id}</p></div>
                    <span className="rounded border border-line px-2 py-1 text-[10px]">{selectedTask.lifecycle.status}</span>
                  </div>
                  <p className="whitespace-pre-wrap text-sm text-ink-dim">{selectedTask.description || "无描述"}</p>
                  <p className="text-[10px] text-ink-mute">类型 {selectedTask.item_type} · 优先级 {selectedTask.priority} · 版本 {selectedTask.lifecycle.version}</p>
                  {cliRequested && <div className="rounded border border-warning/30 bg-warning/5 p-3 text-xs text-warning">Task Card 已绑定当前 Group；Local Runtime session grant、PTY 与会话审计尚未接入，因此 CLI 保持禁用。</div>}
                  <Link href={taskHref()} className="btn w-fit text-xs">返回 Task Card Index</Link>
                </article>
              )}
              {taskState.status === "ready" && !taskLinkUnavailable && !(currentApp === "task-card" && selectedTask) && (
                workItems.length > 0 ? (
                  <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                    {workItems.map((item) => (
                      <Link key={item.work_item_id} href={taskHref(item.work_item_id)} className="card block space-y-2 hover:border-accent/60">
                        <div className="flex items-center justify-between gap-2"><span className="font-mono text-[10px] text-ink-mute">{item.work_item_id.slice(0, 8)}</span><span className="rounded border border-line px-2 py-1 text-[10px]">{item.lifecycle.status}</span></div>
                        <h3 className="text-sm font-medium">{item.title}</h3>
                        <p className="line-clamp-2 text-xs text-ink-dim">{item.description || "无描述"}</p>
                      </Link>
                    ))}
                  </div>
                ) : <div className="card text-xs text-ink-dim">当前 Worktree 没有关联的 canonical WorkItem。</div>
              )}
            </div>
          )}

          {currentApp === "workflow" && <div className="card space-y-2"><h2 className="text-lg font-semibold">Workflow · LangGraph</h2><p className="text-xs text-ink-dim">GroupContext 已验证；LangGraph run/checkpoint API 尚未接入，当前不创建或恢复执行。</p></div>}
          {currentApp === "plugins" && <div className="card space-y-2"><h2 className="text-lg font-semibold">Group Plugin Apps</h2><p className="text-xs text-ink-dim">Group App Registry、manifest 信任、capability gateway 与撤权 API 尚未接入。此页面不展示本地开关作为生产状态。</p></div>}
        </section>
      </div>

      <aside className="fixed bottom-3 left-3 right-3 z-40 mx-auto max-w-[960px] rounded-xl border border-line bg-bg-card/95 p-3 shadow-2xl backdrop-blur md:bottom-4 md:left-[17rem] md:right-5" data-testid="group-chat-bar">
        <div className="flex flex-wrap items-center gap-2">
          <label htmlFor="group-chat-scope" className="text-[10px] font-semibold uppercase tracking-wider text-ink-mute">Scope</label>
          <select id="group-chat-scope" value={scope} onChange={(event) => changeScope(event.target.value as ChatScope)} className="rounded-md border border-line bg-bg-soft px-2 py-1.5 text-xs">
            <option value="WORKTREE">WORKTREE · {worktree.name}</option>
            <option value="GLOBAL">GLOBAL · 需选择授权目标</option>
          </select>
          <input disabled value="" readOnly placeholder="Group Chat API 尚未接入" className="min-w-[180px] flex-1 rounded-md border border-line bg-bg-soft px-3 py-2 text-xs opacity-70" aria-label="Group Chat message" />
          <button type="button" disabled className="btn-primary text-xs opacity-50">发送</button>
          <span className="w-full pl-2 text-[9px] text-warning">{scope === "GLOBAL" ? "GLOBAL 写入需先加载目标并逐个授权。" : "当前只展示授权范围；此输入不会发送消息。"}</span>
        </div>
      </aside>
    </main>
  );
}
