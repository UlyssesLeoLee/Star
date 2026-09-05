"use client";

import { useState, useMemo } from "react";
import { useStore } from "@/lib/store";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Tabs, type TabItem } from "@/components/Tabs";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { MessageCircleWarning, HelpCircle, CheckCircle2, XCircle, AlertCircle, ClipboardCheck, GitBranch, ArrowRight } from "lucide-react";
import { FEEDBACK_SM, type FeedbackStatus } from "@/types/ids";
import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";

type FeedbackTab = "inbox" | "statemachine" | "history";

const TABS: TabItem[] = [
  { id: "inbox", label: "Open Feedback", icon: <HelpCircle size={13} /> },
  { id: "statemachine", label: "State Machine (6-State)", icon: <GitBranch size={13} /> },
  { id: "history", label: "Review History", icon: <ClipboardCheck size={13} /> },
];

const severityIcon: Record<string, React.ReactNode> = {
  info: <HelpCircle size={12} className="text-info" />,
  minor: <AlertCircle size={12} className="text-warn" />,
  major: <AlertCircle size={12} className="text-err" />,
  critical: <XCircle size={12} className="text-err" />,
};

export default function FeedbackPage() {
  const { t } = useTranslation();
  const { feedbacks, transitionFeedback } = useStore();
  const [activeTab, setActiveTab] = useState<FeedbackTab>("inbox");
  const [selected, setSelected] = useState<string | null>(feedbacks[0]?.id ?? null);

  const fb = feedbacks.find((f) => f.id === selected);
  const allowed = fb ? FEEDBACK_SM.transitions.filter((t) => t.from === fb.status).map((t) => t.to) : [];

  const openCount = feedbacks.filter((f) => f.status === "open" || f.status === "acknowledged").length;
  const resolvedCount = feedbacks.filter((f) => f.status === "resolved" || f.status === "wontfix").length;
  const pendingAnswer = feedbacks.filter((f) => f.status === "open" || f.status === "in_progress").length;

  const openFeedbacks = useMemo(
    () => feedbacks.filter((f) => f.status === "open" || f.status === "acknowledged" || f.status === "in_progress"),
    [feedbacks],
  );
  const closedFeedbacks = useMemo(
    () => feedbacks.filter((f) => f.status === "resolved" || f.status === "wontfix" || f.status === "reopened"),
    [feedbacks],
  );

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/feedback'].title}
        subtitle="6 状态机 (§7.3) + INV-FB-01~02。Agent 在执行中向人类发问；人类 answer 后回到 agent 上下文。"
        icon={<MessageCircleWarning className="text-accent" size={20} />}
        track="B"
        count={`${openCount} open · ${pendingAnswer} awaiting answer`}
      />

      <Tabs
        items={TABS}
        active={activeTab}
        onChange={(id) => setActiveTab(id as FeedbackTab)}
        variant="underline"
        size="md"
        ariaLabel="Feedback navigation tabs"
      />

      {activeTab === "inbox" && (
        <div className="space-y-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <Stat label="Total Feedback" value={feedbacks.length} tone="default" />
            <Stat label="Open / Acknowledged" value={openCount} tone="warn" hint="pending human action" />
            <Stat label="Awaiting Answer" value={pendingAnswer} tone="err" />
            <Stat label="Resolved / Closed" value={resolvedCount} tone="ok" />
          </div>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
            <div className="lg:col-span-2 space-y-2">
              {openFeedbacks.length === 0 && (
                <div className="card text-center py-12 text-ink-mute text-xs">No open feedback. All agent queries answered. 🎉</div>
              )}
              {openFeedbacks.map((f) => (
                <div
                  key={f.id}
                  onClick={() => setSelected(f.id)}
                  className={clsx("card cursor-pointer transition-all space-y-2", selected === f.id ? "border-accent/60 bg-accent/5" : "hover:border-line/80")}
                >
                  <div className="flex items-start gap-3">
                    <div className="mt-0.5">{severityIcon[f.severity]}</div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-mono text-xs text-ink-mute">{f.id}</span>
                        <StatusPill value={f.status} size="xs" />
                        <StatusPill value={f.severity} size="xs" />
                        <StatusPill value={f.category} size="xs" />
                      </div>
                      <p className="text-sm font-medium text-ink">{f.question}</p>
                      {f.answer && (
                        <div className="mt-1.5 pl-2 border-l-2 border-ok/40 text-xs text-ink-dim">
                          <span className="text-ok font-medium">A:</span> {f.answer}
                        </div>
                      )}
                      <div className="mt-1.5 text-[10px] text-ink-mute font-mono">
                        agent={f.agent_session_id} · asked_by={f.asked_by}{f.answered_by ? ` · answered_by=${f.answered_by}` : ""}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
            <div>
              {fb ? (
                <div className="card sticky top-16 space-y-4">
                  <div className="flex items-center justify-between pb-2 border-b border-line">
                    <div>
                      <div className="font-mono text-xs text-ink-mute">{fb.id}</div>
                      <div className="text-sm font-semibold text-ink mt-0.5 flex items-center gap-1.5">
                        {severityIcon[fb.severity]} Feedback Detail
                      </div>
                    </div>
                    <StatusPill value={fb.status} />
                  </div>
                  <div>
                    <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Question</div>
                    <p className="text-xs text-ink">{fb.question}</p>
                  </div>
                  {fb.answer ? (
                    <div>
                      <div className="text-[10px] uppercase tracking-wider text-ok mb-1 flex items-center gap-1">
                        <CheckCircle2 size={10} /> Answer
                      </div>
                      <p className="text-xs text-ink-dim">{fb.answer}</p>
                    </div>
                  ) : (
                    <div className="p-2.5 rounded border border-warn/30 bg-warn/5 text-xs text-warn">等待回答 — Agent session 已 block</div>
                  )}
                  <dl className="text-xs space-y-1.5">
                    <Row label="Category" value={<StatusPill value={fb.category} size="xs" />} />
                    <Row label="Severity" value={<StatusPill value={fb.severity} size="xs" />} />
                    <Row label="Worktree" value={<span className="font-mono">{fb.worktree_id}</span>} />
                    <Row label="Session" value={<span className="font-mono">{fb.agent_session_id}</span>} />
                  </dl>
                  {allowed.length > 0 && (
                    <div className="border-t border-line pt-3 space-y-2">
                      <div className="text-[10px] uppercase tracking-wider text-ink-mute">Transition</div>
                      <div className="flex flex-wrap gap-1.5">
                        {allowed.map((to) => (
                          <button key={to} type="button" onClick={() => transitionFeedback(fb.id, to as FeedbackStatus)} className="btn-primary text-xs flex items-center gap-1">
                            <ArrowRight size={10} /> {to}
                          </button>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                <div className="card text-center py-12 text-ink-mute text-xs">Select a feedback item to inspect.</div>
              )}
            </div>
          </div>
        </div>
      )}

      {activeTab === "statemachine" && (
        <div className="space-y-4">
          <StateMachineDiagram sm={FEEDBACK_SM} highlightState={fb?.status} />
          <div className="card space-y-3">
            <SectionTitle>Feedback Invariants (INV-FB-01~02)</SectionTitle>
            <ul className="text-xs space-y-2 text-ink-dim">
              <li className="flex gap-2"><span className="font-mono text-warn shrink-0">INV-FB-01</span><span>同一 agent_session_id 同一 category 同一天内最多 5 条未回答反馈 — 超出返回 429。</span></li>
              <li className="flex gap-2"><span className="font-mono text-warn shrink-0">INV-FB-02</span><span>critical severity 的反馈 30min 未应答 → 自动发 PagerDuty + 写入 audit_event。</span></li>
              <li className="flex gap-2"><span className="font-mono text-info shrink-0">NOTE</span><span>状态机: open → acknowledged → in_progress → resolved / wontfix; reopened 可重开。</span></li>
            </ul>
          </div>
        </div>
      )}

      {activeTab === "history" && (
        <div className="card space-y-3">
          <SectionTitle>Resolved & Closed Feedback History</SectionTitle>
          {closedFeedbacks.length === 0 ? (
            <div className="text-center py-12 text-ink-mute text-xs">No resolved feedback yet.</div>
          ) : (
            <table className="table">
              <thead><tr><th>ID</th><th>Category</th><th>Severity</th><th>Status</th><th>Question</th><th>Answered By</th></tr></thead>
              <tbody>
                {closedFeedbacks.map((f) => (
                  <tr key={f.id} className="opacity-80">
                    <td className="font-mono text-xs text-ink-mute">{f.id}</td>
                    <td><StatusPill value={f.category} size="xs" /></td>
                    <td><StatusPill value={f.severity} size="xs" /></td>
                    <td><StatusPill value={f.status} size="xs" /></td>
                    <td className="text-xs text-ink line-clamp-1 max-w-[280px]">{f.question}</td>
                    <td className="font-mono text-xs text-ink-mute">{f.answered_by ?? "—"}</td>
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

function Row({ label, value }: { label: React.ReactNode; value: React.ReactNode }) {
  return (
    <div className="flex justify-between items-center text-xs">
      <dt className="text-ink-mute">{label}</dt>
      <dd className="text-ink font-mono text-[11px]">{value}</dd>
    </div>
  );
}