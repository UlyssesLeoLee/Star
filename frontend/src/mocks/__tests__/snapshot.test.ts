// frontend/src/mocks/__tests__/snapshot.test.ts
// 整体 snapshot 锁住 mock 输出 (per mock-data-isolation.md §3.4).
// 改 mock 时, 必须 review snapshot diff.

import { describe, it, expect } from "vitest";
import { MOCK_AGENTS, MOCK_NOTIFS, MOCK_KPI, COST_SERIES, KANBAN_COLUMNS } from "@/mocks/data";

describe("mock snapshot (lock fixture output)", () => {
  it("MOCK_AGENTS stable", () => {
    expect(MOCK_AGENTS).toMatchSnapshot();
  });
  it("MOCK_NOTIFS stable", () => {
    expect(MOCK_NOTIFS).toMatchSnapshot();
  });
  it("MOCK_KPI stable", () => {
    expect(MOCK_KPI).toMatchSnapshot();
  });
  it("COST_SERIES stable", () => {
    expect(COST_SERIES).toMatchSnapshot();
  });
  it("KANBAN_COLUMNS stable", () => {
    expect(KANBAN_COLUMNS).toMatchSnapshot();
  });
});
