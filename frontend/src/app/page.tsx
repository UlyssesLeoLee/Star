"use client";

import { useStore } from "@/lib/store";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { WORKTREE_SM, AGENT_SM, FEEDBACK_SM, PR_SM } from "@/types/ids";
import { Activity, Cpu, GitBranch, GitPullRequest, MessageCircleWarning, FileText, ShieldCheck, Zap, Users } from "lucide-react";
import Link from "next/link";

export default function Dashboard() {
  const s = useStore();

  // 状态分布
  const wtByStatus = s.worktrees.reduce<Record<string, number>>((acc, w) => {
    acc[w.status] = (acc[w.status] ?? 0) + 1; return acc;
  }, {});
  const agByStatus = s.agentSessions.reduce<Record<string, number>>((acc, a) => {
    acc[a.status] = (acc[a.status] ?? 0) + 1; return acc;
  }, {});
  const prByStatus = s.pullRequests.reduce<Record<string, number>>((acc, p) => {
    acc[p.status] = (acc[p.status] ?? 0) + 1; return acc;
  }, {});
  const fbByStatus = s.feedbacks.reduce<Record<string, number>>((acc, f) => {
    acc[f.status] = (acc[f.status] ?? 0) + 1; return acc;
  }, {});

  const activeAgents = s.agentSessions.filter((a) => !["completed","failed","cancelled"].includes(a.status)).length;
  const totalToken24h = s.agentSessions.reduce((sum, a) => sum + a.token_usage.total, 0);
  const totalCost24h = s.agentSessions.reduce((sum, a) => sum + a.cost_summary.usd, 0);
  const openFeedback = s.feedbacks.filter((f) => f.status === "open" || f.status === "acknowledged").length;
  const inFlightPRs = s.pullRequests.filter((p) => !["merged","closed"].includes(p.status)).length;
  const suppressedNotifs = s.notifications.filter((n) => n.status === "suppressed").length;

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Star Control Plane"
        subtitle="Vibe Coding Work Management SaaS — 25 Module, 5 状态机, 357 tests pass."
        icon={<Activity className="text-accent" size={20} />}
      />

      {/* Stat row */}
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-3 mb-6">
        <Stat label="Active Agents" value={activeAgents} hint={`${s.agentSessions.length} total`} tone="info" />
        <Stat label="Tokens (24h)" value={totalToken24h.toLocaleString()} hint="aggregated" />
        <Stat label="Cost (24h)" value={`$${totalCost24h.toFixed(2)}`} tone="warn" />
        <Stat label="In-flight PRs" value={inFlightPRs} hint={`${s.pullRequests.filter(p => p.status === "merged").length} merged`} tone="info" />
        <Stat label="Open Feedback" value={openFeedback} hint={`${s.feedbacks.length} total`} tone="warn" />
        <Stat label="Suppressed Notif" value={suppressedNotifs} hint="INV-N-07" tone="ok" />
      </div>

      {/* 5 状态机汇总 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mb-6">
        <StateSummaryCard
          title="Worktree"
          icon={<GitBranch size={14} />}
          href="/worktree"
          smName="17 状态机"
          distribution={wtByStatus}
          totalStates={WORKTREE_SM.states.length}
        />
        <StateSummaryCard
          title="Agent Session"
          icon={<Cpu size={14} />}
          href="/agent"
          smName="14 状态机"
          distribution={agByStatus}
          totalStates={AGENT_SM.states.length}
        />
        <StateSummaryCard
          title="Pull Request"
          icon={<GitPullRequest size={14} />}
          href="/scm"
          smName="7 状态机"
          distribution={prByStatus}
          totalStates={PR_SM.states.length}
        />
        <StateSummaryCard
          title="Feedback"
          icon={<MessageCircleWarning size={14} />}
          href="/feedback"
          smName="6 状态机"
          distribution={fbByStatus}
          totalStates={FEEDBACK_SM.states.length}
        />
      </div>

      {/* Recent activity */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
        <div className="card">
          <SectionTitle>Recent Agent Sessions</SectionTitle>
          <table className="table">
            <thead>
              <tr><th>ID</th><th>Status</th><th>Cost</th><th>Started</th></tr>
            </thead>
            <tbody>
              {s.agentSessions.slice(0, 6).map((a) => (
                <tr key={a.id}>
                  <td className="font-mono text-xs">{a.id}</td>
                  <td><StatusPill value={a.status} /></td>
                  <td className="font-mono text-xs">${a.cost_summary.usd.toFixed(2)}</td>
                  <td className="text-ink-dim text-xs">{new Date(a.started_at).toLocaleTimeString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="card">
          <SectionTitle>Recent Audit Events</SectionTitle>
          <table className="table">
            <thead>
              <tr><th>Category</th><th>Action</th><th>Actor</th><th>When</th></tr>
            </thead>
            <tbody>
              {s.auditEvents.slice(-6).reverse().map((a) => (
                <tr key={a.id}>
                  <td><StatusPill value={a.category} size="xs" /></td>
                  <td className="font-mono text-xs">{a.action}</td>
                  <td className="font-mono text-xs">{a.actor_id}</td>
                  <td className="text-ink-dim text-xs">{new Date(a.created_at).toLocaleTimeString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

function StateSummaryCard({
  title, icon, href, smName, distribution, totalStates,
}: {
  title: string;
  icon: React.ReactNode;
  href: string;
  smName: string;
  distribution: Record<string, number>;
  totalStates: number;
}) {
  const occupied = Object.keys(distribution).length;
  const entries = Object.entries(distribution).sort(([, a], [, b]) => b - a);
  return (
    <Link href={href} className="card block hover:border-accent/60 transition-colors">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div className="text-accent">{icon}</div>
          <div className="text-sm font-semibold">{title}</div>
          <span className="pill border-line text-ink-dim text-[10px] font-mono">{smName}</span>
        </div>
        <span className="text-[10px] text-ink-mute">
          {occupied} / {totalStates} states in use
        </span>
      </div>
      <div className="flex flex-wrap gap-1.5">
        {entries.map(([k, v]) => (
          <span key={k} className="inline-flex items-center gap-1 text-[11px]">
            <StatusPill value={k} size="xs" />
            <span className="font-mono text-ink-dim">×{v}</span>
          </span>
        ))}
      </div>
    </Link>
  );
}
