// =====================================================================
// theme-tokens.test.ts — 主题 palette hook
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { LIGHT_COLORS, DARK_COLORS, useAgentGameTheme } from "./theme-tokens";

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
