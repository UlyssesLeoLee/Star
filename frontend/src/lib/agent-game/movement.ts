// =====================================================================
// Agent Game — Movement (Roguelike 4-邻接 step)
// =====================================================================
// Per 2026-09-05 12:23 JST 拍板 (ask_8a60a3bc90f779308a69be1d):
//   - 点击移动, 4-邻接 (上下左右)
//   - 每 step 消耗 cost + 触发 cell 效果
//   - 死亡 = all agents freeze 等玩家重开
// =====================================================================

import type { GameMap, MapCell } from "./mapgen";
import { isAdjacent } from "./mapgen";
import type { AgentGameState } from "./types";
import { MAX_HP } from "./types";

/** 移动 1 步的 cost (usd, 跟现有 ⚡ Step 按钮一致) */
export const STEP_COST_USD = 0.1;

/** 移动结果 (事件化, 跟 claimReward 一致风格) */
export type MoveResult =
  | {
      ok: true;
      from: { x: number; y: number };
      to: { x: number; y: number };
      cell: MapCell;
      /** cost 增长 (已应用 HP 扣血 + cell 效果) */
      died: boolean;
      hpAfter: number;
      coinsAfter: number;
      triggerCostRatio: number;
      /** cell 效果 (已应用到 game state) */
      effect: CellEffect;
      /** 是否达到 boss */
      reachedBoss: boolean;
    }
  | {
      ok: false;
      reason: "not_adjacent" | "trap_cell_blocked" | "agent_dead" | "no_map" | "same_cell";
    };

/** Cell 触发效果类型 (per mapgen.ts) */
export type CellEffect =
  | { kind: "none" }
  | { kind: "enemy"; workItemId: string | undefined }
  | { kind: "treasure"; description: string; /** coin delta */ coinsDelta: number; /** hp delta */ hpDelta: number }
  | { kind: "trap"; description: string; coinsDelta: number; hpDelta: number }
  | { kind: "boss" };

/** 计算 cell 效果 (纯函数) */
export function computeCellEffect(cell: MapCell): CellEffect {
  switch (cell.type) {
    case "enemy":
      return { kind: "enemy", workItemId: cell.workItemId };
    case "treasure": {
      // treasure 给 bonus
      const desc = cell.description ?? "+5 coin";
      if (desc.includes("coin")) return { kind: "treasure", description: desc, coinsDelta: 5, hpDelta: 0 };
      if (desc.includes("XP")) return { kind: "treasure", description: desc, coinsDelta: 0, hpDelta: 0 };  // XP 由 claim 触发, 此处跳过
      if (desc.includes("revive")) return { kind: "treasure", description: desc, coinsDelta: 0, hpDelta: 0 };  // P2
      if (desc.includes("HP")) return { kind: "treasure", description: desc, coinsDelta: 0, hpDelta: 20 };
      if (desc.includes("perk")) return { kind: "treasure", description: desc, coinsDelta: 10, hpDelta: 0 };
      return { kind: "treasure", description: desc, coinsDelta: 5, hpDelta: 0 };
    }
    case "trap": {
      const desc = cell.description ?? "-20 HP";
      if (desc.includes("HP")) return { kind: "trap", description: desc, coinsDelta: 0, hpDelta: -20 };
      if (desc.includes("coin")) return { kind: "trap", description: desc, coinsDelta: -10, hpDelta: 0 };
      if (desc.includes("XP")) return { kind: "trap", description: desc, coinsDelta: 0, hpDelta: 0 };
      if (desc.includes("perk")) return { kind: "trap", description: desc, coinsDelta: 0, hpDelta: 0 };  // P2
      if (desc.includes("level")) return { kind: "trap", description: desc, coinsDelta: 0, hpDelta: 0 };  // P2
      return { kind: "trap", description: desc, coinsDelta: 0, hpDelta: -20 };
    }
    case "boss":
      return { kind: "boss" };
    case "blank":
    case "start":
    default:
      return { kind: "none" };
  }
}

/** 应用 cell 效果到 game state (纯函数) */
export function applyCellEffect(
  state: AgentGameState,
  effect: CellEffect,
): AgentGameState {
  if (!state.alive) return state;
  if (effect.kind === "treasure" || effect.kind === "trap") {
    const newCoins = Math.max(0, state.coins + effect.coinsDelta);
    const newHp = Math.max(0, Math.min(MAX_HP, state.hp + effect.hpDelta));
    return { ...state, coins: newCoins, hp: newHp };
  }
  return state;
}

/** 主入口: 移动 1 步 */
export function moveAgent(
  state: AgentGameState,
  map: GameMap | null,
  currentPos: { x: number; y: number } | null,
  targetPos: { x: number; y: number },
  costBudget: number,
): MoveResult {
  if (!state.alive) return { ok: false, reason: "agent_dead" };
  if (!map) return { ok: false, reason: "no_map" };
  if (!currentPos) return { ok: false, reason: "no_map" };
  if (currentPos.x === targetPos.x && currentPos.y === targetPos.y) {
    return { ok: false, reason: "same_cell" };
  }
  if (!isAdjacent(currentPos, targetPos)) {
    return { ok: false, reason: "not_adjacent" };
  }
  const cell = map.cells[targetPos.y]?.[targetPos.x];
  if (!cell) return { ok: false, reason: "not_adjacent" };
  // trap 不可进入 (per 拍板 BFS, trap 也不可穿越)
  if (cell.type === "trap") {
    return { ok: false, reason: "trap_cell_blocked" };
  }

  // 1) 移动成功 → 走 1 step, cost +$0.1
  const costDelta = STEP_COST_USD;
  const ratio = costDelta / costBudget;
  const ironMult = Math.max(0.25, 1 - Math.min(3, state.perks.filter((p) => p === "iron_will").length) * 0.25);
  const hpLoss = Math.round(ratio * MAX_HP * ironMult);
  const newHp = Math.max(0, state.hp - hpLoss);
  const died = newHp <= 0;

  // 2) cell 效果
  const effect = computeCellEffect(cell);
  const stateAfterCell = applyCellEffect({ ...state, hp: newHp }, effect);

  return {
    ok: true,
    from: currentPos,
    to: targetPos,
    cell,
    died,
    hpAfter: stateAfterCell.hp,
    coinsAfter: stateAfterCell.coins,
    triggerCostRatio: ratio,
    effect,
    reachedBoss: cell.type === "boss",
  };
}
