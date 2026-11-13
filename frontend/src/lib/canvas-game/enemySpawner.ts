/*
@cypher schema=1 source_sha256=c76e9c10d830bd1feb30fa5b78eeafc8d6220e2834e7a82b42573fb124549ead
MERGE (self:File {path:"frontend/src/lib/canvas-game/enemySpawner.ts"})
MERGE (fn:Function {name:"enemySpawner::enemiesFromCanvas",language:"typescript"})
MERGE (step:Function {name:"enemySpawner::stepEnemies",language:"typescript"})
MERGE (types:File {path:"frontend/src/lib/canvas-game/types.ts"})
MERGE (self)-[:DEFINES]->(fn)
MERGE (self)-[:DEFINES]->(step)
MERGE (self)-[:IMPORTS]->(types)
@endcypher
*/
// =====================================================================
// enemySpawner — 从 canvas 元素生成敌人 + 敌人 AI 步进 (per WBS §23.36)
// 关卡素材 = canvas 的 work_item / agent 节点 (per star-game ADR-0027: WBS=关卡)
// =====================================================================

import { GAME_WORLD } from "./robotController";
import type { Bullet, Enemy, EnemySpawnSeed } from "./types";
import { enemyAimedBullet, enemyRadialBullets } from "./robotController";

/** canvas 元素 → 敌人种子 (坐标归一化到游戏世界 1200x800) */
export function enemiesFromCanvas(
  seeds: EnemySpawnSeed[],
  enemySeq: () => string,
): Enemy[] {
  return seeds.map((s) => {
    const variant = (s.kind === "agent" ? 2 : 0) + (s.difficulty % 3 === 0 ? 1 : 0);
    return {
      id: enemySeq(),
      x: s.x,
      y: s.y,
      hp: s.kind === "agent" ? 60 : 30,
      maxHp: s.kind === "agent" ? 60 : 30,
      variant: variant % 3,
      speed: s.kind === "agent" ? 40 : 26,
      fireIntervalFrames: s.kind === "agent" ? 90 : 140,
      framesSinceFire: Math.floor(Math.random() * 30),
      sourceElementId: s.id,
      kind: s.kind,
    };
  });
}

/** 敌人 AI 步进: 缓慢追踪玩家 + 定时弹幕 (aimed / radial 交替) */
export function stepEnemies(
  enemies: Enemy[],
  player: { x: number; y: number },
  bullets: Bullet[],
  bulletSeq: () => string,
): { enemies: Enemy[]; newBullets: Bullet[] } {
  const nextBullets = [...bullets];
  const nextEnemies = enemies.map((e) => {
    // 追踪玩家 (agent 类更快)
    const dx = player.x - e.x;
    const dy = player.y - e.y;
    const len = Math.hypot(dx, dy) || 1;
    const nx = e.x + (dx / len) * e.speed;
    const ny = e.y + (dy / len) * e.speed;

    let framesSinceFire = e.framesSinceFire + 1;
    if (framesSinceFire >= e.fireIntervalFrames) {
      framesSinceFire = 0;
      // 弹幕模式: work_item → aimed, agent → radial 8 颗 (per BD §5.1.2)
      if (e.kind === "agent") {
        nextBullets.push(...enemyRadialBullets(e, 8, 150, bulletSeq));
      } else {
        nextBullets.push(enemyAimedBullet(e, player, 200, bulletSeq));
      }
    }

    return {
      ...e,
      x: Math.max(10, Math.min(GAME_WORLD.width - 10, nx)),
      y: Math.max(10, Math.min(GAME_WORLD.height - 10, ny)),
      framesSinceFire,
    };
  });

  return { enemies: nextEnemies, newBullets: nextBullets };
}
