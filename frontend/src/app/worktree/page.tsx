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
import { useCallback, useEffect, useRef, useState } from "react";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";
import { GroupApiError, type WorktreeGroupApiClient } from "@/lib/group/worktreeGroupApi";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { WORKTREE_SM, type Worktree, type WorktreeStatus } from "@/types/ids";
import { GitBranch, GitMerge, Lock, Cpu, AlertCircle } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";
import dynamic from "next/dynamic";

/* CYPHER STRUCTURE MANIFEST ADDENDUM
CREATE
  (indexApi:Variable {name:"worktreeGroupApi",type:"variable",language:"typescript"}),
  (indexState:Variable {name:"projectIndexState",type:"variable",language:"typescript"}),
  (memberState:Variable {name:"memberDirectoryState",type:"variable",language:"typescript"}),
  (memberSequence:Variable {name:"memberRequestSequence",type:"variable",language:"typescript"}),
  (ownerTarget:Variable {name:"ownerTarget",type:"variable",language:"typescript"}),
  (canManageOwners:Variable {name:"canManageOwners",type:"variable",language:"typescript"}),
  (projectMember:Class {name:"ProjectMember",type:"class",language:"typescript"}),
  (memberEnvelope:Class {name:"ProjectMemberDirectoryEnvelope",type:"class",language:"typescript"}),
  (memberDirectoryStateType:Class {name:"ProjectMemberDirectoryState",type:"class",language:"typescript"}),
  (managementPlanType:Class {name:"WorktreeManagementPlan",type:"class",language:"typescript"}),
  (memberEffect:Function {name:"memberDirectoryEffect",type:"function",signature:"useEffect callback(): void",visibility:"private",complexity:"moderate"}),
  (refreshIndex:Function {name:"refreshProjectWorktreeIndex",type:"function",visibility:"private",complexity:"complex"}),
  (mapIndex:Function {name:"mapProjectWorktreeIndexItem",type:"function",visibility:"private",complexity:"moderate"}),
  (liveDetails:Function {name:"LiveWorktreeDetails",type:"function",visibility:"private",complexity:"moderate"}),
  (managementPlan:Function {name:"createArchivePlan",type:"function",visibility:"private",complexity:"moderate"}),
  (managementConfirm:Function {name:"confirmArchivePlan",type:"function",visibility:"private",complexity:"moderate"}),
  (ownerPlan:Function {name:"createOwnerPlan",type:"function",visibility:"private",complexity:"moderate"}),
  (ownerConfirm:Function {name:"confirmOwnerPlan",type:"function",visibility:"private",complexity:"moderate"}),
  (memberApi:Function {name:"WorktreeGroupApiClient.listProjectMembers",type:"function",visibility:"public",complexity:"simple"});
MATCH (page:Function {name:"WorktreePage"}), (api:Variable {name:"worktreeGroupApi"}),
      (state:Variable {name:"projectIndexState"}),
      (members:Variable {name:"memberDirectoryState"}),
      (refresh:Function {name:"refreshProjectWorktreeIndex"}),
      (mapper:Function {name:"mapProjectWorktreeIndexItem"}),
      (details:Function {name:"LiveWorktreeDetails"}),
      (plan:Function {name:"createArchivePlan"}),
      (confirm:Function {name:"confirmArchivePlan"}),
      (directoryEffect:Function {name:"memberDirectoryEffect"}),
      (memberMethod:Function {name:"WorktreeGroupApiClient.listProjectMembers"}),
      (ownerPlan:Function {name:"createOwnerPlan"}),
      (ownerConfirm:Function {name:"confirmOwnerPlan"});
CREATE (page)-[:USES]->(api), (page)-[:USES]->(state), (page)-[:USES]->(members),
       (page)-[:CALLS]->(refresh), (page)-[:CALLS]->(directoryEffect),
       (directoryEffect)-[:CALLS]->(memberMethod),
       (refresh)-[:CALLS]->(mapper), (details)-[:CALLS]->(plan),
       (details)-[:CALLS]->(confirm), (details)-[:CALLS]->(ownerPlan),
       (details)-[:CALLS]->(ownerConfirm);
*/

const WORKTREE_INDEX_STATES: readonly WorktreeStatus[] = [
  "initializing", "cloning", "syncing", "active", "dirty", "behind", "diverged",
  "conflict", "committing", "pushing", "ci_running", "review_requested", "merged",
  "closed", "abandoned", "archived", "reverted",
];

interface ProjectWorktreeIndexItem {
  id: string;
  project_id: string;
  name: string;
  branch: string;
  status: WorktreeStatus;
  owner_user_id: string | null;
  agent_session_id: string | null;
  runtime_id: string | null;
  work_item_id: string | null;
  pull_request_url: string | null;
  version: number;
  last_activity: string;
  created_at: string;
  updated_at: string;
  archived: boolean;
  dirty: boolean;
  ahead: number;
  behind: number;
  locked: boolean;
  health_score: number;
  risk_count: number;
  machine_state: string;
}

interface ProjectWorktreeIndexEnvelope {
  project_id: string;
  role: string;
  next_cursor: string | null;
  worktrees: Array<Record<string, unknown>>;
}

interface ProjectMember {
  user_id: string;
  role: string;
}

interface ProjectMemberDirectoryEnvelope {
  project_id: string;
  role: string;
  members: Array<Record<string, unknown>>;
}

interface ProjectMemberDirectoryState {
  project_id: string;
  mode: "loading" | "ready" | "error";
  members: ProjectMember[];
  role?: string;
  error?: string;
}

interface ProjectWorktreeIndexState {
  project_id: string;
  mode: "loading" | "ready" | "error";
  rows: ProjectWorktreeIndexItem[];
  next_cursor: string | null;
  loading_more: boolean;
  error?: string;
}

interface WorktreeManagementPlan {
  plan_id: string;
  worktree_id: string;
  status: string;
  expires_at: string;
  operation: "set_archived" | "assign_owner";
  archived?: boolean;
  target_owner_user_id?: string;
}

function mapProjectWorktreeIndexItem(raw: Record<string, unknown>, projectId: string): ProjectWorktreeIndexItem {
  const required = (key: string): string => {
    const value = raw[key];
    if (typeof value !== "string" || !value.trim()) throw new Error(`服务端 Worktree 投影缺少 ${key}`);
    return value;
  };
  const optional = (key: string): string | null => typeof raw[key] === "string" ? raw[key] as string : null;
  const boolean = (key: string): boolean => {
    const value = raw[key];
    if (typeof value !== "boolean") throw new Error(`服务端 Worktree 投影中的 ${key} 无效`);
    return value;
  };
  const numeric = (key: string): number => {
    const value = raw[key];
    if (!Number.isSafeInteger(value)) throw new Error(`服务端 Worktree 投影中的 ${key} 无效`);
    return value as number;
  };
  const rowProjectId = required("project_id");
  if (rowProjectId !== projectId) throw new Error("Worktree Index 返回了其他 Project 的记录");
  const archived = boolean("archived");
  const humanState = required("human_state");
  const status = archived ? "archived" : WORKTREE_INDEX_STATES.includes(humanState as WorktreeStatus)
    ? humanState as WorktreeStatus
    : null;
  if (!status) throw new Error("Worktree Index 返回了不支持的生命周期状态");
  const updatedAt = required("updated_at");
  return {
    id: required("id"),
    project_id: rowProjectId,
    name: required("name"),
    branch: required("branch"),
    status,
    owner_user_id: optional("owner_user_id"),
    agent_session_id: optional("agent_session_id"),
    runtime_id: optional("runtime_id"),
    work_item_id: optional("work_item_id"),
    pull_request_url: optional("pull_request_url"),
    version: numeric("version"),
    last_activity: typeof raw.last_activity === "string" ? raw.last_activity : updatedAt,
    created_at: required("created_at"),
    updated_at: updatedAt,
    archived,
    dirty: boolean("dirty"),
    ahead: numeric("ahead"),
    behind: numeric("behind"),
    locked: boolean("locked"),
    health_score: numeric("health_score"),
    risk_count: numeric("risk_count"),
    machine_state: required("machine_state"),
  };
}

function apiErrorMessage(error: unknown): string {
  if (error instanceof GroupApiError) return `${error.code} (${error.status})`;
  return error instanceof Error ? error.message : "Worktree Index 请求失败";
}

function safePullRequestUrl(value: string | null): string | null {
  if (!value) return null;
  try {
    const url = new URL(value);
    return url.protocol === "https:" && !url.username && !url.password ? url.toString() : null;
  } catch {
    return null;
  }
}

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
  const worktreeGroupApi = useWorktreeGroupApi();
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
  const [includeArchived, setIncludeArchived] = useState(false);
  const [indexFilterDraft, setIndexFilterDraft] = useState({ owner_user_id: "", human_state: "" });
  const [indexFilters, setIndexFilters] = useState({ owner_user_id: "", human_state: "" });
  const [indexState, setIndexState] = useState<ProjectWorktreeIndexState | null>(null);
  const [memberDirectoryState, setMemberDirectoryState] = useState<ProjectMemberDirectoryState | null>(null);
  const indexRequestSequence = useRef(0);
  const memberRequestSequence = useRef(0);

  const refreshProjectWorktreeIndex = useCallback(async (projectId: string, cursor?: string) => {
    if (!worktreeGroupApi) return;
    const requestSequence = ++indexRequestSequence.current;
    setIndexState((current) => {
      if (cursor && current?.project_id === projectId) {
        return { ...current, loading_more: true, error: undefined };
      }
      return { project_id: projectId, mode: "loading", rows: [], next_cursor: null, loading_more: false };
    });
    try {
      const response = await worktreeGroupApi.listProjectWorktrees<ProjectWorktreeIndexEnvelope>(projectId, {
        limit: 50,
        cursor,
        include_archived: includeArchived,
        owner_user_id: indexFilters.owner_user_id || undefined,
        human_state: indexFilters.human_state || undefined,
      });
      if (requestSequence !== indexRequestSequence.current) return;
      if (response.project_id !== projectId || !Array.isArray(response.worktrees)) {
        throw new Error("服务端 Worktree Index 响应与当前 Project 不匹配");
      }
      const rows = response.worktrees.map((row) => mapProjectWorktreeIndexItem(row, projectId));
      setIndexState((current) => ({
        project_id: projectId,
        mode: "ready",
        rows: cursor && current?.project_id === projectId ? [...current.rows, ...rows] : rows,
        next_cursor: typeof response.next_cursor === "string" ? response.next_cursor : null,
        loading_more: false,
      }));
    } catch (error) {
      if (requestSequence !== indexRequestSequence.current) return;
      setIndexState((current) => ({
        project_id: projectId,
        mode: "error",
        rows: cursor && current?.project_id === projectId ? current.rows : [],
        next_cursor: cursor && current?.project_id === projectId ? current.next_cursor : null,
        loading_more: false,
        error: apiErrorMessage(error),
      }));
    }
  }, [includeArchived, indexFilters, worktreeGroupApi]);

  useEffect(() => setHasMounted(true), []);

  useEffect(() => {
    if (!selectedProject || !worktreeGroupApi) {
      indexRequestSequence.current += 1;
      setIndexState(null);
      return;
    }
    void refreshProjectWorktreeIndex(selectedProject.id);
    return () => { indexRequestSequence.current += 1; };
  }, [refreshProjectWorktreeIndex, selectedProject?.id, worktreeGroupApi]);

  useEffect(() => {
    if (!selectedProject || !worktreeGroupApi) {
      memberRequestSequence.current += 1;
      setMemberDirectoryState(null);
      return;
    }
    const requestSequence = ++memberRequestSequence.current;
    const projectId = selectedProject.id;
    setMemberDirectoryState({ project_id: projectId, mode: "loading", members: [] });
    void worktreeGroupApi.listProjectMembers<ProjectMemberDirectoryEnvelope>(projectId)
      .then((response) => {
        if (requestSequence !== memberRequestSequence.current) return;
        if (response.project_id !== projectId || typeof response.role !== "string" || !response.role.trim() || !Array.isArray(response.members)) {
          throw new Error("服务端 Project 成员投影与当前 Project 不匹配");
        }
        const members = response.members.map((raw) => {
          const userId = raw.user_id;
          const role = raw.role;
          if (typeof userId !== "string" || !/^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/.test(userId) || typeof role !== "string" || !role.trim()) {
            throw new Error("服务端 Project 成员投影包含无效成员");
          }
          return { user_id: userId, role };
        });
        setMemberDirectoryState({ project_id: projectId, mode: "ready", members, role: response.role });
      })
      .catch((error: unknown) => {
        if (requestSequence !== memberRequestSequence.current) return;
        setMemberDirectoryState({ project_id: projectId, mode: "error", members: [], error: apiErrorMessage(error) });
      });
    return () => { memberRequestSequence.current += 1; };
  }, [selectedProject?.id, worktreeGroupApi]);

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

  const currentProjectIndex = selectedProject && indexState?.project_id === selectedProject.id ? indexState : null;
  const liveIndexRows = currentProjectIndex?.rows ?? [];
  useEffect(() => {
    if (worktreeGroupApi) {
      if (!currentProjectIndex || currentProjectIndex.mode === "loading") return;
      if (!liveIndexRows.some((worktree) => worktree.id === selected)) {
        setSelected(liveIndexRows[0]?.id ?? "");
      }
      return;
    }
    if (!projectWorktrees.some((worktree) => worktree.id === selected)) {
      setSelected(projectWorktrees[0]?.id ?? "");
    }
  }, [currentProjectIndex, liveIndexRows, projectWorktrees, selected, worktreeGroupApi]);

  const wt = projectWorktrees.find((worktree) => worktree.id === selected);
  const liveWorktree = liveIndexRows.find((worktree) => worktree.id === selected);
  const displayedWorktreeCount = worktreeGroupApi ? liveIndexRows.length : projectWorktrees.length;
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
        subtitle="项目级 Worktree 管理 — 先选择 Project，再集中管理每个 Agent Worktree 的归属、执行绑定、生命周期与健康信号。"
        icon={<GitBranch className="text-accent" size={20} />}
        track="B"
        count={displayedWorktreeCount}
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
        <span className="text-xs text-ink-mute">
          {worktreeGroupApi ? "数据来自当前用户授权的 Worktree Index API。" : "本地预览数据；宿主登录与授权 provider 尚未装配。"}
        </span>
        {worktreeGroupApi && (
          <label className="flex items-center gap-2 text-xs text-ink-dim">
            <input type="checkbox" checked={includeArchived} onChange={(event) => setIncludeArchived(event.target.checked)} />
            包含已归档 Worktree
          </label>
        )}
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
            <span className="text-ink-mute">创建 / 导入与 Git checkout 清理仍需 Repository/Runtime 协调器；列表中的归档只管理平台记录，不删除目录。</span>
          </div>

          <SectionTitle>{selectedProject.key} · {selectedProject.name} 的 Worktree</SectionTitle>
          <div className="mb-5">
            <StateMachineDiagram sm={WORKTREE_SM} highlightState={liveWorktree?.status ?? wt?.status} />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        {/* 列表 */}
        <div className="lg:col-span-2">
          <div className="card">
            <SectionTitle>Project Worktree Index ({displayedWorktreeCount})</SectionTitle>
            {worktreeGroupApi && currentProjectIndex?.mode === "loading" && (
              <div role="status" className="mb-3 text-xs text-ink-mute">正在读取当前 Project 的授权 Worktree 投影…</div>
            )}
            {worktreeGroupApi && currentProjectIndex?.mode === "error" && (
              <div role="alert" className="mb-3 flex items-center justify-between gap-3 text-xs text-danger">
                <span>Index 读取失败：{currentProjectIndex.error}。不会回退显示本地 seed。</span>
                <button className="btn" onClick={() => void refreshProjectWorktreeIndex(selectedProject.id)}>重试</button>
              </div>
            )}
            {worktreeGroupApi && (
              <form
                className="mb-3 flex flex-wrap items-end gap-2"
                onSubmit={(event) => {
                  event.preventDefault();
                  setIndexFilters({
                    owner_user_id: indexFilterDraft.owner_user_id.trim(),
                    human_state: indexFilterDraft.human_state,
                  });
                }}
              >
                <label className="grid gap-1 text-[10px] text-ink-mute">
                  Owner user ID
                  <input
                    value={indexFilterDraft.owner_user_id}
                    onChange={(event) => setIndexFilterDraft((current) => ({ ...current, owner_user_id: event.target.value }))}
                    className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink"
                    placeholder="筛选负责人"
                    pattern="[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
                    title="请输入完整 UUID"
                  />
                </label>
                <label className="grid gap-1 text-[10px] text-ink-mute">
                  Worktree state
                  <select
                    value={indexFilterDraft.human_state}
                    onChange={(event) => setIndexFilterDraft((current) => ({ ...current, human_state: event.target.value }))}
                    className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink"
                  >
                    <option value="">全部状态</option>
                    {WORKTREE_INDEX_STATES.filter((status) => status !== "archived").map((status) => <option key={status} value={status}>{status}</option>)}
                  </select>
                </label>
                <button className="btn" type="submit">筛选 Worktree</button>
              </form>
            )}
            <table className="table">
              <thead>
                <tr>
                  <th>Worktree</th>
                  <th>Branch</th>
                  <th>Status</th>
                  <th>Owner</th>
                  <th>Agent / Runtime</th>
                  <th>PR</th>
                  <th>Health / Lock</th>
                  <th>Last activity</th>
                </tr>
              </thead>
              <tbody>
                {worktreeGroupApi ? liveIndexRows.map((w) => (
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
                    <td className="font-mono text-[10px]">{w.owner_user_id ?? "未分配"}</td>
                    <td className="font-mono text-[10px]">
                      <div>{w.agent_session_id ?? "未绑定 Agent"}</div>
                      <div className="text-ink-mute">{w.runtime_id ?? "未绑定 Runtime"}</div>
                    </td>
                    <td className="text-xs">
                      {safePullRequestUrl(w.pull_request_url) ? <a href={safePullRequestUrl(w.pull_request_url) ?? undefined} target="_blank" rel="noreferrer" onClick={(event) => event.stopPropagation()} className="text-info underline">PR</a> : "—"}
                    </td>
                    <td className="text-[10px]">
                      <div>{w.dirty ? "有未提交改动" : "工作区干净"} · +{w.ahead}/−{w.behind}</div>
                      <div className="text-ink-mute">{w.locked ? "锁定" : "未锁定"} · v{w.version} · 风险 {w.risk_count}</div>
                    </td>
                    <td className="text-ink-dim text-xs">{formatTimestamp(w.last_activity)}</td>
                  </tr>
                )) : projectWorktrees.map((w) => (
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
                    <td className="text-[10px] text-ink-mute">本地预览</td>
                    <td className="text-[10px]">
                      <div>{agentForWorktree(w)?.name ?? "未绑定 Agent"}</div>
                      <div className="text-ink-mute">{runtimeForWorktree(w)?.hostname ?? "未绑定 Runtime"}</div>
                    </td>
                    <td className="font-mono text-xs">{w.pr_id ?? "—"}</td>
                    <td className="font-mono text-xs">v{w.lock_version} · seed</td>
                    <td className="text-ink-dim text-xs">{formatTimestamp(w.last_event_at)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {!worktreeGroupApi && projectWorktrees.length === 0 && (
              <p className="py-4 text-xs text-ink-mute">该 Project 的本地预览中没有 Worktree。</p>
            )}
            {worktreeGroupApi && currentProjectIndex?.mode === "ready" && liveIndexRows.length === 0 && (
              <p className="py-4 text-xs text-ink-mute">服务端授权投影中没有匹配的 Worktree。</p>
            )}
            {worktreeGroupApi && currentProjectIndex?.next_cursor && (
              <div className="mt-3 flex items-center justify-between">
                <span className="text-xs text-ink-mute">显示 {liveIndexRows.length} 条；可继续读取下一页。</span>
                <button
                  className="btn"
                  disabled={currentProjectIndex.loading_more}
                  onClick={() => void refreshProjectWorktreeIndex(selectedProject.id, currentProjectIndex.next_cursor ?? undefined)}
                >
                  {currentProjectIndex.loading_more ? "正在读取…" : "加载更多 Worktree"}
                </button>
              </div>
            )}
          </div>
        </div>

        {/* 详情 / 操作面板 */}
        <div>
          {liveWorktree && worktreeGroupApi && (
            <LiveWorktreeDetails
              key={liveWorktree.id}
              row={liveWorktree}
              api={worktreeGroupApi}
              members={memberDirectoryState?.project_id === selectedProject.id && memberDirectoryState.mode === "ready" ? memberDirectoryState.members : []}
              membersLoading={memberDirectoryState?.project_id !== selectedProject.id || memberDirectoryState?.mode === "loading"}
              membersError={memberDirectoryState?.project_id === selectedProject.id && memberDirectoryState.mode === "error" ? memberDirectoryState.error : undefined}
              canManageOwners={memberDirectoryState?.project_id === selectedProject.id && memberDirectoryState.mode === "ready" && (memberDirectoryState.role === "tenant_admin" || memberDirectoryState.role === "project_admin")}
              onRefresh={() => refreshProjectWorktreeIndex(selectedProject.id)}
            />
          )}
          {!worktreeGroupApi && wt && (
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

function LiveWorktreeDetails({
  row,
  api,
  members,
  membersLoading,
  membersError,
  canManageOwners,
  onRefresh,
}: {
  row: ProjectWorktreeIndexItem;
  api: WorktreeGroupApiClient;
  members: ProjectMember[];
  membersLoading: boolean;
  membersError?: string;
  canManageOwners: boolean;
  onRefresh: () => Promise<void>;
}) {
  const [pendingPlan, setPendingPlan] = useState<WorktreeManagementPlan | null>(null);
  const [ownerTarget, setOwnerTarget] = useState(row.owner_user_id ?? "");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const archivedTarget = !row.archived;
  const executionBound = row.agent_session_id !== null || row.runtime_id !== null;
  const [expiryClock, setExpiryClock] = useState(() => Date.now());
  const operationInFlight = useRef(false);
  const planExpiry = pendingPlan ? Date.parse(pendingPlan.expires_at) : null;
  const planExpired = pendingPlan !== null && (planExpiry === null || !Number.isFinite(planExpiry) || planExpiry <= expiryClock);

  useEffect(() => {
    if (planExpiry === null || !Number.isFinite(planExpiry)) return;
    const timeout = window.setTimeout(
      () => setExpiryClock(Date.now()),
      Math.max(0, planExpiry - Date.now()) + 1,
    );
    return () => window.clearTimeout(timeout);
  }, [planExpiry]);

  const createArchivePlan = async () => {
    if (operationInFlight.current) return;
    operationInFlight.current = true;
    setBusy(true);
    setError(null);
    setNotice(null);
    setPendingPlan(null);
    try {
      const response = await api.createManagementPlan<Record<string, unknown>>(
        row.id,
        {
          operation: "set_archived",
          archived: archivedTarget,
          expected_version: row.version,
          correlation_id: crypto.randomUUID(),
        },
        crypto.randomUUID(),
      );
      if (
        typeof response.plan_id !== "string" ||
        response.worktree_id !== row.id ||
        response.operation !== "set_archived" ||
        typeof response.expires_at !== "string" ||
        response.status !== "pending" ||
        !Number.isFinite(Date.parse(response.expires_at)) ||
        Date.parse(response.expires_at) <= Date.now()
      ) {
        throw new Error("服务端返回了不匹配的 Worktree 管理计划");
      }
      setPendingPlan({
        plan_id: response.plan_id,
        worktree_id: row.id,
        status: "pending",
        expires_at: response.expires_at,
        operation: "set_archived",
        archived: archivedTarget,
      });
    } catch (requestError) {
      setError(apiErrorMessage(requestError));
    } finally {
      operationInFlight.current = false;
      setBusy(false);
    }
  };

  const confirmArchivePlan = async () => {
    if (!pendingPlan || pendingPlan.operation !== "set_archived" || planExpired || operationInFlight.current) return;
    operationInFlight.current = true;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await api.confirmManagementPlan(row.id, pendingPlan.plan_id);
      setPendingPlan(null);
      setNotice(pendingPlan.archived ? "Worktree 已归档；Git checkout 不会因此被删除。" : "Worktree 已恢复到 Index。");
      await onRefresh();
    } catch (requestError) {
      setError(apiErrorMessage(requestError));
    } finally {
      operationInFlight.current = false;
      setBusy(false);
    }
  };

  const createOwnerPlan = async () => {
    if (operationInFlight.current || !members.some((member) => member.user_id === ownerTarget) || ownerTarget === row.owner_user_id) return;
    operationInFlight.current = true;
    setBusy(true);
    setError(null);
    setNotice(null);
    setPendingPlan(null);
    try {
      const response = await api.createManagementPlan<Record<string, unknown>>(
        row.id,
        {
          operation: "assign_owner",
          owner_user_id: ownerTarget,
          expected_version: row.version,
          correlation_id: crypto.randomUUID(),
        },
        crypto.randomUUID(),
      );
      if (
        typeof response.plan_id !== "string" ||
        response.worktree_id !== row.id ||
        response.operation !== "assign_owner" ||
        typeof response.expires_at !== "string" ||
        response.status !== "pending" ||
        !Number.isFinite(Date.parse(response.expires_at)) ||
        Date.parse(response.expires_at) <= Date.now()
      ) {
        throw new Error("服务端返回了不匹配的负责人转派计划");
      }
      setPendingPlan({
        plan_id: response.plan_id,
        worktree_id: row.id,
        status: "pending",
        expires_at: response.expires_at,
        operation: "assign_owner",
        target_owner_user_id: ownerTarget,
      });
    } catch (requestError) {
      setError(apiErrorMessage(requestError));
    } finally {
      operationInFlight.current = false;
      setBusy(false);
    }
  };

  const confirmOwnerPlan = async () => {
    if (!pendingPlan || pendingPlan.operation !== "assign_owner" || planExpired || operationInFlight.current) return;
    operationInFlight.current = true;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await api.confirmManagementPlan(row.id, pendingPlan.plan_id);
      setPendingPlan(null);
      setNotice("Worktree 负责人转派已由服务端确认；列表正在重新读取授权投影。");
      await onRefresh();
    } catch (requestError) {
      setError(apiErrorMessage(requestError));
    } finally {
      operationInFlight.current = false;
      setBusy(false);
    }
  };

  return (
    <div className="card sticky top-16" data-testid="live-worktree-details">
      <div className="mb-3 flex items-center justify-between">
        <div>
          <div className="font-mono text-xs text-ink-mute">{row.id}</div>
          <div className="text-base font-semibold">{row.name}</div>
        </div>
        <StatusPill value={row.status} />
      </div>

      <dl className="mb-4 space-y-1.5 text-xs">
        <Row label="Branch" value={<span className="font-mono text-info">{row.branch}</span>} />
        <Row label="Owner" value={<span className="font-mono">{row.owner_user_id ?? "未分配"}</span>} />
        <Row label="Agent Session" value={<span className="font-mono">{row.agent_session_id ?? "未绑定"}</span>} />
        <Row label="Local Runtime" value={<span className="font-mono">{row.runtime_id ?? "未绑定"}</span>} />
        <Row label="Task Card" value={<span className="font-mono">{row.work_item_id ?? "未关联"}</span>} />
        <Row label="Pull Request" value={safePullRequestUrl(row.pull_request_url) ? <a className="text-info underline" href={safePullRequestUrl(row.pull_request_url) ?? undefined} target="_blank" rel="noreferrer">打开 PR</a> : "—"} />
        <Row label="Machine state" value={row.machine_state} />
        <Row label="Working tree" value={row.dirty ? "Dirty" : "Clean"} />
        <Row label="Ahead / behind" value={`+${row.ahead} / −${row.behind}`} />
        <Row label="Health / risk" value={`${row.health_score} / ${row.risk_count}`} />
        <Row label="Lock" value={`${row.locked ? "已锁定" : "未锁定"} · version ${row.version}`} />
        <Row label="Last activity" value={formatTimestamp(row.last_activity)} />
      </dl>

      <section className="mb-4 rounded border border-line p-3" aria-label="Worktree 负责人管理">
        <h3 className="mb-2 text-xs font-semibold">负责人</h3>
        {membersLoading ? (
          <p className="text-[11px] text-ink-mute">正在读取当前 Project 成员…</p>
        ) : membersError ? (
          <p role="alert" className="text-[11px] text-danger">成员目录读取失败：{membersError}。负责人转派已关闭。</p>
        ) : !canManageOwners ? (
          <p className="text-[11px] text-ink-mute">负责人转派仅对 Project 管理员开放；服务端仍会复核权限。</p>
        ) : members.length === 0 ? (
          <p className="text-[11px] text-ink-mute">当前 Project 没有可分配的已授权成员。</p>
        ) : (
          <>
            <label className="grid gap-1 text-[11px] text-ink-mute">
              新负责人
              <select
                value={ownerTarget}
                onChange={(event) => setOwnerTarget(event.target.value)}
                disabled={busy || pendingPlan !== null}
                className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink"
                data-testid="worktree-owner-target"
              >
                <option value="">选择 Project 成员</option>
                {members.map((member) => (
                  <option key={member.user_id} value={member.user_id}>
                    {member.user_id} · {member.role}
                  </option>
                ))}
              </select>
            </label>
            <button
              className="btn mt-2"
              disabled={busy || pendingPlan !== null || !ownerTarget || ownerTarget === row.owner_user_id || !members.some((member) => member.user_id === ownerTarget)}
              onClick={() => void createOwnerPlan()}
              data-testid="worktree-owner-plan"
            >
              生成负责人转派计划
            </button>
          </>
        )}
      </section>

      <div className="mb-4 rounded border border-line p-2 text-[11px] text-ink-mute">
        归档 / 恢复通过服务端版本校验、权限检查和二次确认。归档只隐藏平台记录，不删除 Git checkout；执行绑定存在时服务端会拒绝归档。
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          className="btn"
          disabled={busy || pendingPlan !== null || (archivedTarget && executionBound)}
          onClick={() => void createArchivePlan()}
        >
          {busy ? "处理中…" : archivedTarget ? "生成归档计划" : "生成恢复计划"}
        </button>
        <Link className="btn-primary" href={`/worktree/${encodeURIComponent(row.id)}/group`} data-testid="open-worktree-group">
          展开 Worktree
        </Link>
        <Link className="btn" href={`/worktree/${encodeURIComponent(row.id)}/group?app=canvas`}>
          打开 Canvas
        </Link>
      </div>

      {archivedTarget && executionBound && (
        <p className="mt-2 text-xs text-warning">当前有 Agent Session 或 Runtime 绑定；需先由 Runtime lifecycle 管理停止执行，再生成归档计划。</p>
      )}
      {pendingPlan && (
        <div className="mt-3 rounded border border-warning/40 bg-warning/5 p-3" data-testid="worktree-management-plan">
          <p className="text-xs">
            {pendingPlan.operation === "assign_owner"
              ? <>计划 {pendingPlan.plan_id} 将把负责人转派给 <span className="font-mono">{pendingPlan.target_owner_user_id}</span>。</>
              : <>计划 {pendingPlan.plan_id} 将{pendingPlan.archived ? "归档" : "恢复"}此 Worktree。</>}
          </p>
          <p className="mt-1 text-[10px] text-ink-mute">有效至 {formatTimestamp(pendingPlan.expires_at)}</p>
          <button
            className="btn-primary mt-2"
            disabled={busy || planExpired}
            onClick={() => void (pendingPlan.operation === "assign_owner" ? confirmOwnerPlan() : confirmArchivePlan())}
          >
            {busy ? "正在确认…" : planExpired ? "计划已过期，请重新生成" : pendingPlan.operation === "assign_owner" ? "确认负责人转派" : `确认${pendingPlan.archived ? "归档" : "恢复"}`}
          </button>
        </div>
      )}
      {error && <p role="alert" className="mt-3 text-xs text-danger">管理操作失败：{error}</p>}
      {notice && <p role="status" className="mt-3 text-xs text-success">{notice}</p>}
    </div>
  );
}
