// frontend/src/mocks/__tests__/fixtures-sync.test.ts
// 验证 fixtures/*.json 与 data/*.ts 一致 (改了 data 但忘改 fixture 会 fail)
// per docs/frontend/design/mock-msw-handlers.md §2.5 + §4 缺口 #3

import { describe, it, expect } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { MOCK_AGENTS } from "@/mocks/data";
import { MOCK_NOTIFS } from "@/mocks/data";
import { MOCK_KPI, COST_SERIES } from "@/mocks/data";

function loadFixture(name: string): unknown {
  const path = join(process.cwd(), "src/mocks/fixtures", name);
  return JSON.parse(readFileSync(path, "utf-8"));
}

describe("fixtures sync with data/", () => {
  it("agents.json matches MOCK_AGENTS", () => {
    expect(loadFixture("agents.json")).toEqual(MOCK_AGENTS);
  });
  it("inbox.json matches MOCK_NOTIFS", () => {
    expect(loadFixture("inbox.json")).toEqual(MOCK_NOTIFS);
  });
  it("analytics-kpi.json matches MOCK_KPI", () => {
    expect(loadFixture("analytics-kpi.json")).toEqual(MOCK_KPI);
  });
  it("analytics-cost-series.json matches COST_SERIES", () => {
    expect(loadFixture("analytics-cost-series.json")).toEqual(COST_SERIES);
  });
});
