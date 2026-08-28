// frontend/src/mocks/schemas/inbox.ts
// 零 prod dep, 用 TS type guards (替代 d4b3193 的 zod).

export const NOTIF_KINDS = [
  "agent_decision_required",
  "ci_failed",
  "review_requested",
  "merge_conflict",
  "budget_alert",
  "policy_violation",
  "feedback_question",
] as const;
export type NotifKind = (typeof NOTIF_KINDS)[number];

export function isNotifKind(v: unknown): v is NotifKind {
  return typeof v === "string" && (NOTIF_KINDS as readonly string[]).includes(v);
}

export interface MockNotif {
  id: string;
  kind: NotifKind;
  subject: string;
  body: string;
  read: boolean;
  ago: string;
}

export function isMockNotif(v: unknown): v is MockNotif {
  if (typeof v !== "object" || v === null) return false;
  const n = v as Record<string, unknown>;
  return (
    typeof n.id === "string" &&
    /^n-\d{3}$/.test(n.id) &&
    isNotifKind(n.kind) &&
    typeof n.subject === "string" &&
    n.subject.length > 0 &&
    typeof n.body === "string" &&
    n.body.length > 0 &&
    typeof n.read === "boolean" &&
    typeof n.ago === "string" &&
    n.ago.length > 0
  );
}
