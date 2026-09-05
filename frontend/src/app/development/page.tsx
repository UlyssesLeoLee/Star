"use client";

// =====================================================================
// /development — 研发活动中心 (ChangeSets / Worktrees / Audit)
// =====================================================================

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Tabs } from "@/components/Tabs";
import { Hammer, Plus, Minus, Edit3, GitBranch, History, Shield } from "lucide-react";
import { CHANGESET_SM, type ChangeSetStatus } from "@/types/ids";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { useState } from "react";
import { useTranslation } from "@/lib/i18n";

export default function DevelopmentPage() {
  const { t } = useTranslation();
  const { changeSets, transitionChangeSet, worktrees, auditEvents } = useStore();
  const [tab, setTab] = useState<string>("changesets");
  const [selected, setSelected] = useState<string | null>("cs-003");
  const cs = changeSets.find((c) => c.id === selected);
  const allowed = cs ? CHANGESET_SM.transitions.filter((t) => t.from === cs.status).map((t) => t.to) : [];
  const devAuditEvents = auditEvents.filter((e) => e.category === "data_access" || e.category === "config_change").slice(0, 20);

  return (
    <div className="max-w-7xl mx-auto">
      <PageHeader
        title={t.pageTitles['/development'].title}
        subtitle="ChangeSets 代码变更、Worktree 沙箱与研发审计日志"
        icon={<Hammer className="text-accent" size={20} />}
        track="D"
        count={`${changeSets.length} changesets`}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="ChangeSets" value={changeSets.length} tone="info" />
        <Stat label="Worktrees" value={worktrees.length} tone="ok" />
        <Stat label="Merged" value={changeSets.filter((c) => c.status === "merged").length} tone="ok" />
        <Stat label="Draft" value={changeSets.filter((c) => c.status === "draft").length} tone="warn" />
      </div>

      <Tabs
        active={tab}
        onChange={setTab}
        items={[
          { id: "changesets", label: "ChangeSets 变更集",  icon: <Hammer size={12} />,    badge: changeSets.length },
          { id: "worktrees",  label: "Worktrees 沙箱",     icon: <GitBranch size={12} />, badge: worktrees.length },
          { id: "statemachine", label: "State Machine 状态机", icon: <Shield size={12} /> },
          { id: "audit",     label: "Dev Audit 研发审计",   icon: <History size={12} />,   badge: devAuditEvents.length },
        ]}
      />

      {tab === "changesets" && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-3" data-testid="tab-changesets">
          <div className="lg:col-span-2 card">
            <SectionTitle>ChangeSet 列表</SectionTitle>
            <table className="table mt-2">
              <thead>
                <tr><th>ID</th><th>Title</th><th>Worktree</th><th>Status</th><th>Diff</th></tr>
              </thead>
              <tbody>
                {changeSets.map((c) => (
                  <tr
                    key={c.id}
                    onClick={() => setSelected(c.id)}
                    className={`cursor-pointer transition-colors ${selected === c.id ? "bg-accent/5 border-l-2 border-accent" : "hover:bg-bg-soft/50"}`}
                  >
                    <td className="font-mono text-xs text-accent">{c.id}</td>
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
              <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5 font-mono">// SYMBOL INDEX</div>
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
              {allowed.length > 0 && (
                <>
                  <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5 font-mono">// TRANSITION</div>
                  <div className="flex flex-wrap gap-1.5">
                    {allowed.map((to) => (
                      <button key={to} onClick={() => transitionChangeSet(cs.id, to as ChangeSetStatus)} className="btn-primary">
                        → {to}
                      </button>
                    ))}
                  </div>
                </>
              )}
            </div>
          )}
        </div>
      )}

      {tab === "worktrees" && (
        <div className="card" data-testid="tab-worktrees">
          <SectionTitle>Active Worktrees 沙箱列表</SectionTitle>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 mt-3">
            {worktrees.map((w) => (
              <div key={w.id} className="p-3 rounded border border-line bg-bg-soft/40 hover:border-accent/40 transition-colors">
                <div className="flex items-center justify-between mb-1.5">
                  <span className="font-mono text-sm text-accent font-semibold">{w.id}</span>
                  <StatusPill value={w.status} size="xs" />
                </div>
                <div className="text-xs text-ink-dim mb-1">{w.branch || "—"}</div>
                <div className="grid grid-cols-2 gap-1 text-[10px] font-mono text-ink-mute">
                  <span>agent: {w.agent_session_id || "—"}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === "statemachine" && (
        <div className="card" data-testid="tab-statemachine">
          <SectionTitle>ChangeSet State Machine — 5 States</SectionTitle>
          <div className="mt-3">
            <StateMachineDiagram sm={CHANGESET_SM} highlightState={cs?.status} />
          </div>
        </div>
      )}

      {tab === "audit" && (
        <div className="card overflow-x-auto" data-testid="tab-audit">
          <SectionTitle>研发审计日志 (Dev Category)</SectionTitle>
          {devAuditEvents.length === 0 ? (
            <p className="text-xs text-ink-mute mt-3 font-mono">// No dev audit events in mock data</p>
          ) : (
            <table className="table mt-2">
              <thead>
                <tr><th>ID</th><th>Category</th><th>Action</th><th>Actor</th><th>When</th></tr>
              </thead>
              <tbody>
                {devAuditEvents.map((e) => (
                  <tr key={e.id}>
                    <td className="font-mono text-xs">{e.id}</td>
                    <td><StatusPill value={e.category} size="xs" /></td>
                    <td className="font-mono text-xs">{e.action}</td>
                    <td className="font-mono text-xs">{e.actor_id}</td>
                    <td className="text-ink-dim text-xs">{new Date(e.created_at).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
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


