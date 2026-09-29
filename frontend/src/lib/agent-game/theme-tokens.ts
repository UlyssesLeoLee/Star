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

/** 圣诞像素调色板 (圣夜像素风格, 冬日经典红绿金纯像素调) */
export const CHRISTMAS_COLORS = {
  // 圣夜密林深黑底
  inkBlack: "#0b130e",
  inkDark: "#122017",
  inkMid: "#1a2f23",
  inkLight: "#234330",
  // 圣夜绯红 (温暖壁炉 / 缎带)
  vermilion: "#d42426",
  vermilionGlow: "#ff4d6d",
  // 常青冬青松绿 (替换 neonCyan)
  neonCyan: "#165b33",
  neonCyanGlow: "#22c55e",
  // 伯利恒星芒烛金
  gold: "#f8b229",
  goldGlow: "#fcd34d",
  // 圣诞夜空紫
  cyberPurple: "#9333ea",
  cyberPurpleGlow: "#c084fc",
  // 雾杉灰
  ash: "#7a9a8b",
  ashLight: "#9ebcab",
  // 初雪霜白 (高对比文字)
  paper: "#f5f8f5",
} as const;

/** 极魅像素调色板 (歧路旅人 HD-2D 神恩紫红金复古纯像素调) */
export const CHARISMA_COLORS = {
  // 剧院黑曜深渊底
  inkBlack: "#0c0a14",
  inkDark: "#171326",
  inkMid: "#221c38",
  inkLight: "#2c2245",
  // 醉梦丝绒红 (舞娘丝绒帷幕)
  vermilion: "#e11d48",
  vermilionGlow: "#fb7185",
  // 神恩紫罗兰 (歧路旅人神恩法术与引力光球)
  neonCyan: "#8b5cf6",
  neonCyanGlow: "#a78bfa",
  // 帝国流金 (烛光与古典浮雕)
  gold: "#f59e0b",
  goldGlow: "#fbbf24",
  // 圣殿极光紫
  cyberPurple: "#a855f7",
  cyberPurpleGlow: "#c084fc",
  // 暮霭紫灰
  ash: "#948da5",
  ashLight: "#a79bb7",
  // 香槟丝白 (高对比文本)
  paper: "#fdf4ff",
} as const;

/** 老上海月份牌调色板 (民国海派摩登擦笔水彩与 Art Deco 暖象牙宣纸调) */
export const SHANGHAI_COLORS = {
  // 暖象牙宣纸底
  inkBlack: "#f5e8c7",
  inkDark: "#ebd8b0",
  inkMid: "#faf2de",
  inkLight: "#dfca9f",
  // 旗袍朱砂茜红
  vermilion: "#b8282b",
  vermilionGlow: "#d43f42",
  // 翡翠碧玉墨绿 (替换青色)
  neonCyan: "#1b5e48",
  neonCyanGlow: "#298265",
  // 留声机老鎏金
  gold: "#d49e35",
  goldGlow: "#e5b452",
  // 胭脂紫红
  cyberPurple: "#7c2d4a",
  cyberPurpleGlow: "#9e3f63",
  // 老上海黛灰
  ash: "#8c7e70",
  ashLight: "#a39587",
  // 老炭墨黑 (高对比文本)
  paper: "#221c16",
} as const;

/** 日式赛璐璐调色板 (80-90s 经典日漫手绘赛璐璐胶片画与机械墨线平涂) */
export const CEL_COLORS = {
  // 赛璐璐胶片透光暖白底
  inkBlack: "#f8fafc",
  inkDark: "#eef2f6",
  inkMid: "#ffffff",
  inkLight: "#e2e8f0",
  // 赛璐璐活力朱红
  vermilion: "#ea580c",
  vermilionGlow: "#f97316",
  // 赛璐璐 EVA 蓝
  neonCyan: "#2563eb",
  neonCyanGlow: "#3b82f6",
  // 赛璐璐明黄
  gold: "#f59e0b",
  goldGlow: "#fbbf24",
  // 经典紫电青紫
  cyberPurple: "#4f46e5",
  cyberPurpleGlow: "#6366f1",
  // 赛璐璐描线炭灰
  ash: "#334155",
  ashLight: "#64748b",
  // 赛璐璐机械转印漆黑线 (高对比文字)
  paper: "#111827",
} as const;

/** 主题 mode (dark | light | christmas | charisma | shanghai | cel) */
export type ThemeMode = "dark" | "light" | "christmas" | "charisma" | "shanghai" | "cel";

/** Agent Game 调色板结构类型 */
export type AgentGamePalette = Record<keyof typeof DARK_COLORS, string>;

/**
 * useAgentGameTheme — 客户端 hook, 跟随 next-themes 切换 palette
 *   - dark 模式: DARK (墨黑底 + 高饱和霓虹)
 *   - light 模式: LIGHT (宣纸底 + 高饱和印刷)
 *   - christmas 模式: CHRISTMAS (圣夜密林黑 + 绯红 + 松绿 + 烛金)
 *   - charisma 模式: CHARISMA (剧院黑曜深渊 + 神恩紫 + 丝绒红 + 帝国金)
 *   - shanghai 模式: SHANGHAI (象牙暖宣纸 + 旗袍朱砂红 + 翡翠墨绿 + 留声机金)
 *   - cel 模式: CEL (日式手绘赛璐璐 + EVA天青蓝 + 活力朱红 + 胶片暖白 + 机械墨线)
 *   - 默认值: dark (per 守门 #13, dark 优先)
 *   - mount 前返回 dark (避免 hydration 闪烁)
 */
export function useAgentGameTheme() {
  const { theme, resolvedTheme } = useTheme();
  return useMemo(() => {
    // 用 resolvedTheme (per next-themes, 处理 system 默认)
    let mode: ThemeMode = "dark";
    if (resolvedTheme === "light" || theme === "light") {
      mode = "light";
    } else if (resolvedTheme === "christmas" || theme === "christmas") {
      mode = "christmas";
    } else if (resolvedTheme === "charisma" || theme === "charisma") {
      mode = "charisma";
    } else if (resolvedTheme === "shanghai" || theme === "shanghai") {
      mode = "shanghai";
    } else if (resolvedTheme === "cel" || theme === "cel") {
      mode = "cel";
    }

    let colors: AgentGamePalette = DARK_COLORS;
    if (mode === "light") {
      colors = LIGHT_COLORS;
    } else if (mode === "christmas") {
      colors = CHRISTMAS_COLORS;
    } else if (mode === "charisma") {
      colors = CHARISMA_COLORS;
    } else if (mode === "shanghai") {
      colors = SHANGHAI_COLORS;
    } else if (mode === "cel") {
      colors = CEL_COLORS;
    }

    return {
      mode,
      colors,
    };
  }, [theme, resolvedTheme]);
}
