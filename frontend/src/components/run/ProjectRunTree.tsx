/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/ProjectRunTree.tsx",type:"file",language:"tsx"}),
 (tree:Function {name:"ProjectRunTree",type:"function"}),(live:Function {name:"AuthenticatedProjectRunTree",type:"function"}),
 (row:Function {name:"DirectoryRow",type:"function"}),(label:Function {name:"nodeScopeLabel",type:"function"}),
 (key:Function {name:"moveTreeFocus",type:"function"}),(start:Function {name:"mountDirectoryTree",type:"function"}),(clamp:Function {name:"clampTreeViewport",type:"function"}),
 (model:Class {name:"RunDirectoryTree",type:"class"}),(session:Function {name:"useRunDirectory",type:"function"}),
 (href:Function {name:"canonicalRunWorktreeHref",type:"function"}),
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

import Link from "next/link";
import { ChevronDown, ChevronRight, FolderKanban, GitBranch, Layers, Monitor, RefreshCw } from "lucide-react";
import { useEffect, useMemo, useRef, useState, useSyncExternalStore, type KeyboardEvent } from "react";
import { canonicalRunWorktreeHref, type RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import { RunDirectoryTree, type DirectoryNode, type DirectoryTreeRow } from "@/lib/run/runDirectoryTree";

const ROW_HEIGHT = 36;
const VIEWPORT_HEIGHT = 288;
const OVERSCAN = 3;

export function ProjectRunTree({ collapsed }: { collapsed: boolean }) {
  const session = useRunDirectory();
  if (collapsed) return <div className="flex justify-center py-2" title="展开侧栏以浏览 Project → Branch → Run → Worktree"><FolderKanban size={20} aria-label="Project 目录" /></div>;
  return <section aria-label="Project 目录" data-testid="project-run-directory" className="border-b border-line pb-3">
    <div className="px-2 py-1 text-xs font-bold">Projects</div>
    <p className="px-2 pb-2 text-[10px] text-ink-mute">云端 Branch → Engineering Run → 本地 Worktree</p>
    {session.status === "ready" && session.client ? <AuthenticatedProjectRunTree client={session.client} /> :
      <p role="status" className="px-2 py-3 text-xs text-ink-mute">{session.message}</p>}
  </section>;
}

function AuthenticatedProjectRunTree({ client }: { client: RunDirectoryApiClient }) {
  const { focus, clearFocus } = useRunDirectory();
  const model = useMemo(() => new RunDirectoryTree(client), [client]);
  const rows = useSyncExternalStore(model.subscribe, model.getSnapshot, model.getSnapshot);
  const viewport = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
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
    else if (event.key === "ArrowRight" && row.kind === "node" && row.node.kind !== "worktree") {
      if (!row.expanded) { model.toggle(row.node); return; }
      next += 1;
    } else if (event.key === "ArrowLeft" && row.kind === "node") {
      if (row.expanded && row.node.kind !== "worktree") { model.toggle(row.node); return; }
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
  return <>
    <button type="button" onClick={() => { model.refresh(); if (viewport.current) viewport.current.scrollTop = 0; setScrollTop(0); }} className="mb-1 flex items-center gap-1 px-2 text-[10px] text-ink-mute hover:text-ink"><RefreshCw size={11} />刷新授权目录</button>
    <div ref={viewport} role="tree" aria-label="Project、Branch、Engineering Run、Worktree" onKeyDown={moveTreeFocus}
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)} className="relative overflow-y-auto overflow-x-hidden" style={{ height: VIEWPORT_HEIGHT }}>
      <div role="presentation" style={{ height: rows.length * ROW_HEIGHT }}>
        {visible.map((row, offset) => <DirectoryRow key={row.key} row={row} index={start + offset} model={model}
          selectedWorktreeId={focus?.focus?.worktree_id} clearFocus={clearFocus} />)}
      </div>
    </div>
    <p className="px-2 pt-1 text-[10px] text-ink-mute">展开层级；选择 Worktree 后验证 Run 焦点。</p>
  </>;
}

function nodeScopeLabel(node: DirectoryNode): string {
  switch (node.kind) {
    case "project": return `Project · ${node.entity.role}`;
    case "branch": return `云端 Branch · ${node.entity.state}`;
    case "run": return `Engineering Run · ${node.entity.state}`;
    case "worktree": return `本地 Worktree · ${node.entity.archived ? "已归档" : node.entity.machine_state}`;
  }
}
function DirectoryRow({ row, index, model, selectedWorktreeId, clearFocus }: {
  row: DirectoryTreeRow; index: number; model: RunDirectoryTree; selectedWorktreeId?: string; clearFocus: () => void;
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
  const contents = <><span className="flex shrink-0 items-center">{node.kind === "worktree" ? <span className="w-3" /> : expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}<Icon size={13} /></span>
    <span className="min-w-0"><span className="block truncate text-xs">{node.label}</span><span className="block truncate text-[9px] text-ink-mute">{nodeScopeLabel(node)}</span></span></>;
  const className = "flex w-full items-center gap-1 rounded pr-1 text-left hover:bg-accent/10 focus:outline focus:outline-1 focus:outline-accent";
  if (node.kind === "worktree") {
    const selected = selectedWorktreeId === node.entity.worktree_id;
    return <Link role="treeitem" aria-level={depth + 1} aria-selected={selected} aria-current={selected ? "page" : undefined}
      data-directory-index={index} style={style} title={`${node.label} · ${node.entity.worktree_id}`}
      className={`${className} ${selected ? "bg-accent/15 text-accent" : ""}`} prefetch={false}
      onClick={(event) => { if (!selected && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey) clearFocus(); }}
      href={canonicalRunWorktreeHref(node.entity.project_id, node.branchId, node.entity.engineering_run_id, node.entity.worktree_id)}>{contents}</Link>;
  }
  return <button role="treeitem" aria-level={depth + 1} aria-expanded={expanded} type="button" data-directory-index={index}
    style={style} title={node.label} className={className} onClick={() => model.toggle(node)}>{contents}</button>;
}
