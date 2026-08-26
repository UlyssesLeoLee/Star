"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Network, ArrowRight } from "lucide-react";

export default function RelationPage() {
  const relations = useStore((s) => s.relations);
  const wts = useStore((s) => s.worktrees);
  const wis = useStore((s) => s.workItems);
  const ags = useStore((s) => s.agentSessions);
  const css = useStore((s) => s.changeSets);

  const lookup = (kind: string, id: string): string => {
    if (kind === "work_item") return wis.find((w) => w.id === id)?.key ?? id;
    if (kind === "worktree") return wts.find((w) => w.id === id)?.name ?? id;
    if (kind === "agent_session") return id;
    if (kind === "changeset") return id;
    return id;
  };

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Relation"
        subtitle="跨实体关系图(blocks / duplicates / relates_to / parent_of / cloned_from)。BFS 5 层 + 2 层 prefetch buffer。"
        icon={<Network className="text-accent" size={20} />}
        track="E"
        count={relations.length}
      />

      <div className="card">
        <table className="table">
          <thead>
            <tr>
              <th>From</th><th>Kind</th><th></th><th>To</th><th>Created</th>
            </tr>
          </thead>
          <tbody>
            {relations.map((r) => (
              <tr key={r.id}>
                <td>
                  <div className="font-mono text-xs">
                    <StatusPill value={r.from_kind} size="xs" />
                    <span className="ml-2 text-info">{lookup(r.from_kind, r.from_id)}</span>
                  </div>
                </td>
                <td>
                  <span className={`pill font-mono text-[10px] ${
                    r.kind === "blocks" ? "border-err/40 text-err bg-err/10" :
                    r.kind === "duplicates" ? "border-warn/40 text-warn bg-warn/10" :
                    r.kind === "cloned_from" ? "border-info/40 text-info bg-info/10" :
                    "border-line text-ink-dim"
                  }`}>
                    {r.kind}
                  </span>
                </td>
                <td><ArrowRight size={12} className="text-ink-mute" /></td>
                <td>
                  <div className="font-mono text-xs">
                    <StatusPill value={r.to_kind} size="xs" />
                    <span className="ml-2 text-info">{lookup(r.to_kind, r.to_id)}</span>
                  </div>
                </td>
                <td className="text-ink-dim text-xs">{new Date(r.created_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
