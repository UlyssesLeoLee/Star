"use client";

// =====================================================================
// /inbox — 通知列表 (minimal placeholder)
// =====================================================================
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. notifications 全部 mock (10 条) — 真实 notification service 接入 P3
//   2. read/unread 仅本地 state, 不持久化, 不联动 store
//   3. 3 column layout (源 / 列表 / 详情) 简化为 1 column
//   4. 实时 SSE 推送 (Phase I+) P3
//   5. light mode (per §7) P3
// =====================================================================

import { useState } from "react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Inbox, Bell } from "lucide-react";

type MockNotif = {
  id: string;
  kind: "agent_decision_required" | "ci_failed" | "review_requested" | "merge_conflict" | "budget_alert" | "policy_violation" | "feedback_question";
  subject: string;
  body: string;
  read: boolean;
  ago: string;
};

const MOCK_NOTIFS: ReadonlyArray<MockNotif> = [
  { id: "n-001", kind: "agent_decision_required", subject: "ag-005 awaiting decision",          body: "Spec excerpt conflicts with INV-RT-03, please confirm",    read: false, ago: "2 min ago" },
  { id: "n-002", kind: "ci_failed",                subject: "CI failed: Physis-builder #482",  body: "test:integration:rt_step failed 3/12",                    read: false, ago: "8 min ago" },
  { id: "n-003", kind: "review_requested",         subject: "Review: feat/w2-gantt → main",    body: "@Ulysses review required (5 files, +420 -118)",            read: false, ago: "23 min ago" },
  { id: "n-004", kind: "merge_conflict",           subject: "Merge conflict on planning/page", body: "Resolve conflict before pushing (per §4 fix/*)",           read: true,  ago: "1 h ago"   },
  { id: "n-005", kind: "budget_alert",             subject: "Budget 80% used",                 body: "ag-002 Physis-builder hit $4.00/$5.00 daily cap",          read: false, ago: "1 h ago"   },
  { id: "n-006", kind: "policy_violation",         subject: "Policy violation: INV-FB-02",    body: "ag-005 reported feedback required, lease paused",          read: true,  ago: "2 h ago"   },
  { id: "n-007", kind: "feedback_question",        subject: "ag-007 asks: which spec?",       body: "Multiple matches for SPEC-001 vs SPEC-002",                read: false, ago: "3 h ago"   },
  { id: "n-008", kind: "ci_failed",                subject: "CI failed: Star-frontend #117",  body: "typecheck exit 1 — see log",                                read: true,  ago: "5 h ago"   },
  { id: "n-009", kind: "review_requested",         subject: "Review: DTL-036 v1.4 hotfix",     body: "3 P1/P2/P3 violations — please review",                     read: true,  ago: "1 d ago"   },
  { id: "n-010", kind: "budget_alert",             subject: "Weekly cost summary",            body: "Total $87.42 across 5 agents (down 12% vs last week)",     read: true,  ago: "2 d ago"   },
];

export default function InboxPage() {
  // local-only read/unread toggle (per 缺标, 不联动 store)
  const [readSet, setReadSet] = useState<Set<string>>(
    () => new Set(MOCK_NOTIFS.filter((n) => n.read).map((n) => n.id)),
  );
  const toggle = (id: string) => {
    setReadSet((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };
  const unread = MOCK_NOTIFS.filter((n) => !readSet.has(n.id)).length;

  return (
    <div className="max-w-5xl mx-auto" data-testid="inbox-page">
      <PageHeader
        title="Inbox"
        subtitle="notification / comment / audit (10 mock; 真实 notification service P3 缺口)"
        icon={<Inbox className="text-accent" size={20} />}
        count={`${unread} unread`}
      />

      <div className="card">
        <SectionTitle>Notifications (mock, local read state)</SectionTitle>
        <ul className="divide-y divide-line/40" data-testid="inbox-list">
          {MOCK_NOTIFS.map((n) => {
            const isRead = readSet.has(n.id);
            return (
              <li
                key={n.id}
                data-testid={`inbox-item-${n.id}`}
                className="py-2.5 flex items-start gap-3"
              >
                <button
                  type="button"
                  onClick={() => toggle(n.id)}
                  aria-label={isRead ? "mark unread" : "mark read"}
                  className="mt-1 shrink-0"
                >
                  <Bell
                    size={12}
                    className={isRead ? "text-ink-mute" : "text-accent fill-accent/30"}
                  />
                </button>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <StatusPill value={isRead ? "read" : "pending"} size="xs" />
                    <span className="text-sm text-ink truncate">{n.subject}</span>
                    <span className="font-mono text-[10px] text-ink-mute ml-auto shrink-0">
                      {n.ago}
                    </span>
                  </div>
                  <div className="text-xs text-ink-dim mt-0.5">{n.body}</div>
                </div>
              </li>
            );
          })}
        </ul>
      </div>

      <div className="card mt-3 text-xs text-ink-dim">
        <SectionTitle>Notification Service — P3 缺口</SectionTitle>
        <ul className="space-y-1.5 list-disc pl-4 text-ink-dim">
          <li>当前 read/unread 仅本地 state, 刷新页面丢失, 不联动 <span className="font-mono text-ink-mute">useStore.notifications</span></li>
          <li>真实 <span className="font-mono text-ink-mute">/api/notifications</span> 接入待 Phase I+</li>
          <li>3 column layout (源 / 列表 / 详情) 简化为 1 column, 详情侧栏 P2</li>
          <li>实时 SSE 推送 (Phase I+) P3</li>
        </ul>
      </div>
    </div>
  );
}
