/*
@cypher schema=1 source_sha256=7afb6b8f911cfa0c5fff57730926bea85e6fe25f037b94a250095e788c25c332
MERGE (self:File {path:"frontend/src/components/run/ProjectRunTree.tsx"})
MERGE (tree:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (live:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::AuthenticatedProjectRunTree",kind:"component"})
MERGE (row:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::DirectoryRow",kind:"component"})
MERGE (label:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::nodeScopeLabel",kind:"function"})
MERGE (worktreeStatus:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::worktreeStatusLabel",kind:"function"})
MERGE (stateLabel:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::stateLabel",kind:"function"})
MERGE (i18n:File {path:"frontend/src/lib/i18n/index.ts"})
MERGE (dictionary:Type {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary",kind:"interface"})
MERGE (worktreeType:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::RunWorktree",kind:"interface"})
MERGE (runHref:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::canonicalRunHref",kind:"function"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (pathHook:ExternalService {name:"next/navigation usePathname"})
MERGE (guardedLink:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (selectProject:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::selectProject",kind:"function"})
MERGE (setSelectedProject:Symbol {id:"frontend/src/lib/nav/navStore.ts::setSelectedProjectId",kind:"action"})
MERGE (setSidebarScope:Symbol {id:"frontend/src/lib/nav/navStore.ts::setSidebarScope",kind:"action"})
MERGE (self)-[:IMPORTS]->(guardedLink)
MERGE (self)-[:DEFINES]->(tree)
MERGE (self)-[:DEFINES]->(live)
MERGE (self)-[:DEFINES]->(row)
MERGE (row)-[:USES]->(guardedLink)
MERGE (self)-[:DEFINES]->(label)
MERGE (self)-[:DEFINES]->(worktreeStatus)
MERGE (self)-[:DEFINES]->(stateLabel)
MERGE (self)-[:IMPORTS]->(i18n)
MERGE (tree)-[:USES_TYPE]->(dictionary)
MERGE (live)-[:USES_TYPE]->(dictionary)
MERGE (self)-[:IMPORTS]->(navStore)
MERGE (self)-[:IMPORTS]->(pathHook)
MERGE (worktreeStatus)-[:USES_TYPE]->(worktreeType)
MERGE (worktreeStatus)-[:USES_TYPE]->(dictionary)
MERGE (row)-[:USES_TYPE]->(dictionary)
MERGE (live)-[:CALLS]->(row)
MERGE (live)-[:CALLS]->(selectProject)
MERGE (live)-[:CALLS]->(navStore)
MERGE (live)-[:CALLS]->(pathHook)
MERGE (row)-[:CALLS]->(label)
MERGE (row)-[:CALLS]->(worktreeStatus)
MERGE (label)-[:CALLS]->(worktreeStatus)
MERGE (worktreeStatus)-[:CALLS]->(stateLabel)
MERGE (row)-[:CALLS]->(runHref)
MERGE (row)-[:CALLS]->(selectProject)
MERGE (selectProject)-[:CALLS]->(setSelectedProject)
MERGE (selectProject)-[:CALLS]->(setSidebarScope)
// P0-lane: 服务端 Lane 层级 + 演示 badge
MERGE (laneHref:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::canonicalLaneHref",kind:"function"})
MERGE (laneWtHref:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::canonicalLaneWorktreeHref",kind:"function"})
MERGE (copy:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::LANE_DIRECTORY_COPY",kind:"constant"})
MERGE (self)-[:DEFINES]->(copy)
MERGE (row)-[:CALLS]->(laneHref)
MERGE (row)-[:CALLS]->(laneWtHref)
@endcypher
*/

/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/ProjectRunTree.tsx",type:"file",language:"tsx"}),
 (tree:Function {name:"ProjectRunTree",type:"function"}),(live:Function {name:"AuthenticatedProjectRunTree",type:"function"}),
 (row:Function {name:"DirectoryRow",type:"function"}),(label:Function {name:"nodeScopeLabel",type:"function"}),
 (worktreeStatus:Function {name:"worktreeStatusLabel",type:"function"}),(stateLabel:Function {name:"stateLabel",type:"function"}),
 (key:Function {name:"moveTreeFocus",type:"function"}),(start:Function {name:"mountDirectoryTree",type:"function"}),(clamp:Function {name:"clampTreeViewport",type:"function"}),
 (model:Class {name:"RunDirectoryTree",type:"class"}),(session:Function {name:"useRunDirectory",type:"function"}),
 (href:Function {name:"canonicalRunWorktreeHref",type:"function"}),(runHref:Function {name:"canonicalRunHref",type:"function"}),
(project:Function {name:"selectProject",type:"function"}),(navStore:Function {name:"useNavStore",type:"function"}),(pathname:Function {name:"usePathname",type:"function"}),
(setProject:Function {name:"setSelectedProjectId",type:"function"}),(setScope:Function {name:"setSidebarScope",type:"function"}),
 (height:Variable {name:"ROW_HEIGHT",type:"variable"}),(viewport:Variable {name:"VIEWPORT_HEIGHT",type:"variable"}),(overscan:Variable {name:"OVERSCAN",type:"variable"}),
 (store:Function {name:"useSyncExternalStore",type:"function"}),(memo:Function {name:"useMemo",type:"function"}),
 (effect:Function {name:"useEffect",type:"function"}),(state:Function {name:"useState",type:"function"}),(ref:Function {name:"useRef",type:"function"}),
 (f)-[:CONTAINS]->(tree),(f)-[:CONTAINS]->(live),(f)-[:CONTAINS]->(row),(f)-[:CONTAINS]->(label),(f)-[:CONTAINS]->(height),(f)-[:CONTAINS]->(viewport),
 (live)-[:CONTAINS]->(key),(live)-[:CONTAINS]->(start),(live)-[:CONTAINS]->(clamp),(f)-[:CONTAINS]->(overscan),(tree)-[:CALLS]->(session),(tree)-[:CALLS]->(live),
 (live)-[:CALLS]->(session),(live)-[:CALLS]->(model),(live)-[:CALLS]->(store),(live)-[:CALLS]->(memo),(live)-[:CALLS]->(effect),
 (live)-[:CALLS]->(state),(live)-[:CALLS]->(ref),(live)-[:CALLS]->(row),(row)-[:CALLS]->(label),(row)-[:CALLS]->(worktreeStatus),(row)-[:CALLS]->(href),
 (live)-[:CALLS]->(project),(live)-[:CALLS]->(navStore),(live)-[:CALLS]->(pathname),(row)-[:CALLS]->(project),
 (project)-[:CALLS]->(setProject),(project)-[:CALLS]->(setScope),
 (label)-[:CALLS]->(worktreeStatus),(worktreeStatus)-[:CALLS]->(stateLabel),
 (live)-[:USES]->(overscan),(live)-[:USES]->(height),(live)-[:USES]->(viewport),(row)-[:USES]->(height),(key)-[:USES]->(height),(key)-[:USES]->(viewport),(clamp)-[:USES]->(height),(clamp)-[:USES]->(viewport);
*/

"use client";

import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { usePathname } from "next/navigation";
import { ChevronDown, ChevronRight, FolderKanban, GitBranch, Layers, Monitor, Plus, RefreshCw } from "lucide-react";
import { useEffect, useMemo, useRef, useState, useSyncExternalStore, type FormEvent, type KeyboardEvent } from "react";
import { canonicalRunHref, canonicalRunWorktreeHref, canonicalLaneHref, canonicalLaneWorktreeHref, type RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { useNavStore } from "@/lib/nav/navStore";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import { useFallbackRunDirectory } from "@/lib/run/useFallbackRunDirectory";
import { directoryErrorMessage, RunDirectoryTree, type DirectoryNode, type DirectoryTreeRow } from "@/lib/run/runDirectoryTree";
import { useTranslation, type Dictionary } from "@/lib/i18n";
import { useStore } from "@/lib/store";
import type { Worktree } from "@/types/ids";

const ROW_HEIGHT = 36;
const VIEWPORT_HEIGHT = 288;
const OVERSCAN = 3;

// =====================================================================
// P0-lane 局部三语文案 (共享 dictionary 不碰; 新增 i18n key 清单见报告父代理待办)
// =====================================================================
const LANE_DIRECTORY_COPY = {
  en: {
    demoBadge: "Demo directory",
    demoBadgeTitle: "Demo data injected for the web fallback; real mode never mixes this data",
    newLane: "New Lane",
    newLaneTitle: "Create Lane",
    laneName: "Lane name",
    laneNamePlaceholder: "e.g. core-runtime",
    laneHostIndependentHint: "No remote repository or branch is required to create a Lane.",
    invalidLaneName: "Lane name must be 1-128 characters.",
    creatingLane: "Creating…",
    createLaneButton: "Create",
    laneChildren: "Worktrees in this Lane",
    selectLane: "Select Lane",
    laneUnboundRemote: "no remote bound",
    laneRunUnbound: "no Run bound (honest)",
    laneBindingVersion: "Lane binding v{version}",
    checkoutStatusUnavailable: "local checkout status not provided by server",
    demoCheckoutStatus: "demo checkout",
  },
  "zh-CN": {
    demoBadge: "演示目录",
    demoBadgeTitle: "网页版降级注入的演示数据; 真实模式绝不混入",
    newLane: "新建 Lane",
    newLaneTitle: "创建 Lane",
    laneName: "Lane 名称",
    laneNamePlaceholder: "如 core-runtime",
    laneHostIndependentHint: "无需远端仓库或分支关联即可创建 Lane。",
    invalidLaneName: "Lane 名称为 1-128 字符。",
    creatingLane: "创建中…",
    createLaneButton: "创建",
    laneChildren: "该 Lane 下的 Worktree",
    selectLane: "选择 Lane",
    laneUnboundRemote: "未绑定远端",
    laneRunUnbound: "未挂 Run (诚实标注)",
    laneBindingVersion: "Lane 绑定 v{version}",
    checkoutStatusUnavailable: "本地 checkout 状态: 服务端未提供",
    demoCheckoutStatus: "演示 checkout",
  },
  ja: {
    demoBadge: "デモディレクトリ",
    demoBadgeTitle: "Web 版フォールバック注入のデモデータ; リアルモードでは混入しない",
    newLane: "新規 Lane",
    newLaneTitle: "Lane を作成",
    laneName: "Lane 名",
    laneNamePlaceholder: "例: core-runtime",
    laneHostIndependentHint: "リモートリポジトリやブランチなしで Lane を作成できます。",
    invalidLaneName: "Lane 名は 1-128 文字です。",
    creatingLane: "作成中…",
    createLaneButton: "作成",
    laneChildren: "この Lane の Worktree",
    selectLane: "Lane を選択",
    laneUnboundRemote: "リモート未バインド",
    laneRunUnbound: "Run 未バインド (正直な表記)",
    laneBindingVersion: "Lane バインド v{version}",
    checkoutStatusUnavailable: "ローカル checkout 状態: サーバー未提供",
    demoCheckoutStatus: "デモ checkout",
  },
} as const;
type LaneCopy = (typeof LANE_DIRECTORY_COPY)["zh-CN"];

export function ProjectRunTree({ collapsed }: { collapsed: boolean }) {
  const session = useRunDirectory();
  const { t } = useTranslation();
  if (collapsed) return <div className="flex justify-center py-2" title={t.sidebar.openProjectDirectory}><FolderKanban size={20} aria-label={t.sidebar.projectNavigation} /></div>;
  return <section aria-label={t.sidebar.projectDirectory} data-testid="project-run-directory" className="border-b border-line pb-3">
    <div className="px-2 py-1 text-xs font-bold">{t.sidebar.projectNavigation}</div>
    <p className="px-2 pb-2 text-[10px] text-ink-mute">{t.runDirectory.hierarchyHint}</p>
    {session.status === "ready" && session.client ? <AuthenticatedProjectRunTree client={session.client} /> :
      <FallbackProjectRunTree />}
  </section>;
}

// =====================================================================
// FallbackProjectRunTree — 宿主会话未装配时的可达降级目录
// (per 2026-11-14 用户发令 "确保导航可达": web 版无宿主时不再显示
//  "暂不可用"死块, 而是用 seed 数据渲染可点击的
//  Project → LANE → Worktree → Branch 层级, 全部可达 /worktree)
// =====================================================================
function FallbackProjectRunTree() {
  const { t, language } = useTranslation();
  const copy = LANE_DIRECTORY_COPY[language];
  // 网页版降级目录引导: store 为空时从独立演示模块注入层级
  // (per 2026-11-19: Project 导航必须渲染树形结构, 不能只剩提示文字;
  //  per P0-lane 2026-11-20: 演示数据独立存放并明确「演示目录」badge 标注)
  useFallbackRunDirectory();
  const projects = useStore((s) => s.projects);
  const worktrees = useStore((s) => s.worktrees);
  const pathname = usePathname() ?? "/";
  const selectedProjectId = useNavStore((s) => s.selectedProjectId);
  const setSelectedProjectId = useNavStore((s) => s.setSelectedProjectId);
  const setSidebarScope = useNavStore((s) => s.setSidebarScope);
  const selectedLaneId = useNavStore((s) => s.selectedLaneId);
  const selectedWorktreeId = useNavStore((s) => s.selectedWorktreeId);
  const setLaneContext = useNavStore((s) => s.setLaneContext);
  const lanes = useStore((s) => s.lanes);
  const [expandedProjects, setExpandedProjects] = useState<Record<string, boolean>>({});
  const [expandedWorktrees, setExpandedWorktrees] = useState<Record<string, boolean>>({});

  const byProject = useMemo(() => {
    const map = new Map<string, Worktree[]>();
    for (const w of worktrees) {
      const list = map.get(w.project_id) ?? [];
      list.push(w);
      map.set(w.project_id, list);
    }
    return map;
  }, [worktrees]);

  function selectProject(projectId: string) {
    setSelectedProjectId(projectId);
    setSidebarScope("project");
  }

  /** Lane 点选 (per 2026-11-15: Lane 是管理层级, 可单独选中; Worktree/Branch 随 Lane 清空) */
  function selectLane(laneId: string) {
    setLaneContext(laneId, "", "");
  }
  /** Worktree 点选: Lane + Worktree + Branch 一键进入统一授权上下文 */
  function selectWorktree(laneId: string, worktreeId: string, branch: string) {
    setLaneContext(laneId, worktreeId, branch);
  }
  const toggleProject = (id: string) =>
    setExpandedProjects((prev) => ({ ...prev, [id]: !prev[id] }));
  const toggleWorktree = (id: string) =>
    setExpandedWorktrees((prev) => ({ ...prev, [id]: !prev[id] }));

  return <div data-testid="project-run-directory-fallback">
    <p role="status" className="px-2 pb-1 text-[10px] text-ink-mute">
      <span data-testid="demo-directory-badge" title={copy.demoBadgeTitle}
        className="mr-1 rounded border border-warn/40 bg-warn/10 px-1 text-[9px] text-warn">{copy.demoBadge}</span>
      {t.runDirectory.fallbackHint}
    </p>
    {projects.length === 0 && (
      <p data-testid="project-run-directory-empty" className="px-2 pb-1 text-[10px] text-warn">
        {t.runDirectory.emptyHint}
      </p>
    )}
    <ul role="tree" aria-label={t.sidebar.projectDirectory} className="max-h-[288px] overflow-y-auto">
      {projects.map((project) => {
        const laneWorktrees = byProject.get(project.id) ?? [];
        const expanded = Boolean(expandedProjects[project.id]);
        const selected = selectedProjectId === project.id;
        return (
          <li key={project.id}>
            <div className="flex items-center gap-1 px-1">
              <button type="button" aria-expanded={expanded} tabIndex={-1} className="flex h-5 w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10"
                onClick={() => toggleProject(project.id)}>
                {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
              </button>
              <Link role="treeitem" aria-level={1} aria-expanded={expanded} aria-selected={selected}
                href={`/worktree?project_id=${encodeURIComponent(project.id)}`} prefetch={false}
                onNavigate={() => selectProject(project.id)}
                className={`flex min-w-0 flex-1 items-center gap-1 rounded py-0.5 pr-1 text-left hover:bg-accent/10 ${selected ? "bg-accent/15 text-accent" : ""}`}>
                <FolderKanban size={13} className="shrink-0" />
                <span className="min-w-0">
                  <span className="block truncate text-xs">{project.name}</span>
                  <span className="block truncate text-[9px] text-ink-mute">Project · {project.key}</span>
                </span>
              </Link>
            </div>
            {expanded && (
              <ul className="ml-3 border-l border-line/60 pl-1">
                {lanes.filter((l) => l.project_id === project.id).map((lane) => {
                  const laneSel = selectedLaneId === lane.id;
                  const laneWts = laneWorktrees.filter((wt) => wt.lane_id === lane.id);
                  return (
                    <li key={lane.id} aria-level={2}>
                      <button
                        type="button"
                        role="treeitem"
                        aria-selected={laneSel}
                        onClick={() => selectLane(lane.id)}
                        data-testid={`project-run-lane-${lane.id}`}
                        className={`flex w-full items-center gap-1 rounded py-0.5 pl-3 pr-1 text-left hover:bg-accent/10 ${laneSel ? "bg-accent/15 text-accent" : "text-ink-mute"}`}
                      >
                        <Layers size={11} className="shrink-0" />
                        <span className="truncate text-[10px]">{lane.name}</span>
                        {!lane.git_remote_url && (
                          <span title="未绑定远端" className="ml-auto shrink-0 rounded border border-warn/40 bg-warn/10 px-1 text-[8px] text-warn">未绑定</span>
                        )}
                      </button>
                      {laneWts.map((wt) => (
                        <li key={wt.id} className="ml-2">
                          <div className="flex items-center gap-1">
                            <button type="button" aria-expanded={Boolean(expandedWorktrees[wt.id])} tabIndex={-1} className="flex h-5 w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10"
                              onClick={() => toggleWorktree(wt.id)}>
                              {expandedWorktrees[wt.id] ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                            </button>
                            <Link role="treeitem" aria-level={3} aria-selected={selectedWorktreeId === wt.id}
                              href={`/worktree?project_id=${encodeURIComponent(project.id)}&lane_id=${encodeURIComponent(lane.id)}&worktree_id=${encodeURIComponent(wt.id)}&branch=${encodeURIComponent(wt.branch)}`}
                              prefetch={false}
                              onNavigate={() => selectWorktree(lane.id, wt.id, wt.branch)}
                              className={`flex min-w-0 flex-1 items-center gap-1 rounded py-0.5 pr-1 text-left hover:bg-accent/10 ${selectedWorktreeId === wt.id ? "bg-accent/15 text-accent" : ""}`}>
                              <GitBranch size={11} className="shrink-0" />
                              <span className="min-w-0">
                                <span className="block truncate text-[11px]">{wt.name}</span>
                                <span className="block truncate text-[9px] text-ink-mute">Worktree · {wt.branch}</span>
                                <span className="block truncate text-[9px] text-ink-mute">{copy.demoCheckoutStatus}: {wt.status}</span>
                              </span>
                            </Link>
                          </div>
                        </li>
                      ))}
                    </li>
                  );
                })}
                {(() => {
                  const uncat = laneWorktrees.filter((wt) => wt.lane_id === undefined);
                  if (uncat.length === 0) return null;
                  return (
                    <li key="uncat" aria-level={2}>
                      <span className="flex w-full items-center gap-1 rounded py-0.5 pl-3 pr-1 text-left text-[10px] text-ink-mute">
                        <Layers size={11} className="shrink-0" />
                        <span className="truncate">{t.runDirectory.fallbackLane}</span>
                        <span className="ml-auto font-mono">{uncat.length}</span>
                      </span>
                      {uncat.map((wt) => (
                        <li key={wt.id} className="ml-2">
                          <div className="flex items-center gap-1">
                            <button type="button" aria-expanded={Boolean(expandedWorktrees[wt.id])} tabIndex={-1} className="flex h-5 w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10"
                              onClick={() => toggleWorktree(wt.id)}>
                              {expandedWorktrees[wt.id] ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                            </button>
                            <Link role="treeitem" aria-level={3} aria-selected={selectedWorktreeId === wt.id}
                              href={`/worktree?project_id=${encodeURIComponent(project.id)}&worktree_id=${encodeURIComponent(wt.id)}&branch=${encodeURIComponent(wt.branch)}`}
                              prefetch={false}
                              onNavigate={() => selectWorktree("", wt.id, wt.branch)}
                              className={`flex min-w-0 flex-1 items-center gap-1 rounded py-0.5 pr-1 text-left hover:bg-accent/10 ${selectedWorktreeId === wt.id ? "bg-accent/15 text-accent" : ""}`}>
                              <GitBranch size={11} className="shrink-0" />
                              <span className="min-w-0">
                                <span className="block truncate text-[11px]">{wt.name}</span>
                                <span className="block truncate text-[9px] text-ink-mute">Worktree · {wt.branch}</span>
                                <span className="block truncate text-[9px] text-ink-mute">{copy.demoCheckoutStatus}: {wt.status}</span>
                              </span>
                            </Link>
                          </div>
                        </li>
                      ))}
                    </li>
                  );
                })()}
                {lanes.filter((l) => l.project_id === project.id).length === 0 && laneWorktrees.filter((wt) => wt.lane_id === undefined).length === 0 && (
                  <li className="flex items-center gap-1 py-0.5 pl-3 text-[10px] text-ink-mute" aria-level={2}>
                    <Layers size={11} className="shrink-0" />
                    <span className="truncate">{t.runDirectory.fallbackLane}</span>
                  </li>
                )}
</ul>
            )}
          </li>
        );
      })}
    </ul>
    <p className="px-2 pt-1 text-[10px] text-ink-mute">{t.sidebar.projectDirectoryHint}</p>
  </div>;
}

function AuthenticatedProjectRunTree({ client }: { client: RunDirectoryApiClient }) {
  const { focus, clearFocus } = useRunDirectory();
  const { t, language } = useTranslation();
  const copy = LANE_DIRECTORY_COPY[language];
  const pathname = usePathname() ?? "/";
  const selectedProjectId = useNavStore((state) => state.selectedProjectId);
  const setSelectedProjectId = useNavStore((state) => state.setSelectedProjectId);
  const setSidebarScope = useNavStore((state) => state.setSidebarScope);
  const selectedLaneId = useNavStore((state) => state.selectedLaneId);
  const selectedWorktreeId = useNavStore((state) => state.selectedWorktreeId);
  const setLaneContext = useNavStore((state) => state.setLaneContext);
  const lanes = useStore((s) => s.lanes);
  const model = useMemo(() => new RunDirectoryTree(client), [client]);
  const rows = useSyncExternalStore(model.subscribe, model.getSnapshot, model.getSnapshot);
  const viewport = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [createTarget, setCreateTarget] = useState<{ project: ProjectNode; wasExpanded: boolean } | null>(null);
  const [createTitle, setCreateTitle] = useState("");
  const [createBusy, setCreateBusy] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);
  const pendingCreate = useRef<{ projectId: string; title: string; idempotencyKey: string; correlationId: string } | null>(null);
  useEffect(function mountDirectoryTree() { setScrollTop(0); model.start(); return () => model.dispose(); }, [model]);
  useEffect(function clampTreeViewport() {
    const element = viewport.current;
    if (!element) return;
    const maximum = Math.max(0, rows.length * ROW_HEIGHT - VIEWPORT_HEIGHT);
    if (element.scrollTop > maximum) { element.scrollTop = maximum; setScrollTop(maximum); }
  }, [rows.length]);
  const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const visible = rows.slice(start, start + Math.ceil(VIEWPORT_HEIGHT / ROW_HEIGHT) + 2 * OVERSCAN);
  function selectProject(projectId: string) {
    setSelectedProjectId(projectId);
    setSidebarScope("project");
  }

  /** Lane 点选 (per 2026-11-15: Lane 是管理层级, 可单独选中; Worktree/Branch 随 Lane 清空) */
  function selectLane(laneId: string) {
    setLaneContext(laneId, "", "");
  }
  /** Worktree 点选: Lane + Worktree + Branch 一键进入统一授权上下文 */
  function selectWorktree(laneId: string, worktreeId: string, branch: string) {
    setLaneContext(laneId, worktreeId, branch);
  }
  function moveTreeFocus(event: KeyboardEvent<HTMLDivElement>) {
    const target = (event.target as HTMLElement).closest<HTMLElement>("[data-directory-index]");
    if (!target || !["ArrowDown", "ArrowUp", "ArrowRight", "ArrowLeft", "Home", "End"].includes(event.key)) return;
    const current = Number(target.dataset.directoryIndex);
    const row = rows[current];
    if (!row) return;
    event.preventDefault();
    let next = current;
    if (event.key === "ArrowDown") next += 1;
    else if (event.key === "ArrowUp") next -= 1;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = rows.length - 1;
    else if (event.key === "ArrowRight" && row.kind === "node" && row.node.kind !== "branch") {
      if (!row.expanded) { model.toggle(row.node); return; }
      next += 1;
    } else if (event.key === "ArrowLeft" && row.kind === "node") {
      if (row.expanded && row.node.kind !== "branch") { model.toggle(row.node); return; }
      const parentIndex = rows.findIndex((candidate) => candidate.key === row.node.parentKey);
      if (parentIndex >= 0) next = parentIndex;
    }
    next = Math.max(0, Math.min(rows.length - 1, next));
    const element = viewport.current;
    if (!element) return;
    if (next * ROW_HEIGHT < element.scrollTop) element.scrollTop = next * ROW_HEIGHT;
    else if ((next + 1) * ROW_HEIGHT > element.scrollTop + VIEWPORT_HEIGHT) element.scrollTop = (next + 1) * ROW_HEIGHT - VIEWPORT_HEIGHT;
    setScrollTop(element.scrollTop);
    requestAnimationFrame(() => element.querySelector<HTMLElement>(`[data-directory-index="${next}"]`)?.focus());
  }
  function openCreateLane(project: ProjectNode, wasExpanded: boolean) {
    const prior = pendingCreate.current;
    const retry = prior?.projectId === project.entity.project_id ? prior : null;
    setCreateTarget({ project, wasExpanded });
    setCreateTitle(retry?.title ?? "");
    setCreateError(retry ? t.runDirectory.unknownCreateResult : null);
  }
  async function submitCreateLane(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!createTarget || createBusy) return;
    const title = createTitle.trim();
    if (!title || [...title].length > 128) {
      setCreateError(copy.invalidLaneName);
      return;
    }
    setCreateBusy(true);
    setCreateError(null);
    try {
      const projectId = createTarget.project.entity.project_id;
      const previous = pendingCreate.current;
      const command = previous?.projectId === projectId && previous.title === title
        ? previous
        : { projectId, title, idempotencyKey: globalThis.crypto.randomUUID(), correlationId: globalThis.crypto.randomUUID() };
      pendingCreate.current = command;
      // P0-lane: 创建服务端 Lane 实体 (无远端仓库/分支要求)
      await client.createLane(projectId, title, command.idempotencyKey, command.correlationId);
      pendingCreate.current = null;
      if (createTarget.wasExpanded) model.refresh(createTarget.project);
      else model.toggle(createTarget.project);
      setCreateTarget(null);
    } catch (error: unknown) {
      setCreateError(directoryErrorMessage(error));
    } finally {
      setCreateBusy(false);
    }
  }
  return <>
    <button type="button" onClick={() => { model.refresh(); if (viewport.current) viewport.current.scrollTop = 0; setScrollTop(0); }} className="mb-1 flex items-center gap-1 px-2 text-[10px] text-ink-mute hover:text-ink"><RefreshCw size={11} />{t.runDirectory.refreshDirectory}</button>
    <div ref={viewport} role="tree" aria-label={t.sidebar.projectDirectory} onKeyDown={moveTreeFocus}
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)} className="relative overflow-y-auto overflow-x-hidden" style={{ height: VIEWPORT_HEIGHT }}>
      <div role="presentation" style={{ height: rows.length * ROW_HEIGHT }}>
        {visible.map((row, offset) => <DirectoryRow key={row.key} row={row} index={start + offset} model={model}
          selectedWorktreeId={focus?.focus?.worktree_id} clearFocus={clearFocus} onCreateLane={openCreateLane}
          selectedProjectId={selectedProjectId} selectedLaneId={selectedLaneId} pathname={pathname}
          onSelectProject={selectProject} onSelectLane={selectLane}
          newLaneLabel={copy.newLane} laneCopy={copy} statusLabels={t.runDirectory} />)}
      </div>
    </div>
    <p className="px-2 pt-1 text-[10px] text-ink-mute">{t.sidebar.projectDirectoryHint}</p>
    {createTarget && <div role="dialog" aria-modal="true" aria-labelledby="create-lane-title" className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <form onSubmit={submitCreateLane} className="w-full max-w-md rounded-lg border border-line bg-bg p-5 shadow-xl">
        <h2 id="create-lane-title" className="text-base font-semibold">{copy.newLaneTitle}</h2>
        <p className="mt-1 break-all text-xs text-ink-mute">{t.runDirectory.projectIdLabel} {createTarget.project.entity.project_id} · {copy.laneHostIndependentHint}</p>
        <label className="mt-4 grid gap-1 text-sm">{copy.laneName}
          <input value={createTitle} onChange={(event) => setCreateTitle(event.target.value)} autoFocus required maxLength={128}
            placeholder={copy.laneNamePlaceholder} className="rounded border border-line bg-bg-soft px-3 py-2" />
        </label>
        {createError && <p role="alert" className="mt-3 text-sm text-danger">{createError}</p>}
        <div className="mt-4 flex justify-end gap-2">
          <button type="button" disabled={createBusy} onClick={() => setCreateTarget(null)} className="rounded border border-line px-3 py-2 text-sm disabled:opacity-50">{t.runDirectory.cancel}</button>
          <button type="submit" disabled={createBusy || !createTitle.trim()} className="rounded bg-accent px-3 py-2 text-sm text-white disabled:opacity-50">{createBusy ? copy.creatingLane : copy.createLaneButton}</button>
        </div>
      </form>
    </div>}
  </>;
}

type ProjectNode = Extract<DirectoryNode, { kind: "project" }>;

type RunDirectoryLabels = Dictionary["runDirectory"];

function stateLabel(value: string, labels: object, unknownStateLabel: string): string {
  const normalized = value.trim().toLowerCase();
  const known = (labels as Record<string, string>)[normalized];
  if (known) return known;
  const safe = value.replace(/[\u0000-\u001f\u007f]/g, " ").trim().slice(0, 32);
  return safe ? `${unknownStateLabel}: ${safe}` : unknownStateLabel;
}

function worktreeStatusLabel(worktree: Extract<DirectoryNode, { kind: "worktree" }>['entity'], labels: RunDirectoryLabels) {
  const humanState = stateLabel(worktree.human_state, labels.humanStates, labels.unknownState);
  const machineState = stateLabel(worktree.machine_state, labels.machineStates, labels.unknownState);
  const summary = [
    `${labels.humanState}: ${humanState}`,
    `${labels.machineState}: ${machineState}`,
    worktree.agent_id !== null ? labels.agentAssigned : labels.agentUnassigned,
    worktree.agent_session_id !== null ? labels.sessionBound : labels.sessionUnbound,
    worktree.archived ? labels.archived : labels.notArchived,
  ].join(" · ");
  const details = [
    worktree.runtime_id !== null ? labels.runtimeBound : labels.runtimeUnbound,
    labels.runBindingVersion.replace("{version}", String(worktree.binding_version)),
    labels.projectBindingVersion.replace("{version}", String(worktree.project_binding_version)),
    labels.lockUnknown,
    labels.dirtyDetailsUnknown,
    labels.observedAtUnknown,
    labels.branchFreshnessUnknown,
  ].join(" · ");
  return { summary, details };
}

/** Lane 维度 Worktree 状态标签 (P0-lane): 无 run 绑定时诚实标注「未挂 Run」 */
function laneWorktreeStatusLabel(worktree: Extract<DirectoryNode, { kind: "laneWorktree" }>['entity'], labels: RunDirectoryLabels, copy: LaneCopy) {
  const humanState = stateLabel(worktree.human_state, labels.humanStates, labels.unknownState);
  const machineState = stateLabel(worktree.machine_state, labels.machineStates, labels.unknownState);
  const summary = [
    `${labels.humanState}: ${humanState}`,
    `${labels.machineState}: ${machineState}`,
    worktree.agent_id !== null ? labels.agentAssigned : labels.agentUnassigned,
    worktree.agent_session_id !== null ? labels.sessionBound : labels.sessionUnbound,
    worktree.archived ? labels.archived : labels.notArchived,
  ].join(" · ");
  const details = [
    worktree.runtime_id !== null ? labels.runtimeBound : labels.runtimeUnbound,
    copy.laneBindingVersion.replace("{version}", String(worktree.binding_version)),
    worktree.engineering_run_id === null ? copy.laneRunUnbound : `Run ${worktree.engineering_run_id}`,
    labels.lockUnknown,
    labels.dirtyDetailsUnknown,
    labels.observedAtUnknown,
    labels.branchFreshnessUnknown,
  ].join(" · ");
  return { summary, details };
}

function nodeScopeLabel(node: DirectoryNode, labels: RunDirectoryLabels, copy: LaneCopy): string {
  switch (node.kind) {
    case "project": return `Project · ${node.entity.role}`;
    // P0-lane: 服务端 Lane 实体 (对外统一称 Lane; 版本 + 远端绑定诚实标注)
    case "lane": return `LANE · v${node.entity.lock_version}${node.entity.repository_binding === null ? ` · ${copy.laneUnboundRemote}` : ""}`;
    case "laneWorktree": return `${labels.localWorktree} · ${laneWorktreeStatusLabel(node.entity, labels, copy).summary}`;
    case "run": return `LANE · ${node.entity.state}`;
    case "worktree": return `${labels.localWorktree} · ${worktreeStatusLabel(node.entity, labels).summary}`;
    case "branch": return node.entity.name
      ? `${labels.branchRecord} · ${node.entity.checkoutStatus ?? copy.checkoutStatusUnavailable}`
      : labels.branchUnavailable;
  }
}
function DirectoryRow({ row, index, model, selectedWorktreeId, clearFocus, selectedProjectId, selectedLaneId, pathname, onSelectProject, onSelectLane, onCreateLane, newLaneLabel, laneCopy, statusLabels }: {
  row: DirectoryTreeRow; index: number; model: RunDirectoryTree; selectedWorktreeId?: string; clearFocus: () => void;
  selectedProjectId: string; selectedLaneId: string; pathname: string; onSelectProject: (projectId: string) => void;
  onSelectLane: (laneId: string) => void; onCreateLane: (project: ProjectNode, expanded: boolean) => void;
  newLaneLabel: string; laneCopy: LaneCopy; statusLabels: RunDirectoryLabels;
}) {
  const depth = row.kind === "node" ? row.node.depth : row.depth;
  const style = { position: "absolute" as const, top: index * ROW_HEIGHT, height: ROW_HEIGHT, left: 0, right: 0, paddingLeft: 4 + depth * 12 };
  if (row.kind === "hint") {
    const actionable = row.state !== "loading";
    return <button role="treeitem" aria-level={depth + 1} type="button" data-directory-index={index} style={style}
      aria-disabled={!actionable} onClick={() => {
        if (row.state === "budget") model.nextWindow(row.parent);
        else if (["empty", "refresh"].includes(row.state)) model.refresh(row.parent);
        else if (actionable) model.loadMore(row.parent);
      }}
      title={row.message} className="w-full truncate pr-1 text-left text-[10px] text-ink-mute focus:outline focus:outline-1 focus:outline-accent">
      {row.message}{row.state === "error" && ` ${statusLabels.retryAction}`}
    </button>;
  }
  const { node, expanded } = row;
  const Icon = { project: FolderKanban, branch: GitBranch, lane: Layers, laneWorktree: Monitor, run: Layers, worktree: Monitor }[node.kind];
  const scopeLabel = nodeScopeLabel(node, statusLabels, laneCopy);
  const contents = <><span className="flex shrink-0 items-center"><Icon size={13} /></span>
    <span className="min-w-0"><span className="block truncate text-xs">{node.label}</span><span className="block truncate text-[9px] text-ink-mute">{scopeLabel}</span></span></>;
  const className = "flex w-full min-w-0 items-center gap-1 rounded pr-1 text-left hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent";
  if (node.kind === "project") {
    const selected = selectedProjectId === node.entity.project_id;
    const projectHref = `/worktree?project_id=${encodeURIComponent(node.entity.project_id)}`;
    return <div role="presentation" style={style} className="flex items-center gap-1">
      <button type="button" tabIndex={-1} aria-label={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${node.label}`}
        aria-expanded={expanded} title={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${statusLabels.projectChildren}`}
        className="flex h-full w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent"
        onClick={() => model.toggle(node)}>{expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}</button>
      <Link role="treeitem" aria-level={depth + 1} aria-expanded={expanded} aria-selected={selected}
        aria-current={selected && pathname === "/worktree" ? "page" : undefined} data-directory-index={index} href={projectHref} prefetch={false}
        aria-label={`${statusLabels.selectNode} ${node.label}`} title={node.label} className={`${className} min-w-0 flex-1 ${selected ? "bg-accent/15 text-accent" : ""}`}
        onNavigate={() => { onSelectProject(node.entity.project_id); clearFocus(); }}>{contents}</Link>
      <button type="button" aria-label={`${node.label}: ${newLaneLabel}`} title={newLaneLabel}
        className="flex h-7 w-7 shrink-0 items-center justify-center rounded text-ink-mute hover:bg-accent/10 hover:text-accent focus:outline focus:outline-1 focus:outline-accent"
        onClick={() => onCreateLane(node, expanded)}><Plus size={14} /></button>
    </div>;
  }
  // P0-lane: Lane 节点 — 服务端实体, 点选即选中 Lane (Worktree/Branch 随 Lane 清空)
  if (node.kind === "lane") {
    const href = canonicalLaneHref(node.entity.project_id, node.entity.lane_id);
    const selected = selectedLaneId === node.entity.lane_id;
    return <div role="presentation" style={style} className="flex items-center gap-1">
      <button type="button" tabIndex={-1} aria-label={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${node.label}`}
        aria-expanded={expanded} title={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${laneCopy.laneChildren}`}
        className="flex h-full w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent"
        onClick={() => model.toggle(node)}>{expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}</button>
      <Link role="treeitem" aria-level={depth + 1} aria-expanded={expanded} aria-selected={selected}
        aria-current={selected ? "page" : undefined} data-directory-index={index} href={href} prefetch={false}
        aria-label={`${laneCopy.selectLane} ${node.label}`} title={scopeLabel}
        className={`${className} min-w-0 flex-1 ${selected ? "bg-accent/15 text-accent" : ""}`}
        onNavigate={() => { onSelectProject(node.entity.project_id); onSelectLane(node.entity.lane_id); clearFocus(); }}>{contents}</Link>
    </div>;
  }
  // P0-lane: Lane Worktree 节点 — 选择 Worktree 后打开该 Lane 的 App 页签 (深链恢复复用 canonicalLaneWorktreeHref)
  if (node.kind === "laneWorktree") {
    const selected = selectedWorktreeId === node.entity.worktree_id;
    const status = laneWorktreeStatusLabel(node.entity, statusLabels, laneCopy);
    const statusDescription = `${node.label}. ${status.summary}. ${status.details}`;
    return <div role="presentation" style={style} className="flex items-center">
      <button type="button" tabIndex={-1} aria-label={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${node.label}`}
        aria-expanded={expanded} title={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${statusLabels.worktreeBranches}`}
        className="flex h-full w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent"
        onClick={() => model.toggle(node)}>
        {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
      </button>
      <Link role="treeitem" aria-level={depth + 1} aria-expanded={expanded} aria-selected={selected} aria-current={selected ? "page" : undefined}
        aria-label={statusDescription} data-directory-index={index} title={statusDescription}
        className={`${className} w-auto flex-1 ${selected ? "bg-accent/15 text-accent" : ""}`} prefetch={false}
        onNavigate={() => {
          if (!selected) clearFocus();
        }}
        href={canonicalLaneWorktreeHref(node.entity.project_id, node.entity.lane_id, node.entity.worktree_id)}>
        <span className="flex shrink-0 items-center"><Monitor size={13} /></span>
        <span className="min-w-0"><span className="block truncate text-xs">{node.label}</span><span className="block truncate text-[9px] text-ink-mute">{scopeLabel}</span></span>
        <span aria-hidden="true" title={status.details} className="mr-1 shrink-0 rounded border border-line px-1 text-[9px] leading-4 text-ink-mute">?</span>
      </Link>
    </div>;
  }
  if (node.kind === "worktree") {
    const selected = selectedWorktreeId === node.entity.worktree_id;
    const status = worktreeStatusLabel(node.entity, statusLabels);
    const statusDescription = `${node.label}. ${status.summary}. ${status.details}`;
    return <div role="presentation" style={style} className="flex items-center">
      <button type="button" tabIndex={-1} aria-label={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${node.label}`}
        aria-expanded={expanded} title={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${statusLabels.worktreeBranches}`}
        className="flex h-full w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent"
        onClick={() => model.toggle(node)}>
        {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
      </button>
      <Link role="treeitem" aria-level={depth + 1} aria-expanded={expanded} aria-selected={selected} aria-current={selected ? "page" : undefined}
        aria-label={statusDescription} data-directory-index={index} title={statusDescription}
        className={`${className} w-auto flex-1 ${selected ? "bg-accent/15 text-accent" : ""}`} prefetch={false}
        onNavigate={() => {
          if (!selected) clearFocus();
        }}
        href={canonicalRunWorktreeHref(node.entity.project_id, node.entity.engineering_run_id, node.entity.worktree_id)}>
        <span className="flex shrink-0 items-center"><Monitor size={13} /></span>
        <span className="min-w-0"><span className="block truncate text-xs">{node.label}</span><span className="block truncate text-[9px] text-ink-mute">{scopeLabel}</span></span>
        <span aria-hidden="true" title={status.details} className="mr-1 shrink-0 rounded border border-line px-1 text-[9px] leading-4 text-ink-mute">?</span>
      </Link>
    </div>;
  }
  if (node.kind === "run") {
    const href = canonicalRunHref(node.entity.project_id, node.entity.engineering_run_id);
    const selected = pathname === href;
    return <div role="presentation" style={style} className="flex items-center gap-1">
      <button type="button" tabIndex={-1} aria-label={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${node.label}`}
        aria-expanded={expanded} title={`${expanded ? statusLabels.collapseNode : statusLabels.expandNode} ${statusLabels.runChildren}`}
        className="flex h-full w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent"
        onClick={() => model.toggle(node)}>{expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}</button>
      <Link role="treeitem" aria-level={depth + 1} aria-expanded={expanded} aria-selected={selected}
        aria-current={selected ? "page" : undefined} data-directory-index={index} href={href} prefetch={false}
        aria-label={`${statusLabels.openRunOverview} ${node.label}`} title={statusLabels.runOverviewFocusHint}
        className={`${className} min-w-0 flex-1 ${selected ? "bg-accent/15 text-accent" : ""}`}
        onNavigate={() => { onSelectProject(node.entity.project_id); clearFocus(); }}>{contents}</Link>
    </div>;
  }
  if (node.kind === "branch") return <div role="treeitem" aria-level={depth + 1} tabIndex={-1}
    data-directory-index={index} style={style}
    title={node.entity.name ? `${statusLabels.branchRecord} · ${node.entity.checkoutStatus ?? laneCopy.checkoutStatusUnavailable}: ${node.entity.name}` : statusLabels.branchUnavailable}
    className={className}>
    <span className="flex shrink-0 items-center"><GitBranch size={13} /></span>
    <span className="min-w-0"><span className="block truncate text-xs">{node.label}</span>
      <span className="block truncate text-[9px] text-ink-mute">{node.entity.checkoutStatus ?? laneCopy.checkoutStatusUnavailable}</span></span>
  </div>;
  return null;
}
