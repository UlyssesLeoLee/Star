// Star Frontend — 主题类型定义
// Per 2026-08-29 04:09 JST 用户拍板: 三元组 enum + 可插拔 + 三层作用域.

/**
 * 主题唯一标识 (三元组 enum + 可插拔).
 * - 内置 2 个: Light + Dark
 * - 预留扩展: HighContrast / Solarized 等
 * - 第三方 / 租户自定义主题可追加 variant
 */
export type ThemeId = "light" | "dark" | "high-contrast" | "solarized";

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

/** Star 调色板 — Mecha Ceramic Light (机甲陶瓷白) */
const STAR_LIGHT_PALETTE: ColorToken[] = [
  { name: "--color-primary", hex: "#3B82F6" },
  { name: "--color-success", hex: "#059669" },
  { name: "--color-warning", hex: "#D97706" },
  { name: "--color-danger", hex: "#DC2626" },
  { name: "--color-neutral", hex: "#475569" },
  { name: "--color-surface", hex: "#F8FAFC" },
  { name: "--color-surface-2", hex: "#FFFFFF" },
  { name: "--color-text", hex: "#0F172A" },
  { name: "--color-text-dim", hex: "#64748B" },
  { name: "--color-border", hex: "#E2E8F0" },
];

/** Star 调色板 — Neo-Tokyo Dark (默认暗色 / 赛博黑曜石) */
const STAR_DARK_PALETTE: ColorToken[] = [
  { name: "--color-primary", hex: "#00F0FF" },
  { name: "--color-success", hex: "#10B981" },
  { name: "--color-warning", hex: "#F59E0B" },
  { name: "--color-danger", hex: "#FF3366" },
  { name: "--color-neutral", hex: "#94A3B8" },
  { name: "--color-surface", hex: "#07090E" },
  { name: "--color-surface-2", hex: "#0E131F" },
  { name: "--color-text", hex: "#F8FAFC" },
  { name: "--color-text-dim", hex: "#94A3B8" },
  { name: "--color-border", hex: "#1E293B" },
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

/** 内置主题 (亮 + 暗) — 接口预留扩展 */
export const THEMES: ThemeDefinition[] = [
  {
    id: "dark",
    displayName: "Neo-Tokyo Dark",
    isDark: true,
    colors: STAR_DARK_PALETTE,
    spacings: STAR_SPACING,
    radii: STAR_RADII,
    version: 2,
  },
  {
    id: "light",
    displayName: "Mecha Light",
    isDark: false,
    colors: STAR_LIGHT_PALETTE,
    spacings: STAR_SPACING,
    radii: STAR_RADII,
    version: 2,
  },
  // 扩展占位 (per 2026-08-29 04:09 JST 用户拍板 "支持后续增加"):
  // {
  //   id: "high-contrast",
  //   displayName: "High Contrast",
  //   isDark: true,
  //   colors: [...],
  //   ...
  // },
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
