/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/RunWorkspaceShell.tsx",type:"file",language:"tsx"}),
 (route:Class {name:"RunWorkspaceRoute",type:"class"}),(load:Class {name:"ContextLoad",type:"class"}),
 (shell:Function {name:"RunWorkspaceShell",type:"function"}),(resolve:Function {name:"resolveCanonicalRunContext",type:"function"}),
 (detail:Function {name:"RunContextDetails",type:"function"}),(session:Function {name:"useRunDirectory",type:"function"}),
 (href:Function {name:"canonicalRunWorktreeHref",type:"function"}),(context:Function {name:"RunDirectoryApiClient.getRunContext",type:"function"}),
 (error:Function {name:"directoryErrorMessage",type:"function"}),(tabs:Variable {name:"RUN_APP_TABS",type:"variable"}),
 (state:Function {name:"useState",type:"function"}),(effect:Function {name:"useEffect",type:"function"}),
 (f)-[:CONTAINS]->(route),(f)-[:CONTAINS]->(load),(f)-[:CONTAINS]->(shell),(f)-[:CONTAINS]->(detail),(f)-[:CONTAINS]->(tabs),
 (shell)-[:CONTAINS]->(resolve),(shell)-[:CALLS]->(session),(shell)-[:CALLS]->(href),(shell)-[:CALLS]->(state),(shell)-[:CALLS]->(effect),
 (shell)-[:CALLS]->(detail),(resolve)-[:CALLS]->(context),(resolve)-[:CALLS]->(error),(shell)-[:USES]->(tabs);
*/

"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { canonicalRunWorktreeHref, RunDirectoryError, type RunContextEnvelope, type RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import { directoryErrorMessage } from "@/lib/run/runDirectoryTree";

export interface RunWorkspaceRoute { project_id: string; branch_id: string; engineering_run_id: string; worktree_id: string }
type ContextLoad = { client: RunDirectoryApiClient; route: string; status: "ready"; envelope: RunContextEnvelope } |
  { client: RunDirectoryApiClient; route: string; status: "error"; message: string };
const RUN_APP_TABS = ["Inbox", "Task Cards", "Canvas", "Workflow / LangGraph", "Run BI / Benchmark", "Plugin Apps"] as const;

export function RunWorkspaceShell({ route }: { route: RunWorkspaceRoute }) {
  const { client, status, message, setFocus, clearFocus } = useRunDirectory();
  const [load, setLoad] = useState<ContextLoad | null>(null);
  const [retry, setRetry] = useState(0);
  const [chatScope, setChatScope] = useState<"WORKTREE" | "GLOBAL">("WORKTREE");
  let href = "";
  try { href = canonicalRunWorktreeHref(route.project_id, route.branch_id, route.engineering_run_id, route.worktree_id); } catch { /* Invalid routes never reach the API. */ }
  const projectId = route.project_id.toLowerCase();
  const branchId = route.branch_id.toLowerCase();
  const runId = route.engineering_run_id.toLowerCase();
  const worktreeId = route.worktree_id.toLowerCase();
  useEffect(function resolveCanonicalRunContext() {
    clearFocus();
    setChatScope("WORKTREE");
    if (!client || !href || status !== "ready") return;
    const controller = new AbortController();
    void client.getRunContext(runId, worktreeId, controller.signal).then((envelope) => {
      if (controller.signal.aborted) return;
      const context = envelope.run_context;
      if (context.project_id !== projectId || context.branch_id !== branchId || context.engineering_run_id !== runId || envelope.focus?.worktree_id !== worktreeId) {
        throw new RunDirectoryError(0, "scope_mismatch", "当前路径与授权 RunContext 不一致，checkout 焦点未设置。");
      }
      setLoad({ client, route: href, status: "ready", envelope });
      setFocus(envelope);
    }).catch((error: unknown) => {
      if (!controller.signal.aborted) setLoad({ client, route: href, status: "error", message: directoryErrorMessage(error) });
    });
    return () => { controller.abort(); clearFocus(); };
  }, [branchId, clearFocus, client, href, projectId, retry, runId, setFocus, status, worktreeId]);
  const current = load && load.client === client && load.route === href ? load : null;
  if (!href) return <section className="p-6" role="alert"><h1 className="text-lg font-semibold">Run 路径无效</h1><p className="mt-2 text-sm text-ink-mute">Project、Branch、Run 与 Worktree 必须使用有效目录身份。</p></section>;
  if (status !== "ready" || !client) return <section className="p-6" role="status"><h1 className="text-lg font-semibold">Engineering Run</h1><p className="mt-2 text-sm text-ink-mute">{message}</p></section>;
  if (!current) return <section className="p-6" role="status">正在验证 Project、云端 Branch、Engineering Run 与 Worktree 焦点…</section>;
  if (current.status === "error") return <section className="p-6" role="alert"><h1 className="text-lg font-semibold">Run 工作区暂不可用</h1><p className="my-3 text-sm text-ink-mute">{current.message}</p><button type="button" className="rounded border border-line px-3 py-1 text-sm" onClick={() => { setLoad(null); setRetry((value) => value + 1); }}>重试授权上下文</button></section>;
  const { envelope } = current;
  const focus = envelope.focus;
  if (!focus) return null;
  const hooksQuery = new URLSearchParams({ project_id: projectId, worktree_id: worktreeId });
  return <section data-testid="engineering-run-workspace" data-engineering-run-id={runId} data-focus-worktree-id={worktreeId} className="flex min-h-[calc(100vh-4rem)] flex-col p-4 md:p-6">
    <header className="border-b border-line pb-4">
      <p className="break-all text-xs text-ink-mute">Project {projectId} / 云端 Branch {envelope.branch.name} / Engineering Run / Worktree</p>
      <h1 className="mt-2 text-xl font-semibold">{envelope.engineering_run.title}</h1>
      <p className="mt-1 text-sm">Checkout 焦点：<strong>{focus.name}</strong> · {focus.machine_state}{focus.archived && " · 已归档"}</p>
      <p className="mt-2 text-xs text-ink-mute">协作、任务、Canvas、Workflow、分析与插件的工作区属于此 Engineering Run。Worktree 提供本地 checkout 焦点。</p>
    </header>
    <div role="tablist" aria-label="Engineering Run Apps" className="mt-4 flex flex-wrap gap-2">
      {RUN_APP_TABS.map((tab) => <button role="tab" aria-selected={false} disabled type="button" key={tab} className="rounded border border-line px-3 py-2 text-xs text-ink-mute">{tab} · 待迁移</button>)}
    </div>
    <div role="status" className="my-4 rounded border border-line bg-bg-soft p-4 text-sm">
      <p>{envelope.capabilities.run_owned_apps_available ? "此入口的 Run Apps 适配器尚未装配。" : "服务端尚未开放 Run 所有的 Apps。"} Task Cards 与 Canvas 为同级 App，当前仅提供已授权目录与 checkout 焦点。</p>
      <p className="mt-2">{envelope.capabilities.execution_admission_available ? "此入口的任务执行适配器尚未装配。" : "服务端执行准入尚未开放。"} CLI 将从任务卡内打开；任务绑定、runtime、取消、独立验证和结果回写完成前，执行保持禁用。</p>
    </div>
    <RunContextDetails envelope={envelope} />
    <div className="mt-4 flex flex-wrap items-center gap-3 text-xs">
      <span className="text-ink-mute">有效 Hook 策略与执行结果投影待接入。</span>
      <Link href={`/settings/advanced/hooks?${hooksQuery}`} className="text-accent underline">高级设置 → Hooks</Link>
    </div>
    <footer className="mt-auto border-t border-line pt-4">
      <div className="flex items-center gap-2 text-xs"><span>聊天范围</span>
        {(["WORKTREE", "GLOBAL"] as const).map((scope) => <button type="button" aria-pressed={chatScope === scope} key={scope} onClick={() => setChatScope(scope)} className={`rounded border border-line px-2 py-1 ${chatScope === scope ? "bg-accent/15 text-accent" : "text-ink-mute"}`}>{scope}</button>)}
      </div>
      <div className="mt-2 flex gap-2"><input disabled aria-label={`${chatScope} 聊天`} placeholder="聊天 runtime 尚未装配" className="min-w-0 flex-1 rounded border border-line bg-bg-soft px-3 py-2 text-sm" /><button disabled type="button" className="rounded border border-line px-3 text-xs text-ink-mute">发送</button></div>
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
      <dt className="text-ink-mute">Worktree</dt><dd>{focus?.worktree_id}</dd>
      <dt className="text-ink-mute">Checkout 版本</dt><dd>{envelope.focus_version}</dd>
      <dt className="text-ink-mute">Workspace</dt><dd>{focus?.workspace_id}</dd>
      <dt className="text-ink-mute">Agent / Session</dt><dd>{focus?.agent_id ?? "未绑定"} / {focus?.agent_session_id ?? "未绑定"}</dd>
      <dt className="text-ink-mute">Runtime</dt><dd>{focus?.runtime_id ?? "未绑定"}</dd>
      <dt className="text-ink-mute">授权</dt><dd>Project {context.authorization.project.role} v{context.authorization.project.version} · Branch {context.authorization.branch.role} v{context.authorization.branch.version} · Run {context.authorization.engineering_run.role} v{context.authorization.engineering_run.version}</dd>
    </dl>
  </details>;
}
