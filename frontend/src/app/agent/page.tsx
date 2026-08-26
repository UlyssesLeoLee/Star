"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Bot, Cpu, DollarSign, Hash } from "lucide-react";
import { AGENT_SM, type AgentStatus } from "@/types/ids";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { useState } from "react";

export default function AgentPage() {
  const { agentSessions, transitionAgent } = useStore();
  const [selected, setSelected] = useState<string | null>("ag-003");
  const ag = agentSessions.find((a) => a.id === selected);
  const allowed = ag ? AGENT_SM.transitions.filter((t) => t.from === ag.status).map((t) => t.to) : [];

  const activeCount = agentSessions.filter((a) => !["completed","failed","cancelled"].includes(a.status)).length;
  const totalCost = agentSessions.reduce((s, a) => s + a.cost_summary.usd, 0);

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Agent Sessions"
        subtitle="14 状态机 (§7.4) + INV-AGT-N01~N14。每个 session 绑 worktree + 1 种 agent kind + token/cost 预算。"
        icon={<Bot className="text-accent" size={20} />}
        track="B"
        count={`${activeCount} active / ${agentSessions.length}`}
      />

      <div className="mb-5">
        <StateMachineDiagram sm={AGENT_SM} highlightState={ag?.status} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <div className="lg:col-span-2 card">
          <table className="table">
            <thead>
              <tr>
                <th>ID</th><th>Kind</th><th>Status</th><th>Step</th>
                <th>Tokens</th><th>Cost</th>
              </tr>
            </thead>
            <tbody>
              {agentSessions.map((a) => {
                const pct = a.cost_summary.budget_usd > 0 ? Math.round((a.cost_summary.usd / a.cost_summary.budget_usd) * 100) : 0;
                return (
                  <tr key={a.id} onClick={() => setSelected(a.id)} className="cursor-pointer">
                    <td className="font-mono text-xs">{a.id}</td>
                    <td><span className="font-mono text-xs text-info">{a.agent_kind}</span></td>
                    <td><StatusPill value={a.status} size="xs" /></td>
                    <td className="font-mono text-xs text-ink-dim">{a.current_step}</td>
                    <td className="font-mono text-xs">{a.token_usage.total.toLocaleString()}</td>
                    <td className="font-mono text-xs">
                      <span className={pct > 80 ? "text-warn" : "text-ink"}>
                        ${a.cost_summary.usd.toFixed(2)} <span className="text-ink-mute">/ ${a.cost_summary.budget_usd.toFixed(2)}</span>
                      </span>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {ag && (
          <div className="card sticky top-16">
            <div className="flex items-center justify-between mb-3">
              <div>
                <div className="text-xs text-ink-mute font-mono">{ag.id}</div>
                <div className="text-base font-semibold">{ag.agent_kind}</div>
              </div>
              <StatusPill value={ag.status} />
            </div>
            <dl className="text-xs space-y-1.5 mb-3">
              <Row label={<><Cpu size={10} className="inline mr-1" />Step</>} value={ag.current_step} />
              <Row label="Worktree" value={<span className="font-mono text-info">{ag.worktree_id}</span>} />
              <Row label={<><Hash size={10} className="inline mr-1" />Tokens</>} value={`${ag.token_usage.total.toLocaleString()} (in ${ag.token_usage.input.toLocaleString()} / out ${ag.token_usage.output.toLocaleString()})`} />
              <Row label={<><DollarSign size={10} className="inline mr-1" />Cost</>} value={`$${ag.cost_summary.usd.toFixed(2)} / $${ag.cost_summary.budget_usd.toFixed(2)}`} />
              <Row label="Started" value={new Date(ag.started_at).toLocaleString()} />
              {ag.ended_at && <Row label="Ended" value={new Date(ag.ended_at).toLocaleString()} />}
            </dl>
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5">Transition</div>
            <div className="flex flex-wrap gap-1.5">
              {allowed.map((to) => (
                <button key={to} onClick={() => transitionAgent(ag.id, to as AgentStatus)} className="btn-primary">→ {to}</button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function Row({ label, value }: { label: React.ReactNode; value: React.ReactNode }) {
  return (
    <div className="flex justify-between">
      <dt className="text-ink-mute">{label}</dt>
      <dd className="text-ink font-mono text-[11px]">{value}</dd>
    </div>
  );
}
