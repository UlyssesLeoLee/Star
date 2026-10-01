/* CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"TaskRunHistoryPanel.tsx",type:"file",language:"typescript"}),
  (panel:Function {name:"TaskRunHistoryPanel",type:"function",language:"typescript",visibility:"public",complexity:"moderate"}),
  (refresh:Function {name:"refreshTaskRuns",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
  (loadMore:Function {name:"loadMoreTaskRuns",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
  (openRun:Function {name:"openTaskRunDetail",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
  (api:Interface {name:"WorktreeGroupApiClient",type:"interface",language:"typescript"});
CREATE (file)-[:CONTAINS]->(panel),(file)-[:CONTAINS]->(refresh),(file)-[:CONTAINS]->(loadMore),(file)-[:CONTAINS]->(openRun),
       (panel)-[:USES]->(api),(panel)-[:CALLS]->(refresh),(panel)-[:CALLS]->(loadMore),(panel)-[:CALLS]->(openRun);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"TaskRunHistoryPanel.tsx"}),(panel:Function {name:"TaskRunHistoryPanel"});
CREATE (profileSnapshot:Logic {name:"run_profile_snapshot_projection",type:"logic",language:"typescript"});
CREATE (file)-[:CONTAINS]->(profileSnapshot),(panel)-[:USES]->(profileSnapshot);
*/

"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { RefreshCw } from "lucide-react";

import type {
  TaskRunDetail,
  TaskRunPage,
  TaskRunSummary,
  WorktreeGroupApiClient,
} from "@/lib/group/worktreeGroupApi";

interface TaskRunHistoryPanelProps {
  worktreeId: string;
  workItemId: string;
  api: WorktreeGroupApiClient | null;
}

const MAX_LOCAL_RUNS = 100;

export function TaskRunHistoryPanel({ worktreeId, workItemId, api }: TaskRunHistoryPanelProps) {
  const [runs, setRuns] = useState<TaskRunSummary[]>([]);
  const [nextCursor, setNextCursor] = useState<TaskRunPage["next_cursor"]>(null);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [detail, setDetail] = useState<TaskRunDetail | null>(null);
  const [detailPending, setDetailPending] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const requestGeneration = useRef(0);
  const detailRequestGeneration = useRef(0);

  useEffect(() => {
    const generation = ++requestGeneration.current;
    detailRequestGeneration.current += 1;
    setRuns([]);
    setNextCursor(null);
    setSelectedRunId(null);
    setDetail(null);
    setLoadingMore(false);
    setDetailError(null);
    if (!api) {
      setLoading(false);
      setError(null);
      return;
    }

    setLoading(true);
    setError(null);
    void api.listTaskRuns(worktreeId, workItemId, { limit: 20 })
      .then((page) => {
        if (requestGeneration.current !== generation) return;
        setRuns(page.runs.slice(0, MAX_LOCAL_RUNS));
        setNextCursor(page.next_cursor);
      })
      .catch((cause: unknown) => {
        if (requestGeneration.current !== generation) return;
        setError(cause instanceof Error ? cause.message : "无法读取 Run 历史。");
      })
      .finally(() => {
        if (requestGeneration.current === generation) setLoading(false);
      });

    return () => {
      if (requestGeneration.current === generation) {
        requestGeneration.current += 1;
        detailRequestGeneration.current += 1;
      }
    };
  }, [api, refreshKey, workItemId, worktreeId]);

  const refreshTaskRuns = useCallback(() => setRefreshKey((value) => value + 1), []);

  const loadMoreTaskRuns = useCallback(async () => {
    if (!api || !nextCursor || loadingMore || runs.length >= MAX_LOCAL_RUNS) return;
    const generation = requestGeneration.current;
    setLoadingMore(true);
    setError(null);
    try {
      const page = await api.listTaskRuns(worktreeId, workItemId, { limit: 20, cursor: nextCursor });
      if (requestGeneration.current !== generation) return;
      setRuns((current) => {
        const known = new Set(current.map((run) => run.run_id));
        return [...current, ...page.runs.filter((run) => !known.has(run.run_id))].slice(0, MAX_LOCAL_RUNS);
      });
      setNextCursor(page.next_cursor);
    } catch (cause) {
      if (requestGeneration.current === generation) {
        setError(cause instanceof Error ? cause.message : "无法继续读取 Run 历史。");
      }
    } finally {
      if (requestGeneration.current === generation) setLoadingMore(false);
    }
  }, [api, loadingMore, nextCursor, runs.length, workItemId, worktreeId]);

  const openTaskRunDetail = useCallback(async (runId: string) => {
    if (!api) return;
    const generation = ++detailRequestGeneration.current;
    setSelectedRunId(runId);
    setDetail(null);
    setDetailError(null);
    setDetailPending(true);
    try {
      const result = await api.getTaskRunDetail(worktreeId, workItemId, runId);
      if (detailRequestGeneration.current === generation) setDetail(result);
    } catch (cause) {
      if (detailRequestGeneration.current === generation) {
        setDetailError(cause instanceof Error ? cause.message : "无法读取 Run 详情。");
      }
    } finally {
      if (detailRequestGeneration.current === generation) setDetailPending(false);
    }
  }, [api, workItemId, worktreeId]);

  return (
    <section className="card space-y-3" aria-label="Task Execution Run 历史">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h3 className="text-sm font-semibold">Execution Runs</h3>
          <p className="text-[10px] text-ink-mute">按当前 Task 与 Worktree 授权读取；仅显示结构化状态、事件和脱敏 Evidence 元数据。</p>
        </div>
        <button type="button" className="btn flex items-center gap-1.5 text-[10px]" onClick={refreshTaskRuns} disabled={!api || loading}>
          <RefreshCw size={12} className={loading ? "animate-spin" : ""} />
          {loading ? "读取中…" : "刷新 Runs"}
        </button>
      </div>

      {!api && <p className="rounded border border-warning/30 bg-warning/5 p-2 text-[10px] text-warning">当前为预览上下文；Run 历史需连接已授权的 Group API。</p>}
      {error && <p role="alert" className="text-[10px] text-err">{error}</p>}
      {api && !loading && !error && runs.length === 0 && <p className="text-[10px] text-ink-mute">该 Task 尚无可见的执行 Run。</p>}

      {runs.length > 0 && (
        <ul className="grid gap-2 md:grid-cols-2">
          {runs.map((run) => (
            <li key={run.run_id}>
              <button
                type="button"
                className={`w-full rounded border p-2 text-left text-[10px] ${selectedRunId === run.run_id ? "border-accent bg-accent/5" : "border-line bg-bg-soft/30"}`}
                aria-pressed={selectedRunId === run.run_id}
                onClick={() => void openTaskRunDetail(run.run_id)}
              >
                <span className="flex flex-wrap items-center justify-between gap-2">
                  <span className="font-mono">{run.run_id}</span>
                  <span className="rounded border border-line px-1.5 py-0.5">{run.execution_state ?? "unknown"}</span>
                </span>
                <span className="mt-1 block text-ink-mute">
                  {run.run_origin} · {run.execution_channel} · {new Date(run.started_at).toLocaleString()}
                </span>
                <span className="mt-1 block text-ink-mute">
                  verify: {run.verification_state ?? "unknown"} · acceptance: {run.human_acceptance_state ?? "unknown"}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}

      {nextCursor && runs.length < MAX_LOCAL_RUNS && (
        <button type="button" className="btn text-[10px]" onClick={() => void loadMoreTaskRuns()} disabled={!api || loadingMore}>
          {loadingMore ? "读取中…" : "加载更早的 Runs"}
        </button>
      )}

      {selectedRunId && (
        <div className="space-y-2 border-t border-line pt-3" aria-label="Run 详情">
          <div className="flex items-center justify-between gap-2">
            <h4 className="text-xs font-semibold">Run 事件与 Evidence</h4>
            <span className="font-mono text-[10px] text-ink-mute">{selectedRunId}</span>
          </div>
          {detailPending && <p className="text-[10px] text-ink-mute">读取 Run 详情…</p>}
          {detailError && <p role="alert" className="text-[10px] text-err">{detailError}</p>}
          {detail && (
            <>
              <section className="rounded border border-line/70 p-2" aria-label="Run Profile 快照">
                <h5 className="mb-1 text-[10px] font-semibold">Profile 与 spawn fence</h5>
                <dl className="grid gap-x-3 gap-y-1 text-[10px] sm:grid-cols-[max-content_1fr]">
                  <dt className="text-ink-mute">Approved Launch Profile</dt>
                  <dd className="break-all font-mono">
                    {detail.run.approved_launch_profile_id
                      ? `${detail.run.approved_launch_profile_id} · v${detail.run.approved_launch_profile_version ?? "?"} · ${detail.run.approved_launch_profile_digest ?? "no digest"}`
                      : "未记录（历史 Run）"}
                  </dd>
                  <dt className="text-ink-mute">Agent Execution Profile</dt>
                  <dd className="break-all font-mono">
                    {detail.run.execution_profile_id
                      ? `${detail.run.execution_profile_id} · v${detail.run.execution_profile_version ?? "?"} · ${detail.run.execution_profile_digest ?? "no digest"}`
                      : "未记录（历史 Run）"}
                  </dd>
                  <dt className="text-ink-mute">Spawn binding digest</dt>
                  <dd className="break-all font-mono">
                    {detail.run.spawn_fence_binding_digest ?? "未记录（历史 Run）"}
                  </dd>
                </dl>
              </section>
              <ol className="max-h-64 space-y-1 overflow-auto rounded border border-line p-2">
                {detail.events.map((event) => (
                  <li key={event.event_id} className="flex flex-wrap items-center justify-between gap-2 border-b border-line/60 py-1 text-[10px] last:border-0">
                    <span>{event.event_type}{event.loop_iteration_no != null ? ` · iteration ${event.loop_iteration_no}` : ""}</span>
                    <span className="text-ink-mute">{event.execution_state ?? event.verification_state ?? event.hook_decision ?? "recorded"} · {new Date(event.occurred_at).toLocaleString()}</span>
                  </li>
                ))}
                {detail.events_truncated && <li className="text-[10px] text-warning">仅展示最近 100 条事件；更早事件仍保存在 Run 历史中。</li>}
                {detail.events.length === 0 && <li className="text-[10px] text-ink-mute">暂无事件。</li>}
              </ol>
              <div className="space-y-1">
                <h5 className="text-[10px] font-semibold">Evidence metadata</h5>
                {detail.evidence.length === 0
                  ? <p className="text-[10px] text-ink-mute">暂无 Evidence。</p>
                  : <ul className="space-y-1">
                      {detail.evidence.map((item) => (
                        <li key={item.evidence_id} className="rounded border border-line/70 p-2 text-[10px]">
                          <span className="font-medium">{item.evidence_kind}</span>
                          <span className="ml-2 text-ink-mute">{item.sensitivity} · {item.byte_length ?? "?"} bytes · {item.sha256_digest ?? "no digest"}</span>
                          {item.summary && <p className="mt-1 break-words text-ink-dim">{item.summary}</p>}
                        </li>
                      ))}
                    </ul>}
                {detail.evidence_truncated && <p className="text-[10px] text-warning">Evidence 超过单次展示上限。</p>}
              </div>
            </>
          )}
        </div>
      )}
    </section>
  );
}
