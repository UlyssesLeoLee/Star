"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { GitFork, GitPullRequest } from "lucide-react";
import { PR_SM, type PullRequestStatus } from "@/types/ids";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { useState } from "react";
import { useTranslation } from "@/lib/i18n";

export default function ScmPage() {
  const { t } = useTranslation();
  const repos = useStore((s) => s.repositories);
  const prs = useStore((s) => s.pullRequests);
  const { transitionPR } = useStore();
  const [selected, setSelected] = useState<string | null>("pr-003");
  const pr = prs.find((p) => p.id === selected);
  const allowed = pr ? PR_SM.transitions.filter((t) => t.from === pr.status).map((t) => t.to) : [];

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/scm'].title}
        subtitle="Repository / Branch / Commit / PR + 7 状态机 (§7.5) + Webhook Idempotency-Key。SCM 是 ACL 边界,所有 git 操作经 domain-scm。"
        icon={<GitFork className="text-accent" size={20} />}
        track="C"
        count={`${repos.length} repos / ${prs.length} PRs`}
      />

      <div className="mb-5">
        <StateMachineDiagram sm={PR_SM} highlightState={pr?.status} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-3 mb-5">
        <div className="lg:col-span-2 card">
          <SectionTitle>Pull Requests</SectionTitle>
          <table className="table">
            <thead>
              <tr>
                <th>PR</th><th>Title</th><th>Repo</th><th>Source → Target</th>
                <th>Status</th><th>Review</th><th>CI</th>
              </tr>
            </thead>
            <tbody>
              {prs.map((p) => {
                const repo = repos.find((r) => r.id === p.repository_id);
                return (
                  <tr key={p.id} onClick={() => setSelected(p.id)} className="cursor-pointer">
                    <td className="font-mono text-xs">{repo?.provider}#{p.number}</td>
                    <td className="font-medium">{p.title}</td>
                    <td className="font-mono text-xs text-info">{repo?.full_name}</td>
                    <td className="font-mono text-xs">{p.source_branch} → {p.target_branch}</td>
                    <td><StatusPill value={p.status} size="xs" /></td>
                    <td><StatusPill value={p.review_state} size="xs" /></td>
                    <td><StatusPill value={p.ci_state} size="xs" /></td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {pr && (
          <div className="card sticky top-16">
            <div className="flex items-center justify-between mb-3">
              <div>
                <div className="text-xs text-ink-mute font-mono">{pr.id} · #{pr.number}</div>
                <div className="text-base font-semibold">{pr.title}</div>
              </div>
              <StatusPill value={pr.status} />
            </div>
            <dl className="text-xs space-y-1.5 mb-3">
              <Row label="Author" value={pr.author_id} />
              <Row label="Branch" value={<span className="font-mono text-info">{pr.source_branch}</span>} />
              <Row label="→ Target" value={<span className="font-mono">{pr.target_branch}</span>} />
              <Row label="Review" value={<StatusPill value={pr.review_state} size="xs" />} />
              <Row label="CI" value={<StatusPill value={pr.ci_state} size="xs" />} />
              {pr.merged_at && <Row label="Merged" value={new Date(pr.merged_at).toLocaleString()} />}
            </dl>
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5">Transition</div>
            <div className="flex flex-wrap gap-1.5">
              {allowed.map((to) => (
                <button key={to} onClick={() => transitionPR(pr.id, to as PullRequestStatus)} className="btn-primary">→ {to}</button>
              ))}
            </div>
          </div>
        )}
      </div>

      <SectionTitle>Repositories</SectionTitle>
      <div className="card">
        <table className="table">
          <thead>
            <tr>
              <th>ID</th><th>Provider</th><th>Full name</th><th>Default</th>
              <th>Webhook idempotency key</th><th>Last event</th>
            </tr>
          </thead>
          <tbody>
            {repos.map((r) => (
              <tr key={r.id}>
                <td className="font-mono text-xs">{r.id}</td>
                <td><StatusPill value={r.provider} size="xs" /></td>
                <td className="font-mono text-info">{r.full_name}</td>
                <td className="font-mono text-xs">{r.default_branch}</td>
                <td className="font-mono text-xs text-warn">{r.webhook_idempotency_key ?? "—"}</td>
                <td className="text-ink-dim text-xs">{r.last_event_at ? new Date(r.last_event_at).toLocaleString() : "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
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
