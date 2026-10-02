/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/RunTaskCardsPanel.tsx",type:"file",language:"tsx"}),
 (props:Class {name:"RunTaskCardsPanelProps",type:"class",language:"typescript"}),
 (panel:Function {name:"RunTaskCardsPanel",type:"function",signature:"RunTaskCardsPanel(props) -> JSX.Element",visibility:"public",complexity:"moderate"}),
 (load:Function {name:"loadRunTaskPage",type:"function",language:"typescript",complexity:"moderate"}),
 (success:Function {name:"taskPageSuccessCallback",type:"function",language:"typescript",complexity:"simple"}),(failure:Function {name:"taskPageFailureCallback",type:"function",language:"typescript",complexity:"simple"}),(cleanup:Function {name:"abortTaskPageRequest",type:"function",language:"typescript",complexity:"simple"}),
 (next:Function {name:"goNextTaskCardPage",type:"function",language:"typescript",complexity:"simple"}),(previous:Function {name:"goPreviousTaskCardPage",type:"function",language:"typescript",complexity:"simple"}),(retry:Function {name:"retryTaskCardPage",type:"function",language:"typescript",complexity:"simple"}),
 (renderCard:Function {name:"renderTaskCard",type:"function",language:"typescript",complexity:"moderate"}),(renderLabel:Function {name:"renderTaskLabel",type:"function",language:"typescript",complexity:"simple"}),
 (cursorStack:Variable {name:"cursorStack",type:"variable"}),(pageIndex:Variable {name:"pageIndex",type:"variable"}),(revision:Variable {name:"revision",type:"variable"}),(phase:Variable {name:"phase",type:"variable"}),(page:Variable {name:"page",type:"variable"}),(error:Variable {name:"error",type:"variable"}),(cursor:Variable {name:"cursor",type:"variable"}),(available:Variable {name:"available",type:"variable"}),(pageLimit:Variable {name:"RUN_TASK_CARD_MAX_PAGE_COUNT",type:"variable"}),(pageSize:Variable {name:"RUN_TASK_CARD_PAGE_SIZE",type:"variable"}),
 (setCursor:Function {name:"setCursorStack",type:"function"}),(setIndex:Function {name:"setPageIndex",type:"function"}),(setRevision:Function {name:"setRevision",type:"function"}),(setPhase:Function {name:"setPhase",type:"function"}),(setPage:Function {name:"setPage",type:"function"}),(setError:Function {name:"setError",type:"function"}),(useState:Function {name:"useState",type:"function"}),(useEffect:Function {name:"useEffect",type:"function"}),(createAbort:Function {name:"AbortController",type:"function"}),(api:Function {name:"RunDirectoryApiClient.listRunWorkItems",type:"function"}),(errorMessage:Function {name:"directoryErrorMessage",type:"function"}),(abort:Function {name:"AbortController.abort",type:"function"}),
 (f)-[:CONTAINS]->(props),(f)-[:CONTAINS]->(panel),(f)-[:CONTAINS]->(load),(f)-[:CONTAINS]->(success),(f)-[:CONTAINS]->(failure),(f)-[:CONTAINS]->(cleanup),(f)-[:CONTAINS]->(next),(f)-[:CONTAINS]->(previous),(f)-[:CONTAINS]->(retry),(f)-[:CONTAINS]->(renderCard),(f)-[:CONTAINS]->(renderLabel),(f)-[:CONTAINS]->(pageLimit),(f)-[:CONTAINS]->(pageSize),
 (panel)-[:CALLS]->(useState),(panel)-[:CALLS]->(useEffect),(panel)-[:CONTAINS]->(next),(panel)-[:CONTAINS]->(previous),(panel)-[:CONTAINS]->(retry),(panel)-[:USES]->(cursorStack),(panel)-[:USES]->(pageIndex),(panel)-[:USES]->(revision),(panel)-[:USES]->(phase),(panel)-[:USES]->(page),(panel)-[:USES]->(error),(panel)-[:USES]->(cursor),(panel)-[:USES]->(available),(panel)-[:CALLS]->(renderCard),(renderCard)-[:CALLS]->(renderLabel),
 (panel)-[:CALLS]->(setCursor),(panel)-[:CALLS]->(setIndex),(panel)-[:CALLS]->(setPage),(panel)-[:CALLS]->(setError),(panel)-[:CALLS]->(setPhase),(panel)-[:USES]->(pageSize),
 (load)-[:CALLS]->(api),(load)-[:CALLS]->(createAbort),(load)-[:CALLS]->(setCursor),(load)-[:CALLS]->(setIndex),(load)-[:CALLS]->(setPage),(load)-[:CALLS]->(setError),(load)-[:CALLS]->(setPhase),(load)-[:CALLS]->(abort),(load)-[:CALLS]->(success),(load)-[:CALLS]->(failure),(load)-[:CALLS]->(cleanup),(load)-[:USES]->(available),(load)-[:USES]->(cursor),(load)-[:USES]->(page),(load)-[:USES]->(phase),(success)-[:CALLS]->(setPage),(success)-[:CALLS]->(setPhase),(success)-[:USES]->(page),(success)-[:USES]->(phase),(failure)-[:CALLS]->(errorMessage),(failure)-[:CALLS]->(setPage),(failure)-[:CALLS]->(setError),(failure)-[:CALLS]->(setPhase),(failure)-[:USES]->(page),(failure)-[:USES]->(error),(failure)-[:USES]->(phase),(cleanup)-[:CALLS]->(abort),
 (next)-[:CALLS]->(setCursor),(next)-[:CALLS]->(setIndex),(next)-[:USES]->(page),(next)-[:USES]->(cursorStack),(next)-[:USES]->(pageIndex),(next)-[:USES]->(pageLimit),(previous)-[:CALLS]->(setIndex),(previous)-[:USES]->(pageIndex),(retry)-[:CALLS]->(setRevision),(retry)-[:USES]->(revision),(renderCard)-[:USES]->(page),(renderLabel)-[:USES]->(pageSize);
*/

"use client";

import { useEffect, useState } from "react";
import { directoryErrorMessage } from "@/lib/run/runDirectoryTree";
import { RUN_TASK_CARD_PAGE_SIZE, type EngineeringRun, type RunDirectoryApiClient, type RunTaskCardPage } from "@/lib/run/runDirectoryApi";

interface RunTaskCardsPanelProps {
  client: RunDirectoryApiClient;
  run: EngineeringRun;
  available: boolean;
}

const RUN_TASK_CARD_MAX_PAGE_COUNT = 100;

export function RunTaskCardsPanel({ client, run, available }: RunTaskCardsPanelProps) {
  const [cursorStack, setCursorStack] = useState<Array<string | undefined>>([undefined]);
  const [pageIndex, setPageIndex] = useState(0);
  const [revision, setRevision] = useState(0);
  const [phase, setPhase] = useState<"loading" | "ready" | "error">("loading");
  const [page, setPage] = useState<RunTaskCardPage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const cursor = cursorStack[pageIndex];

  useEffect(function loadRunTaskPage() {
    if (!available) {
      setCursorStack([undefined]);
      setPageIndex(0);
      setPage(null);
      setError(null);
      setPhase("ready");
      return;
    }
    const controller = new AbortController();
    setPhase("loading");
    setError(null);
    void client.listRunWorkItems(run, cursor, controller.signal).then((result) => {
      if (controller.signal.aborted) return;
      setPage(result);
      setPhase("ready");
    }).catch((reason: unknown) => {
      if (controller.signal.aborted) return;
      setPage(null);
      setError(directoryErrorMessage(reason));
      setPhase("error");
    });
    return () => controller.abort();
  }, [available, client, cursor, revision, run]);

  if (!available) return <section role="tabpanel" aria-label="Task Cards" className="rounded border border-line p-4">
    <h2 className="font-semibold">Task Cards</h2>
    <p className="mt-2 text-sm text-ink-mute">服务端尚未开放 Run-owned Apps。此处不会读取 Worktree 旧任务投影或本地演示数据；Owner migration、数据库角色与 RLS 验收完成后才会加载真实 Run 任务。</p>
  </section>;

  const canGoNext = page?.next_after !== null && page?.next_after !== undefined && pageIndex + 1 < RUN_TASK_CARD_MAX_PAGE_COUNT;
  const canGoPrevious = pageIndex > 0;
  function goNextTaskCardPage() {
    if (!page?.next_after || !canGoNext) return;
    setCursorStack([...cursorStack.slice(0, pageIndex + 1), page.next_after]);
    setPageIndex(pageIndex + 1);
  }
  function goPreviousTaskCardPage() { setPageIndex(Math.max(0, pageIndex - 1)); }
  function retryTaskCardPage() { setRevision(revision + 1); }

  return <section role="tabpanel" aria-label="Task Cards" className="rounded border border-line p-4" data-engineering-run-id={run.engineering_run_id}>
    <header className="flex flex-wrap items-start justify-between gap-3">
      <div>
        <h2 className="font-semibold">Task Cards</h2>
        <p className="mt-1 text-xs text-ink-mute">按 Engineering Run 所有权列出。当前 Worktree 仅是 checkout 焦点。</p>
      </div>
      <button type="button" disabled={phase === "loading"} onClick={retryTaskCardPage} className="rounded border border-line px-3 py-1 text-xs disabled:opacity-50">刷新</button>
    </header>

    {phase === "loading" && <p className="mt-4 text-sm text-ink-mute" role="status">正在读取 Run 任务…</p>}
    {phase === "error" && <div className="mt-4 text-sm" role="alert"><p>{error}</p><button type="button" onClick={retryTaskCardPage} className="mt-2 rounded border border-line px-3 py-1 text-xs">重试</button></div>}
    {phase === "ready" && page?.rows.length === 0 && <p className="mt-4 text-sm text-ink-mute">此 Engineering Run 当前没有已归属的任务。未归属历史任务不会自动迁入。</p>}
    {phase === "ready" && page && page.rows.length > 0 && <>
      <ul className="mt-4 grid gap-3">
        {page.rows.map((card) => <li key={card.work_item_id} className="rounded border border-line bg-bg-soft p-3" data-work-item-id={card.work_item_id}>
          <article>
            <header className="flex flex-wrap items-start justify-between gap-2">
              <div className="min-w-0">
                <h3 className="break-words font-medium">{card.title}</h3>
                <p className="mt-1 text-xs text-ink-mute">{card.item_type} · {card.priority} · {card.lifecycle.status} · Review {card.lifecycle.review_state}</p>
              </div>
              <button type="button" disabled aria-label={`在任务卡 ${card.title} 内打开 CLI（尚未启用）`} title="执行准入、Runtime sandbox、独立验证与结果回写尚未装配" className="shrink-0 rounded border border-line px-3 py-1 text-xs text-ink-mute disabled:cursor-not-allowed disabled:opacity-50">打开 CLI · 暂不可用</button>
            </header>
            {card.labels.length > 0 && <ul aria-label="任务标签" className="mt-2 flex flex-wrap gap-1">{card.labels.slice(0, 8).map((label) => <li key={label} className="rounded bg-bg px-2 py-0.5 text-[11px]">{label}</li>)}{card.labels.length > 8 && <li className="text-[11px] text-ink-mute">+{card.labels.length - 8}</li>}</ul>}
            {card.description && <details className="mt-2 text-sm"><summary className="cursor-pointer text-xs text-ink-mute">任务描述</summary><p className="mt-2 whitespace-pre-wrap break-words">{card.description}</p></details>}
            <p className="mt-2 break-all text-[11px] text-ink-mute">Work Item {card.work_item_id} · Active Worktree {card.lifecycle.active_worktree_id ?? "未绑定"}</p>
          </article>
        </li>)}
      </ul>
      <nav aria-label="Task Card 分页" className="mt-4 flex items-center justify-between gap-2 text-xs">
        <button type="button" disabled={!canGoPrevious} onClick={goPreviousTaskCardPage} className="rounded border border-line px-3 py-1 disabled:opacity-50">上一页</button>
        <span className="text-ink-mute">第 {pageIndex + 1} 页 · 每页最多 {RUN_TASK_CARD_PAGE_SIZE} 项</span>
        <button type="button" disabled={!canGoNext} onClick={goNextTaskCardPage} className="rounded border border-line px-3 py-1 disabled:opacity-50">下一页</button>
      </nav>
      {pageIndex + 1 === RUN_TASK_CARD_MAX_PAGE_COUNT && canGoNext && <p className="mt-2 text-xs text-ink-mute">已达到单次浏览页数上限，请刷新后从第一批继续浏览。</p>}
    </>}
  </section>;
}
