/*
@cypher schema=1 source_sha256=1c7060cbb8510003939fd73157574d3c56b197a8a2fb57d78385a9c823c25475
MERGE (self:File {path:"frontend/src/components/run/RunTaskCardsPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::RunTaskCardsPanel",kind:"function"})
MERGE (submit:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::submitTaskCardCreate",kind:"function"})
MERGE (target:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::taskCardCreateTarget",kind:"function"})
MERGE (sameTarget:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::sameTaskCardCreateTarget",kind:"function"})
MERGE (api:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.createRunTaskCard",kind:"method"})
MERGE (context:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::RunContextEnvelope",kind:"type"})
MERGE (input:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::CreateRunTaskCardInput",kind:"type"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:DEFINES]->(submit)
MERGE (self)-[:DEFINES]->(target)
MERGE (self)-[:DEFINES]->(sameTarget)
MERGE (panel)-[:USES_TYPE]->(context)
MERGE (submit)-[:CALLS]->(target)
MERGE (submit)-[:CALLS]->(sameTarget)
MERGE (submit)-[:CALLS]->(api)
MERGE (submit)-[:USES_TYPE]->(input)
@endcypher
*/
/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/RunTaskCardsPanel.tsx",type:"file",language:"tsx"}),
 (props:Class {name:"RunTaskCardsPanelProps",type:"class",language:"typescript"}),(pendingType:Class {name:"PendingTaskCardCreate",type:"class",language:"typescript"}),
 (panel:Function {name:"RunTaskCardsPanel",type:"function",signature:"RunTaskCardsPanel(props) -> JSX.Element",visibility:"public",complexity:"moderate"}),
 (openForm:Function {name:"openTaskCardForm",type:"function",complexity:"simple"}),(closeForm:Function {name:"closeTaskCardForm",type:"function",complexity:"simple"}),(submitCreate:Function {name:"submitTaskCardCreate",type:"function",complexity:"moderate"}),
 (load:Function {name:"loadRunTaskPage",type:"function",language:"typescript",complexity:"moderate"}),
 (success:Function {name:"taskPageSuccessCallback",type:"function",language:"typescript",complexity:"simple"}),(failure:Function {name:"taskPageFailureCallback",type:"function",language:"typescript",complexity:"simple"}),(cleanup:Function {name:"abortTaskPageRequest",type:"function",language:"typescript",complexity:"simple"}),
 (next:Function {name:"goNextTaskCardPage",type:"function",language:"typescript",complexity:"simple"}),(previous:Function {name:"goPreviousTaskCardPage",type:"function",language:"typescript",complexity:"simple"}),(retry:Function {name:"retryTaskCardPage",type:"function",language:"typescript",complexity:"simple"}),
 (renderCard:Function {name:"renderTaskCard",type:"function",language:"typescript",complexity:"moderate"}),(renderLabel:Function {name:"renderTaskLabel",type:"function",language:"typescript",complexity:"simple"}),
 (cursorStack:Variable {name:"cursorStack",type:"variable"}),(pageIndex:Variable {name:"pageIndex",type:"variable"}),(revision:Variable {name:"revision",type:"variable"}),(phase:Variable {name:"phase",type:"variable"}),(page:Variable {name:"page",type:"variable"}),(error:Variable {name:"error",type:"variable"}),(cursor:Variable {name:"cursor",type:"variable"}),(available:Variable {name:"available",type:"variable"}),(pageLimit:Variable {name:"RUN_TASK_CARD_MAX_PAGE_COUNT",type:"variable"}),(pageSize:Variable {name:"RUN_TASK_CARD_PAGE_SIZE",type:"variable"}),
 (context:Variable {name:"context",type:"variable"}),(run:Variable {name:"run",type:"variable"}),(focus:Variable {name:"focus",type:"variable"}),(canCreate:Variable {name:"canCreateTaskCard",type:"variable"}),(formOpen:Variable {name:"createFormOpen",type:"variable"}),(createPhase:Variable {name:"createPhase",type:"variable"}),(createError:Variable {name:"createError",type:"variable"}),(createSuccess:Variable {name:"createSuccess",type:"variable"}),(pendingCreate:Variable {name:"pendingCreate",type:"variable"}),
 (setCursor:Function {name:"setCursorStack",type:"function"}),(setIndex:Function {name:"setPageIndex",type:"function"}),(setRevision:Function {name:"setRevision",type:"function"}),(setPhase:Function {name:"setPhase",type:"function"}),(setPage:Function {name:"setPage",type:"function"}),(setError:Function {name:"setError",type:"function"}),(useState:Function {name:"useState",type:"function"}),(useEffect:Function {name:"useEffect",type:"function"}),(createAbort:Function {name:"AbortController",type:"function"}),(api:Function {name:"RunDirectoryApiClient.listRunWorkItems",type:"function"}),(errorMessage:Function {name:"directoryErrorMessage",type:"function"}),(abort:Function {name:"AbortController.abort",type:"function"}),
 (setFormOpen:Function {name:"setCreateFormOpen",type:"function"}),(setCreatePhase:Function {name:"setCreatePhase",type:"function"}),(setCreateError:Function {name:"setCreateError",type:"function"}),(setCreateSuccess:Function {name:"setCreateSuccess",type:"function"}),(setPendingCreate:Function {name:"setPendingCreate",type:"function"}),(createApi:Function {name:"RunDirectoryApiClient.createRunTaskCard",type:"function"}),(formEvent:Class {name:"FormEvent",type:"class"}),(formData:Class {name:"FormData",type:"class"}),(randomUuid:Function {name:"crypto.randomUUID",type:"function"}),(taskPriorities:Variable {name:"RUN_TASK_CARD_PRIORITIES",type:"variable"}),(titleLimit:Variable {name:"RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS",type:"variable"}),(descriptionLimit:Variable {name:"RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS",type:"variable"}),
 (f)-[:CONTAINS]->(props),(f)-[:CONTAINS]->(pendingType),(f)-[:CONTAINS]->(panel),(f)-[:CONTAINS]->(load),(f)-[:CONTAINS]->(success),(f)-[:CONTAINS]->(failure),(f)-[:CONTAINS]->(cleanup),(f)-[:CONTAINS]->(next),(f)-[:CONTAINS]->(previous),(f)-[:CONTAINS]->(retry),(f)-[:CONTAINS]->(openForm),(f)-[:CONTAINS]->(closeForm),(f)-[:CONTAINS]->(submitCreate),(f)-[:CONTAINS]->(renderCard),(f)-[:CONTAINS]->(renderLabel),(f)-[:CONTAINS]->(pageLimit),(f)-[:CONTAINS]->(pageSize),
 (panel)-[:CALLS]->(useState),(panel)-[:CALLS]->(useEffect),(panel)-[:CONTAINS]->(next),(panel)-[:CONTAINS]->(previous),(panel)-[:CONTAINS]->(retry),(panel)-[:CONTAINS]->(openForm),(panel)-[:CONTAINS]->(closeForm),(panel)-[:CONTAINS]->(submitCreate),(panel)-[:USES]->(cursorStack),(panel)-[:USES]->(pageIndex),(panel)-[:USES]->(revision),(panel)-[:USES]->(phase),(panel)-[:USES]->(page),(panel)-[:USES]->(error),(panel)-[:USES]->(cursor),(panel)-[:USES]->(available),(panel)-[:USES]->(context),(panel)-[:USES]->(run),(panel)-[:USES]->(focus),(panel)-[:USES]->(canCreate),(panel)-[:USES]->(formOpen),(panel)-[:USES]->(createPhase),(panel)-[:USES]->(createError),(panel)-[:USES]->(createSuccess),(panel)-[:USES]->(pendingCreate),(panel)-[:CALLS]->(renderCard),(renderCard)-[:CALLS]->(renderLabel),
 (panel)-[:CALLS]->(setCursor),(panel)-[:CALLS]->(setIndex),(panel)-[:CALLS]->(setPage),(panel)-[:CALLS]->(setError),(panel)-[:CALLS]->(setPhase),(panel)-[:USES]->(pageSize),
 (load)-[:CALLS]->(api),(load)-[:CALLS]->(createAbort),(load)-[:CALLS]->(setCursor),(load)-[:CALLS]->(setIndex),(load)-[:CALLS]->(setPage),(load)-[:CALLS]->(setError),(load)-[:CALLS]->(setPhase),(load)-[:CALLS]->(abort),(load)-[:CALLS]->(success),(load)-[:CALLS]->(failure),(load)-[:CALLS]->(cleanup),(load)-[:USES]->(available),(load)-[:USES]->(cursor),(load)-[:USES]->(page),(load)-[:USES]->(phase),(success)-[:CALLS]->(setPage),(success)-[:CALLS]->(setPhase),(success)-[:USES]->(page),(success)-[:USES]->(phase),(failure)-[:CALLS]->(errorMessage),(failure)-[:CALLS]->(setPage),(failure)-[:CALLS]->(setError),(failure)-[:CALLS]->(setPhase),(failure)-[:USES]->(page),(failure)-[:USES]->(error),(failure)-[:USES]->(phase),(cleanup)-[:CALLS]->(abort),
 (next)-[:CALLS]->(setCursor),(next)-[:CALLS]->(setIndex),(next)-[:USES]->(page),(next)-[:USES]->(cursorStack),(next)-[:USES]->(pageIndex),(next)-[:USES]->(pageLimit),(previous)-[:CALLS]->(setIndex),(previous)-[:USES]->(pageIndex),(retry)-[:CALLS]->(setRevision),(retry)-[:USES]->(revision),(openForm)-[:CALLS]->(setFormOpen),(openForm)-[:CALLS]->(setCreateError),(openForm)-[:CALLS]->(setCreateSuccess),(closeForm)-[:CALLS]->(setFormOpen),(closeForm)-[:USES]->(pendingCreate),(submitCreate)-[:CALLS]->(formData),(submitCreate)-[:CALLS]->(randomUuid),(submitCreate)-[:CALLS]->(createApi),(submitCreate)-[:CALLS]->(errorMessage),(submitCreate)-[:CALLS]->(setPendingCreate),(submitCreate)-[:CALLS]->(setCreatePhase),(submitCreate)-[:CALLS]->(setCreateError),(submitCreate)-[:CALLS]->(setCreateSuccess),(submitCreate)-[:CALLS]->(setCursor),(submitCreate)-[:CALLS]->(setIndex),(submitCreate)-[:CALLS]->(setRevision),(submitCreate)-[:USES]->(context),(submitCreate)-[:USES]->(pendingCreate),(submitCreate)-[:USES]->(createPhase),(submitCreate)-[:USES]->(canCreate),(submitCreate)-[:USES]->(taskPriorities),(submitCreate)-[:USES]->(titleLimit),(submitCreate)-[:USES]->(descriptionLimit),(renderCard)-[:USES]->(page),(renderLabel)-[:USES]->(pageSize);
*/

"use client";

import { useEffect, useState, type FormEvent } from "react";
import { directoryErrorMessage } from "@/lib/run/runDirectoryTree";
import {
  RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS,
  RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS,
  RUN_TASK_CARD_PAGE_SIZE,
  RUN_TASK_CARD_PRIORITIES,
  type CreateRunTaskCardInput,
  type RunContextEnvelope,
  type RunDirectoryApiClient,
  type RunTaskCardPage,
  type RunTaskCardPriority,
} from "@/lib/run/runDirectoryApi";

interface RunTaskCardsPanelProps {
  client: RunDirectoryApiClient;
  context: RunContextEnvelope;
  available: boolean;
}

interface PendingTaskCardCreate {
  input: CreateRunTaskCardInput;
  idempotency_key: string;
  target: {
    project_id: string;
    repository_id: string;
    branch_id: string;
    engineering_run_id: string;
    worktree_id: string;
  };
}

function taskCardCreateTarget(context: RunContextEnvelope): PendingTaskCardCreate["target"] | null {
  const focus = context.focus;
  if (!focus) return null;
  return {
    project_id: context.engineering_run.project_id,
    repository_id: context.engineering_run.repository_id,
    branch_id: context.engineering_run.branch_id,
    engineering_run_id: context.engineering_run.engineering_run_id,
    worktree_id: focus.worktree_id,
  };
}

function sameTaskCardCreateTarget(left: PendingTaskCardCreate["target"], right: PendingTaskCardCreate["target"] | null): boolean {
  return right !== null && left.project_id === right.project_id && left.repository_id === right.repository_id &&
    left.branch_id === right.branch_id && left.engineering_run_id === right.engineering_run_id && left.worktree_id === right.worktree_id;
}

const RUN_TASK_CARD_MAX_PAGE_COUNT = 100;

export function RunTaskCardsPanel({ client, context, available }: RunTaskCardsPanelProps) {
  const run = context.engineering_run;
  const focus = context.focus;
  const [cursorStack, setCursorStack] = useState<Array<string | undefined>>([undefined]);
  const [pageIndex, setPageIndex] = useState(0);
  const [revision, setRevision] = useState(0);
  const [phase, setPhase] = useState<"loading" | "ready" | "error">("loading");
  const [page, setPage] = useState<RunTaskCardPage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [createFormOpen, setCreateFormOpen] = useState(false);
  const [createPhase, setCreatePhase] = useState<"idle" | "submitting" | "error">("idle");
  const [createError, setCreateError] = useState<string | null>(null);
  const [createSuccess, setCreateSuccess] = useState<string | null>(null);
  const [pendingCreate, setPendingCreate] = useState<PendingTaskCardCreate | null>(null);
  const cursor = cursorStack[pageIndex];
  const runId = run.engineering_run_id;
  const canCreateTaskCard = Boolean(focus && !focus.archived &&
    context.run_context.project_id === run.project_id &&
    context.run_context.branch_id === run.branch_id &&
    context.run_context.engineering_run_id === runId &&
    context.branch.project_id === run.project_id &&
    context.branch.repository_id === run.repository_id &&
    context.branch.branch_id === run.branch_id &&
    focus.project_id === run.project_id &&
    focus.repository_id === run.repository_id &&
    focus.engineering_run_id === runId &&
    context.run_context.granted_scopes.includes("work-item:write"));

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
  function openTaskCardForm() {
    setCreateFormOpen(true);
    setCreateError(null);
    setCreateSuccess(null);
  }
  function closeTaskCardForm() {
    if (pendingCreate) return;
    setCreateFormOpen(false);
    setCreateError(null);
  }
  async function submitTaskCardCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canCreateTaskCard || createPhase === "submitting") return;
    let command = pendingCreate;
    try {
      if (!command) {
        const form = new FormData(event.currentTarget);
        const title = String(form.get("title") ?? "").trim();
        const description = String(form.get("description") ?? "");
        const priority = String(form.get("priority") ?? "medium") as RunTaskCardPriority;
        if (!title || title.length > RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS || description.length > RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS || !RUN_TASK_CARD_PRIORITIES.includes(priority)) {
          setCreateError("请检查标题、描述长度与优先级。");
          setCreatePhase("error");
          return;
        }
        const target = taskCardCreateTarget(context);
        if (!target) {
          setCreateError("当前没有有效 Worktree 焦点，不能创建任务卡。");
          setCreatePhase("error");
          return;
        }
        command = {
          input: {
            title,
            description,
            priority,
            correlation_id: globalThis.crypto.randomUUID(),
          },
          idempotency_key: globalThis.crypto.randomUUID(),
          target,
        };
        setPendingCreate(command);
      } else if (!sameTaskCardCreateTarget(command.target, taskCardCreateTarget(context))) {
        setCreateError("当前 Worktree 焦点已变化。请返回原目标 Run/Worktree，再使用相同请求重试。");
        setCreatePhase("error");
        return;
      }
      setCreatePhase("submitting");
      setCreateError(null);
      await client.createRunTaskCard(context, command.input, command.idempotency_key);
      setPendingCreate(null);
      setCreatePhase("idle");
      setCreateFormOpen(false);
      setCreateSuccess("Task Card 已创建，正在刷新当前 Run 列表。");
      setCursorStack([undefined]);
      setPageIndex(0);
      setRevision((value) => value + 1);
    } catch (reason: unknown) {
      if (command) setPendingCreate(command);
      setCreateError(directoryErrorMessage(reason));
      setCreatePhase("error");
    }
  }

  return <section role="tabpanel" aria-label="Task Cards" className="rounded border border-line p-4" data-engineering-run-id={run.engineering_run_id}>
    <header className="flex flex-wrap items-start justify-between gap-3">
      <div>
        <h2 className="font-semibold">Task Cards</h2>
        <p className="mt-1 text-xs text-ink-mute">按 Engineering Run 所有权列出。当前 Worktree 仅是 checkout 焦点。</p>
      </div>
      <div className="flex flex-wrap gap-2">
        <button type="button" disabled={!canCreateTaskCard || phase === "loading" || Boolean(pendingCreate)} onClick={openTaskCardForm} className="rounded border border-line px-3 py-1 text-xs disabled:opacity-50">新建 Task Card</button>
        <button type="button" disabled={phase === "loading"} onClick={retryTaskCardPage} className="rounded border border-line px-3 py-1 text-xs disabled:opacity-50">刷新</button>
      </div>
    </header>

    {!canCreateTaskCard && <p className="mt-3 text-xs text-ink-mute" role="note">需要当前 Run 中具有 work-item:write 授权且未归档的 Worktree 焦点才能创建任务卡。</p>}
    {createSuccess && <p className="mt-3 text-sm" role="status">{createSuccess}</p>}
    {createFormOpen && <form onSubmit={submitTaskCardCreate} className="mt-4 grid gap-3 rounded border border-line bg-bg-soft p-3">
      <p className="text-xs text-ink-mute">新卡属于当前 Engineering Run；焦点 Worktree 只记录为初始 checkout 关联。</p>
      <fieldset disabled={createPhase === "submitting" || Boolean(pendingCreate)} className="grid gap-3">
        <label className="grid gap-1 text-sm">标题
          <input name="title" required maxLength={RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS} autoFocus className="rounded border border-line bg-bg px-3 py-2" />
        </label>
        <label className="grid gap-1 text-sm">描述
          <textarea name="description" maxLength={RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS} rows={4} className="rounded border border-line bg-bg px-3 py-2" />
        </label>
        <label className="grid gap-1 text-sm">优先级
          <select name="priority" defaultValue="medium" className="rounded border border-line bg-bg px-3 py-2">
            <option value="low">低</option><option value="medium">中</option><option value="high">高</option><option value="urgent">紧急</option>
          </select>
        </label>
      </fieldset>
      {createError && <p className="text-sm" role="alert">{createError}</p>}
      <div className="flex gap-2">
        <button type="submit" disabled={createPhase === "submitting"} className="rounded border border-line px-3 py-1 text-xs disabled:opacity-50">
          {createPhase === "submitting" ? "正在创建…" : pendingCreate ? "使用同一请求重试" : "创建任务卡"}
        </button>
        {!pendingCreate && <button type="button" onClick={closeTaskCardForm} className="rounded border border-line px-3 py-1 text-xs">取消</button>}
      </div>
    </form>}

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
