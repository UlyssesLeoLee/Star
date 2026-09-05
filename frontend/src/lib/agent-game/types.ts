// =====================================================================
// Agent Game — Types (拟人化游戏化 v0.1, per 2026-09-05 11:42 JST 拍板)
// =====================================================================
// 拍板结果 (per ask_user ask_2a41eb2fff282abd3d58e45e):
//   - 死亡触发: cost 预算超支 = 死亡, 回 Lv 1, 保留 50% 金币
//   - 肉鸽随机: 5 选 1 power-up (per-life, 升级时选)
//   - 视觉变化: Lv 1..10 渐进 (色/大小/光环)
//   - 钱币 + 复活: 1 种金币, 完成 work-item 给 1-5 金币, 死亡扣 50 复活
//
// 数据模型 (5 维):
//   level      1..10 (满级后攒 xp 没用)
//   xp         当前等级内的 xp, 升级后清零
//   coins      金币, 死后保留 50%
//   hp         0..100, ≤0 = 死亡, cost 增长时扣
//   perks      5 选 1 累计 (per-life, 死后清空)
//
// 跟 store.ts (zustand) 集成: agentGameStates 存到现有 store (避免再开一个 store)
// =====================================================================

import type { WorkItem, AgentSession } from "@/types/ids";

/** Agent 等级范围 (per 拍板 Lv 1..10) */
export const MIN_LEVEL = 1;
export const MAX_LEVEL = 10;

/** HP 上限 (per 拍板) */
export const MAX_HP = 100;

/** 5 个 Power-up (per 拍板 #2, 5 选 1 累计 per-life) */
export type PerkId = "xp_boost" | "coin_magnet" | "bounty_hunter" | "iron_will" | "lucky_star";

/** Perk 定义 */
export interface PerkDefinition {
  id: PerkId;
  name: string;
  description: string;
  icon: string;             // emoji 简化渲染
  /** 累计叠加模式: boolean (true = 同 perk 可选多次叠加) */
  stackable: boolean;
}

/** 5 个 Perk 静态定义 */
export const PERKS: ReadonlyArray<PerkDefinition> = [
  {
    id: "xp_boost",
    name: "XP Boost",
    description: "+25% xp 获得 (累计叠加, 上限 +100%)",
    icon: "📈",
    stackable: true,
  },
  {
    id: "coin_magnet",
    name: "Coin Magnet",
    description: "+1 coin / completion (累计叠加)",
    icon: "🪙",
    stackable: true,
  },
  {
    id: "bounty_hunter",
    name: "Bounty Hunter",
    description: "p0 完成给 2x coins (累计, 最多 3x)",
    icon: "🎯",
    stackable: true,
  },
  {
    id: "iron_will",
    name: "Iron Will",
    description: "-25% HP 损失 (累计, 最多 -75%)",
    icon: "🛡️",
    stackable: true,
  },
  {
    id: "lucky_star",
    name: "Lucky Star",
    description: "1% 概率双倍升级 (每次升级独立判定)",
    icon: "🍀",
    stackable: false,
  },
] as const;

/** Agent 完整游戏化状态 */
export interface AgentGameState {
  agentId: string;
  level: number;                    // 1..10
  xp: number;                       // 当前等级内的 xp
  coins: number;                    // 金币
  hp: number;                       // 0..100
  alive: boolean;                   // 活着?
  perks: PerkId[];                  // 选过的 perk (per-life, 死后清空)
  deaths: number;                   // 死亡次数 (跨 life 累计)
  revives: number;                  // 复活次数 (跨 life 累计)
  completedMissions: number;        // 完成任务数 (跨 life 累计)
  highestLevel: number;             // 历史最高级 (跨 life 累计)
  /** last claim: { wiId: timestamp } 防重复刷 */
  lastClaimAt: Record<string, string>;
}

/** 初始状态 (新 agent / 重开后) */
export function createInitialGameState(agentId: string, budgetUsd: number): AgentGameState {
  return {
    agentId,
    level: 1,
    xp: 0,
    coins: 0,
    hp: MAX_HP,
    alive: true,
    perks: [],
    deaths: 0,
    revives: 0,
    completedMissions: 0,
    highestLevel: 1,
    lastClaimAt: {},
  };
}

/** 完成 work-item 获得的 xp (per priority) */
export function xpForWorkItem(wi: Pick<WorkItem, "priority">): number {
  switch (wi.priority) {
    case "p0": return 20;
    case "p1": return 10;
    case "p2": return 5;
    case "p3": return 2;
    default:  return 5;
  }
}

/** 完成 work-item 获得的 base coin (per priority, 不含 perk 加成) */
export function baseCoinForWorkItem(wi: Pick<WorkItem, "priority">): number {
  switch (wi.priority) {
    case "p0": return 5;
    case "p1": return 3;
    case "p2": return 2;
    case "p3": return 1;
    default:  return 2;
  }
}

/** Level 升级所需 xp (sigmoid 曲线, 10 满级) */
export const XP_TO_NEXT_LEVEL: ReadonlyArray<number> = [
  50,    // Lv 1 → 2
  100,   // Lv 2 → 3
  200,   // Lv 3 → 4
  350,   // Lv 4 → 5
  500,   // Lv 5 → 6
  750,   // Lv 6 → 7
  1000,  // Lv 7 → 8
  1500,  // Lv 8 → 9
  2500,  // Lv 9 → 10
  Infinity, // Lv 10 满级
];

/** Agent 视觉配置 (per 拍板 #3, Lv 1..10 渐进) */
export interface AgentVisualTier {
  level: number;
  /** 主色 (hex) */
  color: string;
  /** 节点大小 (相对, 1.0 = 基准 220x110) */
  scale: number;
  /** 边框宽度 (px) */
  borderWidth: number;
  /** 装饰 emoji (空 = 无) */
  decoration: string;
  /** tier 名称 */
  tierName: string;
}

/** 10 段视觉等级 (per 拍板 #3) */
export const AGENT_VISUAL_TIERS: ReadonlyArray<AgentVisualTier> = [
  { level: 1,  color: "#6e7681", scale: 0.80, borderWidth: 1,   decoration: "",      tierName: "新手" },
  { level: 2,  color: "#8b949e", scale: 0.85, borderWidth: 1,   decoration: "",      tierName: "学徒" },
  { level: 3,  color: "#58a6ff", scale: 0.90, borderWidth: 1.5, decoration: "",      tierName: "熟练" },
  { level: 4,  color: "#388bfd", scale: 0.95, borderWidth: 1.5, decoration: "",      tierName: "进阶" },
  { level: 5,  color: "#3fb950", scale: 1.00, borderWidth: 2,   decoration: "✨",    tierName: "老兵" },
  { level: 6,  color: "#2ea043", scale: 1.05, borderWidth: 2,   decoration: "✨",    tierName: "精英" },
  { level: 7,  color: "#a371f7", scale: 1.10, borderWidth: 2.5, decoration: "💫",    tierName: "专家" },
  { level: 8,  color: "#8957e5", scale: 1.15, borderWidth: 2.5, decoration: "💫",    tierName: "大师" },
  { level: 9,  color: "#d29922", scale: 1.20, borderWidth: 3,   decoration: "🌟",    tierName: "传奇" },
  { level: 10, color: "#f0b429", scale: 1.30, borderWidth: 3,   decoration: "👑",    tierName: "神话" },
] as const;

/** 由 level 取视觉 tier (clamp) */
export function visualForLevel(level: number): AgentVisualTier {
  const idx = Math.max(0, Math.min(MAX_LEVEL - 1, Math.floor(level) - 1));
  return AGENT_VISUAL_TIERS[idx];
}

/** 复活 cost (per 拍板 #4, 50 金币) */
export const REVIVE_COST = 50;

/** 死亡时金币保留率 (per 拍板 #1, 50%) */
export const DEATH_COIN_KEEP_RATIO = 0.5;

/** Death trigger 事件 payload */
export interface DeathEvent {
  agentId: string;
  /** death 触发原因 (cost 比例) */
  triggerCostRatio: number;
  /** 死亡时的 coins / level / hp 快照 (供 modal 展示) */
  snapshotCoins: number;
  snapshotLevel: number;
  snapshotHp: number;
  /** 死亡时是否可复活 (coins >= REVIVE_COST) */
  canRevive: boolean;
  timestamp: string;
}

/** Level up 事件 payload */
export interface LevelUpEvent {
  agentId: string;
  fromLevel: number;
  toLevel: number;
  /** 满级标记 (Lv 10 之后不再 level up) */
  isMaxLevel: boolean;
  /** 选 perk 用的 5 选 1 列表 (per 拍板) */
  perkChoices: ReadonlyArray<PerkDefinition>;
  /** lucky_star 触发双倍升级 (per Perk) */
  doubleTriggered: boolean;
  timestamp: string;
}

/** Claim 奖励事件 payload (work-item 完成领奖) */
export interface ClaimEvent {
  agentId: string;
  workItemId: string;
  xp: number;
  coins: number;
  leveledUp: boolean;
  timestamp: string;
}
