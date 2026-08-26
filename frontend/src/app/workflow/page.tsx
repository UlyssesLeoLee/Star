"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Workflow, ArrowRight } from "lucide-react";

export default function WorkflowPage() {
  const workflows = useStore((s) => s.workflows);
  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Workflows"
        subtitle="状态机配置:每个 project 可绑定一个 workflow。state.category 决定 work-item status 字段;transition.trigger + guard(CEL) 控制迁移。"
        icon={<Workflow className="text-accent" size={20} />}
        track="D"
        count={workflows.length}
      />
      <div className="grid grid-cols-1 gap-4">
        {workflows.map((wf) => (
          <div key={wf.id} className="card">
            <div className="flex items-center justify-between mb-3">
              <div>
                <div className="text-sm font-semibold">{wf.name}</div>
                <div className="text-xs text-ink-mute font-mono">{wf.id} · project {wf.project_id}</div>
              </div>
              <div className="flex items-center gap-2">
                {wf.is_default && <span className="pill border-accent/40 text-accent bg-accent/10 text-[10px]">default</span>}
                <span className="text-xs text-ink-dim font-mono">{wf.states.length} states · {wf.transitions.length} transitions</span>
              </div>
            </div>

            {/* State flow */}
            <div className="flex items-center gap-1 overflow-x-auto pb-2">
              {wf.states.map((s, i) => (
                <div key={s.id} className="flex items-center gap-1 shrink-0">
                  <div className={`px-3 py-2 rounded-md border text-xs font-mono ${
                    s.kind === "initial" ? "border-accent/40 bg-accent/10 text-accent" :
                    s.kind === "final"   ? "border-ok/40 bg-ok/10 text-ok" :
                    "border-line bg-bg-soft text-ink"
                  }`}>
                    <div className="font-semibold">{s.name}</div>
                    <div className="text-[9px] text-ink-mute">{s.category}</div>
                  </div>
                  {i < wf.states.length - 1 && <ArrowRight size={12} className="text-ink-mute" />}
                </div>
              ))}
            </div>

            {/* Transitions */}
            <div className="mt-3 pt-3 border-t border-line">
              <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-2">Transitions</div>
              <table className="table">
                <thead><tr><th>From</th><th>To</th><th>Trigger</th><th>Guard</th></tr></thead>
                <tbody>
                  {wf.transitions.map((t, i) => {
                    const from = wf.states.find((s) => s.id === t.from_state_id);
                    const to = wf.states.find((s) => s.id === t.to_state_id);
                    return (
                      <tr key={i}>
                        <td><StatusPill value={from?.category ?? "?"} size="xs" /></td>
                        <td><StatusPill value={to?.category ?? "?"} size="xs" /></td>
                        <td className="font-mono text-xs">{t.trigger}</td>
                        <td className="font-mono text-xs text-ink-dim">{t.guard ?? "—"}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
