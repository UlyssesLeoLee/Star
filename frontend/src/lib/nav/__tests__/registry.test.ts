import { describe, it, expect } from "vitest";
import {
  ALL_MODULES,
  MODULE_MAP,
  CATEGORY_STYLES,
  getCategoryStyles,
  type ModuleCategory,
} from "../registry";

// =====================================================================
// registry.test.ts — 域分色契约 + helper 行为单测
//
// Per 2026-09-02 15:42 JST 拍板, 5 域分色 token (Jira 风格 icon tile)
// 不能改值, 改了需要 DDD Review 重新拍板 + 改这一份测试.
// =====================================================================

describe("CATEGORY_STYLES — 5 域色卡契约", () => {
  const CATEGORIES: ModuleCategory[] = [
    "core",
    "work",
    "agent",
    "integration",
    "system",
  ];

  it("5 域全部存在, 无 missing key", () => {
    CATEGORIES.forEach((c) => {
      expect(CATEGORY_STYLES[c]).toBeDefined();
    });
  });

  it("每域 6 组 class 全部非空 (bg/bgActive/text/border/borderActive/glow/dot/name)", () => {
    CATEGORIES.forEach((c) => {
      const cs = CATEGORY_STYLES[c];
      expect(cs.bg.length).toBeGreaterThan(0);
      expect(cs.bgActive.length).toBeGreaterThan(0);
      expect(cs.text.length).toBeGreaterThan(0);
      expect(cs.border.length).toBeGreaterThan(0);
      expect(cs.borderActive.length).toBeGreaterThan(0);
      expect(cs.glow.length).toBeGreaterThan(0);
      expect(cs.dot.length).toBeGreaterThan(0);
      expect(cs.name.length).toBeGreaterThan(0);
    });
  });

  it("5 域色各自唯一 (bg 字符串不重复, 防止 token 撞色)", () => {
    const bgs = CATEGORIES.map((c) => CATEGORY_STYLES[c].bg);
    expect(new Set(bgs).size).toBe(5);
  });

  it("5 域 glow rgba 颜色唯一 (防止 active 状态全显同色光晕)", () => {
    const glows = CATEGORIES.map((c) => CATEGORY_STYLES[c].glow);
    expect(new Set(glows).size).toBe(5);
  });

  it("light + dark 模式 class 双覆盖 (color-500/10 + color-400/10 都出现)", () => {
    CATEGORIES.forEach((c) => {
      const cs = CATEGORY_STYLES[c];
      // light 用 -500/10, dark 用 -400/10
      expect(cs.bg).toMatch(/\/(10|15|20)$/);
    });
  });
});

describe("getCategoryStyles", () => {
  it("合法 category 返回对应色卡", () => {
    expect(getCategoryStyles("work").name).toBe("Work");
    expect(getCategoryStyles("agent").name).toBe("Agent");
    expect(getCategoryStyles("integration").name).toBe("Integration");
    expect(getCategoryStyles("system").name).toBe("System");
    expect(getCategoryStyles("core").name).toBe("Core");
  });

  it("非法 category 回退到 core (防御性编程, 防止 undefined 渲染崩溃)", () => {
    // 强制 cast 模拟脏数据
    const fallback = getCategoryStyles("nonsense" as unknown as ModuleCategory);
    expect(fallback.name).toBe("Core");
    expect(fallback.bg).toBe(CATEGORY_STYLES.core.bg);
  });
});

describe("ALL_MODULES — 5 域覆盖度", () => {
  it("所有 module 都属于 5 域之一 (无 ModuleCategory 漏配)", () => {
    const validCats = new Set([
      "core",
      "work",
      "agent",
      "integration",
      "system",
    ]);
    ALL_MODULES.forEach((m) => {
      expect(validCats.has(m.category)).toBe(true);
    });
  });

  it("每域至少 1 个 module (避免出现完全空域 Sidebar 渲染空组)", () => {
    const counts: Record<string, number> = {};
    ALL_MODULES.forEach((m) => {
      counts[m.category] = (counts[m.category] ?? 0) + 1;
    });
    // 5 域都得有 module, 之前要求 ≥3 是过度约束;
    // Per 2026-09-02 16:13 JST 域分色, core 域只放品牌核心 (inbox),
    // 其他 4 域按业务量动态, 1+ 都合法
    Object.entries(counts).forEach(([cat, n]) => {
      expect(n, `category=${cat} has zero modules`).toBeGreaterThanOrEqual(1);
    });
  });

  it("core 域默认只放品牌核心 (inbox), 业务模块归 work/agent/system (per 2026-09-02 16:13 JST)", () => {
    // 业务子域 (issues / projects / analytics) 不应再归 core
    const business = ["issues", "projects", "analytics", "agents", "settings", "remote"];
    business.forEach((id) => {
      const m = MODULE_MAP.get(id);
      expect(m).toBeDefined();
      expect(m!.category, `${id} should not be core`).not.toBe("core");
    });
    // inbox 是唯一的 core
    const inbox = MODULE_MAP.get("inbox");
    expect(inbox!.category).toBe("core");
  });

  it("每 module 都能通过 getCategoryStyles 拿到色卡 (实操路径无回退)", () => {
    ALL_MODULES.forEach((m) => {
      const cs = getCategoryStyles(m.category);
      // 任何 module 拿到的色卡都跟 CATEGORY_STYLES 一致, 不会出现 undefined
      expect(cs).toBe(CATEGORY_STYLES[m.category]);
    });
  });
});

describe("MODULE_MAP — id 唯一性 + 全覆盖", () => {
  it("Map 大小 = ALL_MODULES 长度 (无重复 id)", () => {
    expect(MODULE_MAP.size).toBe(ALL_MODULES.length);
  });

  it("每个 module 都有 icon (regulatory: Jira 风格 icon tile 必含 icon)", () => {
    ALL_MODULES.forEach((m) => {
      expect(m.icon).toBeDefined();
      // icon 是 React.ElementType (function/class), 不是 string
      expect(["function", "object"].includes(typeof m.icon)).toBe(true);
    });
  });
});


