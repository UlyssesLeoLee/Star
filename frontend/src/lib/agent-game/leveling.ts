// =====================================================================
// Agent Game — Leveling (纯函数, xp/level/coin/HP 计算)
// =====================================================================
// Per 2026-09-05 11:42 JST 拍板 + types.ts 定义:
//   - xp 累加到 XP_TO_NEXT_LEVEL[level-1] 触发升级
//   - 满级 Lv 10 后不再升级
//   - lucky_star perk 1% 概率双倍升级 (每次升级独立判定)
//   - iron_will perk -25% HP 损失 (累计, 最多 -75%)
//   - coin_magnet perk +1 coin / completion (累计)
//   - bounty_hunter perk p0 给 2x coins (累计, 最多 3x)
//   - xp_boost perk +25% xp (累计, 上限 +100%)
//
// 关键不变量:
//   - 所有函数纯函数, 无副作用, 无 Date.now(), 无 random (除 lucky_star 判定)
//   - 接受随机种子 (randomSeed) 便于测试 deterministic
// =====================================================================

import type { AgentGameState, PerkId } from "./types";
import {
  MAX_HP, MAX_LEVEL, REVIVE_COST, DEATH_COIN_KEEP_RATIO,
  XP_TO_NEXT_LEVEL,
  xpForWorkItem, baseCoinForWorkItem,
  createInitialGameState,
} from "./types";
import type { WorkItem, AgentSession } from "@/types/ids";

// re-export 让 store.ts 一处 import
export { createInitialGameState };

// ---- Perk 加成计算 (纯函数) ----

/** xp_boost: +25% per stack, 上限 +100% (= 2x) */
export function xpMultiplier(perks: PerkId[]): number {
  const stacks = perks.filter((p) => p === "xp_boost").length;
  return 1 + Math.min(4, stacks) * 0.25;
}

/** coin_magnet: +1 coin per stack, 上限 +5 (跟 ui 简化) */
export function coinMagnetBonus(perks: PerkId[]): number {
  return perks.filter((p) => p === "coin_magnet").length;
}

/** bounty_hunter: p0 时 coin × (1 + 0.5*stack), 最多 3 stacks (= 2.5x) */
export function bountyMultiplier(perks: PerkId[], priority: WorkItem["priority"]): number {
  if (priority !== "p0") return 1;
  const stacks = perks.filter((p) => p === "bounty_hunter").length;
  return 1 + Math.min(3, stacks) * 0.5;
}

/** iron_will: HP 损失乘 (1 - 0.25*stack), 最多 3 stacks (= 0.25) */
export function ironWillMultiplier(perks: PerkId[]): number {
  const stacks = perks.filter((p) => p === "iron_will").length;
  return Math.max(0.25, 1 - Math.min(3, stacks) * 0.25);
}

/** lucky_star 触发判定 (1% per level-up, 独立) */
export function luckyStarTriggered(perks: PerkId[], random01: number): boolean {
  if (!perks.includes("lucky_star")) return false;
  return random01 < 0.01;
}

// ---- 核心计算 ----

/**
 * 计算 claim 奖励 (完成 work-item → xp + coins)
 *   - 纯函数, 接受 perks + wi + randomSeed (0..1) for lucky_star
 *   - 返回 { xp, coins }
 */
export function computeClaim(
  wi: Pick<WorkItem, "priority">,
  perks: PerkId[],
  _random01: number,        // reserved for future randomness (currently lucky_star 不用)
): { xp: number; coins: number } {
  void _random01;
  const baseXp = xpForWorkItem(wi);
  const baseCoin = baseCoinForWorkItem(wi);
  const xpMult = xpMultiplier(perks);
  const coinBonus = coinMagnetBonus(perks);
  const bountyMult = bountyMultiplier(perks, wi.priority);
  return {
    xp: Math.round(baseXp * xpMult),
    coins: Math.round((baseCoin * bountyMult) + coinBonus),
  };
}

/**
 * 应用 claim 奖励到 game state
 *   - 纯函数, 返回新 state (不 mutate)
 *   - 检测升级, 返回 { state, leveledUp, levelsGained, isMaxLevel }
 */
export function applyClaim(
  state: AgentGameState,
  xp: number,
  coins: number,
  random01ForLucky: number,
): { state: AgentGameState; leveledUp: boolean; levelsGained: number; isMaxLevel: boolean } {
  if (!state.alive) return { state, leveledUp: false, levelsGained: 0, isMaxLevel: false };
  if (state.level >= MAX_LEVEL) {
    // 满级: xp 不再涨, coins 仍累
    return {
      state: { ...state, coins: state.coins + coins, completedMissions: state.completedMissions + 1 },
      leveledUp: false,
      levelsGained: 0,
      isMaxLevel: true,
    };
  }
  let newXp = state.xp + xp;
  let newLevel = state.level;
  let levelsGained = 0;
  while (newLevel < MAX_LEVEL) {
    const need = XP_TO_NEXT_LEVEL[newLevel - 1];
    if (newXp < need) break;
    newXp -= need;
    newLevel += 1;
    levelsGained += 1;
  }
  const isMaxLevel = newLevel >= MAX_LEVEL;
  return {
    state: {
      ...state,
      xp: newXp,
      level: newLevel,
      coins: state.coins + coins,
      completedMissions: state.completedMissions + 1,
      highestLevel: Math.max(state.highestLevel, newLevel),
    },
    leveledUp: levelsGained > 0,
    levelsGained,
    isMaxLevel,
  };
}

/** 选 perk (升级时 5 选 1) */
export function applyPerkChoice(
  state: AgentGameState,
  perkId: PerkId,
): AgentGameState {
  return { ...state, perks: [...state.perks, perkId] };
}

/**
 * 应用 cost 增长 (HP 扣血)
 *   - costDelta: 本次 cost 增长 (usd)
 *   - costBudget: 预算上限 (usd, 通常从 agent.cost_summary.budget_usd)
 *   - 返回 { state, died }
 *   - iron_will 应用: hpLoss = round(costDelta / costBudget * 100 * ironWillMult)
 *   - hp <= 0 = died (caller 处理)
 */
export function applyCostSpend(
  state: AgentGameState,
  costDelta: number,
  costBudget: number,
): { state: AgentGameState; died: boolean; triggerCostRatio: number } {
  if (!state.alive || costDelta <= 0 || costBudget <= 0) {
    return { state, died: false, triggerCostRatio: 0 };
  }
  const ratio = costDelta / costBudget;
  const ironMult = ironWillMultiplier(state.perks);
  const hpLoss = Math.round(ratio * MAX_HP * ironMult);
  const newHp = Math.max(0, state.hp - hpLoss);
  const died = newHp <= 0;
  return {
    state: { ...state, hp: newHp },
    died,
    triggerCostRatio: ratio,
  };
}

/**
 * 死亡处理: 保留 50% 金币, level/xp/perks 清零, HP 满, alive=false
 *   - 死亡次数 +1
 */
export function applyDeath(state: AgentGameState): AgentGameState {
  return {
    ...state,
    alive: false,
    level: 1,
    xp: 0,
    coins: Math.floor(state.coins * DEATH_COIN_KEEP_RATIO),
    hp: MAX_HP,
    perks: [],
    deaths: state.deaths + 1,
  };
}

/**
 * 复活 (扣 REVIVE_COST 金币, HP 满, level/xp 重置, perks 清空, alive=true, revives+1)
 *   - 复活时 level 也是 1 (per 拍板: 复活 vs 重开都是 Lv 1)
 *   - 如果 coins < REVIVE_COST → 返回 { ok: false, reason: "insufficient_coins" }
 */
export function applyRevive(
  state: AgentGameState,
): { state: AgentGameState; ok: boolean; reason?: "insufficient_coins" | "already_alive" } {
  if (state.alive) return { state, ok: false, reason: "already_alive" };
  if (state.coins < REVIVE_COST) return { state, ok: false, reason: "insufficient_coins" };
  return {
    state: {
      ...state,
      alive: true,
      level: 1,
      xp: 0,
      coins: state.coins - REVIVE_COST,
      hp: MAX_HP,
      perks: [],
      revives: state.revives + 1,
    },
    ok: true,
  };
}

/**
 * 重开 (不扣币, level/xp/perks 清零, HP 满, alive=true)
 *   - 不增 deaths (跟 复活 区别)
 *   - 用于 "已死但没钱复活" 时的备选
 */
export function applyRestart(state: AgentGameState): AgentGameState {
  return {
    ...state,
    alive: true,
    level: 1,
    xp: 0,
    hp: MAX_HP,
    perks: [],
  };
}

/**
 * 从 agent session 重新初始化 (新 agent 或重置)
 *   - 用于 store 初始化, 跟 types.ts createInitialGameState 一致
 */
export function freshGameState(agent: AgentSession): AgentGameState {
  return {
    agentId: agent.id,
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

/** Utility: 当前等级还需要多少 xp 满级 */
export function xpProgress(state: AgentGameState): { current: number; need: number; ratio: number } {
  if (state.level >= MAX_LEVEL) return { current: 0, need: 0, ratio: 1 };
  const need = XP_TO_NEXT_LEVEL[state.level - 1];
  return { current: state.xp, need, ratio: state.xp / need };
}
