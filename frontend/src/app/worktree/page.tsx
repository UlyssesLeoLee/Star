/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/worktree/page.tsx", type:"file", language:"tsx"}),
  (page:Function {name:"WorktreePage", type:"function", signature:"WorktreePage(): JSX.Element", visibility:"public", complexity:"complex"}),
  (row:Function {name:"Row", type:"function", signature:"Row(props: { label: string; value: React.ReactNode }): JSX.Element", visibility:"private", complexity:"simple"}),
  (apps:Variable {name:"WORKTREE_GROUP_APPS", type:"variable", language:"typescript"}),
  (editor:Variable {name:"MonacoEditor", type:"variable", language:"typescript"}),
  (projects:Variable {name:"projects", type:"variable", language:"typescript"}),
  (agentSessions:Variable {name:"agentSessions", type:"variable", language:"typescript"}),
  (localRuntimes:Variable {name:"localRuntimes", type:"variable", language:"typescript"}),
  (project:Variable {name:"selectedProject", type:"variable", language:"typescript"}),
  (requestedProjectId:Variable {name:"requestedProjectId", type:"variable", language:"typescript"}),
  (persistedProject:Variable {name:"persistedProject", type:"variable", language:"typescript"}),
  (router:Variable {name:"router", type:"variable", language:"typescript"}),
  (searchParams:Variable {name:"searchParams", type:"variable", language:"typescript"}),
  (worktrees:Variable {name:"projectWorktrees", type:"variable", language:"typescript"}),
  (worktree:Variable {name:"wt", type:"variable", language:"typescript"}),
  (projectSelectionGate:Logic {name:"projectSelectionGate", type:"logic", language:"tsx", complexity:"simple"}),
  (nextStates:Variable {name:"allowedNext", type:"variable", language:"typescript"}),
  (selectProject:Function {name:"selectProject", type:"function", signature:"onChange(event): void", visibility:"private", complexity:"simple"}),
  (syncProjectSelection:Function {name:"syncProjectSelection", type:"function", signature:"useEffect callback(): void", visibility:"private", complexity:"moderate"}),
  (projectMap:Function {name:"projectMap", type:"function", signature:"projects.map(project => JSX.Element)", visibility:"private", complexity:"simple"}),
  (worktreeMap:Function {name:"worktreeMap", type:"function", signature:"projectWorktrees.map(worktree => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (transitionMap:Function {name:"transitionMap", type:"function", signature:"allowedNext.map(status => JSX.Element)", visibility:"private", complexity:"simple"}),
  (appMap:Function {name:"appMap", type:"function", signature:"WORKTREE_GROUP_APPS.map(app => JSX.Element)", visibility:"private", complexity:"simple"}),
  (filterProject:Function {name:"filterProject", type:"function", signature:"worktrees.filter(worktree => project_id matches)", visibility:"private", complexity:"simple"}),
  (agentForWorktree:Function {name:"agentForWorktree", type:"function", signature:"agentSessions.find(session => session.worktree_id === worktree.id)", visibility:"private", complexity:"simple"}),
  (runtimeForWorktree:Function {name:"runtimeForWorktree", type:"function", signature:"localRuntimes.find(runtime => runtime.id === worktree.local_runtime_id)", visibility:"private", complexity:"simple"}),
  (formatTimestamp:Function {name:"formatTimestamp", type:"function", signature:"formatTimestamp(value: string): string", visibility:"private", complexity:"simple"}),
  (deriveTransitions:Function {name:"deriveTransitions", type:"function", signature:"WORKTREE_SM.transitions.filter(...).map(...)", visibility:"private", complexity:"simple"}),
  (file)-[:CONTAINS]->(page),
  (file)-[:CONTAINS]->(row),
  (file)-[:CONTAINS]->(apps),
  (file)-[:CONTAINS]->(editor),
  (file)-[:CONTAINS]->(projectSelectionGate),
  (page)-[:USES]->(projects),
  (page)-[:USES]->(agentSessions),
  (page)-[:USES]->(localRuntimes),
  (page)-[:USES]->(project),
  (page)-[:USES]->(requestedProjectId),
  (page)-[:USES]->(persistedProject),
  (page)-[:USES]->(router),
  (page)-[:USES]->(searchParams),
  (page)-[:USES]->(worktrees),
  (page)-[:USES]->(projectSelectionGate),
  (page)-[:USES]->(wt),
  (page)-[:USES]->(nextStates),
  (page)-[:CALLS]->(filterProject),
  (page)-[:CALLS]->(agentForWorktree),
  (page)-[:CALLS]->(runtimeForWorktree),
  (page)-[:CALLS]->(formatTimestamp),
  (page)-[:CALLS]->(deriveTransitions),
  (page)-[:CALLS]->(projectMap),
  (page)-[:CALLS]->(worktreeMap),
  (page)-[:CALLS]->(transitionMap),
  (page)-[:CALLS]->(appMap),
  (page)-[:CALLS]->(selectProject),
  (page)-[:CALLS]->(syncProjectSelection),
  (selectProject)-[:USES]->(router),
  (selectProject)-[:USES]->(searchParams),
  (syncProjectSelection)-[:USES]->(router),
  (syncProjectSelection)-[:USES]->(searchParams),
  (syncProjectSelection)-[:USES]->(persistedProject),
  (syncProjectSelection)-[:USES]->(project),
  (page)-[:CALLS]->(row),
  (page)-[:USES]->(apps),
  (page)-[:USES]->(editor);
*/

"use client";

import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { WORKTREE_SM, type Worktree, type WorktreeStatus } from "@/types/ids";
import { GitBranch, GitMerge, Lock, Cpu, AlertCircle } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";
import dynamic from "next/dynamic";

function formatTimestamp(value: string): string {
  const timestamp = new Date(value);
  return Number.isNaN(timestamp.getTime())
    ? "—"
    : `${timestamp.toISOString().slice(0, 16).replace("T", " ")} UTC`;
}

const WORKTREE_GROUP_APPS = [
  { id: "multica", label: "Multica" },
  { id: "jira", label: "Jira 视图" },
  { id: "task-card", label: "Task Cards" },
  { id: "canvas", label: "Infinite Canvas" },
  { id: "workflow", label: "Workflow / LangGraph" },
  { id: "plugins", label: "Plugins" },
] as const;

// ULYS-98-W1.1 — Monaco editor (loaded client-side only).
const MonacoEditor = dynamic(
  () => import("@/components/editor/MonacoEditor"),
  {
    ssr: false,
    loading: () => (
      <div className="flex h-32 items-center justify-center text-xs text-ink-mute">
        Loading editor…
      </div>
    ),
  },
);

export default function WorktreePage() {
  const { t } = useTranslation();
  const router = useRouter();
  const searchParams = useSearchParams();
  const { projects, worktrees, agentSessions, localRuntimes, transitionWorktree } = useStore();
  const selectedProjectId = useNavStore((state) => state.selectedProjectId);
  const setSelectedProjectId = useNavStore((state) => state.setSelectedProjectId);
  const [hasMounted, setHasMounted] = useState(false);
  const requestedProjectId = searchParams.get("project_id");
  const persistedProject = projects.find((project) => project.id === (hasMounted ? selectedProjectId : null)) ?? null;
  const selectedProject = requestedProjectId
    ? projects.find((project) => project.id === requestedProjectId) ?? null
    : persistedProject;
  const projectWorktrees = worktrees.filter((worktree) => worktree.project_id === selectedProject?.id);
  const [selected, setSelected] = useState<string>("");

  useEffect(() => setHasMounted(true), []);

  useEffect(() => {
    if (!hasMounted || !selectedProject) return;

    if (selectedProjectId !== selectedProject.id) {
      setSelectedProjectId(selectedProject.id);
    }

    if (requestedProjectId !== selectedProject.id) {
      const query = new URLSearchParams(searchParams.toString());
      query.set("project_id", selectedProject.id);
      router.replace(`/worktree?${query.toString()}`, { scroll: false });
    }
  }, [hasMounted, requestedProjectId, router, searchParams, selectedProject, selectedProjectId, setSelectedProjectId]);

  useEffect(() => {
    if (!projectWorktrees.some((worktree) => worktree.id === selected)) {
      setSelected(projectWorktrees[0]?.id ?? "");
    }
  }, [projectWorktrees, selected]);

  const wt = projectWorktrees.find((worktree) => worktree.id === selected);
  const agentForWorktree = (worktree: Worktree) =>
    agentSessions.find((session) => session.id === worktree.agent_session_id && session.worktree_id === worktree.id);
  const runtimeForWorktree = (worktree: Worktree) =>
    localRuntimes.find((runtime) => runtime.id === worktree.local_runtime_id && runtime.tenant_id === worktree.tenant_id);
  const selectedAgent = wt ? agentForWorktree(wt) : undefined;
  const selectedRuntime = wt ? runtimeForWorktree(wt) : undefined;
  const allowedNext = wt
    ? Array.from(new Set(
        WORKTREE_SM.transitions.filter((t) => t.from === wt.status).map((t) => t.to),
      ))
    : [];

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/worktree'].title}
        subtitle="项目级 Worktree 管理 — 先选择 Project，再比较 Agent、Runtime、PR、冲突与锁信号；当前展示本地原型数据。"
        icon={<GitBranch className="text-accent" size={20} />}
        track="B"
        count={projectWorktrees.length}
      />

      <section className="card mb-4 flex flex-wrap items-center gap-3" aria-label="项目 Worktree 范围">
        <label htmlFor="worktree-project" className="text-xs font-semibold text-ink-dim">Project</label>
        <select
          id="worktree-project"
          value={selectedProject?.id ?? ""}
          onChange={(event) => {
            const projectId = event.target.value;
            setSelectedProjectId(projectId);
            setSelected("");
            const query = new URLSearchParams(searchParams.toString());
            query.set("project_id", projectId);
            router.replace(`/worktree?${query.toString()}`, { scroll: false });
          }}
          className="rounded-md border border-line bg-bg-soft px-3 py-2 text-sm"
          data-testid="worktree-project-selector"
        >
          {!selectedProject && <option value="" disabled>选择 Project</option>}
          {projects.map((project) => (
            <option key={project.id} value={project.id}>{project.key} · {project.name}</option>
          ))}
        </select>
        <span className="text-xs text-ink-mute">原型按路由中的 Project 过滤本地数据；服务端授权尚未接入。</span>
      </section>

      {requestedProjectId && !selectedProject && (
        <div role="status" className="card mb-4 text-sm text-warning" data-testid="worktree-project-unavailable">
          当前链接中的 Project 无法在本地原型数据中解析；请选择一个可用 Project。
        </div>
      )}

      {selectedProject ? (
        <>
          <div className="card mb-4 flex flex-wrap items-center gap-3 text-xs">
            <span className="font-semibold text-ink-dim">Worktree 创建 / 导入</span>
            <span className="text-ink-mute">等待 Project→Repository 绑定和服务端受权 API；当前页面不发起创建或清理操作。</span>
          </div>

          <SectionTitle>{selectedProject.key} · {selectedProject.name} 的 Worktree</SectionTitle>
          <div className="mb-5">
            <StateMachineDiagram sm={WORKTREE_SM} highlightState={wt?.status} />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        {/* 列表 */}
        <div className="lg:col-span-2">
          <div className="card">
            <SectionTitle>Project Worktree Index ({projectWorktrees.length})</SectionTitle>
            <table className="table">
              <thead>
                <tr>
                  <th>Worktree</th>
                  <th>Branch</th>
                  <th>Status</th>
                  <th>Agent / Runtime</th>
                  <th>PR</th>
                  <th>Lock</th>
                  <th>Last event</th>
                </tr>
              </thead>
              <tbody>
                {projectWorktrees.map((w) => (
                  <tr
                    key={w.id}
                    onClick={() => setSelected(w.id)}
                    aria-expanded={selected === w.id}
                    className={clsx("cursor-pointer", selected === w.id && "bg-accent/5")}
                  >
                    <td>
                      <div className="font-mono text-[10px] text-ink-mute">{w.id}</div>
                      <div className="font-medium">{w.name}</div>
                    </td>
                    <td className="font-mono text-xs text-info">{w.branch}</td>
                    <td><StatusPill value={w.status} /></td>
                    <td className="text-[10px]">
                      <div>{agentForWorktree(w)?.name ?? "未绑定 Agent"}</div>
                      <div className="text-ink-mute">{runtimeForWorktree(w)?.hostname ?? "未绑定 Runtime"}</div>
                    </td>
                    <td className="font-mono text-xs">{w.pr_id ?? "—"}</td>
                    <td className="font-mono text-xs">v{w.lock_version}</td>
                    <td className="text-ink-dim text-xs">{formatTimestamp(w.last_event_at)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* 详情 / 操作面板 */}
        <div>
          {wt && (
            <div className="card sticky top-16">
              <div className="flex items-center justify-between mb-3">
                <div>
                  <div className="text-xs text-ink-mute font-mono">{wt.id}</div>
                  <div className="text-base font-semibold">{wt.name}</div>
                </div>
                <StatusPill value={wt.status} />
              </div>

              <dl className="text-xs space-y-1.5 mb-4">
                <Row label="Branch" value={<span className="font-mono text-info">{wt.branch}</span>} />
                <Row label="Base" value={<span className="font-mono">{wt.base_branch}</span>} />
                <Row label="Local Runtime" value={selectedRuntime ? `${selectedRuntime.hostname} · ${selectedRuntime.status}` : wt.local_runtime_id ?? "未绑定"} />
                <Row label="Agent Session" value={selectedAgent ? `${selectedAgent.name} · ${selectedAgent.status}` : wt.agent_session_id ?? "未绑定"} />
                <Row label="PR" value={wt.pr_id ?? "—"} />
                <Row label="Lock version" value={<span className="font-mono">v{wt.lock_version}</span>} />
                <Row label="Created" value={formatTimestamp(wt.created_at)} />
              </dl>

              <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5 flex items-center gap-1.5">
                <GitMerge size={10} /> Allowed transitions
              </div>
              {allowedNext.length === 0 ? (
                <div className="text-xs text-ink-mute italic">无下游迁移(终态)</div>
              ) : (
                <div className="flex flex-wrap gap-1.5">
                  {allowedNext.map((to) => (
                    <button
                      key={to}
                      onClick={() => transitionWorktree(wt.id, to as WorktreeStatus)}
                      className="btn-primary"
                    >
                      → {to}
                    </button>
                  ))}
                </div>
              )}

              <div className="mt-4 pt-3 border-t border-line">
                <Link
                  href={`/canvas/canvas-001?highlight=${wt.id === "wt-001" ? "el-wt-001" : wt.id === "wt-002" ? "el-wt-002" : wt.id === "wt-003" ? "el-wt-003" : "el-wt-001"}`}
                  className="btn text-[10px]"
                  title="在 Miro 模式画布中查看 worktree node 状态(双击可跳回)"
                >
                  <span className="font-mono">⊞</span> 打开在 Canvas
                </Link>
                <Link
                  href={`/worktree/${encodeURIComponent(wt.id)}/group`}
                  className="btn-primary ml-2 text-[10px]"
                  data-testid="open-worktree-group"
                >
                  展开 Worktree 群组
                </Link>
              </div>

              <div className="mt-4 border-t border-line pt-3" data-testid="expanded-worktree-apps">
                <div className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-ink-mute">
                  {wt.name} · 展开应用
                </div>
                <nav aria-label={`${wt.name} 的 Worktree 群组应用`} className="grid grid-cols-2 gap-1">
                  {WORKTREE_GROUP_APPS.map((app) => (
                    <Link
                      key={app.id}
                      href={`/worktree/${encodeURIComponent(wt.id)}/group?app=${app.id}`}
                      className="rounded border border-line px-2 py-2 text-xs text-ink-dim transition hover:border-accent/50 hover:text-accent"
                      data-testid={`expanded-worktree-app-${app.id}`}
                    >
                      {app.label}
                    </Link>
                  ))}
                </nav>
              </div>

              <div className="mt-4 pt-3 border-t border-line text-[10px] text-ink-mute space-y-1">
                              <div className="flex items-center gap-1.5">
                                <Lock size={10} /> Optimistic lock via lock_version
                              </div>
                              <div className="flex items-center gap-1.5">
                                <Cpu size={10} /> 当前执行绑定；历史 Agent Sessions 待接入
                              </div>
                              <div className="flex items-center gap-1.5">
                                <AlertCircle size={10} /> 状态切换会触发 NATS event `star.events.{"{tenant_id}"}.worktree.worktree.*`
                              </div>
                            </div>

                            {/* ULYS-98-W1.1 — Monaco editor preview. W1 stub: read-only,
                                显示 src/lib/store.ts 的 placeholder 内容; W2 改成真实
                                file-system + AI 补全. */}
                            <div className="mt-4 pt-3 border-t border-line">
                              <div className="flex items-center justify-between mb-2">
                                <div className="text-[10px] uppercase tracking-wider text-ink-mute">
                                  Code preview
                                </div>
                                <div className="text-[10px] font-mono text-ink-mute">
                                  src/lib/store.ts
                                </div>
                              </div>
                              <div className="h-48 rounded border border-line overflow-hidden">
                                <MonacoEditor
                                  path="src/lib/store.ts"
                                  value={`// ULYS-98-W1 — Monaco editor stub preview\n// (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.1")\n//\n// W1: read-only preview only — no AI completion, no Cmd-K, no save.\n// W2: real file-system + Cmd-K inline edit + inline AI suggestion.\n\nexport interface Worktree {\n  id: string;\n  branch: string;\n  status: WorktreeStatus;\n}\n`}
                                  readOnly
                                  testId="monaco-preview"
                                />
                              </div>
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                </>
              ) : (
                <section className="card space-y-2" data-testid="worktree-project-selection-prompt">
                  <h2 className="text-sm font-semibold">先选择 Project</h2>
                  <p className="text-xs text-ink-mute">选择 Project 后，这里才显示该项目的 Worktree 清单和管理信息。</p>
                </section>
              )}
            </div>
                );
              }

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex justify-between">
      <dt className="text-ink-mute">{label}</dt>
      <dd className="text-ink">{value}</dd>
    </div>
  );
}
