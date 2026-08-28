// frontend/src/mocks/__tests__/agents.test.ts
// mock 自身 unit test (per mock-data-isolation.md §3.4)
// 不 mount React 树, 直接 import mock 数据校验.

import { describe, it, expect } from "vitest";
import { MOCK_AGENTS } from "@/mocks/data";
import { AgentRowSchema, AgentStatusSchema } from "@/mocks/schemas/agent";

describe("MOCK_AGENTS", () => {
  it("has 5 rows", () => {
    expect(MOCK_AGENTS).toHaveLength(5);
  });

  it("all rows match zod schema", () => {
    MOCK_AGENTS.forEach((row) => {
      expect(() => AgentRowSchema.parse(row)).not.toThrow();
    });
  });

  it("all ids are unique and match ag-NNN format", () => {
    const ids = MOCK_AGENTS.map((a) => a.id);
    expect(new Set(ids).size).toBe(ids.length);
    ids.forEach((id) => expect(id).toMatch(/^ag-\d{3}$/));
  });

  it("all status values are valid enum", () => {
    MOCK_AGENTS.forEach((a) => {
      expect(() => AgentStatusSchema.parse(a.status)).not.toThrow();
    });
  });

  it("covers at least 3 distinct statuses (per §3.4 invariant)", () => {
    const statuses = new Set(MOCK_AGENTS.map((a) => a.status));
    expect(statuses.size).toBeGreaterThanOrEqual(3);
  });
});
