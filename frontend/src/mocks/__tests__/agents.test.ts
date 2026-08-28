// frontend/src/mocks/__tests__/agents.test.ts
// 替代 d4b3193 zod parse, 用 TS type guard isAgentRow.

import { describe, it, expect } from "vitest";
import { MOCK_AGENTS } from "@/mocks/data";
import { isAgentRow, isAgentStatus, AGENT_STATUSES } from "@/mocks/schemas/agent";

describe("MOCK_AGENTS", () => {
  it("has 5 rows", () => {
    expect(MOCK_AGENTS).toHaveLength(5);
  });

  it("all rows match type guard isAgentRow", () => {
    MOCK_AGENTS.forEach((row) => {
      expect(isAgentRow(row)).toBe(true);
    });
  });

  it("all ids are unique and match ag-NNN format", () => {
    const ids = MOCK_AGENTS.map((a) => a.id);
    expect(new Set(ids).size).toBe(ids.length);
    ids.forEach((id) => expect(id).toMatch(/^ag-\d{3}$/));
  });

  it("all status values are valid enum", () => {
    MOCK_AGENTS.forEach((a) => {
      expect(isAgentStatus(a.status)).toBe(true);
      expect(AGENT_STATUSES as readonly string[]).toContain(a.status);
    });
  });

  it("covers at least 3 distinct statuses (per §3.4 invariant)", () => {
    const statuses = new Set(MOCK_AGENTS.map((a) => a.status));
    expect(statuses.size).toBeGreaterThanOrEqual(3);
  });
});
