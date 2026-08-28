// frontend/src/mocks/schemas/agent.ts
// Per docs/frontend/design/mock-data-isolation.md §2.5 — 零 prod dep, 用 TS type guards (替代 d4b3193 的 zod 假设, zod 未装).
// Backend 真实接入时 (Phase F+) 这个 type 与 backend 真实类型一致, 只改 data 文件.

export const AGENT_STATUSES = [
  "active",
  "in_progress",
  "paused",
  "failed",
  "completed",
] as const;
export type AgentStatus = (typeof AGENT_STATUSES)[number];

export function isAgentStatus(v: unknown): v is AgentStatus {
  return typeof v === "string" && (AGENT_STATUSES as readonly string[]).includes(v);
}

export interface AgentRow {
  id: string;
  name: string;
  status: AgentStatus;
  role: string;
  last_active: string;
}

export function isAgentRow(v: unknown): v is AgentRow {
  if (typeof v !== "object" || v === null) return false;
  const r = v as Record<string, unknown>;
  return (
    typeof r.id === "string" &&
    /^ag-\d{3}$/.test(r.id) &&
    typeof r.name === "string" &&
    r.name.length > 0 &&
    isAgentStatus(r.status) &&
    typeof r.role === "string" &&
    r.role.length > 0 &&
    typeof r.last_active === "string" &&
    r.last_active.length > 0
  );
}
