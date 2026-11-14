/*
@cypher schema=1 source_sha256=c5f6a43da1e091947462f3a767029d65c04b8cf4eb6a5d861542805f70c950ea
MERGE (self:File {path:"frontend/src/app/(app)/review/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/review/page.tsx::ReviewPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /review — ReviewRecord 列表 (per requirements §27.4 RVW-001/002 + ui-3pane-arch §1.2 核心区)
// 落地页 = ReviewRecord 列表按 Status 分组 (IN_PROGRESS 待审置顶)
// Kind: SelfReview | CrossReview | AgentAssistedReview
// =====================================================================
"use client";

import { useMemo } from "react";
import { ShieldCheck, UserRound, Bot, Clock } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

export type ReviewStatus = "IN_PROGRESS" | "APPROVED" | "CHANGES_REQUESTED" | "REJECTED" | "SUPERSEDED";
export type ReviewKind = "SelfReview" | "CrossReview" | "AgentAssistedReview";

export interface ReviewRecord {
  id: string;
  worktreeId: string;
  workItemId: string;
  target: string;
  kind: ReviewKind;
  author: string;
  reviewer: string;
  decision: "Approve" | "RequestChanges" | "Reject" | "";
  status: ReviewStatus;
  startedAt: string;
  completedAt?: string;
}

// mock 数据 (前端现状: 全 mock; 后端 ReviewRecord 域待 P3)
const MOCK_REVIEWS: ReviewRecord[] = [
  {
    id: "rvw-001",
    worktreeId: "wt-physis-1",
    workItemId: "wi-2201",
    target: "ChangeSet cs-88a1",
    kind: "CrossReview",
    author: "usr-arch",
    reviewer: "usr-pm",
    decision: "RequestChanges",
    status: "CHANGES_REQUESTED",
    startedAt: "2026-11-13 09:12",
    completedAt: "2026-11-13 10:40",
  },
  {
    id: "rvw-002",
    worktreeId: "wt-physis-2",
    workItemId: "wi-2204",
    target: "ChangeSet cs-91b2",
    kind: "SelfReview",
    author: "usr-dev-1",
    reviewer: "usr-dev-1",
    decision: "",
    status: "IN_PROGRESS",
    startedAt: "2026-11-14 08:05",
  },
  {
    id: "rvw-003",
    worktreeId: "wt-stargate-1",
    workItemId: "wi-1190",
    target: "DesignArtifact da-001",
    kind: "CrossReview",
    author: "usr-arch",
    reviewer: "usr-lead",
    decision: "Approve",
    status: "APPROVED",
    startedAt: "2026-11-12 14:20",
    completedAt: "2026-11-12 15:01",
  },
  {
    id: "rvw-004",
    worktreeId: "wt-physis-3",
    workItemId: "wi-2210",
    target: "ChangeSet cs-93c4",
    kind: "AgentAssistedReview",
    author: "ag-003",
    reviewer: "ag-005",
    decision: "Reject",
    status: "REJECTED",
    startedAt: "2026-11-13 16:30",
    completedAt: "2026-11-13 16:58",
  },
  {
    id: "rvw-005",
    worktreeId: "wt-mobile-1",
    workItemId: "wi-0802",
    target: "ChangeSet cs-77d9",
    kind: "SelfReview",
    author: "usr-dev-2",
    reviewer: "usr-dev-2",
    decision: "",
    status: "SUPERSEDED",
    startedAt: "2026-11-11 10:00",
    completedAt: "2026-11-11 11:30",
  },
];

const STATUS_ORDER: ReviewStatus[] = ["IN_PROGRESS", "CHANGES_REQUESTED", "APPROVED", "REJECTED", "SUPERSEDED"];

export default function ReviewPage() {
  const { t } = useTranslation();

  const grouped = useMemo(() => {
    const map = new Map<ReviewStatus, ReviewRecord[]>();
    for (const s of STATUS_ORDER) map.set(s, []);
    for (const r of MOCK_REVIEWS) map.get(r.status)?.push(r);
    return [...map.entries()].filter(([, list]) => list.length > 0);
  }, []);

  return (
    <div className="mx-auto max-w-5xl p-6">
      <header className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight text-ink">{t.review.title}</h1>
        <p className="mt-1 text-sm text-ink-dim">{t.review.subtitle}</p>
      </header>

      {grouped.map(([status, items]) => (
        <section key={status} className="mb-6" data-testid={`review-group-${status.toLowerCase()}`}>
          <div className="mb-2 flex items-center gap-2">
            <span
              className="px-2 py-0.5 rounded text-[11px] font-mono font-bold border"
              data-status={status}
            >
              {t.review.status[status]}
            </span>
            <span className="text-xs text-ink-mute font-mono">{items.length}</span>
          </div>
          <ul className="space-y-2">
            {items.map((r) => (
              <li
                key={r.id}
                data-testid={`review-item-${r.id}`}
                className="rounded-xl border border-line bg-bg-card p-4 flex items-start gap-3"
              >
                <div className="size-8 rounded-lg grid place-items-center border border-cyan-500/40 bg-cyan-500/15 text-cyan-400 shrink-0">
                  {r.kind === "AgentAssistedReview" ? <Bot size={15} /> : <UserRound size={15} />}
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-semibold text-ink text-sm">{r.id}</span>
                    <span className="text-[10px] font-mono px-1.5 py-0.5 rounded border border-line/60 text-ink-mute">
                      {r.kind}
                    </span>
                    {r.decision && (
                      <span className="text-[10px] font-mono px-1.5 py-0.5 rounded border border-line/60 text-ink-dim">
                        {r.decision}
                      </span>
                    )}
                  </div>
                  <div className="mt-1 text-xs text-ink-dim">
                    {r.target} · {r.worktreeId} · {r.workItemId}
                  </div>
                  <div className="mt-1 text-xs text-ink-mute flex items-center gap-3">
                    <span>
                      {t.review.author}: {r.author}
                    </span>
                    <span>
                      {t.review.reviewer}: {r.reviewer}
                    </span>
                    <span className="flex items-center gap-1">
                      <Clock size={11} /> {r.startedAt}
                      {r.completedAt ? ` → ${r.completedAt}` : ""}
                    </span>
                  </div>
                </div>
                <ShieldCheck size={16} className="text-ink-mute shrink-0" aria-hidden="true" />
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
