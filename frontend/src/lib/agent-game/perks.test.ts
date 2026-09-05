// =====================================================================
// perks.test.ts — Perk 5 选 1 选择器
// =====================================================================
// 覆盖:
//   1. getPerkChoices 返回完整 5 个列表
//   2. isPerkStackable 4 个 true, 1 个 false (lucky_star)
//   3. perkCounts 正确累加
// =====================================================================

import { describe, it, expect } from "vitest";
import { getPerkChoices, isPerkStackable, perkCounts } from "./perks";
import { PERKS } from "./types";

describe("getPerkChoices", () => {
  it("返回完整 5 个 perk", () => {
    const choices = getPerkChoices();
    expect(choices).toHaveLength(5);
  });

  it("等于 PERKS 常量", () => {
    expect(getPerkChoices()).toEqual(PERKS);
  });
});

describe("isPerkStackable", () => {
  it("xp_boost / coin_magnet / bounty_hunter / iron_will 都 stackable", () => {
    expect(isPerkStackable("xp_boost")).toBe(true);
    expect(isPerkStackable("coin_magnet")).toBe(true);
    expect(isPerkStackable("bounty_hunter")).toBe(true);
    expect(isPerkStackable("iron_will")).toBe(true);
  });

  it("lucky_star 不可 stackable", () => {
    expect(isPerkStackable("lucky_star")).toBe(false);
  });
});

describe("perkCounts", () => {
  it("空 perks → 全 0", () => {
    const c = perkCounts([]);
    expect(c.xp_boost).toBe(0);
    expect(c.coin_magnet).toBe(0);
    expect(c.bounty_hunter).toBe(0);
    expect(c.iron_will).toBe(0);
    expect(c.lucky_star).toBe(0);
  });

  it("2x xp_boost + 1x iron_will", () => {
    const c = perkCounts(["xp_boost", "xp_boost", "iron_will"]);
    expect(c.xp_boost).toBe(2);
    expect(c.coin_magnet).toBe(0);
    expect(c.bounty_hunter).toBe(0);
    expect(c.iron_will).toBe(1);
    expect(c.lucky_star).toBe(0);
  });
});
