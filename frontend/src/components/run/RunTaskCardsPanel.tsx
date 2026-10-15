/*
@cypher schema=1 source_sha256=d46af69faae392e45f7344b114cf3afa866e8e1b283011e449f256dbf47e0b76
MERGE (self:File {path:"frontend/src/components/run/RunTaskCardsPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::RunTaskCardsPanel",kind:"function"})
MERGE (submit:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::submitTaskCardCreate",kind:"function"})
MERGE (restore:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::restorePendingTaskCardCreate",kind:"effect"})
MERGE (discard:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::discardPendingTaskCardCreate",kind:"function"})
MERGE (mutate:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::mutateRunTaskCard",kind:"function"})
MERGE (submitReject:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::submitTaskCardRejection",kind:"function"})
MERGE (scope:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryScopeKey",kind:"function"})
MERGE (target:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryTarget",kind:"function"})
MERGE (targetMatches:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryTargetMatchesContext",kind:"function"})
MERGE (store:Type {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore",kind:"interface"})
MERGE (loadStore:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore.load",kind:"method"})
MERGE (saveStore:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore.saveIfAbsent",kind:"method"})
MERGE (removeStore:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore.removeIfMatches",kind:"method"})
MERGE (getStore:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::getTaskCardCreateRecoveryStore",kind:"function"})
MERGE (recoveryFile:File {path:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts"})
MERGE (api:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.createRunTaskCard",kind:"method"})
MERGE (legacyApi:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.createLegacyRunTaskCard",kind:"method"})
MERGE (transitionApi:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.transitionRunTaskCard",kind:"method"})
MERGE (reviewApi:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.reviewRunTaskCard",kind:"method"})
MERGE (context:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::RunContextEnvelope",kind:"type"})
MERGE (input:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::CreateRunTaskCardInput",kind:"type"})
MERGE (maxAge:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TASK_CARD_CREATE_RECOVERY_MAX_AGE_MS",kind:"constant"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:DEFINES]->(submit)
MERGE (panel)-[:USES_TYPE]->(context)
MERGE (self)-[:IMPORTS]->(recoveryFile)
MERGE (panel)-[:CALLS]->(scope)
MERGE (panel)-[:CALLS]->(target)
MERGE (panel)-[:CALLS]->(targetMatches)
MERGE (panel)-[:CALLS]->(getStore)
MERGE (panel)-[:USES_TYPE]->(store)
MERGE (self)-[:DEFINES]->(restore)
MERGE (self)-[:DEFINES]->(discard)
MERGE (self)-[:DEFINES]->(mutate)
MERGE (self)-[:DEFINES]->(submitReject)
MERGE (restore)-[:CALLS]->(loadStore)
MERGE (restore)-[:READS]->(scope)
MERGE (discard)-[:CALLS]->(removeStore)
MERGE (discard)-[:READS]->(scope)
MERGE (submit)-[:CALLS]->(targetMatches)
MERGE (submit)-[:CALLS]->(saveStore)
MERGE (submit)-[:CALLS]->(removeStore)
MERGE (submit)-[:CALLS]->(api)
MERGE (submit)-[:CALLS]->(legacyApi)
MERGE (mutate)-[:CALLS]->(transitionApi)
MERGE (mutate)-[:CALLS]->(reviewApi)
MERGE (submitReject)-[:CALLS]->(mutate)
MERGE (panel)-[:CALLS]->(mutate)
MERGE (submit)-[:USES_TYPE]->(input)
MERGE (submit)-[:READS]->(maxAge)
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
 (setFormOpen:Function {name:"setCreateFormOpen",type:"function"}),(setCreatePhase:Function {name:"setCreatePhase",type:"function"}),(setCreateError:Function {name:"setCreateError",type:"function"}),(setCreateSuccess:Function {name:"setCreateSuccess",type:"function"}),(setPendingCreate:Function {name:"setPendingCreate",type:"function"}),(createApi:Function {name:"RunDirectoryApiClient.createRunTaskCard",type:"function"}),(createLegacyApi:Function {name:"RunDirectoryApiClient.createLegacyRunTaskCard",type:"function"}),(targetMatches:Function {name:"taskCardCreateRecoveryTargetMatchesContext",type:"function"}),(formEvent:Class {name:"FormEvent",type:"class"}),(formData:Class {name:"FormData",type:"class"}),(randomUuid:Function {name:"crypto.randomUUID",type:"function"}),(taskPriorities:Variable {name:"RUN_TASK_CARD_PRIORITIES",type:"variable"}),(titleLimit:Variable {name:"RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS",type:"variable"}),(descriptionLimit:Variable {name:"RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS",type:"variable"}),
 (f)-[:CONTAINS]->(props),(f)-[:CONTAINS]->(pendingType),(f)-[:CONTAINS]->(panel),(f)-[:CONTAINS]->(load),(f)-[:CONTAINS]->(success),(f)-[:CONTAINS]->(failure),(f)-[:CONTAINS]->(cleanup),(f)-[:CONTAINS]->(next),(f)-[:CONTAINS]->(previous),(f)-[:CONTAINS]->(retry),(f)-[:CONTAINS]->(openForm),(f)-[:CONTAINS]->(closeForm),(f)-[:CONTAINS]->(submitCreate),(f)-[:CONTAINS]->(renderCard),(f)-[:CONTAINS]->(renderLabel),(f)-[:CONTAINS]->(pageLimit),(f)-[:CONTAINS]->(pageSize),
 (panel)-[:CALLS]->(useState),(panel)-[:CALLS]->(useEffect),(panel)-[:CONTAINS]->(next),(panel)-[:CONTAINS]->(previous),(panel)-[:CONTAINS]->(retry),(panel)-[:CONTAINS]->(openForm),(panel)-[:CONTAINS]->(closeForm),(panel)-[:CONTAINS]->(submitCreate),(panel)-[:USES]->(cursorStack),(panel)-[:USES]->(pageIndex),(panel)-[:USES]->(revision),(panel)-[:USES]->(phase),(panel)-[:USES]->(page),(panel)-[:USES]->(error),(panel)-[:USES]->(cursor),(panel)-[:USES]->(available),(panel)-[:USES]->(context),(panel)-[:USES]->(run),(panel)-[:USES]->(focus),(panel)-[:USES]->(canCreate),(panel)-[:USES]->(formOpen),(panel)-[:USES]->(createPhase),(panel)-[:USES]->(createError),(panel)-[:USES]->(createSuccess),(panel)-[:USES]->(pendingCreate),(panel)-[:CALLS]->(renderCard),(renderCard)-[:CALLS]->(renderLabel),
 (panel)-[:CALLS]->(setCursor),(panel)-[:CALLS]->(setIndex),(panel)-[:CALLS]->(setPage),(panel)-[:CALLS]->(setError),(panel)-[:CALLS]->(setPhase),(panel)-[:USES]->(pageSize),
 (load)-[:CALLS]->(api),(load)-[:CALLS]->(createAbort),(load)-[:CALLS]->(setCursor),(load)-[:CALLS]->(setIndex),(load)-[:CALLS]->(setPage),(load)-[:CALLS]->(setError),(load)-[:CALLS]->(setPhase),(load)-[:CALLS]->(abort),(load)-[:CALLS]->(success),(load)-[:CALLS]->(failure),(load)-[:CALLS]->(cleanup),(load)-[:USES]->(available),(load)-[:USES]->(cursor),(load)-[:USES]->(page),(load)-[:USES]->(phase),(success)-[:CALLS]->(setPage),(success)-[:CALLS]->(setPhase),(success)-[:USES]->(page),(success)-[:USES]->(phase),(failure)-[:CALLS]->(errorMessage),(failure)-[:CALLS]->(setPage),(failure)-[:CALLS]->(setError),(failure)-[:CALLS]->(setPhase),(failure)-[:USES]->(page),(failure)-[:USES]->(error),(failure)-[:USES]->(phase),(cleanup)-[:CALLS]->(abort),
 (next)-[:CALLS]->(setCursor),(next)-[:CALLS]->(setIndex),(next)-[:USES]->(page),(next)-[:USES]->(cursorStack),(next)-[:USES]->(pageIndex),(next)-[:USES]->(pageLimit),(previous)-[:CALLS]->(setIndex),(previous)-[:USES]->(pageIndex),(retry)-[:CALLS]->(setRevision),(retry)-[:USES]->(revision),(openForm)-[:CALLS]->(setFormOpen),(openForm)-[:CALLS]->(setCreateError),(openForm)-[:CALLS]->(setCreateSuccess),(closeForm)-[:CALLS]->(setFormOpen),(closeForm)-[:USES]->(pendingCreate),(submitCreate)-[:CALLS]->(formData),(submitCreate)-[:CALLS]->(randomUuid),(submitCreate)-[:CALLS]->(createApi),(submitCreate)-[:CALLS]->(createLegacyApi),(submitCreate)-[:CALLS]->(targetMatches),(submitCreate)-[:CALLS]->(errorMessage),(submitCreate)-[:CALLS]->(setPendingCreate),(submitCreate)-[:CALLS]->(setCreatePhase),(submitCreate)-[:CALLS]->(setCreateError),(submitCreate)-[:CALLS]->(setCreateSuccess),(submitCreate)-[:CALLS]->(setCursor),(submitCreate)-[:CALLS]->(setIndex),(submitCreate)-[:CALLS]->(setRevision),(submitCreate)-[:USES]->(context),(submitCreate)-[:USES]->(pendingCreate),(submitCreate)-[:USES]->(createPhase),(submitCreate)-[:USES]->(canCreate),(submitCreate)-[:USES]->(taskPriorities),(submitCreate)-[:USES]->(titleLimit),(submitCreate)-[:USES]->(descriptionLimit),(panel)-[:CALLS]->(targetMatches),(renderCard)-[:USES]->(page),(renderLabel)-[:USES]->(pageSize);
*/

"use client";

import { useEffect, useState, type FormEvent } from "react";
import { directoryErrorMessage } from "@/lib/run/runDirectoryTree";
import {
  getTaskCardCreateRecoveryStore,
  taskCardCreateRecoveryScopeKey,
  taskCardCreateRecoveryTarget,
  taskCardCreateRecoveryTargetMatchesContext,
  TASK_CARD_CREATE_RECOVERY_MAX_AGE_MS,
  TaskCardCreateRecoveryError,
  type PendingTaskCardCreate,
  type TaskCardCreateRecoveryStore,
} from "@/lib/run/taskCardCreateRecoveryStore";
import {
  RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS,
  RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS,
  RUN_TASK_CARD_PAGE_SIZE,
  RUN_TASK_CARD_PRIORITIES,
  type RunContextEnvelope,
  type RunDirectoryApiClient,
  type RunTaskCard,
  type RunTaskCardLifecycleCommand,
  type RunTaskCardPage,
  type RunTaskCardPriority,
  type RunTaskCardReviewCommand,
} from "@/lib/run/runDirectoryApi";

type RunTaskCardMutation =
  | { kind: "lifecycle"; input: Omit<RunTaskCardLifecycleCommand, "expected_version" | "correlation_id"> }
  | { kind: "review"; input: Omit<RunTaskCardReviewCommand, "expected_version" | "correlation_id"> };

interface RunTaskCardsPanelProps {
  client: RunDirectoryApiClient;
  context: RunContextEnvelope;
  available: boolean;
  recoveryStore?: TaskCardCreateRecoveryStore;
}

const RUN_TASK_CARD_MAX_PAGE_COUNT = 100;

function taskCardCreateErrorMessage(reason: unknown): string {
  return reason instanceof TaskCardCreateRecoveryError ? reason.message : directoryErrorMessage(reason);
}

export function RunTaskCardsPanel({ client, context, available, recoveryStore: suppliedRecoveryStore }: RunTaskCardsPanelProps) {
  const recoveryStore = suppliedRecoveryStore ?? getTaskCardCreateRecoveryStore();
  const run = context.engineering_run;
  const focus = context.focus;
  const recoveryScopeKey = taskCardCreateRecoveryScopeKey(context);
  const currentRecoveryTarget = taskCardCreateRecoveryTarget(context);
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
  const [commandBusyId, setCommandBusyId] = useState<string | null>(null);
  const [commandError, setCommandError] = useState<string | null>(null);
  const [commandSuccess, setCommandSuccess] = useState<string | null>(null);
  const [rejectingCardId, setRejectingCardId] = useState<string | null>(null);
  const [rejectReason, setRejectReason] = useState("");
  const [pendingCreate, setPendingCreate] = useState<PendingTaskCardCreate | null>(null);
  const [recoveryState, setRecoveryState] = useState<{
    scope_key: string | null;
    phase: "loading" | "ready" | "error";
    error: string | null;
  }>({ scope_key: null, phase: "loading", error: null });
  const cursor = cursorStack[pageIndex];
  const runId = run.engineering_run_id;
  const taskOwnerAvailable = Boolean(context.capabilities.task_cards_available &&
    context.run_context.project_id === run.project_id && context.run_context.engineering_run_id === runId &&
    context.run_context.repository_id === run.repository_id && context.run_context.branch_id === run.branch_id &&
    ((run.branch_id === null && context.branch === null) || (context.branch !== null && context.branch.branch_id === run.branch_id &&
      context.branch.project_id === run.project_id && context.branch.repository_id === run.repository_id)));
  const canCreateTaskCard = Boolean(taskOwnerAvailable && context.run_context.granted_scopes.includes("work-item:write") &&
    (!focus || (!focus.archived && focus.project_id === run.project_id &&
      (run.repository_id === null || focus.repository_id === run.repository_id) && focus.engineering_run_id === runId)));
  const canMutateTaskCards = Boolean(taskOwnerAvailable && context.run_context.granted_scopes.includes("work-item:write"));
  const canReviewTaskCards = Boolean(canMutateTaskCards &&
    [context.run_context.authorization.project.role, context.run_context.authorization.engineering_run.role]
      .every((role) => ["tenant_admin", "project_admin", "developer"].includes(role)));
  const pendingTargetMatches = pendingCreate
    ? taskCardCreateRecoveryTargetMatchesContext(pendingCreate, context)
    : false;
  const pendingAgeMs = pendingCreate ? Date.now() - pendingCreate.created_at_ms : 0;
  const pendingRetryExpired = Boolean(pendingCreate && (pendingAgeMs < 0 || pendingAgeMs >= TASK_CARD_CREATE_RECOVERY_MAX_AGE_MS));
  const canRetryPendingCreate = Boolean(pendingCreate && pendingTargetMatches && !pendingRetryExpired);

  useEffect(function restorePendingTaskCardCreate() {
    let active = true;
    setPendingCreate(null);
    setCreateFormOpen(false);
    setRecoveryState({ scope_key: recoveryScopeKey, phase: recoveryScopeKey ? "loading" : "ready", error: null });
    if (!recoveryScopeKey) return () => { active = false; };

    void recoveryStore.load(recoveryScopeKey).then((record) => {
      if (!active) return;
      setPendingCreate(record);
      setCreateFormOpen(Boolean(record));
      setCreateError(null);
      setRecoveryState({ scope_key: recoveryScopeKey, phase: "ready", error: null });
    }).catch((reason: unknown) => {
      if (!active) return;
      setRecoveryState({ scope_key: recoveryScopeKey, phase: "error", error: taskCardCreateErrorMessage(reason) });
    });
    return () => { active = false; };
  }, [recoveryScopeKey, recoveryStore]);

  useEffect(function loadRunTaskPage() {
    if (!available || !taskOwnerAvailable) {
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
  }, [available, client, cursor, revision, run, taskOwnerAvailable]);

  if (!available || !taskOwnerAvailable) return <section role="tabpanel" aria-label="Task Cards" className="rounded border border-line p-4">
    <h2 className="font-semibold">Task Cards</h2>
    <p className="mt-2 text-sm text-ink-mute">{available
      ? "当前 Run 上下文或授权未开放 Task Cards。此处不会读取旧任务投影或本地演示数据。"
      : "服务端尚未开放 Run-owned Apps。此处不会读取旧任务投影或本地演示数据；Owner migration、数据库角色与 RLS 验收完成后才会加载真实 Run 任务。"}</p>
    {pendingCreate && <p className="mt-2 text-xs" role="status">已保留待确认创建“{pendingCreate.input.title}”；原焦点 Worktree：{pendingCreate.schema_version === 1 ? pendingCreate.target.worktree_id : pendingCreate.target.focus_worktree_id ?? "无"}。恢复匹配上下文前不会重放。</p>}
    {recoveryState.phase === "error" && <p className="mt-2 text-xs" role="alert">本地待恢复请求无法读取：{recoveryState.error} 创建入口保持关闭。</p>}
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
  async function mutateRunTaskCard(card: RunTaskCard, mutation: RunTaskCardMutation) {
    if (!canMutateTaskCards || commandBusyId !== null) return;
    setCommandBusyId(card.work_item_id);
    setCommandError(null);
    setCommandSuccess(null);
    try {
      const key = globalThis.crypto.randomUUID();
      if (mutation.kind === "lifecycle") {
        const command: RunTaskCardLifecycleCommand = {
          ...mutation.input,
          expected_version: card.lifecycle.version,
          correlation_id: globalThis.crypto.randomUUID(),
        };
        await client.transitionRunTaskCard(context, card, command, key);
      } else {
        const command: RunTaskCardReviewCommand = {
          ...mutation.input,
          expected_version: card.lifecycle.version,
          correlation_id: globalThis.crypto.randomUUID(),
        };
        await client.reviewRunTaskCard(context, card, command, key);
      }
      if (mutation.kind === "review" && mutation.input.action === "reject") {
        setRejectingCardId(null);
        setRejectReason("");
      }
      setCommandSuccess(`Task Card「${card.title}」已更新，正在刷新列表。`);
    } catch (reason: unknown) {
      setCommandError(`操作结果未确认，正在刷新列表；请在新状态加载后继续，不会自动重放。${directoryErrorMessage(reason)}`);
    } finally {
      setCommandBusyId(null);
      setRevision((value) => value + 1);
    }
  }
  async function submitTaskCardRejection(event: FormEvent<HTMLFormElement>, card: RunTaskCard) {
    event.preventDefault();
    const reason = rejectReason.trim();
    if (!reason || reason.length > 4_000) return;
    await mutateRunTaskCard(card, { kind: "review", input: { action: "reject", reason } });
  }
  function openTaskCardForm() {
    if (recoveryState.phase !== "ready" || recoveryState.scope_key !== recoveryScopeKey || pendingCreate) return;
    setCreateFormOpen(true);
    setCreateError(null);
    setCreateSuccess(null);
  }
  function closeTaskCardForm() {
    if (pendingCreate) return;
    setCreateFormOpen(false);
    setCreateError(null);
  }
  async function discardPendingTaskCardCreate() {
    if (!pendingCreate || !recoveryScopeKey || typeof globalThis.confirm !== "function") return;
    const confirmed = globalThis.confirm("请先在当前 Run 的 Task Cards 列表中核对该任务是否已经创建。确认后将清除本地恢复记录，后续创建会使用新的幂等键。继续？");
    if (!confirmed) return;
    try {
      await recoveryStore.removeIfMatches(recoveryScopeKey, pendingCreate.idempotency_key);
      setPendingCreate(null);
      setCreateFormOpen(false);
      setCreateError(null);
      setCreatePhase("idle");
      setRecoveryState({ scope_key: recoveryScopeKey, phase: "ready", error: null });
    } catch (reason: unknown) {
      setCreateError(taskCardCreateErrorMessage(reason));
      setCreatePhase("error");
    }
  }
  async function submitTaskCardCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canCreateTaskCard || createPhase === "submitting" || !recoveryScopeKey ||
        recoveryState.phase !== "ready" || recoveryState.scope_key !== recoveryScopeKey) return;
    let command = pendingCreate;
    let recordPersisted = Boolean(command);
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
        if (!currentRecoveryTarget) {
          setCreateError("当前 Run、授权或 Worktree 焦点上下文无效，不能创建任务卡。");
          setCreatePhase("error");
          return;
        }
        command = {
          schema_version: 2,
          scope_key: recoveryScopeKey,
          created_at_ms: Date.now(),
          input: {
            title,
            description,
            priority,
            correlation_id: globalThis.crypto.randomUUID(),
          },
          idempotency_key: globalThis.crypto.randomUUID(),
          target: currentRecoveryTarget,
        };
        let stored: PendingTaskCardCreate | null;
        try {
          stored = await recoveryStore.saveIfAbsent(command);
        } catch {
          throw new TaskCardCreateRecoveryError("无法保存本地恢复记录，创建请求未发送。");
        }
        if (!stored) throw new TaskCardCreateRecoveryError("本地恢复记录未能保存，创建请求未发送。");
        if (stored.idempotency_key !== command.idempotency_key) {
          setPendingCreate(stored);
          setCreateFormOpen(true);
          setCreateError("此 Run 已有待确认的创建请求。已恢复原请求；请先处理它，再开始新的创建。");
          setCreatePhase("error");
          return;
        }
        command = stored;
        recordPersisted = true;
        setPendingCreate(command);
      }
      if (!taskCardCreateRecoveryTargetMatchesContext(command, context)) {
        setCreateError("当前 Run、焦点 Worktree、binding 或授权版本已变化。请恢复原目标与授权状态后，再使用相同请求重试。");
        setCreatePhase("error");
        return;
      }
      const ageMs = Date.now() - command.created_at_ms;
      if (ageMs < 0 || ageMs >= TASK_CARD_CREATE_RECOVERY_MAX_AGE_MS) {
        setCreateError("此恢复请求已超过安全重试期限。请先在 Run Task Cards 列表核对结果，再清除恢复记录。");
        setCreatePhase("error");
        return;
      }
      setCreatePhase("submitting");
      setCreateError(null);
      if (command.schema_version === 1) {
        await client.createLegacyRunTaskCard(context, command.input, command.idempotency_key, command.target.worktree_id);
      } else {
        await client.createRunTaskCard(context, command.input, command.idempotency_key);
      }
      try {
        await recoveryStore.removeIfMatches(recoveryScopeKey, command.idempotency_key);
      } catch {
        setPendingCreate(command);
        setCreatePhase("error");
        setCreateSuccess("服务端已确认创建，但本地恢复记录未能清理。可使用同一请求安全重试并完成对账。");
        setCreateError("本地恢复记录仍保留；请使用同一请求重试，勿开始新的创建。");
        return;
      }
      setPendingCreate(null);
      setCreatePhase("idle");
      setCreateFormOpen(false);
      setCreateSuccess("Task Card 已创建，正在刷新当前 Run 列表。");
      setCursorStack([undefined]);
      setPageIndex(0);
      setRevision((value) => value + 1);
    } catch (reason: unknown) {
      if (command && recordPersisted) setPendingCreate(command);
      setCreateError(taskCardCreateErrorMessage(reason));
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
        <button type="button" disabled={!canCreateTaskCard || phase === "loading" || Boolean(pendingCreate) || recoveryState.phase !== "ready" || recoveryState.scope_key !== recoveryScopeKey} onClick={openTaskCardForm} className="rounded border border-line px-3 py-1 text-xs disabled:opacity-50">新建 Task Card</button>
        <button type="button" disabled={phase === "loading"} onClick={retryTaskCardPage} className="rounded border border-line px-3 py-1 text-xs disabled:opacity-50">刷新</button>
      </div>
    </header>

    {!canCreateTaskCard && <p className="mt-3 text-xs text-ink-mute" role="note">需要当前 Run 的 work-item:write 授权；Worktree 焦点可选，提供时必须属于当前 Run 且未归档。</p>}
    {recoveryState.phase === "loading" && <p className="mt-3 text-xs text-ink-mute" role="status">正在检查本地待恢复请求；检查完成前创建入口保持关闭。</p>}
    {recoveryState.phase === "error" && <p className="mt-3 text-xs" role="alert">本地待恢复请求无法读取：{recoveryState.error} 为避免丢失幂等键，创建入口保持关闭。</p>}
    {pendingCreate && <div className="mt-3 rounded border border-amber-500/50 bg-amber-500/5 p-3 text-xs" role={pendingTargetMatches && !pendingRetryExpired ? "status" : "alert"}>
      <p>发现待确认创建“{pendingCreate.input.title}”，原焦点 Worktree：{pendingCreate.schema_version === 1 ? pendingCreate.target.worktree_id : pendingCreate.target.focus_worktree_id ?? "无"}。</p>
      {!pendingTargetMatches && <p className="mt-1">当前 Run、焦点 Worktree、binding 或授权版本不同；不会向当前目标重放。请恢复原目标后重试。</p>}
      {pendingRetryExpired && <p className="mt-1">该请求已超过安全重试期限；请先核对 Run Task Cards 列表，再清除恢复记录。</p>}
      {pendingTargetMatches && !pendingRetryExpired && <p className="mt-1">请求正文与幂等键已从本地恢复；重试会使用原 Run target{pendingCreate.schema_version === 1 ? "/Worktree 路由" : "及焦点"}。</p>}
      <button type="button" onClick={discardPendingTaskCardCreate} disabled={createPhase === "submitting"} className="mt-2 rounded border border-line px-2 py-1 disabled:opacity-50">我已核对列表，清除恢复记录</button>
    </div>}
    {createSuccess && <p className="mt-3 text-sm" role="status">{createSuccess}</p>}
    {commandSuccess && <p className="mt-3 text-sm" role="status">{commandSuccess}</p>}
    {commandError && <p className="mt-3 text-sm" role="alert">{commandError}</p>}
    {createFormOpen && <form onSubmit={submitTaskCardCreate} className="mt-4 grid gap-3 rounded border border-line bg-bg-soft p-3">
      <p className="text-xs text-ink-mute">{pendingCreate ? "已恢复原请求；字段固定，不会生成新的请求正文或幂等键。" : "新卡属于当前 Engineering Run；可选 Worktree 焦点只记录为初始 checkout 关联。"}</p>
      <fieldset disabled={createPhase === "submitting" || Boolean(pendingCreate) || !canCreateTaskCard} className="grid gap-3">
        <label className="grid gap-1 text-sm">标题
          <input name="title" required maxLength={RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS} defaultValue={pendingCreate?.input.title ?? ""} autoFocus className="rounded border border-line bg-bg px-3 py-2" />
        </label>
        <label className="grid gap-1 text-sm">描述
          <textarea name="description" maxLength={RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS} defaultValue={pendingCreate?.input.description ?? ""} rows={4} className="rounded border border-line bg-bg px-3 py-2" />
        </label>
        <label className="grid gap-1 text-sm">优先级
          <select name="priority" defaultValue={pendingCreate?.input.priority ?? "medium"} className="rounded border border-line bg-bg px-3 py-2">
            <option value="low">低</option><option value="medium">中</option><option value="high">高</option><option value="urgent">紧急</option>
          </select>
        </label>
      </fieldset>
      {createError && <p className="text-sm" role="alert">{createError}</p>}
      <div className="flex gap-2">
        <button type="submit" disabled={createPhase === "submitting" || !canCreateTaskCard || recoveryState.phase !== "ready" || (Boolean(pendingCreate) && !canRetryPendingCreate)} className="rounded border border-line px-3 py-1 text-xs disabled:opacity-50">
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
            {canMutateTaskCards && ["claimed", "in_progress"].includes(card.lifecycle.status) && card.lifecycle.active_worktree_id !== focus?.worktree_id && <p className="mt-2 text-xs text-ink-mute">此任务当前绑定到 Worktree {card.lifecycle.active_worktree_id}；请切换到该 Worktree 后继续。</p>}
            {canMutateTaskCards && <div className="mt-3 flex flex-wrap gap-2" aria-label={`Task Card 操作：${card.title}`}>
              {card.lifecycle.status === "pending" && <>
                <button type="button" disabled={commandBusyId !== null || !focus} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "claimed" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">领取到当前 Worktree</button>
                <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "cancelled" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">取消任务</button>
                {!focus && <span className="self-center text-xs text-ink-mute">选择 Run 下的 Worktree 后可领取执行。</span>}
              </>}
              {card.lifecycle.status === "claimed" && card.lifecycle.active_worktree_id === focus?.worktree_id && <>
                {card.lifecycle.claimed_by_user_id === context.run_context.actor_id && card.lifecycle.claim_expires_at !== null && Date.parse(card.lifecycle.claim_expires_at) > Date.now()
                  ? <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "in_progress" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">开始处理</button>
                  : <span className="self-center text-xs text-ink-mute">{card.lifecycle.claimed_by_user_id === context.run_context.actor_id
                    ? "领取已过期或状态不完整；请刷新后重新领取。"
                    : "此任务由其他成员领取，当前成员不能开始执行。"}</span>}
                {card.lifecycle.claim_expires_at !== null && Date.parse(card.lifecycle.claim_expires_at) <= Date.now() && <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "pending" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">重新排队</button>}
                <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "cancelled" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">取消任务</button>
              </>}
              {card.lifecycle.status === "in_progress" && card.lifecycle.active_worktree_id === focus?.worktree_id && <>
                {card.lifecycle.review_state !== "pending_review" && <>
                  {card.lifecycle.claimed_by_user_id === context.run_context.actor_id
                    ? <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "review", input: { action: "submit" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">提交审查</button>
                    : <span className="self-center text-xs text-ink-mute">只有当前领取任务的成员可以提交审查。</span>}
                  <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "completed" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">标记完成</button>
                </>}
                <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "cancelled" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">取消任务</button>
              </>}
              {card.lifecycle.status === "in_progress" && card.lifecycle.review_state === "pending_review" && canReviewTaskCards && card.lifecycle.claimed_by_user_id !== context.run_context.actor_id && <>
                <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "review", input: { action: "accept" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">批准审查</button>
                {rejectingCardId === card.work_item_id
                  ? <form onSubmit={(event) => void submitTaskCardRejection(event, card)} className="flex w-full flex-wrap items-end gap-2">
                    <label className="grid min-w-56 flex-1 gap-1 text-xs">退回原因
                      <textarea value={rejectReason} onChange={(event) => setRejectReason(event.currentTarget.value)} maxLength={4_000} required rows={2} className="rounded border border-line bg-bg px-2 py-1 text-sm" />
                    </label>
                    <button type="submit" disabled={commandBusyId !== null || !rejectReason.trim()} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">确认退回</button>
                    <button type="button" disabled={commandBusyId !== null} onClick={() => { setRejectingCardId(null); setRejectReason(""); }} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">取消</button>
                  </form>
                  : <button type="button" disabled={commandBusyId !== null} onClick={() => { setRejectingCardId(card.work_item_id); setRejectReason(""); }} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">退回修改</button>}
              </>}
              {card.lifecycle.status === "in_progress" && card.lifecycle.review_state === "pending_review" && canReviewTaskCards && card.lifecycle.claimed_by_user_id === context.run_context.actor_id && <p className="self-center text-xs text-ink-mute">执行成员不能审查自己提交的任务。</p>}
              {card.lifecycle.status === "in_progress" && card.lifecycle.review_state === "pending_review" && !canReviewTaskCards && <p className="self-center text-xs text-ink-mute">等待具有 Run 审查权限的成员处理。</p>}
              {card.lifecycle.status === "failed" && <button type="button" disabled={commandBusyId !== null} onClick={() => void mutateRunTaskCard(card, { kind: "lifecycle", input: { target_status: "pending" } })} className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">重新排队</button>}
            </div>}
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
