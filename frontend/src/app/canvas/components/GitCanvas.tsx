/*
@cypher schema=1 source_sha256=dd5be7d2a418bf1cded4b9f002540b707587dfc75b45072ff008e0e2b1f232da
MERGE (self:File {path:"frontend/src/app/canvas/components/GitCanvas.tsx"})
MERGE (comp:Symbol {id:"frontend/src/app/canvas/components/GitCanvas.tsx::GitCanvas",kind:"component"})
MERGE (self)-[:DEFINES]->(comp)
@endcypher
*/
"use client";

// =====================================================================
// GitCanvas — Git 提交图画布 (per 2026-11-15 docs/prototypes/2 统一画布 mockup:
//   提交/合并提交/冲突/待合并 4 类节点 + lane 列表 + 提交列表表 + 时间排序 + 只看我的 Worktree)
// 数据: store.pullRequests + store.worktrees (预览模式, 宿主 git 投影接入后切换)
// =====================================================================

import { useMemo, useState } from "react";
import { GitCommitHorizontal, GitMerge, AlertTriangle, Clock, GitBranch, Check } from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";

type NodeKind = "commit" | "merge" | "conflict" | "pending";

const KIND_TONE: Record<NodeKind, string> = {
  commit: "bg-ok text-white",
  merge: "bg-accent text-white",
  conflict: "bg-err text-white",
  pending: "bg-warn text-ink",
};

const LEGEND: Array<{ kind: NodeKind; label: string }> = [
  { kind: "commit", label: "提交" },
  { kind: "merge", label: "合并提交" },
  { kind: "conflict", label: "冲突" },
  { kind: "pending", label: "待合并" },
];

function prKind(status: string): NodeKind {
  if (status === "merged") return "merge";
  if (status === "ci_failed" || status === "review_required") return "conflict";
  return "pending";
}

export function GitCanvas() {
  const pullRequests = useStore((s) => s.pullRequests);
  const worktrees = useStore((s) => s.worktrees);
  const [sort, setSort] = useState<"time" | "branch">("time");
  const [onlyMine, setOnlyMine] = useState(false);

  const rows = useMemo(() => {
    let list = pullRequests.map((pr) => {
      const wt = worktrees.find((w) => w.branch === pr.source_branch);
      const kind = prKind(pr.status);
      return {
        id: pr.id,
        sha: `#${pr.number}`,
        msg: pr.title,
        branch: pr.source_branch,
        worktree: wt?.name ?? "—",
        ab: wt ? (wt.status === "dirty" ? "+3/-1" : wt.status === "behind" ? "0/-2" : "0/0") : "—",
        task: pr.repository_id.replace("repo", "T-"),
        mergeState: pr.status,
        kind,
        at: pr.created_at,
      };
    });
    if (onlyMine) list = list.filter((r) => r.branch.startsWith("feat/"));
    if (sort === "time") list.sort((a, b) => b.at.localeCompare(a.at));
    else list.sort((a, b) => a.branch.localeCompare(b.branch));
    return list;
  }, [pullRequests, worktrees, sort, onlyMine]);

  // 分支 lane (来自 worktrees, 去重取前 8)
  const lanes = useMemo(() => {
    const seen = new Set<string>();
    const out: { name: string; wt: string; status: string }[] = [];
    for (const wt of worktrees) {
      if (seen.has(wt.branch)) continue;
      seen.add(wt.branch);
      out.push({ name: wt.branch, wt: wt.name, status: wt.status });
      if (out.length >= 8) break;
    }
    return out;
  }, [worktrees]);

  return (
    <div className="flex h-full flex-col" data-testid="git-canvas">
      {/* 工具栏 */}
      <div className="flex items-center gap-2 border-b border-line bg-bg-card px-3 py-2 flex-wrap">
        <span className="text-[10px] font-mono uppercase tracking-wider text-ink-mute">Git 画布</span>
        <div className="flex items-center gap-1.5">
          {LEGEND.map((l) => (
            <span key={l.kind} className="flex items-center gap-1 text-[10px] text-ink-dim">
              <span className={clsx("size-3 rounded-full grid place-items-center", KIND_TONE[l.kind])}>
                {l.kind === "merge" ? <GitMerge size={8} /> : l.kind === "conflict" ? <AlertTriangle size={8} /> : <GitCommitHorizontal size={8} />}
              </span>
              {l.label}
            </span>
          ))}
        </div>
        <span className="flex-1" />
        <select
          value={sort}
          onChange={(e) => setSort(e.target.value as "time" | "branch")}
          aria-label="排序"
          data-testid="git-canvas-sort"
          className="h-7 rounded border border-line bg-bg-soft px-1.5 text-[10px]"
        >
          <option value="time">时间 ▾</option>
          <option value="branch">分支</option>
        </select>
        <button
          type="button"
          onClick={() => setOnlyMine((v) => !v)}
          aria-pressed={onlyMine}
          data-testid="git-canvas-only-mine"
          className={clsx(
            "h-7 rounded border px-2 text-[10px] font-semibold transition-colors",
            onlyMine ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-mute hover:text-ink",
          )}
        >
          只看我的 Worktree
        </button>
      </div>

      {/* 画布区: 分支 lane + 提交节点 */}
      <div className="min-h-0 flex-1 overflow-auto p-3" data-testid="git-canvas-stage">
        <p className="mb-2 text-[10px] text-ink-mute">拖到画布，或点节点右侧的 + 接在后面</p>
        <div className="space-y-1.5">
          {lanes.map((lane) => {
            const laneCommits = rows.filter((r) => r.branch === lane.name);
            return (
              <div key={lane.name} className="flex items-center gap-2 rounded-lg border border-line bg-bg-card/60 px-2.5 py-1.5" data-testid={`git-canvas-lane-${lane.name}`}>
                <GitBranch size={12} className="shrink-0 text-ok" />
                <span className="w-40 shrink-0 truncate font-mono text-[10px] text-ink">{lane.name}</span>
                <span className="w-28 shrink-0 truncate text-[9px] text-ink-mute">{lane.wt}</span>
                <span className="text-[9px] font-mono text-ink-dim">{lane.status}</span>
                <span className="flex-1" />
                <div className="flex items-center gap-1 overflow-x-auto">
                  {laneCommits.slice(0, 4).map((c) => (
                    <span
                      key={c.id}
                      title={`${c.sha} ${c.msg}`}
                      data-testid={`git-canvas-node-${c.id}`}
                      className={clsx("size-4 shrink-0 rounded-full grid place-items-center", KIND_TONE[c.kind])}
                    >
                      {c.kind === "merge" ? <GitMerge size={8} /> : c.kind === "conflict" ? <AlertTriangle size={8} /> : <Clock size={8} />}
                    </span>
                  ))}
                  <span className="text-[9px] text-ink-mute shrink-0">+ 接在后面</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 提交列表表 (mockup 列: 提交/分支/Worktree/相对 dev/关联任务卡/合并状态) */}
      <div className="shrink-0 border-t border-line overflow-auto max-h-56" data-testid="git-canvas-table">
        <table className="w-full text-[10px]">
          <thead className="sticky top-0 bg-bg-card">
            <tr className="border-b border-line text-left text-[9px] font-semibold uppercase tracking-wider text-ink-mute">
              <th className="px-3 py-1.5">提交</th>
              <th className="px-2 py-1.5">分支</th>
              <th className="px-2 py-1.5">Worktree</th>
              <th className="px-2 py-1.5">相对 dev</th>
              <th className="px-2 py-1.5">关联任务卡</th>
              <th className="px-2 py-1.5">合并状态</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id} className="border-b border-line/60 hover:bg-bg-soft/60" data-testid={`git-canvas-row-${r.id}`}>
                <td className="px-3 py-1 font-mono">
                  <span className="flex items-center gap-1">
                    <span className={clsx("size-3 rounded-full grid place-items-center shrink-0", KIND_TONE[r.kind])}>
                      {r.kind === "merge" ? <GitMerge size={7} /> : r.kind === "conflict" ? <AlertTriangle size={7} /> : <Check size={7} />}
                    </span>
                    {r.sha} · <span className="truncate max-w-40 inline-block align-bottom">{r.msg}</span>
                  </span>
                </td>
                <td className="px-2 py-1 font-mono">{r.branch}</td>
                <td className="px-2 py-1">{r.worktree}</td>
                <td className="px-2 py-1 font-mono">{r.ab}</td>
                <td className="px-2 py-1 font-mono">{r.task}</td>
                <td className="px-2 py-1">
                  <span className={clsx("rounded px-1.5 py-0.5 text-[8px] font-semibold", r.kind === "merge" ? "bg-accent/10 text-accent" : r.kind === "conflict" ? "bg-err/10 text-err" : "bg-warn/10 text-warn")}>
                    {r.mergeState}
                  </span>
                </td>
              </tr>
            ))}
            {rows.length === 0 && (
              <tr><td colSpan={6} className="px-3 py-4 text-center text-ink-mute">暂无提交 (宿主 git 投影接入后显示真实提交图)</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
