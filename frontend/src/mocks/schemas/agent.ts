// frontend/src/mocks/schemas/agent.ts
// zod schema for AgentRow (per docs/frontend/design/mock-data-isolation.md §2.1)
// Backend真实接入时 (Phase F+) 这个 schema 与 backend 真实类型一致, 只改 data 文件.

import { z } from "zod";

export const AgentStatusSchema = z.enum([
  "active",
  "in_progress",
  "paused",
  "failed",
  "completed",
]);
export type AgentStatus = z.infer<typeof AgentStatusSchema>;

export const AgentRowSchema = z.object({
  id: z.string().regex(/^ag-\d{3}$/, "agent id must be ag-NNN format"),
  name: z.string().min(1),
  status: AgentStatusSchema,
  role: z.string().min(1),
  last_active: z.string().min(1), // free-text "2 min ago" — locale-agnostic mock
});
export type AgentRow = z.infer<typeof AgentRowSchema>;
