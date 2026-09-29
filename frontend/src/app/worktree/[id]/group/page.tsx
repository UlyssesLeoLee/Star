/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/worktree/[id]/group/page.tsx", type:"file", language:"tsx"}),
  (page:Function {name:"GroupWorkspacePage", type:"function", signature:"GroupWorkspacePage(props: PageProps): JSX.Element", visibility:"public", complexity:"complex"}),
  (apps:Variable {name:"GROUP_APPS", type:"variable", language:"typescript"}),
  (pluginPreview:Variable {name:"PLUGIN_PREVIEW", type:"variable", language:"typescript"}),
  (statusOptions:Variable {name:"STATUS_OPTIONS", type:"variable", language:"typescript"}),
  (appType:Variable {name:"AppId", type:"variable", language:"typescript"}),
  (scopeType:Variable {name:"ChatScope", type:"variable", language:"typescript"}),
  (isAppId:Function {name:"isAppId", type:"function", signature:"isAppId(value: string | null): value is AppId", visibility:"private", complexity:"simple"}),
  (enabledPlugins:Variable {name:"enabledPlugins", type:"variable", language:"typescript"}),
  (groupCanvas:Variable {name:"groupCanvas", type:"variable", language:"typescript"}),
  (canvasLinkError:Variable {name:"canvasLinkError", type:"variable", language:"typescript"}),
  (setCanvasLinkError:Variable {name:"setCanvasLinkError", type:"variable", language:"typescript"}),
  (projects:Variable {name:"projects", type:"variable", language:"typescript"}),
  (worktrees:Variable {name:"worktrees", type:"variable", language:"typescript"}),
  (worktree:Variable {name:"worktree", type:"variable", language:"typescript"}),
  (projectWorkItems:Variable {name:"projectWorkItems", type:"variable", language:"typescript"}),
  (currentApp:Variable {name:"currentApp", type:"variable", language:"typescript"}),
  (scope:Variable {name:"scope", type:"variable", language:"typescript"}),
  (project:Variable {name:"project", type:"variable", language:"typescript"}),
  (taskHref:Function {name:"taskHref", type:"function", signature:"taskHref(workItemId: string, cli?: boolean): string", visibility:"private", complexity:"simple"}),
  (appHref:Function {name:"appHref", type:"function", signature:"appHref(appId: AppId, params?: Record<string, string>): string", visibility:"private", complexity:"simple"}),
  (changeScope:Function {name:"changeScope", type:"function", signature:"changeScope(nextScope: ChatScope): void", visibility:"private", complexity:"simple"}),
  (openCanvasTask:Function {name:"openCanvasTask", type:"function", signature:"openCanvasTask(workItemId: string, refWorktreeId?: string): void", visibility:"private", complexity:"moderate"}),
  (taskMap:Function {name:"taskMap", type:"function", signature:"projectWorkItems.map(workItem => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (appMap:Function {name:"appMap", type:"function", signature:"GROUP_APPS.map(app => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (jiraColumnMap:Function {name:"jiraColumnMap", type:"function", signature:"STATUS_OPTIONS.map(status => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (jiraTaskMap:Function {name:"jiraTaskMap", type:"function", signature:"projectWorkItems.map(workItem => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (statusMap:Function {name:"statusMap", type:"function", signature:"STATUS_OPTIONS.map(status => JSX.Element)", visibility:"private", complexity:"simple"}),
  (statusClick:Function {name:"statusClick", type:"function", signature:"onClick(): void", visibility:"private", complexity:"simple"}),
  (pluginToggle:Function {name:"pluginToggle", type:"function", signature:"onChange(event): void", visibility:"private", complexity:"simple"}),
  (pluginStateUpdate:Function {name:"pluginStateUpdate", type:"function", signature:"setEnabledPlugins(current => nextState)", visibility:"private", complexity:"simple"}),
  (paneRenderer:Function {name:"paneRenderer", type:"function", signature:"renderPane(paneId: string): JSX.Element", visibility:"private", complexity:"simple"}),
  (file)-[:CONTAINS]->(page),
  (file)-[:CONTAINS]->(apps),
  (file)-[:CONTAINS]->(pluginPreview),
  (file)-[:CONTAINS]->(statusOptions),
  (file)-[:CONTAINS]->(appType),
  (file)-[:CONTAINS]->(scopeType),
  (file)-[:CONTAINS]->(isAppId),
  (page)-[:USES]->(apps),
  (page)-[:USES]->(pluginPreview),
  (page)-[:USES]->(statusOptions),
  (page)-[:USES]->(scope),
  (page)-[:USES]->(enabledPlugins),
  (page)-[:USES]->(groupCanvas),
  (page)-[:USES]->(canvasLinkError),
  (page)-[:USES]->(setCanvasLinkError),
  (page)-[:USES]->(projectWorkItems),
  (page)-[:USES]->(currentApp),
  (page)-[:USES]->(projects),
  (page)-[:USES]->(worktrees),
  (page)-[:USES]->(worktree),
  (page)-[:USES]->(project),
  (page)-[:CALLS]->(isAppId),
  (page)-[:CONTAINS]->(taskHref),
  (page)-[:CONTAINS]->(appHref),
  (page)-[:CONTAINS]->(changeScope),
  (page)-[:CONTAINS]->(openCanvasTask),
  (page)-[:CONTAINS]->(taskMap),
  (page)-[:CONTAINS]->(appMap),
  (page)-[:CONTAINS]->(jiraColumnMap),
  (page)-[:CONTAINS]->(jiraTaskMap),
  (page)-[:CONTAINS]->(statusMap),
  (page)-[:CONTAINS]->(statusClick),
  (page)-[:CONTAINS]->(pluginToggle),
  (page)-[:CONTAINS]->(pluginStateUpdate),
  (page)-[:CONTAINS]->(paneRenderer),
  (page)-[:CALLS]->(taskHref),
  (page)-[:CALLS]->(appHref),
  (page)-[:CALLS]->(changeScope),
  (page)-[:CALLS]->(openCanvasTask),
  (openCanvasTask)-[:CALLS]->(setCanvasLinkError),
  (page)-[:CALLS]->(taskMap),
  (page)-[:CALLS]->(appMap),
  (page)-[:CALLS]->(jiraColumnMap),
  (page)-[:CALLS]->(jiraTaskMap),
  (page)-[:CALLS]->(statusMap),
  (page)-[:CALLS]->(statusClick),
  (page)-[:CALLS]->(pluginToggle),
  (page)-[:CALLS]->(pluginStateUpdate),
  (page)-[:CALLS]->(paneRenderer);
*/

"use client";

import { use as ReactUse, useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import {
  ArrowUpRight,
  Blocks,
  CheckSquare,
  ClipboardList,
  GitBranch,
  LayoutGrid,
  MessageSquareText,
  Plug,
  SquareKanban,
  Workflow,
  type LucideIcon,
} from "lucide-react";

import { CanvasView } from "@/components/CanvasView";
import { StatusPill } from "@/components/StatusPill";
import { TerminalStackContainer } from "@/components/terminal/TerminalStackContainer";
import { useStore } from "@/lib/store";
import type { WorkItemStatus } from "@/types/ids";

interface PageProps {
  params: Promise<{ id: string }>;
}

type AppId = "multica" | "jira" | "task-card" | "canvas" | "workflow" | "plugins";
type ChatScope = "WORKTREE" | "GLOBAL";

function isAppId(value: string | null): value is AppId {
  return GROUP_APPS.some((app) => app.id === value);
}

const GROUP_APPS: Array<{
  id: AppId;
  label: string;
  subtitle: string;
  icon: LucideIcon;
}> = [
  { id: "multica", label: "Multica", subtitle: "任务生命周期", icon: Blocks },
  { id: "jira", label: "Jira 视图", subtitle: "Board / Backlog / Sprint", icon: SquareKanban },
  { id: "task-card", label: "Task Cards", subtitle: "统一任务入口", icon: ClipboardList },
  { id: "canvas", label: "Infinite Canvas", subtitle: "Miro 式协作画布", icon: LayoutGrid },
  { id: "workflow", label: "Workflow", subtitle: "LangGraph / L0", icon: Workflow },
  { id: "plugins", label: "Plugins", subtitle: "群组应用与能力", icon: Plug },
];

const PLUGIN_PREVIEW = [
  { id: "canvas-insights", name: "Canvas Insights", capability: "canvas.read / relation.query" },
  { id: "release-helper", name: "Release Helper", capability: "work_item.read / workflow.run" },
  { id: "scm-bridge", name: "SCM Bridge", capability: "pull_request.read / status.subscribe" },
];

const STATUS_OPTIONS: Array<{ value: WorkItemStatus; label: string }> = [
  { value: "todo", label: "待办" },
  { value: "in_progress", label: "进行中" },
  { value: "review", label: "评审中" },
  { value: "blocked", label: "阻塞" },
  { value: "done", label: "完成" },
  { value: "wontfix", label: "不处理" },
];

export default function GroupWorkspacePage({ params }: PageProps) {
  const { id: worktreeId } = ReactUse(params);
  const router = useRouter();
  const searchParams = useSearchParams();
  const scope: ChatScope = searchParams.get("scope") === "GLOBAL" ? "GLOBAL" : "WORKTREE";
  const [canvasLinkError, setCanvasLinkError] = useState<string | null>(null);
  const [enabledPlugins, setEnabledPlugins] = useState<Record<string, boolean>>({
    "canvas-insights": true,
    "release-helper": true,
    "scm-bridge": false,
  });

  const worktrees = useStore((state) => state.worktrees);
  const projects = useStore((state) => state.projects);
  const workItems = useStore((state) => state.workItems);
  const canvases = useStore((state) => state.canvases);
  const canvasElements = useStore((state) => state.canvasElements);
  const canvasConnectors = useStore((state) => state.canvasConnectors);
  const transitionWorkItem = useStore((state) => state.transitionWorkItem);

  const worktree = worktrees.find((item) => item.id === worktreeId);
  const project = projects.find((item) => item.id === worktree?.project_id);
  const projectWorkItems = worktree
    ? workItems.filter((item) =>
        item.project_id === worktree.project_id &&
        (!item.worktree_id || item.worktree_id === worktree.id),
      )
    : [];
  const requestedApp = searchParams.get("app");
  const currentApp = isAppId(requestedApp) ? requestedApp : "multica";
  const currentWorkItemId = searchParams.get("work_item_id");
  const selectedTask = projectWorkItems.find((item) => item.id === currentWorkItemId);
  const cliPreviewOpen = searchParams.get("cli") === "1" && selectedTask?.worktree_id === worktreeId;
  const groupCanvas = worktree
    ? canvases.find((canvas) => canvas.ref_kind === "worktree" && canvas.ref_id === worktree.id)
    : undefined;
  const elements = groupCanvas
    ? canvasElements.filter((element) => element.canvas_id === groupCanvas.id)
    : [];
  const connectors = groupCanvas
    ? canvasConnectors.filter((connector) => connector.canvas_id === groupCanvas.id)
    : [];
  const selectedTaskCanvasElement = selectedTask
    ? elements.find((element) =>
        element.entity_ref?.ref_type === "work_item" &&
        element.entity_ref.ref_id === selectedTask.id &&
        element.entity_ref.worktree_id === worktreeId,
      )
    : undefined;
  const basePath = `/worktree/${encodeURIComponent(worktreeId)}/group`;
  function taskHref(workItemId: string, cli = false) {
    const query = new URLSearchParams({ app: "task-card", work_item_id: workItemId, scope });
    if (cli) query.set("cli", "1");
    return `${basePath}?${query.toString()}`;
  }

  function appHref(appId: AppId, params: Record<string, string> = {}) {
    const query = new URLSearchParams({ app: appId, scope, ...params });
    return `${basePath}?${query.toString()}`;
  }

  function changeScope(nextScope: ChatScope) {
    const query = new URLSearchParams(searchParams.toString());
    query.set("scope", nextScope);
    router.replace(`${basePath}?${query.toString()}`, { scroll: false });
  }

  function openCanvasTask(workItemId: string, refWorktreeId?: string) {
    const taskIsBoundHere = projectWorkItems.some(
      (item) => item.id === workItemId && item.worktree_id === worktreeId,
    );
    if (refWorktreeId !== worktreeId || !taskIsBoundHere) {
      setCanvasLinkError("此 Canvas 引用没有当前 Worktree 的显式授权关联，已阻止跳转。请先在当前 Worktree 中建立 canonical 任务关联。");
      return;
    }
    setCanvasLinkError(null);
    router.push(taskHref(workItemId));
  }

  if (!worktree) {
    return (
      <main className="mx-auto w-full max-w-4xl p-6">
        <section className="card space-y-3">
          <h1 className="text-lg font-semibold">找不到 Worktree</h1>
          <p className="text-sm text-ink-dim">当前 Worktree ID：{worktreeId}</p>
          <Link href="/worktree" className="btn w-fit">返回 Worktree Index</Link>
        </section>
      </main>
    );
  }

  const taskList = (
    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
      {projectWorkItems.map((item) => (
        <Link
          key={item.id}
          href={taskHref(item.id)}
          className="card block transition hover:border-accent/60 hover:bg-bg-soft/60"
          data-testid={`group-task-${item.id}`}
        >
          <div className="mb-2 flex items-center justify-between gap-2">
            <span className="font-mono text-xs text-info">{item.key}</span>
            <StatusPill value={item.status} size="xs" />
          </div>
          <div className="text-sm font-medium">{item.title}</div>
          <div className="mt-3 flex items-center justify-between text-[10px] text-ink-mute">
            <span>{item.kind} · {item.priority}</span>
            <span>{item.worktree_id === worktree.id ? "已绑定当前 Worktree" : "项目级示例任务"}</span>
          </div>
        </Link>
      ))}
    </div>
  );

  return (
    <main className="min-h-full pb-36">
      <header className="border-b border-line bg-bg-soft/70 px-5 py-4 md:px-7">
        <div className="mx-auto flex max-w-[1500px] flex-wrap items-center justify-between gap-4">
          <div className="min-w-0">
            <div className="mb-1 flex items-center gap-2 text-[10px] uppercase tracking-[0.18em] text-ink-mute">
              <GitBranch size={12} /> Project · {project?.name ?? "Unknown Project"} / Worktree Group
            </div>
            <h1 className="truncate text-xl font-semibold">{worktree.name}</h1>
            <div className="mt-1 flex flex-wrap items-center gap-2 font-mono text-xs text-ink-dim">
              <span>{worktree.id}</span><span>·</span><span>{worktree.branch}</span>
              <span>·</span><StatusPill value={worktree.status} size="xs" />
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">前端原型 · mock store</span>
            <Link href={`/worktree?project_id=${encodeURIComponent(worktree.project_id)}`} className="btn text-xs">Project Worktree Index</Link>
          </div>
        </div>
      </header>

      <div className="mx-auto grid max-w-[1500px] gap-5 px-4 py-5 md:grid-cols-[220px_minmax(0,1fr)] md:px-7">
        <nav aria-label="Worktree 群组应用" className="h-fit rounded-lg border border-line bg-bg-card p-2 md:sticky md:top-4">
          <div className="px-2 pb-2 pt-1 text-[10px] font-semibold uppercase tracking-widest text-ink-mute">{worktree.name} · 同级应用</div>
          <div className="grid grid-cols-2 gap-1 md:grid-cols-1">
            {GROUP_APPS.map((app) => {
              const Icon = app.icon;
              const href = appHref(app.id);
              const active = currentApp === app.id;
              return (
                <Link
                  key={app.id}
                  href={href}
                  aria-current={active ? "page" : undefined}
                  className={`flex min-w-0 items-center gap-2 rounded-md px-2.5 py-2 text-left transition ${active ? "bg-accent/15 text-accent" : "text-ink-dim hover:bg-bg-soft hover:text-ink"}`}
                  data-testid={`group-app-${app.id}`}
                >
                  <Icon size={15} className="shrink-0" />
                  <span className="min-w-0">
                    <span className="block truncate text-xs font-medium">{app.label}</span>
                    <span className="hidden truncate text-[9px] text-ink-mute md:block">{app.subtitle}</span>
                  </span>
                </Link>
              );
            })}
          </div>
          <div className="mt-3 border-t border-line px-2 pt-3 text-[10px] leading-relaxed text-ink-mute">
            所有 App 共用 Worktree 上下文与 canonical WorkItem。切换入口不切换根节点。
          </div>
        </nav>

        <section className="min-w-0" aria-live="polite">
          {currentApp === "multica" && (
            <div className="space-y-4">
              <div className="flex flex-wrap items-end justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Multica · 生命周期</h2><p className="text-xs text-ink-mute">领取、执行与评审共用 WorkItem 状态源。</p></div>
                <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">示例数据只在浏览器 store</span>
              </div>
              {taskList}
            </div>
          )}

          {currentApp === "jira" && (
            <div className="space-y-4">
              <div><h2 className="text-lg font-semibold">Jira 等价视图</h2><p className="text-xs text-ink-mute">Board / Backlog / Sprint 视图复用同一组任务卡。</p></div>
              <div className="grid gap-3 xl:grid-cols-3">
                {(["todo", "in_progress", "review"] as WorkItemStatus[]).map((status) => (
                  <section key={status} className="rounded-lg border border-line bg-bg-soft/40 p-3">
                    <h3 className="mb-3 flex items-center justify-between text-xs font-semibold">
                      {STATUS_OPTIONS.find((option) => option.value === status)?.label}
                      <span className="font-mono text-ink-mute">{projectWorkItems.filter((item) => item.status === status).length}</span>
                    </h3>
                    <div className="space-y-2">
                      {projectWorkItems.filter((item) => item.status === status).map((item) => (
                        <Link key={item.id} href={taskHref(item.id)} className="block rounded-md border border-line bg-bg-card p-3 hover:border-accent/50">
                          <div className="font-mono text-[10px] text-info">{item.key}</div>
                          <div className="mt-1 text-xs">{item.title}</div>
                        </Link>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
              <p className="text-[10px] text-ink-mute">当前预览列出三种常用状态；完整自定义 workflow 列配置由 Phase 2 接入。</p>
            </div>
          )}

          {currentApp === "task-card" && (
            <div className="space-y-4">
              <div className="flex flex-wrap items-end justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Task Card Index</h2><p className="text-xs text-ink-mute">Task Card 与 Canvas、Multica、Jira 视图是同级入口。</p></div>
                <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">CLI 仅展示面板预览</span>
              </div>
              {selectedTask ? (
                <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(340px,0.9fr)]">
                  <article className="card space-y-4">
                    <div className="flex items-start justify-between gap-3">
                      <div><div className="font-mono text-xs text-info">{selectedTask.key} · {selectedTask.id}</div><h3 className="mt-2 text-lg font-semibold">{selectedTask.title}</h3></div>
                      <StatusPill value={selectedTask.status} />
                    </div>
                    <p className="text-sm text-ink-dim">{selectedTask.description}</p>
                    <div className="flex flex-wrap gap-2 text-[10px] text-ink-mute">
                      <span className="rounded border border-line px-2 py-1">Worktree: {selectedTask.worktree_id ?? "未绑定（seed）"}</span>
                      <span className="rounded border border-line px-2 py-1">Canonical ID: {selectedTask.id}</span>
                    </div>
                    <div>
                      <div className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-ink-mute">本地预览状态迁移</div>
                      <div className="flex flex-wrap gap-1.5">
                        {STATUS_OPTIONS.map((option) => (
                          <button
                            key={option.value}
                            type="button"
                            disabled={selectedTask.status === option.value}
                            onClick={() => transitionWorkItem(selectedTask.id, option.value)}
                            className="btn text-[10px] disabled:opacity-40"
                          >
                            {option.label}
                          </button>
                        ))}
                      </div>
                    </div>
                    <div className="flex flex-wrap gap-2 border-t border-line pt-3">
                      {selectedTask.worktree_id === worktreeId
                        ? <Link href={taskHref(selectedTask.id, true)} className="btn-primary text-xs"><CheckSquare size={13} /> 卡内 CLI 预览</Link>
                        : <span className="rounded border border-line px-2 py-1 text-[10px] text-ink-mute">先绑定当前 Worktree 才能打开 CLI</span>}
                      {selectedTaskCanvasElement && <Link href={appHref("canvas", { highlight: selectedTaskCanvasElement.id })} className="btn text-xs"><LayoutGrid size={13} /> 在 Canvas 中定位</Link>}
                      <Link href={appHref("jira")} className="btn text-xs"><ArrowUpRight size={13} /> 打开 Jira 视图</Link>
                    </div>
                  </article>
                  <section className="card min-h-[320px] space-y-3">
                    <div className="flex items-center justify-between gap-3">
                      <div><h3 className="text-sm font-semibold">{cliPreviewOpen ? "卡内 CLI" : "执行入口"}</h3><p className="text-[10px] text-ink-mute">TaskExecutionContext / Local Runtime 未连接</p></div>
                      {cliPreviewOpen && <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">mock terminal</span>}
                    </div>
                    {cliPreviewOpen ? (
                      <>
                        <div className="rounded border border-warning/30 bg-warning/5 p-2 text-[10px] leading-relaxed text-warning">
                          这里只展示卡内终端布局。Worktree checkout、权限快照和真实进程未绑定，命令执行不可用。
                        </div>
                        <TerminalStackContainer
                          key={selectedTask.id}
                          renderPane={(paneId) => (
                            <div className="flex h-full flex-col justify-between bg-[#0d1117] p-3 font-mono text-[10px] text-emerald-300" data-testid="task-cli-preview-pane">
                              <div><div>task: {selectedTask.key}</div><div>worktree: {selectedTask.worktree_id ?? "unbound"}</div><div>session: not provisioned</div></div>
                              <div><span className="text-ink-mute">{paneId}</span> $ <span className="animate-pulse">▍</span></div>
                            </div>
                          )}
                        />
                      </>
                    ) : (
                      <div className="flex h-48 flex-col items-center justify-center gap-2 rounded border border-dashed border-line text-center">
                        <CheckSquare size={20} className="text-accent" />
                        <p className="text-xs text-ink-dim">CLI 从此 Task Card 打开，并继承当前 Worktree。</p>
                        <p className="text-[10px] text-ink-mute">生产执行必须由服务端构造 TaskExecutionContext。</p>
                      </div>
                    )}
                  </section>
                </div>
              ) : taskList}
            </div>
          )}

          {currentApp === "canvas" && (
            <div className="space-y-3">
              <div className="flex flex-wrap items-end justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Infinite Canvas · Miro 等价入口</h2><p className="text-xs text-ink-mute">Canvas 与 Task Card 同级；typed EntityRef 指向当前 Worktree 内的 canonical WorkItem。</p></div>
                <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">Worktree scope · 持久化 API 未接</span>
              </div>
              {canvasLinkError && <p role="alert" className="rounded border border-warning/30 bg-warning/5 p-3 text-xs text-warning">{canvasLinkError}</p>}
              {groupCanvas ? (
                <div className="h-[min(68vh,760px)] overflow-hidden rounded-lg border border-line bg-bg-card" data-testid="group-canvas-preview">
                  <div className="border-b border-line px-3 py-2 text-xs font-medium">{groupCanvas.title}</div>
                  <CanvasView
                    canvas={groupCanvas}
                    elements={elements}
                    connectors={connectors}
                    highlightElementId={searchParams.get("highlight") ?? undefined}
                    onOpenWorkItem={openCanvasTask}
                  />
                </div>
              ) : (
                <div className="card flex min-h-56 flex-col items-center justify-center gap-2 px-6 text-center">
                  <p className="text-sm text-ink-dim">当前 Worktree 没有显式绑定的 Group Canvas。</p>
                  <p className="max-w-xl text-xs leading-relaxed text-ink-mute">旧 Project / Free Canvas 不会自动继承到这里；绑定创建、元素保存和 EntityRef API 尚未接通，因此不会用全局 seed 冒充当前 Worktree 画布。</p>
                </div>
              )}
            </div>
          )}

          {currentApp === "workflow" && (
            <div className="space-y-4">
              <div><h2 className="text-lg font-semibold">Workflow · LangGraph / L0</h2><p className="text-xs text-ink-mute">流程执行是 Worktree Group 的同级应用能力，不创建第二个聊天栏或任务事实源。</p></div>
              <section className="card grid gap-4 md:grid-cols-3">
                <div><div className="text-[10px] uppercase tracking-widest text-ink-mute">Owner</div><div className="mt-1 text-sm">Workflow Runtime</div></div>
                <div><div className="text-[10px] uppercase tracking-widest text-ink-mute">Context</div><div className="mt-1 font-mono text-xs">{worktree.id} · {worktree.project_id}</div></div>
                <div><div className="text-[10px] uppercase tracking-widest text-ink-mute">Runtime</div><div className="mt-1 text-sm text-warning">not connected</div></div>
              </section>
              <div className="rounded-lg border border-warning/30 bg-warning/5 p-4 text-xs leading-relaxed text-ink-dim">
                LangGraph 每次 start/resume 都必须重新解析 GroupContext 并复验权限。当前仓库 chat stub 不带 scope；此入口不创建 Flow、不运行 Agent，也不把 checkpoint 当作授权凭据。
              </div>
            </div>
          )}

          {currentApp === "plugins" && (
            <div className="space-y-4">
              <div><h2 className="text-lg font-semibold">Group Plugin Apps</h2><p className="text-xs text-ink-mute">App Registry 与 LangGraph SubAgentRegistry 分离；每次 capability 调用由服务端重新授权。</p></div>
              <div className="rounded-lg border border-warning/30 bg-warning/5 p-3 text-[10px] text-warning">下面的开关只改变本地原型展示，不会安装、卸载或授予任何 capability。</div>
              <div className="grid gap-3 lg:grid-cols-2">
                {PLUGIN_PREVIEW.map((plugin) => (
                  <article key={plugin.id} className="card flex items-center justify-between gap-4">
                    <div className="min-w-0">
                      <div className="flex items-center gap-2 text-sm font-medium"><Plug size={14} className="text-accent" />{plugin.name}</div>
                      <div className="mt-1 break-all font-mono text-[10px] text-ink-mute">{plugin.capability}</div>
                    </div>
                    <label className="flex shrink-0 items-center gap-2 text-xs">
                      <input
                        type="checkbox"
                        checked={enabledPlugins[plugin.id]}
                        onChange={(event) => setEnabledPlugins((current) => ({ ...current, [plugin.id]: event.target.checked }))}
                        aria-label={`${plugin.name} 原型开关`}
                        data-testid={`plugin-preview-${plugin.id}`}
                      />
                      {enabledPlugins[plugin.id] ? "预览启用" : "预览停用"}
                    </label>
                  </article>
                ))}
              </div>
            </div>
          )}
        </section>
      </div>

      {selectedTask && currentApp === "task-card" && (
        <Link href={appHref("task-card")} className="fixed bottom-36 right-4 z-30 rounded-full border border-line bg-bg-card px-3 py-2 text-[10px] text-ink-dim shadow-lg md:bottom-24">返回全部 Task Cards</Link>
      )}

      <aside className="fixed bottom-[4.6rem] left-3 right-3 z-40 mx-auto max-w-[960px] rounded-xl border border-line bg-bg-card/95 p-3 shadow-2xl backdrop-blur md:bottom-4 md:left-[17rem] md:right-5" data-testid="group-chat-bar">
        <div className="flex flex-wrap items-center gap-2">
          <MessageSquareText size={15} className="shrink-0 text-accent" />
          <label htmlFor="group-chat-scope" className="text-[10px] font-semibold uppercase tracking-wider text-ink-mute">Scope</label>
          <select
            id="group-chat-scope"
            value={scope}
            onChange={(event) => changeScope(event.target.value as ChatScope)}
            className="rounded-md border border-line bg-bg-soft px-2 py-1.5 text-xs"
            data-testid="group-chat-scope"
          >
            <option value="WORKTREE">WORKTREE · {worktree.id}</option>
            <option value="GLOBAL">GLOBAL · 需选择授权目标</option>
          </select>
          <div className="min-w-[180px] flex-1">
            <input
              disabled
              value=""
              readOnly
              placeholder={scope === "GLOBAL" ? "Global target ACL/API 未接入" : "Worktree scoped chat API 未接入"}
              className="w-full rounded-md border border-line bg-bg-soft px-3 py-2 text-xs opacity-70"
              aria-label="Group Chat message"
            />
          </div>
          <button type="button" disabled className="btn-primary text-xs opacity-50">发送</button>
          <span className="w-full pl-6 text-[9px] text-warning">
            {scope === "GLOBAL" ? "GLOBAL 写入必须先从服务端加载目标并逐个授权。" : "GroupContext Chat API 尚未接入；此控件不会发送消息。"}
          </span>
        </div>
      </aside>
    </main>
  );
}
