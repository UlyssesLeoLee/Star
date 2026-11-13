/*
@cypher schema=1 source_sha256=259e303aa0b0fbbfa70408ea327b4bd671ce15c8bca14504288f8936f2dbcde6
MERGE (self:File {path:"frontend/src/lib/canvas-game/collision.ts"})
MERGE (fn:Function {name:"collision::aabbHit",language:"typescript"})
MERGE (step:Function {name:"collision::stepBullets",language:"typescript"})
MERGE (types:File {path:"frontend/src/lib/canvas-game/types.ts"})
MERGE (self)-[:DEFINES]->(fn)
MERGE (self)-[:DEFINES]->(step)
MERGE (self)-[:IMPORTS]->(types)
@endcypher
*/
// =====================================================================
// collision — AABB 碰撞 + 子弹推进 (per BD §6.1 canvas-game-physics/collision.ts)
// =====================================================================

import type { Bullet, Enemy, ExplosionParticle, PlayerRobot } from "./types";

/** AABB 圆-圆碰撞 (per BD §3.1.4 FR-GAME-210/211) */
export function aabbHit(a: { x: number; y: number; r: number }, b: { x: number; y: number; r: number }): boolean {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  const rr = a.r + b.r;
  return dx * dx + dy * dy <= rr * rr;
}

export interface BulletStepResult {
  bullets: Bullet[];
  enemies: Enemy[];
  robot: PlayerRobot;
  explosions: ExplosionParticle[];
  killsDelta: number;
}

/**
 * 推进子弹一帧: 移动 + TTL 回收 + 碰撞结算。
 * 单帧内命中同一敌人的多个子弹伤害累计, 再统一结算 (避免重叠结算)。
 */
export function stepBullets(
  bullets: Bullet[],
  enemies: Enemy[],
  robot: PlayerRobot,
  now: number,
  explosionSeq: () => string,
): BulletStepResult {
  const nextBullets: Bullet[] = [];
  const deadEnemyIds = new Set<string>();
  const enemyDamage = new Map<string, number>();
  const explosions: ExplosionParticle[] = [];
  let nextHp = robot.hp;

  for (const b of bullets) {
    const nb = { ...b, x: b.x + b.vx, y: b.y + b.vy, ttlFrames: b.ttlFrames - 1 };
    if (nb.ttlFrames <= 0) continue;
    if (nb.x < -20 || nb.x > 1240 || nb.y < -20 || nb.y > 840) continue;

    let consumed = false;

    if (b.kind === "player") {
      // 玩家子弹 vs 敌人
      for (const e of enemies) {
        if (deadEnemyIds.has(e.id)) continue;
        if (aabbHit({ x: nb.x, y: nb.y, r: nb.radius }, { x: e.x, y: e.y, r: 16 })) {
          const total = (enemyDamage.get(e.id) ?? 0) + b.damage;
          enemyDamage.set(e.id, total);
          if (total >= e.hp) {
            deadEnemyIds.add(e.id);
            explosions.push(makeExplosion(e.x, e.y, explosionSeq()));
          }
          consumed = true;
          break;
        }
      }
    } else {
      // 敌方弹幕 vs 玩家 (rolling 无敌帧 + dying 不结算, per BD §5.1.1)
      const invuln = now < robot.rollInvulnUntil;
      if (!invuln && robot.state !== "dying" && aabbHit({ x: nb.x, y: nb.y, r: nb.radius }, { x: robot.x, y: robot.y, r: 14 })) {
        nextHp = Math.max(0, nextHp - b.damage);
        consumed = true;
      }
    }

    if (!consumed) nextBullets.push(nb);
  }

  const nextEnemies = enemies
    .filter((e) => !deadEnemyIds.has(e.id))
    .map((e) => {
      const dmg = enemyDamage.get(e.id);
      return dmg !== undefined ? { ...e, hp: e.hp - dmg } : e;
    });

  return {
    bullets: nextBullets,
    enemies: nextEnemies,
    robot: { ...robot, hp: nextHp, state: nextHp <= 0 && robot.state !== "dying" ? "dying" : robot.state },
    explosions,
    killsDelta: deadEnemyIds.size,
  };
}

/** 爆炸粒子: 8x8 像素矩阵 (per BD §3.1.8 FR-GAME-711) */
function makeExplosion(x: number, y: number, id: string): ExplosionParticle {
  const cells: number[][] = [];
  for (let r = 0; r < 8; r++) {
    const row: number[] = [];
    for (let c = 0; c < 8; c++) {
      const dist = Math.hypot(c - 3.5, r - 3.5);
      row.push(dist < 1.6 ? 2 : dist < 3.2 ? 1 : 0);
    }
    cells.push(row);
  }
  return { id, x, y, cells, ttlFrames: 18 };
}
