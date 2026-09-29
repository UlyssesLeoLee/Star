// =====================================================================
// theme-tokens.test.ts — 主题 palette hook
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { LIGHT_COLORS, DARK_COLORS, CHRISTMAS_COLORS, CHARISMA_COLORS, SHANGHAI_COLORS, useAgentGameTheme } from "./theme-tokens";

// mock next-themes
const mockUseTheme = vi.fn();
vi.mock("next-themes", () => ({
  useTheme: () => mockUseTheme(),
}));

describe("LIGHT_COLORS", () => {
  it("宣纸底 inkBlack", () => {
    expect(LIGHT_COLORS.inkBlack).toBe("#f4efe6");
  });
  it("朱红 vermilion (Saturated 暗色版)", () => {
    expect(LIGHT_COLORS.vermilion).toBe("#e60033");
  });
  it("霓虹青 neonCyan (Saturated 蓝色 暗色版)", () => {
    expect(LIGHT_COLORS.neonCyan).toBe("#0055ff");
  });
  it("金 gold (橙金)", () => {
    expect(LIGHT_COLORS.gold).toBe("#d48800");
  });
  it("15 色类齐全 (4 墨黑 + 2 朱红 + 2 霓虹青 + 2 金 + 2 紫 + 2 灰 + 1 白)", () => {
    const keys = Object.keys(LIGHT_COLORS);
    expect(keys.length).toBe(15);
    expect(keys).toContain("vermilion");
    expect(keys).toContain("neonCyan");
    expect(keys).toContain("gold");
    expect(keys).toContain("cyberPurple");
  });
});

describe("DARK_COLORS", () => {
  it("DARK_COLORS 等于 theme.COLORS (向后兼容)", () => {
    expect(DARK_COLORS.inkBlack).toBe("#0d0d12");
    expect(DARK_COLORS.vermilion).toBe("#dc2626");
  });
});

describe("CHRISTMAS_COLORS", () => {
  it("圣夜黑底 inkBlack", () => {
    expect(CHRISTMAS_COLORS.inkBlack).toBe("#0b130e");
  });
  it("圣夜绯红 vermilion", () => {
    expect(CHRISTMAS_COLORS.vermilion).toBe("#d42426");
  });
  it("常青松绿 neonCyan (替代青色)", () => {
    expect(CHRISTMAS_COLORS.neonCyan).toBe("#165b33");
  });
  it("伯利恒星金 gold", () => {
    expect(CHRISTMAS_COLORS.gold).toBe("#f8b229");
  });
  it("初雪霜白 paper", () => {
    expect(CHRISTMAS_COLORS.paper).toBe("#f5f8f5");
  });
  it("15 色类齐全", () => {
    const keys = Object.keys(CHRISTMAS_COLORS);
    expect(keys.length).toBe(15);
    expect(keys).toContain("vermilion");
    expect(keys).toContain("neonCyan");
    expect(keys).toContain("gold");
  });
});

describe("CHARISMA_COLORS", () => {
  it("剧院黑曜深渊底 inkBlack", () => {
    expect(CHARISMA_COLORS.inkBlack).toBe("#0c0a14");
  });
  it("醉梦丝绒红 vermilion", () => {
    expect(CHARISMA_COLORS.vermilion).toBe("#e11d48");
  });
  it("神恩紫罗兰 neonCyan (替换青色)", () => {
    expect(CHARISMA_COLORS.neonCyan).toBe("#8b5cf6");
  });
  it("帝国流金 gold", () => {
    expect(CHARISMA_COLORS.gold).toBe("#f59e0b");
  });
  it("香槟丝白 paper", () => {
    expect(CHARISMA_COLORS.paper).toBe("#fdf4ff");
  });
  it("15 色类齐全", () => {
    const keys = Object.keys(CHARISMA_COLORS);
    expect(keys.length).toBe(15);
    expect(keys).toContain("vermilion");
    expect(keys).toContain("neonCyan");
    expect(keys).toContain("gold");
  });
});

describe("SHANGHAI_COLORS", () => {
  it("暖象牙宣纸底 inkBlack", () => {
    expect(SHANGHAI_COLORS.inkBlack).toBe("#f5e8c7");
  });
  it("旗袍朱砂茜红 vermilion", () => {
    expect(SHANGHAI_COLORS.vermilion).toBe("#b8282b");
  });
  it("翡翠墨绿 neonCyan (替代青色)", () => {
    expect(SHANGHAI_COLORS.neonCyan).toBe("#1b5e48");
  });
  it("留声机老鎏金 gold", () => {
    expect(SHANGHAI_COLORS.gold).toBe("#d49e35");
  });
  it("老炭墨黑 paper", () => {
    expect(SHANGHAI_COLORS.paper).toBe("#221c16");
  });
  it("15 色类齐全", () => {
    const keys = Object.keys(SHANGHAI_COLORS);
    expect(keys.length).toBe(15);
    expect(keys).toContain("vermilion");
    expect(keys).toContain("neonCyan");
    expect(keys).toContain("gold");
  });
});

describe("useAgentGameTheme", () => {
  beforeEach(() => {
    mockUseTheme.mockReset();
  });

  it("theme=light → mode=light, colors=LIGHT", () => {
    mockUseTheme.mockReturnValue({ theme: "light", resolvedTheme: "light" });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("light");
    expect(result.current.colors).toBe(LIGHT_COLORS);
  });

  it("theme=dark → mode=dark, colors=DARK", () => {
    mockUseTheme.mockReturnValue({ theme: "dark", resolvedTheme: "dark" });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("dark");
    expect(result.current.colors).toBe(DARK_COLORS);
  });

  it("theme=christmas → mode=christmas, colors=CHRISTMAS", () => {
    mockUseTheme.mockReturnValue({ theme: "christmas", resolvedTheme: "christmas" });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("christmas");
    expect(result.current.colors).toBe(CHRISTMAS_COLORS);
  });

  it("theme=charisma → mode=charisma, colors=CHARISMA", () => {
    mockUseTheme.mockReturnValue({ theme: "charisma", resolvedTheme: "charisma" });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("charisma");
    expect(result.current.colors).toBe(CHARISMA_COLORS);
  });

  it("theme=shanghai → mode=shanghai, colors=SHANGHAI", () => {
    mockUseTheme.mockReturnValue({ theme: "shanghai", resolvedTheme: "shanghai" });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("shanghai");
    expect(result.current.colors).toBe(SHANGHAI_COLORS);
  });

  it("theme=system + resolvedTheme=light → mode=light (per resolvedTheme 优先)", () => {
    mockUseTheme.mockReturnValue({ theme: "system", resolvedTheme: "light" });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("light");
  });

  it("theme=system + resolvedTheme=dark → mode=dark", () => {
    mockUseTheme.mockReturnValue({ theme: "system", resolvedTheme: "dark" });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("dark");
  });

  it("theme=undefined → 默认 dark (per 守门 #13 dark 优先)", () => {
    mockUseTheme.mockReturnValue({ theme: undefined, resolvedTheme: undefined });
    const { result } = renderHook(() => useAgentGameTheme());
    expect(result.current.mode).toBe("dark");
  });
});
