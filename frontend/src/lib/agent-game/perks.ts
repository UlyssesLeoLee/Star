// =====================================================================
// Agent Game — Perk 5 选 1 选择器
// =====================================================================
// Per 2026-09-05 11:42 JST 拍板 #2: 5 选 1 累计, 升级时随机抽 5 个 (perks pool)
//   - lucky_star 不可叠加 (stackable: false), 选了 2 次只算 1 次
//   - 其他 4 个 stackable, 可选多次叠加
//
// 选择算法 (per-life):
//   1. 从 PERKS (5 个) 随机选 5 个
//   2. lucky_star 第二次 (已经 perks 里有) 排除, 所以 pool 是 5 - (1 if already) = 4
//   3. 实际: 5 个中, lucky_star 出现 1 次, 其他 4 个每个可以出现 1-N 次
//   4. 选 5 个 (含 lucky_star 至多 1 次) → 5 选 1 用户点
//
// 简化: 每次升级都给完整 5 个列表 (per 拍板), 用户挑 1 个
//   - lucky_star 第二次选时, 实际效果是 "1% 概率双倍升级", 选多次无加成, 但我们仍让它可选
//   - 或者: lucky_star 第一次选后, 升级时改为 4 选 1 (排除 lucky_star)
//   - 拍板: 5 选 1 永远保持, lucky_star 可重复选, 但效果不叠加 (UI 提示)
// =====================================================================

import type { PerkDefinition, PerkId } from "./types";
import { PERKS } from "./types";

/** 5 选 1 列表 (永远是 5 个完整列表, 用户自由选) */
export function getPerkChoices(): ReadonlyArray<PerkDefinition> {
  return PERKS;
}

/** 检查 perk 是否可叠加 (per PERKS 定义) */
export function isPerkStackable(perkId: PerkId): boolean {
  const def = PERKS.find((p) => p.id === perkId);
  return def?.stackable ?? false;
}

/** 计算已选 perks 的 count map */
export function perkCounts(perks: PerkId[]): Record<PerkId, number> {
  const counts: Record<PerkId, number> = {
    xp_boost: 0,
    coin_magnet: 0,
    bounty_hunter: 0,
    iron_will: 0,
    lucky_star: 0,
  };
  for (const p of perks) {
    counts[p] = (counts[p] ?? 0) + 1;
  }
  return counts;
}
