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
  it("至少 4 个内置主题 (Light + Dark + Christmas + Charisma)", () => {
    expect(THEMES.length).toBeGreaterThanOrEqual(4);
    const ids = THEMES.map((t) => t.id);
    expect(ids).toContain("light");
    expect(ids).toContain("dark");
    expect(ids).toContain("christmas");
    expect(ids).toContain("charisma");
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
    }
  });

  it("圣诞主题符合纯像素风特征 (0px 矢量圆角)", () => {
    const xmas = THEMES.find((t) => t.id === "christmas");
    expect(xmas).toBeDefined();
    for (const r of xmas!.radii) {
      expect(r.px).toBe(0);
    }
  });

  it("极魅主题符合歧路旅人 HD-2D 纯像素风特征 (0px 矢量圆角 + 神恩紫罗兰与丝绒红)", () => {
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
