"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Trello, AlertTriangle } from "lucide-react";

export default function BoardPage() {
  const board = useStore((s) => s.board);
  const workItems = useStore((s) => s.workItems);
  const lookup = Object.fromEntries(workItems.map((w) => [w.id, w]));

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Board"
        subtitle="Kanban 视图 + WIP 限制。每列定义 status + 可选 wip_limit;超限时高亮。"
        icon={<Trello className="text-accent" size={20} />}
        track="E"
        count={`${board.columns.reduce((s, c) => s + c.work_item_ids.length, 0)} cards`}
      />

      <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
        {board.columns.map((col) => {
          const overWip = col.wip_limit !== undefined && col.wip_limit < 99 && col.work_item_ids.length > col.wip_limit;
          return (
            <div key={col.status} className={`card ${overWip ? "border-warn/60" : ""}`}>
              <div className="flex items-center justify-between mb-3">
                <StatusPill value={col.status} />
                <span className="text-[10px] text-ink-mute font-mono">
                  {col.work_item_ids.length}
                  {col.wip_limit !== undefined && col.wip_limit < 99 && ` / ${col.wip_limit}`}
                </span>
              </div>
              {overWip && (
                <div className="mb-2 text-[10px] text-warn flex items-center gap-1">
                  <AlertTriangle size={10} /> WIP 超过限制
                </div>
              )}
              <div className="space-y-2">
                {col.work_item_ids.map((id) => {
                  const w = lookup[id];
                  if (!w) return null;
                  const pColor =
                    w.priority === "p0" ? "border-l-err" :
                    w.priority === "p1" ? "border-l-warn" :
                    w.priority === "p2" ? "border-l-info" : "border-l-ink-mute";
                  return (
                    <div key={id} className={`p-2 rounded border border-line border-l-2 ${pColor} bg-bg-soft/60 hover:bg-bg-soft transition-colors`}>
                      <div className="flex items-center justify-between mb-1">
                        <span className="font-mono text-[10px] text-info">{w.key}</span>
                        <span className="font-mono text-[10px] text-ink-mute">{w.story_points ?? "—"}sp</span>
                      </div>
                      <div className="text-xs line-clamp-2">{w.title}</div>
                      {w.labels.length > 0 && (
                        <div className="mt-1 flex flex-wrap gap-1">
                          {w.labels.slice(0, 2).map((l) => <span key={l} className="text-[9px] text-ink-mute">#{l}</span>)}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
