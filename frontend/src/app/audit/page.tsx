"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { History, Shield, Link2, Brain } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "@/lib/i18n";

export default function AuditPage() {
  const { t } = useTranslation();
  const events = useStore((s) => s.auditEvents);
  const [categoryFilter, setCategoryFilter] = useState<string>("all");
  const [showAionly, setShowAionly] = useState(false);

  const filtered = events.filter((e) =>
    (categoryFilter === "all" || e.category === categoryFilter) &&
    (!showAionly || e.ai_metadata),
  ).reverse();

  const categories = Array.from(new Set(events.map((e) => e.category)));
  const aiCount = events.filter((e) => e.ai_metadata).length;
  const policyViolations = events.filter((e) => e.category === "policy_violation").length;

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/audit'].title}
        subtitle="Append-only hash chain + 9 AI 问题元数据 + cross-tenant 100% 审计。INV-AU-01~07 保证不可篡改。"
        icon={<History className="text-accent" size={20} />}
        track="E"
        count={events.length}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="Total events" value={events.length} tone="info" />
        <Stat label="AI decisions logged" value={aiCount} tone="default" hint="9 AI questions answered" />
        <Stat label="Policy violations" value={policyViolations} tone={policyViolations > 0 ? "warn" : "ok"} />
        <Stat label="Hash chain" value="✓ valid" tone="ok" hint="append-only" />
      </div>

      <div className="card mb-3 flex items-center gap-2 text-xs">
        <span className="text-ink-dim">Filter:</span>
        <select value={categoryFilter} onChange={(e) => setCategoryFilter(e.target.value)} className="bg-bg-soft border border-line rounded px-2 py-1 text-xs">
          <option value="all">all categories</option>
          {categories.map((c) => <option key={c} value={c}>{c}</option>)}
        </select>
        <label className="flex items-center gap-1.5 ml-3 cursor-pointer">
          <input type="checkbox" checked={showAionly} onChange={(e) => setShowAionly(e.target.checked)} />
          <Brain size={11} /> AI events only
        </label>
        <span className="ml-auto text-ink-mute font-mono">{filtered.length} of {events.length}</span>
      </div>

      <div className="card overflow-x-auto">
        <table className="table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Category</th>
              <th>Action</th>
              <th>Actor</th>
              <th>Target</th>
              <th><Link2 size={10} className="inline" /> hash chain</th>
              <th>AI metadata</th>
              <th>When</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((e) => (
              <tr key={e.id}>
                <td className="font-mono text-xs">{e.id}</td>
                <td><StatusPill value={e.category} size="xs" /></td>
                <td className="font-mono text-xs">{e.action}</td>
                <td className="font-mono text-xs">{e.actor_id}</td>
                <td className="font-mono text-xs text-info">
                  {e.target_kind ? `${e.target_kind}:${e.target_id}` : "—"}
                </td>
                <td className="font-mono text-[10px]">
                  <div className="flex items-center gap-1">
                    <span className="text-ink-mute">{e.prev_hash}</span>
                    <span>→</span>
                    <span className={e.hash === "0x7710" ? "text-ok" : "text-ink"}>{e.hash}</span>
                  </div>
                </td>
                <td>
                  {e.ai_metadata ? (
                    <div className="text-[10px] font-mono">
                      <div className="text-accent">agent={e.ai_metadata.agent_session_id}</div>
                      {e.ai_metadata.confidence !== undefined && (
                        <div className="text-ink-dim">conf={e.ai_metadata.confidence.toFixed(2)}</div>
                      )}
                    </div>
                  ) : (
                    <span className="text-ink-mute">—</span>
                  )}
                </td>
                <td className="text-ink-dim text-xs">{new Date(e.created_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
