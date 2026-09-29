// Star Frontend — 主题类型定义
// Per 2026-08-29 04:09 JST 用户拍板: 三元组 enum + 可插拔 + 三层作用域.

/**
 * 主题唯一标识 (三元组 enum + 可插拔).
 * - 内置 2 个: Light + Dark
 * - 预留扩展: HighContrast / Solarized 等
 * - 第三方 / 租户自定义主题可追加 variant
 */
export type ThemeId = "light" | "dark" | "christmas" | "charisma" | "shanghai" | "high-contrast" | "solarized";

/**
 * 主题作用域 (三层解析: Personal > Tenant > Global)
 */
export type ThemeScope = "personal" | "tenant" | "global";

/** 设计令牌 — 颜色 */
export interface ColorToken {
  name: string; // CSS var name, 例: "--color-primary"
  hex: string; // 例: "#5B5BD6"
  alpha?: number; // 0.0 - 1.0
}

/** 设计令牌 — 间距 (4px 基础栅格) */
export interface SpacingToken {
  name: string; // 例: "--space-4"
  px: number; // 4 / 8 / 12 / 16 / 24 / 32 / 48 / 64
}

/** 设计令牌 — 圆角 (3 档) */
export interface RadiusToken {
  name: string; // 例: "--radius-sm"
  px: number; // 4 / 8 / 12
}

/** 完整主题定义 (与后端 ThemeDefinition 对齐) */
export interface ThemeDefinition {
  id: ThemeId;
  displayName: string;
  isDark: boolean;
  colors: ColorToken[];
  spacings: SpacingToken[];
  radii: RadiusToken[];
  version: number;
}

/** Star 调色板 — Mecha Ceramic Light (机甲陶瓷白 / 桜の光) */
const STAR_LIGHT_PALETTE: ColorToken[] = [
  { name: "--color-primary", hex: "#0284C7" },
  { name: "--color-secondary", hex: "#F43F5E" },
  { name: "--color-success", hex: "#059669" },
  { name: "--color-warning", hex: "#D97706" },
  { name: "--color-danger", hex: "#E11D48" },
  { name: "--color-neutral", hex: "#475569" },
  { name: "--color-surface", hex: "#F8FAFD" },
  { name: "--color-surface-2", hex: "#FFFFFF" },
  { name: "--color-text", hex: "#0F172A" },
  { name: "--color-text-dim", hex: "#475569" },
  { name: "--color-border", hex: "#E2E8F0" },
];

/** Star 调色板 — Neo-Tokyo Dark (默认暗色 / 赛博黑曜石 & 电光霓虹) */
const STAR_DARK_PALETTE: ColorToken[] = [
  { name: "--color-primary", hex: "#00F0FF" },
  { name: "--color-secondary", hex: "#FF2A85" },
  { name: "--color-success", hex: "#10B981" },
  { name: "--color-warning", hex: "#F59E0B" },
  { name: "--color-danger", hex: "#FF3366" },
  { name: "--color-neutral", hex: "#94A3B8" },
  { name: "--color-surface", hex: "#080B11" },
  { name: "--color-surface-2", hex: "#0F1420" },
  { name: "--color-text", hex: "#F8FAFC" },
  { name: "--color-text-dim", hex: "#94A3B8" },
  { name: "--color-border", hex: "#1E293B" },
];

/**
 * Star 调色板 — Frost Noel / 圣夜霜雪 (第3套UI主题: 冬日温暖毛玻璃风格)
 * 运用毛玻璃 (Glassmorphism) 与冬日色彩心理学:
 * - 圣夜绯红 (#D42426): 礼物缎带与节日暖意
 * - 常青松针 (#165B33): 冬青与冷杉避风港
 * - 星芒烛金 (#F8B229): 伯利恒暖光与流光倒影
 * - 圣夜霜林 (#0B130E): 半透明毛玻璃微暗底
 * - 初雪霜白 (#F5F8F5): 晶莹高对比文本
 */
const STAR_CHRISTMAS_PALETTE: ColorToken[] = [
  { name: "--color-primary", hex: "#D42426" },
  { name: "--color-secondary", hex: "#165B33" },
  { name: "--color-accent", hex: "#F8B229" },
  { name: "--color-success", hex: "#22C55E" },
  { name: "--color-warning", hex: "#F59E0B" },
  { name: "--color-danger", hex: "#EF4444" },
  { name: "--color-neutral", hex: "#7A9A8B" },
  { name: "--color-surface", hex: "#0B130E" },
  { name: "--color-surface-2", hex: "#122017" },
  { name: "--color-text", hex: "#F5F8F5" },
  { name: "--color-text-dim", hex: "#9EBCAB" },
  { name: "--color-border", hex: "#234330" },
];

/**
 * Star 调色板 — Charisma Pixel / 极魅像素 (第4套UI主题: 歧路旅人 HD-2D 神恩紫红金复古纯像素)
 * 运用色彩心理学与 16 位游戏机 / HD-2D 美术设计:
 * - 神恩紫罗兰 (#8B5CF6): 象征神秘、高贵神圣与极魅引力 (Charisma/Grace)，带来深邃神往感
 * - 醉梦丝绒红 (#E11D48): 剧场帷幕与舞娘红丝绒，高多巴胺舞台聚焦与行动力
 * - 帝国流金 (#F59E0B): 水晶吊灯与古典浮雕鎏金，提供成就感与荣誉高光
 * - 剧院黑曜深渊 (#0C0A14): 沉静极暗紫黑底，营造 HD-2D 剧场聚光灯明暗戏剧张力
 * - 香槟丝白 (#FDF4FF): 带有微暖紫金调的高光文本，WCAG AAA 15.8:1 极致可读性
 */
const STAR_CHARISMA_PALETTE: ColorToken[] = [
  { name: "--color-primary", hex: "#8B5CF6" },
  { name: "--color-secondary", hex: "#E11D48" },
  { name: "--color-accent", hex: "#F59E0B" },
  { name: "--color-success", hex: "#10B981" },
  { name: "--color-warning", hex: "#FBBF24" },
  { name: "--color-danger", hex: "#F43F5E" },
  { name: "--color-neutral", hex: "#948DA5" },
  { name: "--color-surface", hex: "#0C0A14" },
  { name: "--color-surface-2", hex: "#171326" },
  { name: "--color-text", hex: "#FDF4FF" },
  { name: "--color-text-dim", hex: "#A79BB7" },
  { name: "--color-border", hex: "#2C2245" },
];

/**
 * Star 调色板 — Shanghai Yuefenpai / 老上海月份牌 (第5套UI主题: 民国海派摩登擦笔水彩与 Art Deco 暖象牙宣纸)
 * 融合 1920-1930s 民国海派经典月份牌、双线金纹画框与擦笔水彩画法:
 * - 旗袍朱砂茜红 (#B8282B): 典雅海派旗袍经典朱砂红，兼具历史厚度与摩登风韵
 * - 翡翠碧玉墨绿 (#1B5E48): 翡翠耳坠与墨玉手镯，冷暖互补的中式典雅视觉平衡
 * - 留声机老鎏金 (#D49E35): 黄铜号角与古典烫金花边，展现黄金时代的流光溢彩
 * - 象牙暖宣纸底 (#F5E8C7): 经时光沉淀的暖调石版印刷月份牌老宣纸，温润不刺眼
 * - 老上海炭墨黑 (#221C16): 传统徽墨书卷炭黑，字字清晰，温润雅致，WCAG AAA 13.5:1
 */
const STAR_SHANGHAI_PALETTE: ColorToken[] = [
  { name: "--color-primary", hex: "#B8282B" },
  { name: "--color-secondary", hex: "#1B5E48" },
  { name: "--color-accent", hex: "#D49E35" },
  { name: "--color-success", hex: "#1B5E48" },
  { name: "--color-warning", hex: "#D49E35" },
  { name: "--color-danger", hex: "#B8282B" },
  { name: "--color-neutral", hex: "#8C7E70" },
  { name: "--color-surface", hex: "#F5E8C7" },
  { name: "--color-surface-2", hex: "#EBD8B0" },
  { name: "--color-text", hex: "#221C16" },
  { name: "--color-text-dim", hex: "#6B6055" },
  { name: "--color-border", hex: "#C89228" },
];

/** 间距 token (4px 基础栅格) */
const STAR_SPACING: SpacingToken[] = [
  { name: "--space-1", px: 4 },
  { name: "--space-2", px: 8 },
  { name: "--space-3", px: 12 },
  { name: "--space-4", px: 16 },
  { name: "--space-6", px: 24 },
  { name: "--space-8", px: 32 },
  { name: "--space-12", px: 48 },
  { name: "--space-16", px: 64 },
];

/** 圆角 token (3 档, per ui-3pane-arch.md §2.4) */
const STAR_RADII: RadiusToken[] = [
  { name: "--radius-sm", px: 4 },
  { name: "--radius-md", px: 6 },
  { name: "--radius-lg", px: 10 },
];

/** 纯像素圆角 token (8-bit 纯直角阶梯, 0px 矢量圆角 — 仅 Charisma Pixel 独占) */
const PIXEL_RADII: RadiusToken[] = [
  { name: "--radius-sm", px: 0 },
  { name: "--radius-md", px: 0 },
  { name: "--radius-lg", px: 0 },
];

/** 老上海月份牌画框圆角 token (细腻微内圆角 / 民国画框边线) */
const SHANGHAI_RADII: RadiusToken[] = [
  { name: "--radius-sm", px: 2 },
  { name: "--radius-md", px: 4 },
  { name: "--radius-lg", px: 8 },
];

/** 内置主题 (暗夜神格 + 少年原画 + 圣夜霜雪 + 极魅像素 + 老上海月份牌) */
export const THEMES: ThemeDefinition[] = [
  {
    id: "dark",
    displayName: "暗夜神格",
    isDark: true,
    colors: STAR_DARK_PALETTE,
    spacings: STAR_SPACING,
    radii: STAR_RADII,
    version: 2,
  },
  {
    id: "light",
    displayName: "少年原画",
    isDark: false,
    colors: STAR_LIGHT_PALETTE,
    spacings: STAR_SPACING,
    radii: STAR_RADII,
    version: 2,
  },
  {
    id: "christmas",
    displayName: "圣夜霜雪",
    isDark: true,
    colors: STAR_CHRISTMAS_PALETTE,
    spacings: STAR_SPACING,
    radii: STAR_RADII,
    version: 2,
  },
  {
    id: "charisma",
    displayName: "极魅像素",
    isDark: true,
    colors: STAR_CHARISMA_PALETTE,
    spacings: STAR_SPACING,
    radii: PIXEL_RADII,
    version: 1,
  },
  {
    id: "shanghai",
    displayName: "老上海月份牌",
    isDark: false,
    colors: STAR_SHANGHAI_PALETTE,
    spacings: STAR_SPACING,
    radii: SHANGHAI_RADII,
    version: 1,
  },
];

/** 按 id 查找主题 */
export function getTheme(id: ThemeId): ThemeDefinition | undefined {
  return THEMES.find((t) => t.id === id);
}

/** 把主题转 CSS 变量 (注入 :root 或 .dark) */
export function themeToCss(theme: ThemeDefinition): string {
  const lines: string[] = [];
  for (const c of theme.colors) {
    lines.push(`  ${c.name}: ${c.hex};`);
  }
  for (const s of theme.spacings) {
    lines.push(`  ${s.name}: ${s.px}px;`);
  }
  for (const r of theme.radii) {
    lines.push(`  ${r.name}: ${r.px}px;`);
  }
  return lines.join("\n");
}

/** 三层解析优先级 (Personal > Tenant > Global) */
export const SCOPE_PRIORITY: Record<ThemeScope, number> = {
  personal: 3,
  tenant: 2,
  global: 1,
};
