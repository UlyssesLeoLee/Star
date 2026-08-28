// frontend/src/mocks/schemas/inbox.ts
// zod schema for MockNotif (per docs/frontend/design/mock-data-isolation.md §2.1)

import { z } from "zod";

export const NotifKindSchema = z.enum([
  "agent_decision_required",
  "ci_failed",
  "review_requested",
  "merge_conflict",
  "budget_alert",
  "policy_violation",
  "feedback_question",
]);
export type NotifKind = z.infer<typeof NotifKindSchema>;

export const MockNotifSchema = z.object({
  id: z.string().regex(/^n-\d{3}$/, "notif id must be n-NNN format"),
  kind: NotifKindSchema,
  subject: z.string().min(1),
  body: z.string().min(1),
  read: z.boolean(),
  ago: z.string().min(1),
});
export type MockNotif = z.infer<typeof MockNotifSchema>;
