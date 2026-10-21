/*
@cypher schema=1 source_sha256=c54545c8b2e6f472d9fc32594fb900b0c260c9162fb3f6b0d72621a1264d62fc
MERGE (self:File {path:"frontend/src/components/run/ProjectRunTree.tsx"})
MERGE (tree:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (live:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::AuthenticatedProjectRunTree",kind:"component"})
MERGE (row:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::DirectoryRow",kind:"component"})
MERGE (label:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::nodeScopeLabel",kind:"function"})
MERGE (runHref:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::canonicalRunHref",kind:"function"})
MERGE (guardedLink:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (self)-[:IMPORTS]->(guardedLink)
MERGE (self)-[:DEFINES]->(tree)
MERGE (self)-[:DEFINES]->(live)
MERGE (self)-[:DEFINES]->(row)
MERGE (row)-[:USES]->(guardedLink)
MERGE (self)-[:DEFINES]->(label)
MERGE (live)-[:CALLS]->(row)
MERGE (row)-[:CALLS]->(label)
MERGE (row)-[:CALLS]->(runHref)
@endcypher
*/

/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/ProjectRunTree.tsx",type:"file",language:"tsx"}),
 (tree:Function {name:"ProjectRunTree",type:"function"}),(live:Function {name:"AuthenticatedProjectRunTree",type:"function"}),
 (row:Function {name:"DirectoryRow",type:"function"}),(label:Function {name:"nodeScopeLabel",type:"function"}),
 (key:Function {name:"moveTreeFocus",type:"function"}),(start:Function {name:"mountDirectoryTree",type:"function"}),(clamp:Function {name:"clampTreeViewport",type:"function"}),
 (model:Class {name:"RunDirectoryTree",type:"class"}),(session:Function {name:"useRunDirectory",type:"function"}),
 (href:Function {name:"canonicalRunWorktreeHref",type:"function"}),(runHref:Function {name:"canonicalRunHref",type:"function"}),
 (height:Variable {name:"ROW_HEIGHT",type:"variable"}),(viewport:Variable {name:"VIEWPORT_HEIGHT",type:"variable"}),(overscan:Variable {name:"OVERSCAN",type:"variable"}),
 (store:Function {name:"useSyncExternalStore",type:"function"}),(memo:Function {name:"useMemo",type:"function"}),
 (effect:Function {name:"useEffect",type:"function"}),(state:Function {name:"useState",type:"function"}),(ref:Function {name:"useRef",type:"function"}),
 (f)-[:CONTAINS]->(tree),(f)-[:CONTAINS]->(live),(f)-[:CONTAINS]->(row),(f)-[:CONTAINS]->(label),(f)-[:CONTAINS]->(height),(f)-[:CONTAINS]->(viewport),
 (live)-[:CONTAINS]->(key),(live)-[:CONTAINS]->(start),(live)-[:CONTAINS]->(clamp),(f)-[:CONTAINS]->(overscan),(tree)-[:CALLS]->(session),(tree)-[:CALLS]->(live),
 (live)-[:CALLS]->(session),(live)-[:CALLS]->(model),(live)-[:CALLS]->(store),(live)-[:CALLS]->(memo),(live)-[:CALLS]->(effect),
 (live)-[:CALLS]->(state),(live)-[:CALLS]->(ref),(live)-[:CALLS]->(row),(row)-[:CALLS]->(label),(row)-[:CALLS]->(href),
 (live)-[:USES]->(overscan),(live)-[:USES]->(height),(live)-[:USES]->(viewport),(row)-[:USES]->(height),(key)-[:USES]->(height),(key)-[:USES]->(viewport),(clamp)-[:USES]->(height),(clamp)-[:USES]->(viewport);
*/

"use client";

import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { ChevronDown, ChevronRight, ClipboardList, FolderKanban, GitBranch, Layers, Monitor, Plus, RefreshCw } from "lucide-react";
import { useEffect, useMemo, useRef, useState, useSyncExternalStore, type FormEvent, type KeyboardEvent } from "react";
import { canonicalRunHref, canonicalRunWorktreeHref, type RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import { directoryErrorMessage, RunDirectoryTree, type DirectoryNode, type DirectoryTreeRow } from "@/lib/run/runDirectoryTree";
import { useTranslation } from "@/lib/i18n";

const ROW_HEIGHT = 36;
const VIEWPORT_HEIGHT = 288;
const OVERSCAN = 3;

export function ProjectRunTree({ collapsed }: { collapsed: boolean }) {
  const session = useRunDirectory();
  if (collapsed) return <div className="flex justify-center py-2" title="展开侧栏以浏览 Project → Run → Worktree → Branch"><FolderKanban size={20} aria-label="Project 目录" /></div>;
  return <section aria-label="Project 目录" data-testid="project-run-directory" className="border-b border-line pb-3">
    <div className="px-2 py-1 text-xs font-bold">Projects</div>
    <p className="px-2 pb-2 text-[10px] text-ink-mute">Engineering Run → 本地 Worktree → Branch</p>
    {session.status === "ready" && session.client ? <AuthenticatedProjectRunTree client={session.client} /> :
      <p role="status" className="px-2 py-3 text-xs text-ink-mute">{session.message}</p>}
  </section>;
}

function AuthenticatedProjectRunTree({ client }: { client: RunDirectoryApiClient }) {
  const { focus, clearFocus } = useRunDirectory();
  const { t } = useTranslation();
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
  function openCreateRun(project: ProjectNode, wasExpanded: boolean) {
    const prior = pendingCreate.current;
    const retry = prior?.projectId === project.entity.project_id ? prior : null;
    setCreateTarget({ project, wasExpanded });
    setCreateTitle(retry?.title ?? "");
    setCreateError(retry ? "上次提交结果未知；再次提交将复用同一请求标识。" : null);
  }
  async function submitCreateRun(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!createTarget || createBusy) return;
    const title = createTitle.trim();
    if (!title || [...title].length > 500) {
      setCreateError(t.runDirectory.invalidRunName);
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
      await client.createEngineeringRun(projectId, title, command.idempotencyKey, command.correlationId);
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
    <button type="button" onClick={() => { model.refresh(); if (viewport.current) viewport.current.scrollTop = 0; setScrollTop(0); }} className="mb-1 flex items-center gap-1 px-2 text-[10px] text-ink-mute hover:text-ink"><RefreshCw size={11} />刷新授权目录</button>
    <div ref={viewport} role="tree" aria-label="Project、Engineering Run、Worktree、Branch" onKeyDown={moveTreeFocus}
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)} className="relative overflow-y-auto overflow-x-hidden" style={{ height: VIEWPORT_HEIGHT }}>
      <div role="presentation" style={{ height: rows.length * ROW_HEIGHT }}>
        {visible.map((row, offset) => <DirectoryRow key={row.key} row={row} index={start + offset} model={model}
          selectedWorktreeId={focus?.focus?.worktree_id} clearFocus={clearFocus} onCreateRun={openCreateRun} newRunLabel={t.runDirectory.newRun} />)}
      </div>
    </div>
    <p className="px-2 pt-1 text-[10px] text-ink-mute">展开层级；选择 Worktree 设为当前 Run 焦点。</p>
    {createTarget && <div role="dialog" aria-modal="true" aria-labelledby="create-run-title" className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <form onSubmit={submitCreateRun} className="w-full max-w-md rounded-lg border border-line bg-bg p-5 shadow-xl">
        <h2 id="create-run-title" className="text-base font-semibold">{t.runDirectory.newRunTitle}</h2>
        <p className="mt-1 break-all text-xs text-ink-mute">Project {createTarget.project.entity.project_id} · {t.runDirectory.hostIndependentHint}</p>
        <label className="mt-4 grid gap-1 text-sm">{t.runDirectory.runName}
          <input value={createTitle} onChange={(event) => setCreateTitle(event.target.value)} autoFocus required maxLength={500}
            placeholder={t.runDirectory.runNamePlaceholder} className="rounded border border-line bg-bg-soft px-3 py-2" />
        </label>
        {createError && <p role="alert" className="mt-3 text-sm text-danger">{createError}</p>}
        <div className="mt-4 flex justify-end gap-2">
          <button type="button" disabled={createBusy} onClick={() => setCreateTarget(null)} className="rounded border border-line px-3 py-2 text-sm disabled:opacity-50">{t.runDirectory.cancel}</button>
          <button type="submit" disabled={createBusy || !createTitle.trim()} className="rounded bg-accent px-3 py-2 text-sm text-white disabled:opacity-50">{createBusy ? t.runDirectory.creating : t.runDirectory.create}</button>
        </div>
      </form>
    </div>}
  </>;
}

type ProjectNode = Extract<DirectoryNode, { kind: "project" }>;

function nodeScopeLabel(node: DirectoryNode): string {
  switch (node.kind) {
    case "project": return `Project · ${node.entity.role}`;
    case "run": return `Engineering Run · ${node.entity.state}`;
    case "worktree": return `本地 Worktree · ${node.entity.archived ? "已归档" : node.entity.machine_state}`;
    case "branch": return node.entity.name ? "本地 checkout 记录 · 来源/新鲜度未验证" : "本地 checkout 状态未采集";
  }
}
function DirectoryRow({ row, index, model, selectedWorktreeId, clearFocus, onCreateRun, newRunLabel }: {
  row: DirectoryTreeRow; index: number; model: RunDirectoryTree; selectedWorktreeId?: string; clearFocus: () => void;
  onCreateRun: (project: ProjectNode, expanded: boolean) => void; newRunLabel: string;
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
      {row.message}{row.state === "error" && " 点击重试"}
    </button>;
  }
  const { node, expanded } = row;
  const Icon = { project: FolderKanban, branch: GitBranch, run: Layers, worktree: Monitor }[node.kind];
  const contents = <><span className="flex shrink-0 items-center">{node.kind === "branch" ? <span className="w-3" /> : expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}<Icon size={13} /></span>
    <span className="min-w-0"><span className="block truncate text-xs">{node.label}</span><span className="block truncate text-[9px] text-ink-mute">{nodeScopeLabel(node)}</span></span></>;
  const className = "flex w-full min-w-0 items-center gap-1 rounded pr-1 text-left hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent";
  if (node.kind === "project") return <div role="presentation" style={style} className="flex items-center gap-1">
    <button role="treeitem" aria-level={depth + 1} aria-expanded={expanded} type="button" data-directory-index={index}
      className={`${className} min-w-0 flex-1`} onClick={() => model.toggle(node)} title={node.label}>{contents}</button>
    <button type="button" aria-label={`${node.label}: ${newRunLabel}`} title={newRunLabel}
      className="flex h-7 w-7 shrink-0 items-center justify-center rounded text-ink-mute hover:bg-accent/10 hover:text-accent focus:outline focus:outline-1 focus:outline-accent"
      onClick={() => onCreateRun(node, expanded)}><Plus size={14} /></button>
  </div>;
  if (node.kind === "worktree") {
    const selected = selectedWorktreeId === node.entity.worktree_id;
    return <div role="presentation" style={style} className="flex items-center">
      <button type="button" tabIndex={-1} aria-label={`${expanded ? "折叠" : "展开"} ${node.label}`}
        aria-expanded={expanded} title={`${expanded ? "折叠" : "展开"} Worktree 分支状态`}
        className="flex h-full w-5 shrink-0 items-center justify-center rounded hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent"
        onClick={() => model.toggle(node)}>
        {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
      </button>
      <Link role="treeitem" aria-level={depth + 1} aria-expanded={expanded} aria-selected={selected} aria-current={selected ? "page" : undefined}
        data-directory-index={index} title={`${node.label} · ${node.entity.worktree_id}`}
        className={`${className} w-auto flex-1 ${selected ? "bg-accent/15 text-accent" : ""}`} prefetch={false}
        onNavigate={() => {
          if (!selected) clearFocus();
        }}
        href={canonicalRunWorktreeHref(node.entity.project_id, node.entity.engineering_run_id, node.entity.worktree_id)}>
        <span className="flex shrink-0 items-center"><Monitor size={13} /></span>
        <span className="min-w-0"><span className="block truncate text-xs">{node.label}</span><span className="block truncate text-[9px] text-ink-mute">{nodeScopeLabel(node)}</span></span>
      </Link>
    </div>;
  }
  if (node.kind === "run") return <div role="presentation" style={style} className="flex items-center gap-1">
    <button role="treeitem" aria-level={depth + 1} aria-expanded={expanded} type="button" data-directory-index={index}
      className={`${className} min-w-0 flex-1`} onClick={() => model.toggle(node)} title={node.label}>{contents}</button>
    <Link href={canonicalRunHref(node.entity.project_id, node.entity.engineering_run_id)} prefetch={false}
      aria-label={`${node.label}: 打开 Task Cards`} title="打开此 Run 的 Task Cards"
      className="flex h-7 w-7 shrink-0 items-center justify-center rounded text-ink-mute hover:bg-accent/10 hover:text-accent focus:outline focus:outline-1 focus:outline-accent">
      <ClipboardList size={13} />
    </Link>
  </div>;
  if (node.kind === "branch") return <div role="treeitem" aria-level={depth + 1} tabIndex={-1}
    data-directory-index={index} style={style} title={node.entity.name ? `本地 checkout 记录（来源/新鲜度未验证）：${node.entity.name}` : "本地 checkout 状态未采集"} className={className}>{contents}</div>;
  return null;
}
