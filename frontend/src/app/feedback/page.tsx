"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { MessageCircleWarning, HelpCircle, CheckCircle2, XCircle, AlertCircle } from "lucide-react";
import { FEEDBACK_SM, type FeedbackStatus } from "@/types/ids";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";
import { useState } from "react";

const severityIcon = {
  info: <HelpCircle size={12} className="text-info" />,
  minor: <AlertCircle size={12} className="text-warn" />,
  major: <AlertCircle size={12} className="text-err" />,
  critical: <XCircle size={12} className="text-err" />,
};

export default function FeedbackPage() {
  const { feedbacks, transitionFeedback } = useStore();
  const [selected, setSelected] = useState<string | null>("fb-001");
  const fb = feedbacks.find((f) => f.id === selected);
  const allowed = fb ? FEEDBACK_SM.transitions.filter((t) => t.from === fb.status).map((t) => t.to) : [];

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Feedback Inbox"
        subtitle="6 状态机(§7.3) + INV-FB-01~02。Agent 在执行中向人类发问;人类 answer 后回到 agent 上下文。"
        icon={<MessageCircleWarning className="text-accent" size={20} />}
        track="B"
        count={feedbacks.filter(f => f.status === "open" || f.status === "acknowledged").length}
      />

      <div className="mb-5">
        <StateMachineDiagram sm={FEEDBACK_SM} highlightState={fb?.status} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <div className="lg:col-span-2 space-y-2">
          {feedbacks.map((f) => (
            <div
              key={f.id}
              onClick={() => setSelected(f.id)}
              className={`card cursor-pointer transition-colors ${selected === f.id ? "border-accent/60" : "hover:border-line/80"}`}
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
                  <p className="text-sm">{f.question}</p>
                  {f.answer && (
                    <div className="mt-1.5 pl-2 border-l-2 border-ok/40 text-xs text-ink-dim">
                      <span className="text-ok font-medium">A:</span> {f.answer}
                    </div>
                  )}
                  <div className="mt-1.5 text-[10px] text-ink-mute font-mono">
                    agent={f.agent_session_id} · worktree={f.worktree_id} · asked_by={f.asked_by} {f.answered_by && `· answered_by=${f.answered_by}`}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>

        {fb && (
          <div className="card sticky top-16">
            <div className="flex items-center justify-between mb-3">
              <div>
                <div className="text-xs text-ink-mute font-mono">{fb.id}</div>
                <div className="text-base font-semibold flex items-center gap-1.5">
                  {severityIcon[fb.severity]} Feedback Detail
                </div>
              </div>
              <StatusPill value={fb.status} />
            </div>
            <div className="mb-3">
              <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Question</div>
              <p className="text-sm">{fb.question}</p>
            </div>
            {fb.answer ? (
              <div className="mb-3">
                <div className="text-[10px] uppercase tracking-wider text-ok mb-1 flex items-center gap-1">
                  <CheckCircle2 size={10} /> Answer
                </div>
                <p className="text-sm text-ink-dim">{fb.answer}</p>
              </div>
            ) : (
              <div className="mb-3 p-2 rounded border border-warn/30 bg-warn/5 text-xs text-warn">
                等待回答 — Agent session 已 block 在 awaiting_feedback
              </div>
            )}
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5">Transition</div>
            <div className="flex flex-wrap gap-1.5">
              {allowed.map((to) => (
                <button key={to} onClick={() => transitionFeedback(fb.id, to as FeedbackStatus)} className="btn-primary">→ {to}</button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
