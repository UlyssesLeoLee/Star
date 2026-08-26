"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Hammer, Plus, Minus, Edit3 } from "lucide-react";
import { CHANGESET_SM, type ChangeSetStatus } from "@/types/ids";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { useState } from "react";

export default function DevelopmentPage() {
  const { changeSets, transitionChangeSet } = useStore();
  const [selected, setSelected] = useState<string | null>("cs-003");
  const cs = changeSets.find((c) => c.id === selected);
  const allowed = cs ? CHANGESET_SM.transitions.filter((t) => t.from === cs.status).map((t) => t.to) : [];

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Development"
        subtitle="ChangeSet 5 状态机(draft/applied/merged/abandoned/reverted) + INV-DEV-01~05。Symbol index 跟踪 +/- 修改的符号。"
        icon={<Hammer className="text-accent" size={20} />}
        track="D"
        count={changeSets.length}
      />

      <div className="mb-5">
        <StateMachineDiagram sm={CHANGESET_SM} highlightState={cs?.status} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <div className="lg:col-span-2 card">
          <table className="table">
            <thead>
              <tr><th>ID</th><th>Title</th><th>Worktree</th><th>Status</th><th>Diff</th></tr>
            </thead>
            <tbody>
              {changeSets.map((c) => (
                <tr key={c.id} onClick={() => setSelected(c.id)} className="cursor-pointer">
                  <td className="font-mono text-xs">{c.id}</td>
                  <td className="font-medium">{c.title}</td>
                  <td className="font-mono text-xs text-info">{c.worktree_id}</td>
                  <td><StatusPill value={c.status} size="xs" /></td>
                  <td className="font-mono text-xs text-ink-dim">{c.diff_summary}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {cs && (
          <div className="card sticky top-16">
            <div className="flex items-center justify-between mb-3">
              <div>
                <div className="text-xs text-ink-mute font-mono">{cs.id}</div>
                <div className="text-base font-semibold">{cs.title}</div>
              </div>
              <StatusPill value={cs.status} />
            </div>
            <dl className="text-xs space-y-1.5 mb-3">
              <Row label="Worktree" value={<span className="font-mono text-info">{cs.worktree_id}</span>} />
              <Row label="Work item" value={cs.work_item_id} />
              <Row label="Author" value={cs.author_id} />
              <Row label="Created" value={new Date(cs.created_at).toLocaleString()} />
            </dl>
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5">Symbol index</div>
            <div className="grid grid-cols-3 gap-2 mb-4">
              <div className="text-center p-2 rounded border border-ok/30 bg-ok/5">
                <Plus size={10} className="inline text-ok" />
                <div className="text-lg font-mono text-ok">{cs.symbol_index.added}</div>
                <div className="text-[10px] text-ink-mute">added</div>
              </div>
              <div className="text-center p-2 rounded border border-info/30 bg-info/5">
                <Edit3 size={10} className="inline text-info" />
                <div className="text-lg font-mono text-info">{cs.symbol_index.modified}</div>
                <div className="text-[10px] text-ink-mute">modified</div>
              </div>
              <div className="text-center p-2 rounded border border-err/30 bg-err/5">
                <Minus size={10} className="inline text-err" />
                <div className="text-lg font-mono text-err">{cs.symbol_index.removed}</div>
                <div className="text-[10px] text-ink-mute">removed</div>
              </div>
            </div>
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5">Transition</div>
            <div className="flex flex-wrap gap-1.5">
              {allowed.map((to) => (
                <button key={to} onClick={() => transitionChangeSet(cs.id, to as ChangeSetStatus)} className="btn-primary">→ {to}</button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex justify-between">
      <dt className="text-ink-mute">{label}</dt>
      <dd className="text-ink font-mono text-[11px]">{value}</dd>
    </div>
  );
}
