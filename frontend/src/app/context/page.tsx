"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { ListTodo, FileText, Code, History as HistoryIcon, Wrench, Brain, Check, X, Clock } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

const KIND_ICON = {
  spec: <FileText size={12} className="text-info" />,
  code: <Code size={12} className="text-ok" />,
  history: <HistoryIcon size={12} className="text-warn" />,
  tool: <Wrench size={12} className="text-ink-dim" />,
  decision: <Brain size={12} className="text-accent" />,
};

export default function ContextPage() {
  const { t } = useTranslation();
  const packets = useStore((s) => s.contextPackets);
  const decisions = useStore((s) => s.contextDecisions);

  const totalTokens = packets.reduce((s, p) => s + p.token_estimate, 0);

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/context'].title}
        subtitle="ContextPacket (5 字段: priority/kind/payload/provenance/decision) + Decision (3 状态) + INV-CT-01~10。token 预算决定单次 session 可加载量。"
        icon={<ListTodo className="text-accent" size={20} />}
        track="B"
        count={`${packets.length} packets / ${totalTokens.toLocaleString()} tokens`}
      />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mb-5">
        <div className="card">
          <SectionTitle>Context Packets</SectionTitle>
          <table className="table">
            <thead>
              <tr>
                <th>ID</th><th>Kind</th><th>Priority</th><th>Provenance</th>
                <th>Tokens</th><th>Decision</th>
              </tr>
            </thead>
            <tbody>
              {packets.map((p) => (
                <tr key={p.id}>
                  <td className="font-mono text-xs">{p.id}</td>
                  <td className="flex items-center gap-1.5">{KIND_ICON[p.kind]} {p.kind}</td>
                  <td>
                    <span className={`font-mono text-xs ${
                      p.priority === "p0" ? "text-err" :
                      p.priority === "p1" ? "text-warn" :
                      p.priority === "p2" ? "text-info" : "text-ink-dim"
                    }`}>{p.priority.toUpperCase()}</span>
                  </td>
                  <td className="text-xs text-ink-dim">{p.provenance}</td>
                  <td className="font-mono text-xs">{p.token_estimate.toLocaleString()}</td>
                  <td className="font-mono text-xs">{p.decision_id ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="card">
          <SectionTitle>Context Decisions (3 状态: pending/approved/rejected)</SectionTitle>
          <div className="space-y-2">
            {decisions.map((d) => (
              <div key={d.id} className="p-2 rounded border border-line bg-bg-soft/40">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-mono text-xs text-ink-mute">{d.id}</span>
                  <StatusPill value={d.status} size="xs" />
                  <span className="text-[10px] text-ink-mute font-mono ml-auto">{d.agent_session_id}</span>
                </div>
                <p className="text-sm">{d.prompt}</p>
                {d.chosen_option && (
                  <div className="mt-1.5 flex items-center gap-1.5 text-xs">
                    {d.status === "approved" ? <Check size={12} className="text-ok" /> : <X size={12} className="text-err" />}
                    <span className="text-ink-dim">{d.chosen_option}</span>
                    {d.decided_by && <span className="text-[10px] text-ink-mute font-mono ml-auto">by {d.decided_by}</span>}
                  </div>
                )}
                {d.status === "pending" && (
                  <div className="mt-1.5 flex items-center gap-1.5 text-xs text-warn">
                    <Clock size={12} /> 等待人类决策
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
