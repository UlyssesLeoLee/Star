/*
@cypher schema=1 source_sha256=98f852bf7a86d223b26b80c242afbc598e29d9fb7e89acb78ffb10b25b11bba2
CYPHER STRUCTURE MANIFEST
MERGE (self:File {path:"frontend/src/components/AuthenticatedWorktreeIndex.tsx"});
CREATE
  (loadState:Class {name:"IndexLoadState",type:"class"}),
  (component:Function {name:"AuthenticatedWorktreeIndex",type:"function",signature:"AuthenticatedWorktreeIndex(props: { projectId: string }): JSX.Element",visibility:"public",complexity:"complex"}),
  (load:Function {name:"load",type:"function",signature:"load(cursor?: string | null, append?: boolean, signal?: AbortSignal): Promise<void>",visibility:"private",complexity:"moderate"}),
  (loadMore:Function {name:"loadMore",type:"function",signature:"loadMore(): Promise<void>",visibility:"private",complexity:"moderate"}),
  (manage:Function {name:"manageWorktree",type:"function",signature:"manageWorktree(worktree: GroupWorktreeIndexRow, archived: boolean): Promise<void>",visibility:"private",complexity:"moderate"}),
  (errorText:Function {name:"errorText",type:"function",signature:"errorText(error: unknown): string",visibility:"private",complexity:"simple"}),
  (safeUrl:Function {name:"safeExternalUrl",type:"function",signature:"safeExternalUrl(value: string | null): string | null",visibility:"private",complexity:"simple"}),
  (state:Variable {name:"state",type:"variable",language:"typescript"}),
  (api:Variable {name:"api",type:"variable",language:"typescript"}),
  (self)-[:CONTAINS]->(loadState),(self)-[:CONTAINS]->(state),(self)-[:CONTAINS]->(api),(self)-[:CONTAINS]->(component),
  (component)-[:HAS_METHOD]->(load),(component)-[:HAS_METHOD]->(loadMore),(component)-[:HAS_METHOD]->(manage),
  (component)-[:CALLS]->(errorText),(component)-[:CALLS]->(safeUrl),(component)-[:USES]->(state),(component)-[:USES]->(api),
  (load)-[:CALLS]->(api),(loadMore)-[:CALLS]->(load),(manage)-[:CALLS]->(api),(manage)-[:CALLS]->(errorText);
@endcypher
*/

"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useState } from "react";
import { AlertCircle, GitBranch, Lock, RotateCcw, Archive } from "lucide-react";

import {
  GroupApiError,
  GroupContextApiClient,
  type GroupWorktreeIndexResponse,
  type GroupWorktreeIndexRow,
} from "@/lib/group-api/group-context";

type IndexLoadState =
  | { status: "loading" }
  | { status: "ready"; data: GroupWorktreeIndexResponse }
  | { status: "error"; message: string };

function errorText(error: unknown): string {
  if (error instanceof GroupApiError) {
    if (error.status === 401) return "需要有效的登录会话；此 Project Index 不会回退到本地 seed。";
    if (error.status === 403) return "当前身份无权读取此 Project 的 Worktree。";
    if (error.status === 404) return "Project 不存在，或当前身份无权访问。";
    return `Project Worktree API 请求失败：${error.code}`;
  }
  return "无法连接 Project Worktree API；请检查登录会话和服务状态。";
}

function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "—" : date.toISOString().replace("T", " ").slice(0, 16) + " UTC";
}

function safeExternalUrl(value: string | null): string | null {
  if (!value) return null;
  try {
    const url = new URL(value);
    return url.protocol === "https:" || url.protocol === "http:" ? url.toString() : null;
  } catch {
    return null;
  }
}

export function AuthenticatedWorktreeIndex({ projectId }: { projectId: string }) {
  const api = useMemo(() => new GroupContextApiClient(), []);
  const [state, setState] = useState<IndexLoadState>({ status: "loading" });
  const [busyId, setBusyId] = useState<string | null>(null);
  const [loadingMore, setLoadingMore] = useState(false);
  const [operationError, setOperationError] = useState<string | null>(null);
  const [reloadVersion, setReloadVersion] = useState(0);

  const load = useCallback(async (cursor?: string | null, append = false, signal?: AbortSignal) => {
    const result = await api.listProjectWorktrees(projectId, cursor, signal);
    setState((previous) => {
      if (!append || previous.status !== "ready") return { status: "ready", data: result };
      const seen = new Set(previous.data.worktrees.map((row) => row.id));
      return {
        status: "ready",
        data: { ...result, worktrees: [...previous.data.worktrees, ...result.worktrees.filter((row) => !seen.has(row.id))] },
      };
    });
  }, [api, projectId]);

  useEffect(() => {
    const controller = new AbortController();
    setState({ status: "loading" });
    void load(null, false, controller.signal).catch((error: unknown) => {
      if (!controller.signal.aborted) setState({ status: "error", message: errorText(error) });
    });
    return () => controller.abort();
  }, [load, reloadVersion]);

  const loadMore = useCallback(async () => {
    if (state.status !== "ready" || !state.data.next_cursor) return;
    setLoadingMore(true);
    try {
      await load(state.data.next_cursor, true);
    } catch (error) {
      setOperationError(errorText(error));
    } finally {
      setLoadingMore(false);
    }
  }, [load, state]);

  const manageWorktree = useCallback(async (worktree: GroupWorktreeIndexRow, archived: boolean) => {
    const action = archived ? "归档" : "恢复";
    if (archived && (worktree.agent_session_id || worktree.runtime_id)) {
      setOperationError("当前 Worktree 仍绑定 Agent Session 或 Runtime；请先完成运行时停止与状态同步。");
      return;
    }
    setBusyId(worktree.id);
    setOperationError(null);
    try {
      const plan = await api.createWorktreeManagementPlan(worktree, archived);
      if (!window.confirm(`管理计划已创建。确认${action}「${worktree.name}」？\n计划 ${plan.plan_id}\n预期版本 v${plan.expected_version}\n有效至 ${formatDate(plan.expires_at)}`)) return;
      await api.confirmWorktreeManagementPlan(worktree.id, plan.plan_id);
      setReloadVersion((version) => version + 1);
    } catch (error) {
      setOperationError(errorText(error));
    } finally {
      setBusyId(null);
    }
  }, [api]);

  return (
    <main className="mx-auto w-full max-w-7xl space-y-4 p-4" data-testid="authenticated-worktree-index">
      <header className="flex flex-wrap items-start justify-between gap-3 border-b border-line pb-3">
        <div>
          <h1 className="flex items-center gap-2 text-xl font-semibold"><GitBranch size={18} className="text-accent" />Project Worktree Index</h1>
          <p className="mt-1 font-mono text-xs text-ink-mute">Project {projectId} · 服务端授权视图</p>
        </div>
        <Link href="/projects" className="btn text-xs">返回 Projects</Link>
      </header>

      {state.status === "loading" && <section className="card text-sm text-ink-dim">正在读取授权 Worktree…</section>}
      {state.status === "error" && <section className="card space-y-2" role="alert" data-testid="worktree-index-api-error"><h2 className="font-semibold">无法读取 Project Worktree</h2><p className="text-sm text-warning">{state.message}</p></section>}
      {operationError && <p className="rounded border border-warning/30 bg-warning/5 p-3 text-xs text-warning" role="alert">{operationError}</p>}

      {state.status === "ready" && (
        <>
          <section className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-mute">
            <span>{state.data.worktrees.length} Worktrees · Project role {state.data.role}</span>
            <span className="font-mono">{state.data.permission_snapshot_ref}</span>
          </section>
          {state.data.worktrees.length === 0 && <section className="card text-sm text-ink-dim">此 Project 当前没有可见的 Worktree。</section>}
          <section className="grid gap-3 lg:grid-cols-2" aria-label="授权 Worktree 清单">
            {state.data.worktrees.map((worktree) => {
              const canManage = state.data.role === "tenant_admin" || state.data.role === "project_admin";
              const pullRequestUrl = safeExternalUrl(worktree.pull_request_url);
              return (
                <article key={worktree.id} className="card space-y-3" data-testid={`worktree-index-${worktree.id}`}>
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <div><h2 className="font-semibold">{worktree.name}</h2><p className="mt-1 font-mono text-[10px] text-ink-mute">{worktree.id}</p></div>
                    <span className={`rounded border px-2 py-1 text-[10px] ${worktree.archived ? "border-warning/30 text-warning" : "border-accent/30 text-accent"}`}>{worktree.archived ? "archived" : `${worktree.human_state} · ${worktree.machine_state}`}</span>
                  </div>
                  <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
                    <div><dt className="text-ink-mute">Branch</dt><dd className="font-mono text-info">{worktree.branch}</dd></div>
                    <div><dt className="text-ink-mute">Owner</dt><dd className="font-mono">{worktree.owner_user_id ?? "未分配"}</dd></div>
                    <div><dt className="text-ink-mute">Agent / Runtime</dt><dd className="font-mono">{worktree.agent_session_id ?? "—"} / {worktree.runtime_id ?? "—"}</dd></div>
                    <div><dt className="text-ink-mute">Ahead / Behind</dt><dd>{worktree.ahead} / {worktree.behind}</dd></div>
                    <div><dt className="text-ink-mute">Dirty / Risk</dt><dd>{worktree.dirty ? "dirty" : "clean"} / {worktree.risk_count}</dd></div>
                    <div><dt className="text-ink-mute">Health / Tests</dt><dd>{worktree.health_score} / {worktree.test_state}</dd></div>
                    <div><dt className="text-ink-mute">PR</dt><dd>{pullRequestUrl ? <a className="text-info underline" href={pullRequestUrl} target="_blank" rel="noreferrer">Open PR</a> : "—"}</dd></div>
                    <div><dt className="text-ink-mute">Last activity</dt><dd>{formatDate(worktree.last_activity)}</dd></div>
                  </dl>
                  <div className="flex flex-wrap items-center justify-between gap-2 border-t border-line pt-3">
                    <span className="flex items-center gap-1 text-[10px] text-ink-mute"><Lock size={11} />{worktree.locked ? "locked" : "unlocked"} · v{worktree.version}</span>
                    <div className="flex gap-2">
                      {canManage && (
                        <button type="button" disabled={busyId === worktree.id} onClick={() => void manageWorktree(worktree, !worktree.archived)} className="btn text-xs disabled:opacity-50">
                          {worktree.archived ? <RotateCcw size={12} /> : <Archive size={12} />}{worktree.archived ? "恢复" : "归档"}
                        </button>
                      )}
                      <Link href={`/worktree/${encodeURIComponent(worktree.id)}/group`} className="btn-primary text-xs">展开 Worktree</Link>
                    </div>
                  </div>
                  {worktree.archived && <p className="text-[10px] text-ink-mute"><AlertCircle size={10} className="mr-1 inline" />归档只改变 Worktree 管理记录；不执行 Git worktree 物理删除。</p>}
                </article>
              );
            })}
          </section>
          {state.data.next_cursor && <button type="button" disabled={loadingMore} onClick={() => void loadMore()} className="btn mx-auto block text-xs disabled:opacity-50">{loadingMore ? "加载中…" : "加载更多 Worktree"}</button>}
        </>
      )}
    </main>
  );
}
