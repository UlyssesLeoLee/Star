"use client";

import Link from "next/link";
import { useState } from "react";
import Link from "next/link";
import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { WORKTREE_SM, type WorktreeStatus } from "@/types/ids";
import { GitBranch, GitMerge, Lock, Cpu, AlertCircle } from "lucide-react";
import { clsx } from "clsx";

export default function WorktreePage() {
  const { worktrees, transitionWorktree } = useStore();
  const [selected, setSelected] = useState<string>("wt-003");

  const wt = worktrees.find((w) => w.id === selected);
  const allowedNext = wt
    ? Array.from(new Set(
        WORKTREE_SM.transitions.filter((t) => t.from === wt.status).map((t) => t.to),
      ))
    : [];

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Worktree"
        subtitle="17 状态机 (§7.1) — INV-WT-01~04。每个 worktree 是 git checkout 的隔离副本,绑定 local-runtime + agent-session + PR。"
        icon={<GitBranch className="text-accent" size={20} />}
        track="B"
        count={worktrees.length}
      />

      <SectionTitle>状态机可视化</SectionTitle>
      <div className="mb-5">
        <StateMachineDiagram sm={WORKTREE_SM} highlightState={wt?.status} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        {/* 列表 */}
        <div className="lg:col-span-2">
          <div className="card">
            <SectionTitle>Worktrees ({worktrees.length})</SectionTitle>
            <table className="table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Name</th>
                  <th>Branch</th>
                  <th>Status</th>
                  <th>PR</th>
                  <th>Last event</th>
                </tr>
              </thead>
              <tbody>
                {worktrees.map((w) => (
                  <tr
                    key={w.id}
                    onClick={() => setSelected(w.id)}
                    className={clsx("cursor-pointer", selected === w.id && "bg-accent/5")}
                  >
                    <td className="font-mono text-xs">{w.id}</td>
                    <td className="font-medium">{w.name}</td>
                    <td className="font-mono text-xs text-info">{w.branch}</td>
                    <td><StatusPill value={w.status} /></td>
                    <td className="font-mono text-xs">{w.pr_id ?? "—"}</td>
                    <td className="text-ink-dim text-xs">{new Date(w.last_event_at).toLocaleTimeString()}</td>
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
                <Row label="Local Runtime" value={wt.local_runtime_id ?? "—"} />
                <Row label="Agent Session" value={wt.agent_session_id ?? "—"} />
                <Row label="PR" value={wt.pr_id ?? "—"} />
                <Row label="Lock version" value={<span className="font-mono">v{wt.lock_version}</span>} />
                <Row label="Created" value={new Date(wt.created_at).toLocaleString()} />
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
              </div>

              <div className="mt-4 pt-3 border-t border-line text-[10px] text-ink-mute space-y-1">
                <div className="flex items-center gap-1.5">
                  <Lock size={10} /> Optimistic lock via lock_version
                </div>
                <div className="flex items-center gap-1.5">
                  <Cpu size={10} /> 1 worktree ↔ 1 active agent session
                </div>
                <div className="flex items-center gap-1.5">
                  <AlertCircle size={10} /> 状态切换会触发 NATS event star.worktree.*
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
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
