// =====================================================================
// Agent Game — Theme Tokens (dark/light 双调, per 9/5 23:13 JST 拍板)
// =====================================================================
// Per 用户发令 "无限画布也要随着黑白主题切换配色":
//   - 现状: SVG 颜色硬编码 (墨黑底 + 霓虹青/朱红/金 等暗色调)
//   - 目标: 跟随 next-themes 的 useTheme() 切换 dark/light
//   - 风格保持: 日漫 + 武侠 + 赛博朋克 (不变, 只换 palette)
//
// 设计:
//   - DARK_PALETTE: 墨黑 #0d0d12, 朱红 #dc2626, 霓虹青 #06b6d4, 金 #f59e0b, 紫 #a855f7
//   - LIGHT_PALETTE: 宣纸 #f4efe6, 朱红 #e60033, 霓虹青 #0055ff, 金 #d48800, 紫 #6b21a8
//   - 角色/光球/装饰的 SVG 节点: 用 palette 的 color 字段, 跟主题切换
//   - 走 useTheme() hook (per next-themes), 客户端组件
// =====================================================================

import { useTheme } from "next-themes";
import { useMemo } from "react";
import { COLORS as DARK } from "./theme";

/** 亮色调色板 (日漫上色风格, per cel-* variables) */
export const LIGHT_COLORS = {
  // 宣纸底
  inkBlack: "#f4efe6",
  inkDark: "#eae2d6",
  inkMid: "#fbf8f3",
  inkLight: "#d6c8b0",
  // 朱红 (Saturated, 暗色版)
  vermilion: "#e60033",
  vermilionGlow: "#ff4d6d",
  // 霓虹青 (Saturated 蓝色, 暗色版)
  neonCyan: "#0055ff",
  neonCyanGlow: "#3b78ff",
  // 金 (Saturated 橙金)
  gold: "#d48800",
  goldGlow: "#f0a020",
  // 紫 (Saturated 紫罗兰)
  cyberPurple: "#6b21a8",
  cyberPurpleGlow: "#9333ea",
  // 灰 (Saturated 暖灰)
  ash: "#9ca3af",
  ashLight: "#6b7280",
  // 文字 (深墨, 跟背景对比)
  paper: "#0a0d14",
} as const;

/** 暗色调色板 (现状) */
export const DARK_COLORS = DARK;

/** 主题 mode (dark | light) */
export type ThemeMode = "dark" | "light";

/**
 * useAgentGameTheme — 客户端 hook, 跟随 next-themes 切换 palette
 *   - dark 模式: DARK (墨黑底 + 高饱和霓虹)
 *   - light 模式: LIGHT (宣纸底 + 高饱和印刷)
 *   - 默认值: dark (per 守门 #13, dark 优先)
 *   - mount 前返回 dark (避免 hydration 闪烁)
 */
export function useAgentGameTheme() {
  const { theme, resolvedTheme } = useTheme();
  return useMemo(() => {
    // 用 resolvedTheme (per next-themes, 处理 system 默认)
    const mode: ThemeMode = resolvedTheme === "light" || theme === "light" ? "light" : "dark";
    return {
      mode,
      colors: mode === "light" ? LIGHT_COLORS : DARK_COLORS,
    };
  }, [theme, resolvedTheme]);
}
