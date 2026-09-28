// Star Frontend — 主题类型单测
// Per 2026-08-29 04:09 JST 主题决策

import { describe, it, expect } from "vitest";
import {
  THEMES,
  getTheme,
  themeToCss,
  SCOPE_PRIORITY,
  type ThemeId,
  type ThemeScope,
} from "../types";

describe("THEMES", () => {
  it("至少 5 个内置主题 (Light + Dark + Christmas + Charisma + Shanghai)", () => {
    expect(THEMES.length).toBeGreaterThanOrEqual(5);
    const ids = THEMES.map((t) => t.id);
    expect(ids).toContain("light");
    expect(ids).toContain("dark");
    expect(ids).toContain("christmas");
    expect(ids).toContain("charisma");
    expect(ids).toContain("shanghai");
  });

  it("每个主题都有完整 color / spacing / radius token", () => {
    for (const t of THEMES) {
      expect(t.colors.length).toBeGreaterThan(0);
      expect(t.spacings.length).toBeGreaterThan(0);
      expect(t.radii.length).toBeGreaterThan(0);
    }
  });

  it("isDark 字段与主题 ID 语义一致", () => {
    for (const t of THEMES) {
      if (t.id === "light") expect(t.isDark).toBe(false);
      if (t.id === "dark") expect(t.isDark).toBe(true);
      if (t.id === "christmas") expect(t.isDark).toBe(true);
      if (t.id === "charisma") expect(t.isDark).toBe(true);
      if (t.id === "shanghai") expect(t.isDark).toBe(false);
    }
  });

  it("圣诞主题符合常规毛玻璃风格特征 (标准圆角 > 0px)", () => {
    const xmas = THEMES.find((t) => t.id === "christmas");
    expect(xmas).toBeDefined();
    expect(xmas!.displayName).toBe("圣夜霜雪");
    for (const r of xmas!.radii) {
      expect(r.px).toBeGreaterThan(0);
    }
  });

  it("极魅主题为唯一专属 16 位与 HD-2D 纯像素风 (0px 矢量圆角 + 神恩紫罗兰与丝绒红)", () => {
    const charisma = THEMES.find((t) => t.id === "charisma");
    expect(charisma).toBeDefined();
    expect(charisma!.displayName).toBe("极魅像素");
    for (const r of charisma!.radii) {
      expect(r.px).toBe(0);
    }
    const primary = charisma!.colors.find((c) => c.name === "--color-primary");
    expect(primary?.hex).toBe("#8B5CF6");
    const secondary = charisma!.colors.find((c) => c.name === "--color-secondary");
    expect(secondary?.hex).toBe("#E11D48");
    const surface = charisma!.colors.find((c) => c.name === "--color-surface");
    expect(surface?.hex).toBe("#0C0A14");
  });

  it("老上海月份牌主题符合民国海派摩登风格特征 (暖象牙宣纸 + 旗袍朱砂茜红 + 翡翠碧玉墨绿)", () => {
    const shanghai = THEMES.find((t) => t.id === "shanghai");
    expect(shanghai).toBeDefined();
    expect(shanghai!.displayName).toBe("老上海月份牌");
    expect(shanghai!.isDark).toBe(false);
    const primary = shanghai!.colors.find((c) => c.name === "--color-primary");
    expect(primary?.hex).toBe("#B8282B");
    const secondary = shanghai!.colors.find((c) => c.name === "--color-secondary");
    expect(secondary?.hex).toBe("#1B5E48");
    const surface = shanghai!.colors.find((c) => c.name === "--color-surface");
    expect(surface?.hex).toBe("#F5E8C7");
    const border = shanghai!.colors.find((c) => c.name === "--color-border");
    expect(border?.hex).toBe("#C89228");
  });
});

describe("getTheme", () => {
  it("按 id 查找到主题", () => {
    const light = getTheme("light");
    expect(light).toBeDefined();
    expect(light?.id).toBe("light");
  });

  it("未知 id 返回 undefined", () => {
    const unknown = getTheme("nonexistent" as ThemeId);
    expect(unknown).toBeUndefined();
  });
});

describe("themeToCss", () => {
  it("输出含 CSS 变量定义", () => {
    const light = getTheme("light");
    expect(light).toBeDefined();
    const css = themeToCss(light!);
    expect(css).toContain("--color-primary");
    expect(css).toContain("--space-1: 4px");
    expect(css).toContain("--radius-sm: 4px");
  });
});

describe("SCOPE_PRIORITY 三层解析顺序", () => {
  it("Personal > Tenant > Global", () => {
    const scopes: ThemeScope[] = ["personal", "tenant", "global"];
    const sorted = [...scopes].sort(
      (a, b) => SCOPE_PRIORITY[b] - SCOPE_PRIORITY[a]
    );
    expect(sorted).toEqual(["personal", "tenant", "global"]);
  });
});
