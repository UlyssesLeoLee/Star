/*
@cypher schema=1 source_sha256=06eadf84322318ebaeb159eaa5eb1af9cf449927afb770f52231a46aa1e2877c
MERGE (self:File {path:"frontend/src/components/run/RunWorkspaceShell.tsx"})
MERGE (shell:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::RunWorkspaceShell",kind:"function"})
MERGE (loadMore:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::loadMoreWorktreeSelection",kind:"function"})
MERGE (validatePage:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::validateWorktreeSelectionPage",kind:"function"})
MERGE (filtered:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::filteredWorktrees",kind:"variable"})
MERGE (selection:Type {id:"frontend/src/components/run/RunWorkspaceShell.tsx::WorktreeSelectionLoad",kind:"type"})
MERGE (pageSize:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RUN_DIRECTORY_PAGE_SIZE",kind:"constant"})
MERGE (pageBudget:Symbol {id:"frontend/src/lib/run/runDirectoryTree.ts::RUN_DIRECTORY_TREE_BUDGET",kind:"constant"})
MERGE (selectionLimit:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::WORKTREE_SELECTION_MAX_ROWS",kind:"constant"})
MERGE (directoryErrorType:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryError",kind:"class"})
MERGE (panel:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::RunTaskCardsPanel",kind:"function"})
MERGE (worktrees:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.listWorktrees",kind:"method"})
MERGE (worktreeHref:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::canonicalRunWorktreeHref",kind:"function"})
MERGE (worktree:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::RunWorktree",kind:"interface"})
MERGE (context:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::RunContextEnvelope",kind:"type"})
MERGE (runHref:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::canonicalRunHref",kind:"function"})
MERGE (self)-[:DEFINES]->(shell)
MERGE (self)-[:DEFINES]->(validatePage)
MERGE (shell)-[:CALLS]->(panel)
MERGE (biPanel:Symbol {id:"frontend/src/components/run/RunBiPanel.tsx::RunBiPanel",kind:"function"})
MERGE (shell)-[:CALLS]->(biPanel)
MERGE (shell)-[:CALLS]->(worktrees)
MERGE (shell)-[:CALLS]->(worktreeHref)
MERGE (shell)-[:USES_TYPE]->(context)
MERGE (shell)-[:USES_TYPE]->(worktree)
MERGE (shell)-[:CALLS]->(runHref)
MERGE (shell)-[:CALLS]->(loadMore)
MERGE (shell)-[:CALLS]->(validatePage)
MERGE (loadMore)-[:CALLS]->(worktrees)
MERGE (loadMore)-[:CALLS]->(validatePage)
MERGE (loadMore)-[:CALLS]->(directoryErrorType)
MERGE (loadMore)-[:CALLS]->(error)
MERGE (validatePage)-[:CALLS]->(directoryErrorType)
MERGE (validatePage)-[:USES]->(pageSize)
MERGE (validatePage)-[:USES]->(selectionLimit)
MERGE (shell)-[:USES]->(pageSize)
MERGE (shell)-[:USES]->(pageBudget)
MERGE (shell)-[:USES]->(selectionLimit)
MERGE (shell)-[:USES]->(filtered)
MERGE (shell)-[:USES_TYPE]->(selection)
@endcypher
*/
/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/RunWorkspaceShell.tsx",type:"file",language:"tsx"}),
 (route:Class {name:"RunWorkspaceRoute",type:"class"}),(load:Class {name:"ContextLoad",type:"class"}),
 (shell:Function {name:"RunWorkspaceShell",type:"function"}),(loadMore:Function {name:"loadMoreWorktreeSelection",type:"function"}),(filtered:Variable {name:"filteredWorktrees",type:"variable"}),(selection:Type {name:"WorktreeSelectionLoad",type:"type"}),(resolve:Function {name:"resolveCanonicalRunContext",type:"function"}),
 (detail:Function {name:"RunContextDetails",type:"function"}),(session:Function {name:"useRunDirectory",type:"function"}),(taskCards:Function {name:"RunTaskCardsPanel",type:"function"}),
 (href:Function {name:"canonicalRunHref",type:"function"}),(context:Function {name:"RunDirectoryApiClient.getRunContext",type:"function"}),
 (error:Function {name:"directoryErrorMessage",type:"function"}),(tabs:Variable {name:"RUN_APP_TABS",type:"variable"}),(activeApp:Variable {name:"activeApp",type:"variable"}),
 (state:Function {name:"useState",type:"function"}),(effect:Function {name:"useEffect",type:"function"}),(memo:Function {name:"useMemo",type:"function"}),(ref:Function {name:"useRef",type:"function"}),(pageSize:Variable {name:"RUN_DIRECTORY_PAGE_SIZE",type:"variable"}),(pageBudget:Variable {name:"RUN_DIRECTORY_TREE_BUDGET",type:"variable"}),(selectionLimit:Variable {name:"WORKTREE_SELECTION_MAX_ROWS",type:"variable"}),(directoryErrorType:Class {name:"RunDirectoryError",type:"class"}),
 (f)-[:CONTAINS]->(route),(f)-[:CONTAINS]->(load),(f)-[:CONTAINS]->(shell),(f)-[:CONTAINS]->(loadMore),(f)-[:CONTAINS]->(filtered),(f)-[:CONTAINS]->(selectionLimit),(f)-[:CONTAINS]->(detail),(f)-[:CONTAINS]->(tabs),
 (shell)-[:CONTAINS]->(resolve),(shell)-[:CALLS]->(session),(shell)-[:CALLS]->(href),(shell)-[:CALLS]->(state),(shell)-[:CALLS]->(effect),
 (shell)-[:CALLS]->(detail),(shell)-[:CALLS]->(taskCards),(shell)-[:CALLS]->(loadMore),(loadMore)-[:CALLS]->(worktrees),(loadMore)-[:CALLS]->(directoryErrorType),(loadMore)-[:CALLS]->(error),(shell)-[:CALLS]->(memo),(shell)-[:CALLS]->(ref),(shell)-[:USES]->(pageSize),(shell)-[:USES]->(pageBudget),(shell)-[:USES]->(selectionLimit),(shell)-[:USES]->(filtered),(shell)-[:USES_TYPE]->(selection),(resolve)-[:CALLS]->(context),(resolve)-[:CALLS]->(error),(shell)-[:USES]->(tabs),(shell)-[:USES]->(activeApp);
*/

"use client";

import Link from "next/link";
import { useEffect, useMemo, useRef, useState } from "react";
import { canonicalRunHref, canonicalRunWorktreeHref, RUN_DIRECTORY_PAGE_SIZE, RunDirectoryError, type RunContextEnvelope, type RunDirectoryApiClient, type RunWorktree } from "@/lib/run/runDirectoryApi";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import { directoryErrorMessage, RUN_DIRECTORY_TREE_BUDGET } from "@/lib/run/runDirectoryTree";
import { RunTaskCardsPanel } from "@/components/run/RunTaskCardsPanel";
import { RunBiPanel } from "@/components/run/RunBiPanel";

export interface RunWorkspaceRoute { project_id: string; engineering_run_id: string; worktree_id?: string }
type ContextLoad = { client: RunDirectoryApiClient; route: string; status: "ready"; envelope: RunContextEnvelope } |
  { client: RunDirectoryApiClient; route: string; status: "error"; message: string };
type WorktreeSelectionLoad = { client: RunDirectoryApiClient; route: string; status: "loading" } |
  { client: RunDirectoryApiClient; route: string; status: "ready"; rows: RunWorktree[]; hasMore: boolean; nextCursor: string | null; pagesLoaded: number; cursorHistory: string[]; loadingMore: boolean; pageError: string | null } |
  { client: RunDirectoryApiClient; route: string; status: "error"; message: string };
const WORKTREE_SELECTION_MAX_ROWS = RUN_DIRECTORY_PAGE_SIZE * RUN_DIRECTORY_TREE_BUDGET.pagesPerParent;
const RUN_APP_TABS = ["Inbox", "Task Cards", "Canvas", "Workflow / LangGraph", "Run BI / Benchmark", "Plugin Apps"] as const;
type WorktreeDirectoryPage = Awaited<ReturnType<RunDirectoryApiClient["listWorktrees"]>>;

function validateWorktreeSelectionPage(
  page: WorktreeDirectoryPage,
  existingRows: readonly RunWorktree[],
  requestedCursor: string | null,
  cursorHistory: readonly string[],
): void {
  if (page.rows.length > RUN_DIRECTORY_PAGE_SIZE || page.rows.length > WORKTREE_SELECTION_MAX_ROWS - existingRows.length) {
    throw new RunDirectoryError(0, "page_limit", "Worktree 分页超过单页或选择器读取上限，请刷新目录后重试。");
  }

  const knownIds = new Set(existingRows.map((row) => row.worktree_id));
  for (const row of page.rows) {
    if (knownIds.has(row.worktree_id)) {
      throw new RunDirectoryError(0, "invalid_page", "Worktree 分页包含重复条目，请刷新目录后重试。");
    }
    knownIds.add(row.worktree_id);
  }

  if (page.next_cursor !== null && (page.next_cursor === requestedCursor || cursorHistory.includes(page.next_cursor))) {
    throw new RunDirectoryError(0, "invalid_page", "Worktree 分页游标重复，请刷新目录后重试。");
  }
}

export function RunWorkspaceShell({ route }: { route: RunWorkspaceRoute }) {
  const { client, status, message, setFocus, clearFocus } = useRunDirectory();
  const [load, setLoad] = useState<ContextLoad | null>(null);
  const [worktreeSelection, setWorktreeSelection] = useState<WorktreeSelectionLoad | null>(null);
  const [retry, setRetry] = useState(0);
  const [worktreeRetry, setWorktreeRetry] = useState(0);
  const [worktreeSearch, setWorktreeSearch] = useState("");
  const selectionController = useRef<AbortController | null>(null);
  const [chatScope, setChatScope] = useState<"WORKTREE" | "GLOBAL">("GLOBAL");
  const [activeApp, setActiveApp] = useState<(typeof RUN_APP_TABS)[number]>("Task Cards");
  let href = "";
  try { href = route.worktree_id
    ? canonicalRunWorktreeHref(route.project_id, route.engineering_run_id, route.worktree_id)
    : canonicalRunHref(route.project_id, route.engineering_run_id); } catch { /* Invalid routes never reach the API. */ }
  const projectId = route.project_id.toLowerCase();
  const runId = route.engineering_run_id.toLowerCase();
  const worktreeId = route.worktree_id?.toLowerCase();
  useEffect(function resolveCanonicalRunContext() {
    clearFocus();
    setChatScope("GLOBAL");
    setActiveApp("Task Cards");
    if (!client || !href || status !== "ready") return;
    const controller = new AbortController();
    void client.getRunContext(runId, worktreeId, controller.signal).then((envelope) => {
      if (controller.signal.aborted) return;
      const context = envelope.run_context;
      if (context.project_id !== projectId || context.engineering_run_id !== runId ||
        (worktreeId ? envelope.focus?.worktree_id !== worktreeId : envelope.focus !== null)) {
        throw new RunDirectoryError(0, "scope_mismatch", "当前路径与授权 RunContext 不一致，Worktree 焦点未设置。");
      }
      setLoad({ client, route: href, status: "ready", envelope });
      if (envelope.focus) { setFocus(envelope); setChatScope("WORKTREE"); }
      else setChatScope("GLOBAL");
    }).catch((error: unknown) => {
      if (!controller.signal.aborted) setLoad({ client, route: href, status: "error", message: directoryErrorMessage(error) });
    });
    return () => { controller.abort(); clearFocus(); };
  }, [clearFocus, client, href, projectId, retry, runId, setFocus, status, worktreeId]);
  const current = load && load.client === client && load.route === href ? load : null;
  useEffect(function loadAuthorizedWorktreeChoices() {
    selectionController.current?.abort();
    selectionController.current = null;
    setWorktreeSelection(null);
    setWorktreeSearch("");
    if (!client || !href || !current || current.status !== "ready" || current.envelope.focus || worktreeId) return;
    const controller = new AbortController();
    selectionController.current = controller;
    setWorktreeSelection({ client, route: href, status: "loading" });
    void client.listWorktrees(current.envelope.engineering_run, undefined, controller.signal).then((page) => {
      if (controller.signal.aborted) return;
      validateWorktreeSelectionPage(page, [], null, []);
      setWorktreeSelection({ client, route: href, status: "ready", rows: page.rows, hasMore: page.next_cursor !== null, nextCursor: page.next_cursor, pagesLoaded: 1, cursorHistory: [], loadingMore: false, pageError: null });
    }).catch((error: unknown) => {
      if (!controller.signal.aborted) setWorktreeSelection({ client, route: href, status: "error", message: directoryErrorMessage(error) });
    });
    return () => {
      controller.abort();
      selectionController.current?.abort();
      selectionController.current = null;
    };
  }, [client, current, href, worktreeId, worktreeRetry]);
  const currentWorktreeSelection = worktreeSelection && worktreeSelection.client === client && worktreeSelection.route === href
    ? worktreeSelection : null;
  async function loadMoreWorktreeSelection() {
    if (!client || !current || current.status !== "ready" || currentWorktreeSelection?.status !== "ready") return;
    const selection = currentWorktreeSelection;
    const cursor = selection.hasMore && selection.pagesLoaded < RUN_DIRECTORY_TREE_BUDGET.pagesPerParent
      ? selection.nextCursor
      : null;
    if (!cursor || selection.loadingMore) return;
    if (selection.cursorHistory.includes(cursor)) {
      setWorktreeSelection({ ...selection, pageError: "Worktree 分页游标重复。请重试目录读取。" });
      return;
    }
    const controller = new AbortController();
    selectionController.current?.abort();
    selectionController.current = controller;
    setWorktreeSelection({ ...selection, loadingMore: true, pageError: null });
    try {
      const page = await client.listWorktrees(current.envelope.engineering_run, cursor, controller.signal);
      if (controller.signal.aborted) return;
      validateWorktreeSelectionPage(page, selection.rows, cursor, selection.cursorHistory);
      setWorktreeSelection((previous) => previous?.status === "ready" && previous.client === client && previous.route === href
        ? { ...previous, rows: [...previous.rows, ...page.rows], hasMore: page.next_cursor !== null, nextCursor: page.next_cursor, pagesLoaded: previous.pagesLoaded + 1, cursorHistory: [...previous.cursorHistory, cursor], loadingMore: false, pageError: null }
        : previous);
    } catch (error) {
      if (!controller.signal.aborted) setWorktreeSelection((previous) => previous?.status === "ready" && previous.client === client && previous.route === href
        ? { ...previous, loadingMore: false, pageError: directoryErrorMessage(error) }
        : previous);
    } finally {
      if (selectionController.current === controller) selectionController.current = null;
    }
  }
  const filteredWorktrees = useMemo(() => {
    if (currentWorktreeSelection?.status !== "ready") return [];
    const needle = worktreeSearch.trim().toLocaleLowerCase();
    if (!needle) return currentWorktreeSelection.rows;
    return currentWorktreeSelection.rows.filter((worktree) =>
      `${worktree.name} ${worktree.branch} ${worktree.worktree_id} ${worktree.machine_state} ${worktree.human_state}`.toLocaleLowerCase().includes(needle));
  }, [currentWorktreeSelection, worktreeSearch]);
  const visibleChatScope = current?.status === "ready" && current.envelope.focus && chatScope === "WORKTREE" ? "WORKTREE" : "GLOBAL";
  if (!href) return <section className="p-6" role="alert"><h1 className="text-lg font-semibold">Run 路径无效</h1><p className="mt-2 text-sm text-ink-mute">Project 与 Run 必须使用有效目录身份。</p></section>;
  if (status !== "ready" || !client) return <section className="p-6" role="status"><h1 className="text-lg font-semibold">Engineering Run</h1><p className="mt-2 text-sm text-ink-mute">{message}</p></section>;
  if (!current) return <section className="p-6" role="status">正在验证 Project 与 Engineering Run 授权…</section>;
  if (current.status === "error") return <section className="p-6" role="alert"><h1 className="text-lg font-semibold">Run 工作区暂不可用</h1><p className="my-3 text-sm text-ink-mute">{current.message}</p><button type="button" className="rounded border border-line px-3 py-1 text-sm" onClick={() => { setLoad(null); setRetry((value) => value + 1); }}>重试授权上下文</button></section>;
  const { envelope } = current;
  const focus = envelope.focus;
  const hooksQuery = new URLSearchParams({ project_id: projectId });
  if (worktreeId) hooksQuery.set("worktree_id", worktreeId);
  return <section data-testid="engineering-run-workspace" data-engineering-run-id={runId} data-focus-worktree-id={worktreeId ?? ""} className="flex min-h-[calc(100vh-4rem)] flex-col p-4 md:p-6">
    <header className="border-b border-line pb-4">
      <p className="break-all text-xs text-ink-mute">Project {projectId} / Engineering Run{focus ? ` / Worktree ${focus.name} / 本地 Branch ${focus.branch.trim() || "未关联"}` : ""}</p>
      <h1 className="mt-2 break-words text-xl font-semibold">{envelope.engineering_run.title}</h1>
      {focus && <p className="mt-1 break-words text-sm">Checkout 焦点：<strong>{focus.name}</strong> · {focus.machine_state}{focus.archived && " · 已归档"}</p>}
      {focus && <p className="mt-2 text-xs text-ink-mute">协作、任务、Canvas、Workflow、分析与插件的工作区属于此 Engineering Run。Worktree 提供本地 checkout 焦点。</p>}
    </header>
    {focus && <div role="tablist" aria-label="Engineering Run Apps" className="mt-4 flex flex-wrap gap-2">
      {RUN_APP_TABS.map((tab) => <button role="tab" aria-selected={activeApp === tab} aria-controls="run-app-content" disabled={tab !== "Task Cards" && tab !== "Run BI / Benchmark"} onClick={() => setActiveApp(tab)} type="button" key={tab} className={`rounded border border-line px-3 py-2 text-xs ${activeApp === tab ? "bg-accent/15 text-accent" : "text-ink-mute"}`}>{tab}{tab === "Task Cards" || tab === "Run BI / Benchmark" ? "" : " · 待迁移"}</button>)}
    </div>}
    {focus ? <>
      <div id="run-app-content" role="tabpanel" aria-label={activeApp} className="my-4">
        {activeApp === "Task Cards" && <RunTaskCardsPanel key={runId} client={current.client} context={envelope} available={envelope.capabilities.task_cards_available} />}
        {activeApp === "Run BI / Benchmark" && <RunBiPanel key={`${projectId}/${runId}/${worktreeId}`} client={current.client} run={envelope.engineering_run} />}
      </div>
      <div role="status" className="mb-4 rounded border border-line bg-bg-soft p-4 text-sm">
        <p>{envelope.capabilities.task_cards_available ? "Task Cards 已按当前 Run 授权开放读取与创建入口。" : "服务端尚未开放 Task Cards 读取能力；页面保持禁用，不会回退到 Worktree 任务或演示数据。"} Canvas、Inbox、Workflow、BI/Benchmark 与插件仍是同级 Run App。</p>
        <p className="mt-2">{envelope.capabilities.execution_admission_available ? "服务端执行准入已开放，但卡内 CLI Runtime 与结果验证界面尚未装配。" : "服务端执行准入尚未开放。"} CLI 将从任务卡内打开；Runtime、取消/恢复、独立验证和结果回写完成前，执行保持禁用。</p>
      </div>
      <div className="mt-4 flex flex-wrap items-center gap-3 text-xs">
        <span className="text-ink-mute">有效 Hook 策略与执行结果投影待接入。</span>
        <Link href={`/settings/advanced/hooks?${hooksQuery}`} className="text-accent underline">高级设置 → Hooks</Link>
      </div>
    </> : <section data-testid="run-worktree-selection" aria-labelledby="run-worktree-selection-title" className="my-4 rounded border border-line bg-bg-soft p-4">
      <h2 id="run-worktree-selection-title" className="font-semibold">选择 Worktree 继续</h2>
      <p className="mt-1 text-sm text-ink-mute">当前 Run 尚未验证 Worktree 焦点。选择此 Run 已授权目录中的 Worktree 后，才会打开 Run App 并加载其数据。</p>
      {currentWorktreeSelection?.status === "loading" && <p role="status" className="mt-3 text-sm text-ink-mute">正在读取此 Run 的授权 Worktree 目录…</p>}
      {currentWorktreeSelection?.status === "error" && <div role="alert" className="mt-3 text-sm">
        <p>{currentWorktreeSelection.message}</p>
        <button type="button" className="mt-2 rounded border border-line px-3 py-1" onClick={() => { setWorktreeSelection(null); setWorktreeRetry((value) => value + 1); }}>重试 Worktree 目录</button>
      </div>}
      {currentWorktreeSelection?.status === "ready" && currentWorktreeSelection.rows.length === 0 && <p className="mt-3 text-sm text-ink-mute">此 Run 当前没有已授权的 Worktree。请先通过受控 Worktree 创建流程完成创建或绑定，再回到此 Run 选择。</p>}
      {currentWorktreeSelection?.status === "ready" && currentWorktreeSelection.rows.length > 0 && <div className="mt-4">
        <label htmlFor="run-worktree-search" className="block text-xs font-medium">搜索已加载的 Worktree</label>
        <input id="run-worktree-search" type="search" maxLength={128} value={worktreeSearch} onChange={(event) => setWorktreeSearch(event.target.value)} placeholder="名称、Branch、状态或 ID" className="mt-1 w-full rounded border border-line bg-bg-card px-3 py-2 text-sm focus:outline focus:outline-2 focus:outline-accent" />
        <p role="status" className="mt-2 text-xs text-ink-mute">显示 {filteredWorktrees.length} / {currentWorktreeSelection.rows.length} 项；搜索范围是已加载条目，单次最多 {WORKTREE_SELECTION_MAX_ROWS} 项。</p>
        {filteredWorktrees.length > 0 ? <ul className="mt-3 grid gap-2 sm:grid-cols-2" aria-label="已加载的授权 Worktree">
          {filteredWorktrees.map((worktree) => <li key={worktree.worktree_id}>
            <Link prefetch={false} href={canonicalRunWorktreeHref(projectId, runId, worktree.worktree_id)} className="block rounded border border-line bg-bg-card p-3 transition-colors hover:border-accent hover:bg-accent/5 focus:outline focus:outline-2 focus:outline-accent">
              <span className="block truncate text-sm font-medium">{worktree.name}</span>
              <span className="mt-1 block truncate text-xs text-ink-mute">{worktree.machine_state}{worktree.archived ? " · 已归档" : ""} · 本地 Branch {worktree.branch.trim() || "未关联"}</span>
            </Link>
          </li>)}
        </ul> : <p role="status" className="mt-3 rounded border border-line p-3 text-sm text-ink-mute">没有匹配当前已加载条目的 Worktree。清除搜索或加载更多条目后再试。</p>}
        {currentWorktreeSelection.loadingMore && <p role="status" className="mt-3 text-sm text-ink-mute">正在读取下一页；当前结果会保留到加载完成。</p>}
        {currentWorktreeSelection.pageError && <div role="alert" className="mt-3 text-sm"><p>{currentWorktreeSelection.pageError}</p><button type="button" disabled={currentWorktreeSelection.loadingMore} className="mt-2 rounded border border-line px-3 py-1 disabled:opacity-50" onClick={() => { setWorktreeSelection(null); setWorktreeRetry((value) => value + 1); }}>刷新 Worktree 目录</button></div>}
        {currentWorktreeSelection.hasMore && currentWorktreeSelection.pagesLoaded < RUN_DIRECTORY_TREE_BUDGET.pagesPerParent
          ? <button type="button" disabled={currentWorktreeSelection.loadingMore} onClick={() => void loadMoreWorktreeSelection()} className="mt-3 rounded border border-line px-3 py-2 text-sm hover:border-accent focus:outline focus:outline-2 focus:outline-accent disabled:opacity-50">{currentWorktreeSelection.loadingMore ? "正在加载…" : "加载更多 Worktree"}</button>
          : currentWorktreeSelection.hasMore && <p className="mt-3 text-xs text-ink-mute">此选择器最多加载 {WORKTREE_SELECTION_MAX_ROWS} 项。请在左侧 Project 目录展开此 Run，继续查看下一组 Worktree。</p>}
      </div>}
    </section>}
    {focus && <RunContextDetails envelope={envelope} />}
    <footer className="mt-auto border-t border-line pt-4">
      <div className="flex items-center gap-2 text-xs"><span>聊天范围</span>
        {(focus ? ["WORKTREE", "GLOBAL"] as const : ["GLOBAL"] as const).map((scope) => <button type="button" aria-pressed={visibleChatScope === scope} key={scope} onClick={() => setChatScope(scope)} className={`rounded border border-line px-2 py-1 ${visibleChatScope === scope ? "bg-accent/15 text-accent" : "text-ink-mute"}`}>{scope}</button>)}
      </div>
      <div className="mt-2 flex gap-2"><input disabled aria-label={`${visibleChatScope} 聊天`} placeholder="聊天 runtime 尚未装配" className="min-w-0 flex-1 rounded border border-line bg-bg-soft px-3 py-2 text-sm" /><button disabled type="button" className="rounded border border-line px-3 text-xs text-ink-mute">发送</button></div>
    </footer>
  </section>;
}

function RunContextDetails({ envelope }: { envelope: RunContextEnvelope }) {
  const { run_context: context, focus } = envelope;
  return <details className="rounded border border-line p-3 text-xs">
    <summary className="cursor-pointer font-semibold">已授权的 Run 与 checkout 上下文</summary>
    <dl className="mt-3 grid grid-cols-[max-content_1fr] gap-x-3 gap-y-2 break-all">
      <dt className="text-ink-mute">Run</dt><dd>{context.engineering_run_id}</dd>
      <dt className="text-ink-mute">Run 授权版本</dt><dd>{context.context_version}</dd>
      <dt className="text-ink-mute">Worktree</dt><dd>{focus?.worktree_id ?? "未绑定"}</dd>
      <dt className="text-ink-mute">Checkout 版本</dt><dd>{envelope.focus_version ?? "无 Worktree 焦点"}</dd>
      <dt className="text-ink-mute">Workspace</dt><dd>{focus?.workspace_id ?? "未绑定"}</dd>
      <dt className="text-ink-mute">Agent / Session</dt><dd>{focus?.agent_id ?? "未绑定"} / {focus?.agent_session_id ?? "未绑定"}</dd>
      <dt className="text-ink-mute">Runtime</dt><dd>{focus?.runtime_id ?? "未绑定"}</dd>
      <dt className="text-ink-mute">授权</dt><dd>Project {context.authorization.project.role} v{context.authorization.project.version} · Run {context.authorization.engineering_run.role} v{context.authorization.engineering_run.version} · 远端 Branch {context.authorization.branch ? `${context.authorization.branch.role} v${context.authorization.branch.version}` : "未关联"}</dd>
    </dl>
  </details>;
}
