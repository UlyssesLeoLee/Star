/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/lib/run/runDirectoryTree.ts",type:"file",language:"typescript"}),
 (model:Class {name:"RunDirectoryTree",type:"class"}),(node:Class {name:"DirectoryNode",type:"class"}),(base:Class {name:"NodeBase",type:"class"}),
 (parent:Class {name:"DirectoryParent",type:"class"}),(row:Class {name:"DirectoryTreeRow",type:"class"}),(entry:Class {name:"ParentCache",type:"class"}),
 (ctor:Function {name:"RunDirectoryTree.constructor",type:"function"}),(subscribe:Function {name:"RunDirectoryTree.subscribe",type:"function"}),
 (snapshot:Function {name:"RunDirectoryTree.getSnapshot",type:"function"}),(start:Function {name:"RunDirectoryTree.start",type:"function"}),
 (dispose:Function {name:"RunDirectoryTree.dispose",type:"function"}),(toggle:Function {name:"RunDirectoryTree.toggle",type:"function"}),
 (refresh:Function {name:"RunDirectoryTree.refresh",type:"function"}),(more:Function {name:"RunDirectoryTree.loadMore",type:"function"}),(window:Function {name:"RunDirectoryTree.nextWindow",type:"function"}),
 (find:Function {name:"RunDirectoryTree.ensureParent",type:"function"}),(remove:Function {name:"RunDirectoryTree.removeSubtree",type:"function"}),
 (cancel:Function {name:"RunDirectoryTree.cancelPending",type:"function"}),(fetch:Function {name:"RunDirectoryTree.fetchPage",type:"function"}),
 (request:Function {name:"RunDirectoryTree.requestPage",type:"function"}),(publish:Function {name:"RunDirectoryTree.publish",type:"function"}),
 (visit:Function {name:"visitParent",type:"function"}),(map:Function {name:"directoryNodes",type:"function"}),
 (error:Function {name:"directoryErrorMessage",type:"function"}),(root:Variable {name:"DIRECTORY_ROOT",type:"variable"}),
 (budget:Variable {name:"RUN_DIRECTORY_TREE_BUDGET",type:"variable"}),
 (projects:Function {name:"RunDirectoryApiClient.listProjects",type:"function"}),(branches:Function {name:"RunDirectoryApiClient.listBranches",type:"function"}),
 (runs:Function {name:"RunDirectoryApiClient.listRuns",type:"function"}),(worktrees:Function {name:"RunDirectoryApiClient.listWorktrees",type:"function"}),
 (f)-[:CONTAINS]->(model),(f)-[:CONTAINS]->(base),(f)-[:CONTAINS]->(node),(f)-[:CONTAINS]->(parent),(f)-[:CONTAINS]->(row),(f)-[:CONTAINS]->(entry),
 (f)-[:CONTAINS]->(map),(f)-[:CONTAINS]->(error),(f)-[:CONTAINS]->(root),(f)-[:CONTAINS]->(budget),
 (model)-[:HAS_METHOD]->(ctor),(model)-[:HAS_METHOD]->(subscribe),(model)-[:HAS_METHOD]->(snapshot),(model)-[:HAS_METHOD]->(start),
 (model)-[:HAS_METHOD]->(dispose),(model)-[:HAS_METHOD]->(toggle),(model)-[:HAS_METHOD]->(refresh),(model)-[:HAS_METHOD]->(more),(model)-[:HAS_METHOD]->(window),
 (model)-[:HAS_METHOD]->(find),(model)-[:HAS_METHOD]->(remove),(model)-[:HAS_METHOD]->(cancel),(model)-[:HAS_METHOD]->(fetch),
 (model)-[:HAS_METHOD]->(request),(model)-[:HAS_METHOD]->(publish),(publish)-[:CONTAINS]->(visit),
 (start)-[:CALLS]->(more),(toggle)-[:CALLS]->(cancel),(toggle)-[:CALLS]->(remove),(toggle)-[:CALLS]->(more),(refresh)-[:CALLS]->(remove),
 (refresh)-[:CALLS]->(more),(window)-[:CALLS]->(remove),(window)-[:CALLS]->(find),(window)-[:CALLS]->(more),(more)-[:CALLS]->(find),(more)-[:CALLS]->(fetch),(find)-[:CALLS]->(remove),(dispose)-[:CALLS]->(cancel),
 (remove)-[:CALLS]->(cancel),(fetch)-[:CALLS]->(request),(fetch)-[:CALLS]->(map),(fetch)-[:CALLS]->(error),(fetch)-[:CALLS]->(publish),
 (request)-[:CALLS]->(projects),(request)-[:CALLS]->(branches),(request)-[:CALLS]->(runs),(request)-[:CALLS]->(worktrees),(publish)-[:CALLS]->(visit),(start)-[:USES]->(root),(dispose)-[:USES]->(root),(refresh)-[:USES]->(root),(publish)-[:USES]->(root),(more)-[:USES]->(budget),(find)-[:USES]->(budget),(publish)-[:USES]->(budget);
*/

import {
  RunDirectoryApiClient, RunDirectoryError, type AuthorizedRunProject, type CloudBranch,
  type DirectoryPage, type EngineeringRun, type RunWorktree,
} from "./runDirectoryApi";

interface NodeBase { key: string; parentKey: string; depth: number; label: string }
export type DirectoryNode =
  | (NodeBase & { kind: "project"; entity: AuthorizedRunProject })
  | (NodeBase & { kind: "branch"; entity: CloudBranch })
  | (NodeBase & { kind: "run"; entity: EngineeringRun })
  | (NodeBase & { kind: "worktree"; entity: RunWorktree; branchId: string });
export type DirectoryParent = { kind: "root"; key: string; depth: number } | Exclude<DirectoryNode, { kind: "worktree" }>;
export type DirectoryTreeRow =
  | { kind: "node"; key: string; node: DirectoryNode; expanded: boolean }
  | { kind: "hint"; key: string; parent: DirectoryParent; depth: number; state: "loading" | "error" | "more" | "budget" | "empty" | "refresh"; message: string };
interface ParentCache {
  parent: DirectoryParent; rows: DirectoryNode[]; pages: number; nextCursor: string | null;
  busy: boolean; error: string | null; touched: number; cursors: Set<string>;
}
export const DIRECTORY_ROOT: DirectoryParent = { kind: "root", key: "projects", depth: -1 };
export const RUN_DIRECTORY_TREE_BUDGET = { parents: 16, pagesPerParent: 4 } as const;

export function directoryErrorMessage(error: unknown): string {
  return error instanceof RunDirectoryError ? error.message : "授权目录暂时不可用，请重试。";
}
function directoryNodes(parent: DirectoryParent, rows: Array<AuthorizedRunProject | CloudBranch | EngineeringRun | RunWorktree>): DirectoryNode[] {
  return rows.map<DirectoryNode>((entity) => {
    const base = { parentKey: parent.key, depth: parent.depth + 1 };
    switch (parent.kind) {
      case "root": {
        const project = entity as AuthorizedRunProject;
        return { ...base, kind: "project", key: `${parent.key}/project:${project.project_id}`, label: `Project ${project.project_id}`, entity: project };
      }
      case "project": {
        const branch = entity as CloudBranch;
        return { ...base, kind: "branch", key: `${parent.key}/branch:${branch.branch_id}`, label: branch.name, entity: branch };
      }
      case "branch": {
        const run = entity as EngineeringRun;
        return { ...base, kind: "run", key: `${parent.key}/run:${run.engineering_run_id}`, label: run.title, entity: run };
      }
      case "run": {
        const worktree = entity as RunWorktree;
        return { ...base, kind: "worktree", key: `${parent.key}/worktree:${worktree.worktree_id}`, label: worktree.name, entity: worktree, branchId: parent.entity.branch_id };
      }
    }
  });
}

/** One session owns this bounded cache. Late responses can only update their live request slot. */
export class RunDirectoryTree {
  private readonly cache = new Map<string, ParentCache>();
  private readonly expanded = new Set<string>();
  private readonly pending = new Map<string, AbortController>();
  private readonly listeners = new Set<() => void>();
  private snapshot: readonly DirectoryTreeRow[] = [];
  private clock = 0;
  private active = false;
  constructor(private readonly client: RunDirectoryApiClient) {}
  subscribe = (listener: () => void): (() => void) => { this.listeners.add(listener); return () => this.listeners.delete(listener); };
  getSnapshot = (): readonly DirectoryTreeRow[] => this.snapshot;
  start(): void { this.active = true; this.loadMore(DIRECTORY_ROOT); }
  dispose(): void {
    this.active = false;
    this.cancelPending(DIRECTORY_ROOT.key);
    this.cache.clear(); this.expanded.clear(); this.publish();
  }
  toggle(node: Exclude<DirectoryNode, { kind: "worktree" }>): void {
    if (!this.active) return;
    if (this.expanded.has(node.key)) {
      this.expanded.delete(node.key);
      this.cancelPending(node.key);
      this.removeSubtree(node.key, false);
      this.publish();
    } else {
      this.expanded.add(node.key);
      const entry = this.cache.get(node.key);
      if (entry) entry.touched = ++this.clock;
      if (!entry || entry.pages === 0) this.loadMore(node);
      else this.publish();
    }
  }
  refresh(parent: DirectoryParent = DIRECTORY_ROOT): void {
    if (!this.active || (parent.kind !== "root" && !this.expanded.has(parent.key))) return;
    this.removeSubtree(parent.key, true);
    if (parent.kind !== "root") this.expanded.add(parent.key);
    this.loadMore(parent);
  }
  nextWindow(parent: DirectoryParent): void {
    const entry = this.cache.get(parent.key);
    if (!this.active || !entry || entry.busy || !entry.nextCursor || entry.pages < RUN_DIRECTORY_TREE_BUDGET.pagesPerParent ||
      (parent.kind !== "root" && !this.expanded.has(parent.key))) return;
    const cursor = entry.nextCursor;
    this.removeSubtree(parent.key, true);
    if (parent.kind !== "root") this.expanded.add(parent.key);
    this.ensureParent(parent).nextCursor = cursor;
    this.loadMore(parent);
  }
  loadMore(parent: DirectoryParent): void {
    if (!this.active || (parent.kind !== "root" && !this.expanded.has(parent.key))) return;
    const entry = this.ensureParent(parent);
    if (entry.busy || entry.pages >= RUN_DIRECTORY_TREE_BUDGET.pagesPerParent || (entry.pages > 0 && !entry.nextCursor)) return;
    const cursor = entry.nextCursor ?? undefined;
    entry.busy = true; entry.error = null; entry.touched = ++this.clock;
    const controller = new AbortController();
    this.pending.set(parent.key, controller);
    this.publish();
    void this.fetchPage(entry, cursor, controller);
  }
  private ensureParent(parent: DirectoryParent): ParentCache {
    const existing = this.cache.get(parent.key);
    if (existing) return existing;
    while (this.cache.size >= RUN_DIRECTORY_TREE_BUDGET.parents) {
      const candidate = [...this.cache.values()]
        .filter((entry) => entry.parent.kind !== "root" && !parent.key.startsWith(`${entry.parent.key}/`))
        .sort((left, right) => left.touched - right.touched)[0];
      if (!candidate) throw new RunDirectoryError(0, "cache_limit", "目录缓存已达上限，请刷新 Project 目录。");
      this.removeSubtree(candidate.parent.key, true);
    }
    const entry: ParentCache = { parent, rows: [], pages: 0, nextCursor: null, busy: false, error: null, touched: ++this.clock, cursors: new Set() };
    this.cache.set(parent.key, entry);
    return entry;
  }
  private removeSubtree(key: string, includeSelf: boolean): void {
    this.cancelPending(key);
    const matches = (candidate: string) => (includeSelf && candidate === key) || candidate.startsWith(`${key}/`);
    for (const candidate of this.cache.keys()) if (matches(candidate)) this.cache.delete(candidate);
    for (const candidate of this.expanded) if (matches(candidate)) this.expanded.delete(candidate);
  }
  private cancelPending(key: string): void {
    for (const [candidate, controller] of this.pending) {
      if (candidate === key || candidate.startsWith(`${key}/`)) {
        this.pending.delete(candidate); controller.abort();
        const entry = this.cache.get(candidate);
        if (entry) entry.busy = false;
      }
    }
  }
  private async fetchPage(entry: ParentCache, cursor: string | undefined, controller: AbortController): Promise<void> {
    try {
      const page = await this.requestPage(entry.parent, cursor, controller.signal);
      if (!this.active || controller.signal.aborted || this.pending.get(entry.parent.key) !== controller || this.cache.get(entry.parent.key) !== entry) return;
      const nodes = directoryNodes(entry.parent, page.rows);
      const previous = new Set(entry.rows.map((row) => row.key));
      if (nodes.some((node) => previous.has(node.key)) || (page.next_cursor && (page.next_cursor === cursor || entry.cursors.has(page.next_cursor)))) {
        throw new RunDirectoryError(0, "invalid_page", "目录分页重复，请刷新该层目录。");
      }
      entry.rows.push(...nodes); entry.pages += 1; entry.nextCursor = page.next_cursor;
      if (cursor) entry.cursors.add(cursor);
    } catch (error) {
      if (!controller.signal.aborted && this.pending.get(entry.parent.key) === controller && this.cache.get(entry.parent.key) === entry) entry.error = directoryErrorMessage(error);
    } finally {
      if (this.pending.get(entry.parent.key) === controller) {
        this.pending.delete(entry.parent.key); entry.busy = false; this.publish();
      }
    }
  }
  private async requestPage(parent: DirectoryParent, cursor: string | undefined, signal: AbortSignal): Promise<DirectoryPage<AuthorizedRunProject | CloudBranch | EngineeringRun | RunWorktree>> {
    switch (parent.kind) {
      case "root": return this.client.listProjects(cursor, signal);
      case "project": return this.client.listBranches(parent.entity.project_id, cursor, signal);
      case "branch": return this.client.listRuns(parent.entity, cursor, signal);
      case "run": return this.client.listWorktrees(parent.entity, cursor, signal);
    }
  }
  private publish(): void {
    const rows: DirectoryTreeRow[] = [];
    const visitParent = (parent: DirectoryParent) => {
      const entry = this.cache.get(parent.key);
      if (!entry) return;
      for (const node of entry.rows) {
        const expanded = this.expanded.has(node.key);
        rows.push({ kind: "node", key: node.key, node, expanded });
        if (expanded && node.kind !== "worktree") visitParent(node);
      }
      let state: "loading" | "error" | "more" | "budget" | "empty" | "refresh" | null = null;
      let message = "";
      if (entry.busy) { state = "loading"; message = "加载中…"; }
      else if (entry.error) { state = "error"; message = entry.error; }
      else if (entry.nextCursor) {
        state = entry.pages >= RUN_DIRECTORY_TREE_BUDGET.pagesPerParent ? "budget" : "more";
        message = state === "budget" ? "查看下一组（替换当前 200 条）" : "加载更多";
      } else if (entry.rows.length === 0) { state = "empty"; message = "暂无授权条目；点击刷新"; }
      else if (parent.kind !== "root") { state = "refresh"; message = "刷新该层目录"; }
      if (state) rows.push({ kind: "hint", key: `${parent.key}/hint`, parent, depth: parent.depth + 1, state, message });
      if (!entry.busy && entry.rows.length > 0 && parent.kind !== "root" && state !== "refresh") {
        rows.push({ kind: "hint", key: `${parent.key}/refresh`, parent, depth: parent.depth + 1, state: "refresh", message: "刷新该层目录" });
      }
    };
    visitParent(DIRECTORY_ROOT);
    this.snapshot = rows;
    for (const listener of this.listeners) listener();
  }
}
