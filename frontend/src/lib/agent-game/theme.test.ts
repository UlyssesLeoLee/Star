// =====================================================================
// theme.test.ts — 色板/角色 tier/敌人 type 纯函数
// =====================================================================

import { describe, it, expect } from "vitest";
import {
  COLORS, FONTS, DECORATION, CHARACTER_TIERS, ENEMY_TYPES,
  enemyTypeForPriority, pickRandomEnemyType,
} from "./theme";

describe("COLORS", () => {
  it("主色板 6 类 (墨黑/朱红/霓虹青/金/紫/灰/白) 存在", () => {
    expect(COLORS.inkBlack).toBe("#0d0d12");
    expect(COLORS.vermilion).toBe("#dc2626");
    expect(COLORS.neonCyan).toBe("#06b6d4");
    expect(COLORS.gold).toBe("#f59e0b");
    expect(COLORS.cyberPurple).toBe("#a855f7");
    expect(COLORS.paper).toBe("#f8f5f0");
  });
});

describe("FONTS", () => {
  it("3 字体: primary / mono / stamp", () => {
    expect(FONTS.primary).toContain("Hiragino");
    expect(FONTS.mono).toContain("SF Mono");
    expect(FONTS.stamp).toContain("Mincho");
  });
});

describe("DECORATION", () => {
  it("装饰常量 (energyRing / inkTrail / stamp / divineHalo) 存在", () => {
    expect(DECORATION.energyRingRadius).toBe(40);
    expect(DECORATION.inkTrailLength).toBe(4);
    expect(DECORATION.stampSize).toBe(12);
    expect(DECORATION.divineHaloRadius).toBe(28);
  });
});

describe("CHARACTER_TIERS (6 段)", () => {
  it("6 段 tier, 名字 + 颜色 + 装饰", () => {
    expect(CHARACTER_TIERS).toHaveLength(6);
    const names = CHARACTER_TIERS.map((t) => t.name);
    expect(names).toEqual(["游侠", "武童", "剑客", "侠客", "剑圣", "神侠"]);
  });

  it("装饰从 Lv 3 开始累加 (sword/cloak Lv 3+, armor Lv 5+, crown/halo Lv 7+)", () => {
    expect(CHARACTER_TIERS[0]!.hasSword).toBe(false);  // Lv 1 游侠
    expect(CHARACTER_TIERS[1]!.hasSword).toBe(false);  // Lv 2 武童
    expect(CHARACTER_TIERS[2]!.hasSword).toBe(true);   // Lv 3 剑客
    expect(CHARACTER_TIERS[2]!.hasCloak).toBe(true);
    expect(CHARACTER_TIERS[3]!.hasArmor).toBe(true);   // Lv 5 侠客
    expect(CHARACTER_TIERS[4]!.hasCrown).toBe(true);   // Lv 7 剑圣
    expect(CHARACTER_TIERS[4]!.hasHalo).toBe(true);
    expect(CHARACTER_TIERS[5]!.hasHalo).toBe(true);   // Lv 10 神侠
  });
});

describe("ENEMY_TYPES (6 种光球)", () => {
  it("6 种光球 (青/朱/金/紫/白/神)", () => {
    expect(ENEMY_TYPES).toHaveLength(6);
    const keys = ENEMY_TYPES.map((e) => e.key);
    expect(keys).toEqual([
      "neon_blue",
      "vermilion_fire",
      "gold_thunder",
      "purple_shadow",
      "white_paper",
      "boss_divine",
    ]);
  });
});

describe("enemyTypeForPriority", () => {
  it("p0 → 朱火光球 (urgest)", () => {
    expect(enemyTypeForPriority("p0").key).toBe("vermilion_fire");
  });
  it("p1 → 金雷光球", () => {
    expect(enemyTypeForPriority("p1").key).toBe("gold_thunder");
  });
  it("p2 → 青光球 (default)", () => {
    expect(enemyTypeForPriority("p2").key).toBe("neon_blue");
  });
  it("p3 → 白纸光球 (low)", () => {
    expect(enemyTypeForPriority("p3").key).toBe("white_paper");
  });
  it("unknown → 紫影光球 (fallback)", () => {
    expect(enemyTypeForPriority("unknown").key).toBe("purple_shadow");
  });
});

describe("pickRandomEnemyType", () => {
  it("同样 seed 同样 type (deterministic)", () => {
    const a = pickRandomEnemyType(42);
    const b = pickRandomEnemyType(42);
    expect(a.key).toBe(b.key);
  });
  it("不同 seed 产出可不同", () => {
    const seen = new Set<string>();
    for (let seed = 0; seed < 30; seed++) {
      seen.add(pickRandomEnemyType(seed).key);
    }
    expect(seen.size).toBeGreaterThan(1);  // 至少 2 种
  });
  it("负数 seed 也工作", () => {
    const t = pickRandomEnemyType(-1);
    expect(ENEMY_TYPES.find((e) => e.key === t.key)).toBeTruthy();
  });
});
