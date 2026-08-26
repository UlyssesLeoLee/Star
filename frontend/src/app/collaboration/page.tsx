"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Boxes, Users, FileText, Plus, MousePointer2 } from "lucide-react";
import Link from "next/link";

export default function CollaborationPage() {
  const canvases = useStore((s) => s.canvases);
  const canvasElements = useStore((s) => s.canvasElements);
  const canvasConnectors = useStore((s) => s.canvasConnectors);
  const presenceCursors = useStore((s) => s.presenceCursors);
  const whiteboards = useStore((s) => s.whiteboards);
  const worktrees = useStore((s) => s.worktrees);
  const agentSessions = useStore((s) => s.agentSessions);
  const feedbacks = useStore((s) => s.feedbacks);

  const activeCursors = presenceCursors.filter((c) => {
    const ageMs = Date.now() - new Date(c.updated_at).getTime();
    return ageMs < 30_000;  // 30s 内算 active
  });

  // canvas 缩略图统计
  const canvasStats = canvases.map((c) => {
    const elements = canvasElements.filter((e) => e.canvas_id === c.id);
    const connectors = canvasConnectors.filter((cn) => cn.canvas_id === c.id);
    const workItemCount = elements.filter((e) => e.kind === "work_item_card").length;
    const worktreeCount = elements.filter((e) => e.kind === "worktree_node").length;
    const agentCount = elements.filter((e) => e.kind === "agent_cursor").length;
    return { ...c, elementCount: elements.length, connectorCount: connectors.length, workItemCount, worktreeCount, agentCount };
  });

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Collaboration"
        subtitle="无限画布(Miro 模式)与实时协作入口。每个 canvas 可关联 Worktree / WorkItem / Project,跨 25 module 双向联动。"
        icon={<Boxes className="text-accent" size={20} />}
        track="E"
        count={canvases.length}
      />

      {/* 实时状态 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat icon={<Users size={12} />} label="Active cursors" value={activeCursors.length} tone="info" />
        <Stat icon={<Boxes size={12} />} label="Canvases" value={canvases.length} tone="default" />
        <Stat icon={<FileText size={12} />} label="Whiteboards (legacy)" value={whiteboards.length} tone="default" hint="snapshot 模式,逐步迁移到 canvas" />
        <Stat icon={<MousePointer2 size={12} />} label="Online users" value={3} tone="ok" hint="3 users in this workspace" />
      </div>

      {/* 画布列表 */}
      <SectionTitle action={
        <button className="btn-primary">
          <Plus size={12} /> New Canvas
        </button>
      }>Infinite Canvases</SectionTitle>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 mb-5">
        {canvasStats.map((c) => (
          <Link
            key={c.id}
            href={`/canvas/${c.id}`}
            className="card block hover:border-accent/60 transition-colors"
          >
            {/* 缩略图(占位,实际是 canvas mini 渲染) */}
            <div className="relative h-32 rounded border border-line bg-bg-soft overflow-hidden mb-3">
              <div className="absolute inset-0 grid grid-cols-3 gap-1 p-2 text-[9px] font-mono text-ink-mute">
                <div className="border border-line rounded flex items-center justify-center text-center">
                  worktree<br />×{c.worktreeCount}
                </div>
                <div className="border border-line rounded flex items-center justify-center text-center">
                  work-item<br />×{c.workItemCount}
                </div>
                <div className="border border-line rounded flex items-center justify-center text-center">
                  agent<br />×{c.agentCount}
                </div>
              </div>
            </div>

            <div className="flex items-center justify-between mb-1">
              <div className="text-sm font-semibold truncate">{c.title}</div>
              {c.ref_kind && (
                <span className="pill border-line text-ink-dim text-[10px] font-mono">
                  ref: {c.ref_kind}
                </span>
              )}
            </div>
            <div className="text-[10px] text-ink-mute font-mono mb-2">
              {c.elementCount} elements / {c.connectorCount} connectors
            </div>
            <div className="text-[10px] text-ink-dim font-mono">
              {c.collaborator_ids.length} collaborators · updated {new Date(c.updated_at).toLocaleString()}
            </div>
          </Link>
        ))}
      </div>

      {/* 旧 Whiteboard(保留,逐步迁移) */}
      <SectionTitle>Legacy Whiteboards (snapshot 模式)</SectionTitle>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-5">
        {whiteboards.map((wb) => (
          <div key={wb.id} className="card">
            <div className="text-sm font-semibold mb-1">{wb.title}</div>
            <div className="text-[10px] text-ink-mute font-mono mb-2">
              workspace {wb.workspace_id}
            </div>
            <div className="text-[10px] text-ink-dim font-mono">
              {wb.collaborator_ids.length} collaborators · updated {new Date(wb.updated_at).toLocaleString()}
            </div>
            {wb.snapshot_url && (
              <div className="mt-2">
                <img src={wb.snapshot_url} alt={wb.title} className="w-full h-20 object-cover rounded border border-line opacity-60" />
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Active cursors 实时展示 */}
      <SectionTitle>Live Presence (in this workspace)</SectionTitle>
      <div className="card">
        <div className="relative h-32 bg-bg-soft/40 rounded border border-line">
          {activeCursors.map((c) => (
            <div
              key={c.user_id}
              className="absolute transition-all duration-300"
              style={{ left: c.x, top: c.y }}
            >
              <MousePointer2 size={14} className="text-accent" />
              <div className="ml-3 -mt-2 inline-block bg-accent text-white text-[10px] px-1.5 py-0.5 rounded">
                {c.user_id}
              </div>
              {c.selection && (
                <div className="ml-3 mt-0.5 text-[10px] text-ink-dim font-mono whitespace-nowrap bg-bg-card/80 px-1.5 py-0.5 rounded">
                  {c.selection}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function Stat({ icon, label, value, hint, tone }: { icon: React.ReactNode; label: string; value: string | number; hint?: string; tone?: "ok" | "warn" | "err" | "info" | "default" }) {
  const color = {
    ok: "text-ok",
    warn: "text-warn",
    err: "text-err",
    info: "text-info",
    default: "text-ink",
  }[tone ?? "default"];
  return (
    <div className="card">
      <div className="flex items-center gap-1.5 text-[10px] uppercase tracking-wider text-ink-mute">
        {icon}{label}
      </div>
      <div className={`text-2xl font-semibold mt-0.5 font-mono ${color}`}>{value}</div>
      {hint && <div className="text-[11px] text-ink-mute mt-0.5">{hint}</div>}
    </div>
  );
}
