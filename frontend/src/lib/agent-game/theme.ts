// =====================================================================
// Agent Game — Theme (日漫 + 武侠 + 赛博朋克 主题系统, per 9/5 12:33 JST 拍板)
// =====================================================================
// 拍板结果 (per ask_user ask_635d0b81cfd9b1dfc63fd70f):
//   - 画法: SVG 手工画
//   - 主色板: 墨黑底 + 朱红 + 霓虹青 + 金
//   - 装饰: 全装饰 (能量光环 + 墨汁拖尾 + 印章/水印 + 神侠风格 Lv 5+)
//   - 范围: RoguelikeCanvas + AgentCanvasView
//
// 设计目标:
//   1. 纯常量模块, 0 副作用, 0 IO
//   2. 可被 characters.ts / enemies.ts / Decorations.tsx 共享
//   3. dark mode 优先, 不依赖 CSS 变量
//   4. 简洁 API, 一次 import 拿到所有 token
// =====================================================================

/** 主色板 (per 拍板 #2) */
export const COLORS = {
  // 墨黑 (背景, 基础)
  inkBlack: "#0d0d12",         // 主背景
  inkDark: "#15151c",          // 次背景 (cell, panel)
  inkMid: "#1f1f28",           // 中背景
  inkLight: "#2a2a35",         // 浅背景
  // 朱红 (强调, 警告, 攻击)
  vermilion: "#dc2626",        // 朱红 (Lv 1-3 默认)
  vermilionGlow: "#ef4444",    // 朱红发光
  // 霓虹青 (accent, 信息, 高亮)
  neonCyan: "#06b6d4",         // 霓虹青
  neonCyanGlow: "#22d3ee",     // 霓虹青发光
  // 金 (最高级, 宝箱, 神)
  gold: "#f59e0b",             // 金
  goldGlow: "#fbbf24",         // 金发光
  // 紫 (高级, 神秘)
  cyberPurple: "#a855f7",      // 赛博紫
  cyberPurpleGlow: "#c084fc",  // 赛博紫发光
  // 灰 (基础, 中性)
  ash: "#6b7280",
  ashLight: "#9ca3af",
  // 白 (文字)
  paper: "#f8f5f0",            // 日式宣纸白
} as const;

/** 字体 (per 拍板, 日式风格) */
export const FONTS = {
  /** 主字体: UI / 标题 (sans-serif, 加粗) */
  primary: '"Hiragino Sans", "Yu Gothic", "Meiryo", system-ui, sans-serif',
  /** 等宽字体: 数据 / 数字 / id (monospace) */
  mono: '"SF Mono", "Cascadia Code", "Consolas", "Menlo", monospace',
  /** 印章字体: agent name (decorative) */
  stamp: '"Hiragino Mincho ProN", "Yu Mincho", "MS Mincho", serif',
} as const;

/** 装饰元素尺寸 (per 拍板 #3, 全装饰) */
export const DECORATION = {
  /** 能量光环 (外圈脉动) */
  energyRingRadius: 40,        // px (相对)
  energyRingStroke: 2,
  /** 墨汁拖尾 (移动残影) */
  inkTrailLength: 4,            // 残影数
  inkTrailOpacityDecay: 0.25,  // 每次衰减
  /** 印章 (agent name 旁) */
  stampSize: 12,
  stampBorder: 1.5,
  /** 神侠光环 (Lv 7+ 才显示) */
  divineHaloRadius: 28,
} as const;

/** 角色等级视觉 tier (per 拍板 #3, 6 段渐进) */
export const CHARACTER_TIERS = [
  { level: 1, name: "游侠", color: COLORS.ash, accent: COLORS.ashLight, hasHalo: false, hasSword: false, hasArmor: false, hasCloak: false, hasCrown: false },
  { level: 2, name: "武童", color: COLORS.ashLight, accent: COLORS.paper, hasHalo: false, hasSword: false, hasArmor: false, hasCloak: false, hasCrown: false },
  { level: 3, name: "剑客", color: COLORS.neonCyan, accent: COLORS.neonCyanGlow, hasHalo: false, hasSword: true, hasArmor: false, hasCloak: true, hasCrown: false },
  { level: 5, name: "侠客", color: COLORS.vermilion, accent: COLORS.vermilionGlow, hasHalo: false, hasSword: true, hasArmor: true, hasCloak: true, hasCrown: false },
  { level: 7, name: "剑圣", color: COLORS.cyberPurple, accent: COLORS.cyberPurpleGlow, hasHalo: true, hasSword: true, hasArmor: true, hasCloak: true, hasCrown: true },
  { level: 10, name: "神侠", color: COLORS.gold, accent: COLORS.goldGlow, hasHalo: true, hasSword: true, hasArmor: true, hasCloak: true, hasCrown: true },
] as const;

/** 敌人光球类型 (per 拍板 #1, 6 种光球) */
export const ENEMY_TYPES = [
  { key: "neon_blue", name: "青光球", color: COLORS.neonCyan, glow: COLORS.neonCyanGlow, emoji: "🔵" },
  { key: "vermilion_fire", name: "朱火光球", color: COLORS.vermilion, glow: COLORS.vermilionGlow, emoji: "🔴" },
  { key: "gold_thunder", name: "金雷光球", color: COLORS.gold, glow: COLORS.goldGlow, emoji: "⚡" },
  { key: "purple_shadow", name: "紫影光球", color: COLORS.cyberPurple, glow: COLORS.cyberPurpleGlow, emoji: "🟣" },
  { key: "white_paper", name: "白纸光球", color: COLORS.paper, glow: COLORS.ashLight, emoji: "⚪" },
  { key: "boss_divine", name: "神光球", color: COLORS.gold, glow: COLORS.goldGlow, emoji: "👁" },
] as const;

/** 由 work-item priority 推敌人类型 (per mapgen / Roguelike) */
export function enemyTypeForPriority(priority: string): typeof ENEMY_TYPES[number] {
  switch (priority) {
    case "p0": return ENEMY_TYPES[1];  // 朱火光球
    case "p1": return ENEMY_TYPES[2];  // 金雷光球
    case "p2": return ENEMY_TYPES[0];  // 青光球
    case "p3": return ENEMY_TYPES[4];  // 白纸光球
    default:  return ENEMY_TYPES[3];  // 紫影光球
  }
}

/** 随机挑 1 个敌人类型 (纯函数, 接受 seed) */
export function pickRandomEnemyType(seed: number): typeof ENEMY_TYPES[number] {
  const idx = Math.abs(Math.floor(seed)) % ENEMY_TYPES.length;
  return ENEMY_TYPES[idx]!;
}
