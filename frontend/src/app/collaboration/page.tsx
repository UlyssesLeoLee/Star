"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Boxes, MousePointer2, Image as ImageIcon } from "lucide-react";

export default function CollaborationPage() {
  const cursors = useStore((s) => s.presenceCursors);
  const boards = useStore((s) => s.whiteboards);
  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Collaboration"
        subtitle="实时协作域:Presence cursor(WS 推送) + Whiteboard snapshot。"
        icon={<Boxes className="text-accent" size={20} />}
        track="E"
        count={cursors.length}
      />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
        <div className="card">
          <SectionTitle><MousePointer2 size={11} className="inline mr-1" /> Live Presence</SectionTitle>
          <div className="relative bg-bg-soft/40 rounded border border-line h-64 overflow-hidden">
            {cursors.map((c) => (
              <div
                key={c.user_id}
                className="absolute transition-all duration-300"
                style={{ left: c.x, top: c.y }}
              >
                <MousePointer2 size={16} className="text-accent" />
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
          <div className="mt-2 text-[10px] text-ink-mute">
            WS 推送频率 ~10Hz · 字段 (x, y, selection, updated_at)
          </div>
        </div>

        <div className="card">
          <SectionTitle><ImageIcon size={11} className="inline mr-1" /> Whiteboards</SectionTitle>
          <div className="space-y-2">
            {boards.map((b) => (
              <div key={b.id} className="p-2 rounded border border-line bg-bg-soft/40">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-medium">{b.title}</span>
                  <span className="text-[10px] text-ink-mute font-mono">{b.id}</span>
                </div>
                <div className="text-[10px] text-ink-mute font-mono mb-1">
                  workspace={b.workspace_id} · {b.collaborator_ids.length} collaborators
                </div>
                <div className="flex items-center gap-2 text-[10px]">
                  <span className="text-ink-dim">Updated {new Date(b.updated_at).toLocaleString()}</span>
                  <a className="text-info hover:underline" href={b.snapshot_url}>snapshot →</a>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
