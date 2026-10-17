/*
@cypher schema=1 source_sha256=575bd35fea8fb824f9ba2907154ca382b514f8a880ab7889bc9eed08132a6c4b
MERGE (self:File {path:"frontend/src/components/run/RunBiPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/run/RunBiPanel.tsx::RunBiPanel",kind:"function"})
MERGE (api:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.listRunTaskEvents",kind:"method"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (panel)-[:CALLS]->(api)
@endcypher
*/
/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"RunBiPanel.tsx",type:"file",language:"tsx"}),
 (panel:Function {name:"RunBiPanel",type:"function"}),
 (query:Function {name:"RunDirectoryApiClient.listRunTaskEvents",type:"function"}),
 (f)-[:CONTAINS]->(panel),(panel)-[:CALLS]->(query);
*/
"use client";

import { useEffect, useState } from "react";
import { RunDirectoryError, type EngineeringRun, type RunBiTaskEventCursor, type RunBiTaskEventPage, type RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { directoryErrorMessage } from "@/lib/run/runDirectoryTree";

const MAX_CURSOR_HISTORY = 100;
type Load = { client: RunDirectoryApiClient; scope: string; cursor: RunBiTaskEventCursor | undefined; retry: number } &
  ({ status: "ready"; page: RunBiTaskEventPage } | { status: "error"; message: string });

export function RunBiPanel({ client, run }: { client: RunDirectoryApiClient; run: EngineeringRun }) {
  const projectId = run.project_id;
  const runId = run.engineering_run_id;
  const scope = `${projectId}/${runId}`;
  const [navigation, setNavigation] = useState<{ scope: string; cursors: (RunBiTaskEventCursor | undefined)[]; index: number }>({ scope, cursors: [undefined], index: 0 });
  const [retry, setRetry] = useState(0);
  const [load, setLoad] = useState<Load | null>(null);
  const currentNavigation = navigation.scope === scope ? navigation : { scope, cursors: [undefined], index: 0 };
  const cursor = currentNavigation.cursors[currentNavigation.index];
  useEffect(() => {
    const controller = new AbortController();
    void client.listRunTaskEvents(run, cursor, controller.signal).then((page) => {
      if (controller.signal.aborted) return;
      if (page.project_id !== projectId || page.engineering_run_id !== runId) throw new Error("分析结果与当前 Run 不一致，请重新加载。");
      setLoad({ client, scope, cursor, retry, status: "ready", page });
    }).catch((error: unknown) => {
      if (!controller.signal.aborted) setLoad({ client, scope, cursor, retry, status: "error", message: error instanceof RunDirectoryError && error.status === 503 ? "Run 分析暂不可用，请稍后重试。" : directoryErrorMessage(error) });
    });
    return () => controller.abort();
  }, [client, run, projectId, runId, scope, cursor, retry]);
  const current = load?.client === client && load.scope === scope && load.cursor === cursor && load.retry === retry ? load : null;
  const page = current?.status === "ready" ? current.page : null;
  return <section aria-label="Run BI 事件" className="rounded border border-line p-4">
    <h2 className="font-semibold">Run BI 事件</h2>
    <p className="mt-2 text-sm text-ink-mute">此视图属于当前 Run，汇总其任务事件。完整性：未知；新鲜度：未知。事件尚未形成完整分析或 Benchmark 结果。</p>
    <div className="my-3 flex flex-wrap items-center gap-3 text-sm">
      <button type="button" className="rounded border border-line px-3 py-1" onClick={() => { setNavigation({ scope, cursors: [undefined], index: 0 }); setRetry((value) => value + 1); }}>刷新首屏</button>
      <span>第 {currentNavigation.index + 1} 页 · 每页最多 50 条</span>
    </div>
    {!current && <p role="status">正在读取当前 Run 的任务事件…</p>}
    {current?.status === "error" && <p role="alert">{current.message}</p>}
    {page && <>
      {page.items.length === 0 ? <p role="status">尚未观察到任务事件；这不表示全部事件已送达。</p> : <div className="overflow-x-auto"><table className="w-full text-left text-xs">
        <thead><tr><th className="p-2">时间</th><th className="p-2">任务 / 事件</th><th className="p-2">状态</th></tr></thead>
        <tbody>{page.items.map((item) => <tr key={item.event_id} className="border-t border-line">
          <td className="p-2 align-top"><time dateTime={item.occurred_at}>{item.occurred_at}</time></td>
          <td className="break-all p-2">{item.work_item_id}<br />{item.event_type}<br /><span className="text-ink-mute">{item.event_id}</span></td>
          <td className="p-2">{item.from_status && item.to_status ? `${item.from_status} → ${item.to_status}` : item.lifecycle_status ?? "未记录"}{item.review_state && ` · ${item.review_state}`}{item.owner_changed && " · 归属变更"}</td>
        </tr>)}</tbody>
      </table></div>}
      <div className="mt-4 flex gap-3 text-sm">
        <button type="button" disabled={currentNavigation.index === 0} className="rounded border border-line px-3 py-1 disabled:opacity-50" onClick={() => setNavigation({ ...currentNavigation, index: currentNavigation.index - 1 })}>上一页</button>
        <button type="button" disabled={!page.has_more || !page.next_cursor || currentNavigation.index + 1 >= MAX_CURSOR_HISTORY} className="rounded border border-line px-3 py-1 disabled:opacity-50" onClick={() => {
          if (!page.next_cursor || currentNavigation.index + 1 >= MAX_CURSOR_HISTORY) return;
          setNavigation({ scope, cursors: [...currentNavigation.cursors.slice(0, currentNavigation.index + 1), page.next_cursor], index: currentNavigation.index + 1 });
        }}>下一页</button>
      </div>
      {page.has_more && currentNavigation.index + 1 >= MAX_CURSOR_HISTORY && <p className="mt-2 text-xs text-ink-mute">已达到本次浏览的 100 页上限，可刷新首屏重新浏览。</p>}
    </>}
  </section>;
}
