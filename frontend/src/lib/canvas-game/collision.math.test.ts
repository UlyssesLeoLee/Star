/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/canvas-game/collision.math.test.ts"})
MERGE (self)-[:TESTS]->(t:File {path:"frontend/src/lib/canvas-game/collision.ts"})
@endcypher
*/

/**
 * collision.ts 数学性质测试 (2026-11-21, per "完善自动测试脚本和mock项目，加入数学测试的内容")
 *
 * 与 canvasGame.test.ts (示例驱动) 互补, 用确定性 PRNG 做性质测试:
 *   C1. aabbHit 对称性: hit(a,b) = hit(b,a)
 *   C2. aabbHit 平移不变性: 两圆同步平移, 命中判定不变
 *   C3. aabbHit 半径单调性: 增大任一半径, false→true 不可逆
 *   C4. aabbHit 与欧氏距离定义一致 (dist ≤ r1+r2 ⇔ hit)
 *   C5. stepBullets 伤害守恒: 敌人总 HP 减少量 = 命中子弹 damage 之和 (未击杀时)
 *   C6. stepBullets 子弹数守恒: 输入 = 输出存活 + TTL 回收 + 越界回收 + 命中消耗
 *   C7. stepBullets 无敌帧免疫: rollInvulnUntil 内敌方弹幕不扣 HP
 *   C8. stepBullets 爆炸粒子几何: 8x8 矩阵、中心对称、色阶 ∈ {0,1,2}
 *
 * 不引入 fast-check (repo 未装), 固定种子 mulberry32 保证可复现。
 */

import { describe, expect, it } from "vitest";
import { aabbHit, stepBullets } from "./collision";
import type { Bullet, Enemy, PlayerRobot } from "./types";

function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const SAMPLES = 300;

function makeRobot(overrides: Partial<PlayerRobot> = {}): PlayerRobot {
  return {
    id: "robot-1",
    x: 600,
    y: 400,
    hp: 100,
    maxHp: 100,
    sp: 100,
    maxSp: 100,
    state: "idle",
    direction: 0,
    speed: 4,
    attackCooldownMs: 300,
    lastAttackAt: -1000,
    rollInvulnUntil: -1,
    ...overrides,
  } as PlayerRobot;
}

function makeBullet(overrides: Partial<Bullet> = {}): Bullet {
  return {
    id: `b-${Math.random().toString(36).slice(2, 8)}`,
    kind: "player",
    x: 600,
    y: 400,
    vx: 0,
    vy: 0,
    damage: 10,
    ttlFrames: 60,
    radius: 4,
    ...overrides,
  };
}

function makeEnemy(overrides: Partial<Enemy> = {}): Enemy {
  return {
    id: `e-${Math.random().toString(36).slice(2, 8)}`,
    x: 600,
    y: 400,
    hp: 30,
    maxHp: 30,
    variant: 0,
    speed: 1,
    fireIntervalFrames: 90,
    framesSinceFire: 0,
    fireCount: 0,
    sourceElementId: "el-1",
    kind: "work_item",
    ...overrides,
  } as Enemy;
}

describe("aabbHit 数学性质", () => {
  it("C1: 对称性 hit(a,b) = hit(b,a)", () => {
    const rand = mulberry32(0xc1);
    for (let i = 0; i < SAMPLES; i++) {
      const a = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      const b = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      expect(aabbHit(a, b)).toBe(aabbHit(b, a));
    }
  });

  it("C2: 平移不变性 — 两圆同步平移, 判定不变", () => {
    const rand = mulberry32(0xc2);
    for (let i = 0; i < SAMPLES; i++) {
      const a = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      const b = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      const dx = (rand() - 0.5) * 1000;
      const dy = (rand() - 0.5) * 1000;
      const hit0 = aabbHit(a, b);
      const hit1 = aabbHit({ ...a, x: a.x + dx, y: a.y + dy }, { ...b, x: b.x + dx, y: b.y + dy });
      expect(hit1).toBe(hit0);
    }
  });

  it("C3: 半径单调性 — 增大半径只会 false→true, 不会 true→false", () => {
    const rand = mulberry32(0xc3);
    for (let i = 0; i < SAMPLES; i++) {
      const a = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      const b = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      const grow = rand() * 50;
      if (aabbHit(a, b)) {
        expect(aabbHit({ ...a, r: a.r + grow }, b)).toBe(true);
        expect(aabbHit(a, { ...b, r: b.r + grow })).toBe(true);
      }
    }
  });

  it("C4: 与欧氏距离定义一致 dist ≤ r1+r2 ⇔ hit", () => {
    const rand = mulberry32(0xc4);
    for (let i = 0; i < SAMPLES; i++) {
      const a = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      const b = { x: (rand() - 0.5) * 200, y: (rand() - 0.5) * 200, r: rand() * 30 };
      const dist = Math.hypot(a.x - b.x, a.y - b.y);
      // 平方比较避免开方误差; 边界附近 (|dist²−rr²| < 1e-9) 跳过
      const rr = a.r + b.r;
      const lhs = (a.x - b.x) ** 2 + (a.y - b.y) ** 2;
      if (Math.abs(lhs - rr * rr) < 1e-9) continue;
      expect(aabbHit(a, b)).toBe(dist <= rr);
    }
  });
});

describe("stepBullets 数学性质", () => {
  it("C5: 伤害守恒 — 未击杀时敌人 HP 减少 = 命中子弹 damage 之和", () => {
    const rand = mulberry32(0xc5);
    for (let i = 0; i < 100; i++) {
      const enemy = makeEnemy({ x: 600, y: 400, hp: 1000, maxHp: 1000 });
      // 3 发静止子弹叠在敌人中心, 必命中且不击杀
      const bullets = [0, 1, 2].map((k) =>
        makeBullet({ id: `b${k}`, x: 600, y: 400, vx: 0, vy: 0, damage: 5 + Math.floor(rand() * 20) }),
      );
      const totalDamage = bullets.reduce((s, b) => s + b.damage, 0);
      const res = stepBullets(bullets, [enemy], makeRobot(), 0, () => "exp");
      expect(res.enemies).toHaveLength(1);
      expect(res.enemies[0].hp).toBe(enemy.hp - totalDamage);
      expect(res.killsDelta).toBe(0);
      expect(res.bullets).toHaveLength(0); // 全部命中消耗
    }
  });

  it("C6: 子弹数守恒 — 输入 = 存活 + TTL 回收 + 越界回收 + 命中消耗", () => {
    const rand = mulberry32(0xc6);
    for (let i = 0; i < 100; i++) {
      const n = 1 + Math.floor(rand() * 8);
      const bullets: Bullet[] = [];
      let expectTtl = 0;
      let expectOob = 0;
      for (let k = 0; k < n; k++) {
        const mode = Math.floor(rand() * 3);
        if (mode === 0) {
          // TTL=1 → 推进后 ttlFrames=0 回收
          bullets.push(makeBullet({ id: `t${k}`, ttlFrames: 1, x: 600, y: 400 }));
          expectTtl++;
        } else if (mode === 1) {
          // 越界: 推进后 x>1240
          bullets.push(makeBullet({ id: `o${k}`, x: 1239, y: 400, vx: 10, vy: 0 }));
          expectOob++;
        } else {
          // 存活: 场地中央静止
          bullets.push(makeBullet({ id: `s${k}`, x: 300 + rand() * 600, y: 200 + rand() * 400, vx: 0, vy: 0 }));
        }
      }
      const res = stepBullets(bullets, [], makeRobot(), 0, () => "exp");
      const survivors = n - expectTtl - expectOob;
      expect(res.bullets).toHaveLength(survivors);
      // 存活子弹位置 = 原位置 + 速度 (运动学积分)
      for (const nb of res.bullets) {
        const ob = bullets.find((b) => b.id === nb.id)!;
        expect(nb.x).toBe(ob.x + ob.vx);
        expect(nb.y).toBe(ob.y + ob.vy);
        expect(nb.ttlFrames).toBe(ob.ttlFrames - 1);
      }
    }
  });

  it("C7: 无敌帧免疫 — rollInvulnUntil 内敌方弹幕不扣 HP", () => {
    const robot = makeRobot({ hp: 50, rollInvulnUntil: 1000 });
    const bullets = [makeBullet({ kind: "enemy", x: 600, y: 400, damage: 20 })];
    const res = stepBullets(bullets, [], robot, 500, () => "exp"); // now=500 < 1000
    expect(res.robot.hp).toBe(50);
    // 无敌帧结束后同样子弹扣血
    const res2 = stepBullets(bullets, [], robot, 1500, () => "exp");
    expect(res2.robot.hp).toBe(30);
  });

  it("C8: 爆炸粒子几何 — 8x8、中心对称、色阶 ∈ {0,1,2}", () => {
    const enemy = makeEnemy({ x: 600, y: 400, hp: 10 });
    const bullets = [makeBullet({ x: 600, y: 400, damage: 10 })];
    const res = stepBullets(bullets, [enemy], makeRobot(), 0, () => "exp-1");
    expect(res.explosions).toHaveLength(1);
    const { cells } = res.explosions[0];
    expect(cells).toHaveLength(8);
    for (let r = 0; r < 8; r++) {
      expect(cells[r]).toHaveLength(8);
      for (let c = 0; c < 8; c++) {
        expect([0, 1, 2]).toContain(cells[r][c]);
        // 中心对称: (r,c) 与 (7-r,7-c) 到中心 (3.5,3.5) 等距 ⇒ 同色
        expect(cells[r][c]).toBe(cells[7 - r][7 - c]);
      }
    }
    // 中心 4 格必为最高阶 2 (dist=√0.5≈0.707 < 1.6)
    expect(cells[3][3]).toBe(2);
    expect(cells[4][4]).toBe(2);
    // 四角必为 0 (dist=√24.5≈4.95 ≥ 3.2)
    expect(cells[0][0]).toBe(0);
    expect(cells[7][7]).toBe(0);
  });
});