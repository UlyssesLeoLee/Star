// frontend/src/mocks/__tests__/analytics.test.ts
// 替代 d4b3193 zod parse, 用 TS type guard isKpiCard / isCostPoint.

import { describe, it, expect } from "vitest";
import { MOCK_KPI, COST_SERIES } from "@/mocks/data";
import { isKpiCard, isCostPoint } from "@/mocks/schemas/analytics";

describe("MOCK_KPI", () => {
  it("has 4 cards", () => {
    expect(MOCK_KPI).toHaveLength(4);
  });

  it("all cards match type guard isKpiCard", () => {
    MOCK_KPI.forEach((k) => {
      expect(isKpiCard(k)).toBe(true);
    });
  });
});

describe("COST_SERIES", () => {
  it("has exactly 7 days (per §3.4 invariant)", () => {
    expect(COST_SERIES).toHaveLength(7);
  });

  it("all points match type guard isCostPoint", () => {
    COST_SERIES.forEach((p) => {
      expect(isCostPoint(p)).toBe(true);
    });
  });

  it("all usd values are non-negative", () => {
    COST_SERIES.forEach((p) => {
      expect(p.usd).toBeGreaterThanOrEqual(0);
    });
  });

  it("all day labels are unique (Mon..Sun 7 unique days)", () => {
    const days = COST_SERIES.map((p) => p.day);
    expect(new Set(days).size).toBe(days.length);
  });

  it("weekly sum is reasonable (10-100 USD per §3.4)", () => {
    const sum = COST_SERIES.reduce((acc, p) => acc + p.usd, 0);
    expect(sum).toBeGreaterThan(10);
    expect(sum).toBeLessThan(100);
  });
});
