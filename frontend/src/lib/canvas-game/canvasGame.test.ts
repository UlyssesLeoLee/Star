/*
@cypher schema=1 source_sha256=b6660eeea8de4cdc4f3fcf126596441f5e4dec6bb566630caeefded9fe00d5f6
MERGE (self:File {path:"frontend/src/lib/canvas-game/canvasGame.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/canvas-game/canvasGame.test.ts::canvasGameCore",kind:"suite"})
MERGE (ctrl:File {path:"frontend/src/lib/canvas-game/robotController.ts"})
MERGE (col:File {path:"frontend/src/lib/canvas-game/collision.ts"})
MERGE (sp:File {path:"frontend/src/lib/canvas-game/enemySpawner.ts"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(ctrl)
MERGE (self)-[:IMPORTS]->(col)
MERGE (self)-[:IMPORTS]->(sp)
MERGE (suite)-[:TESTS]->(ctrl)
MERGE (suite)-[:TESTS]->(col)
MERGE (suite)-[:TESTS]->(sp)
@endcypher
*/
// per WBS §23.36: canvas 游戏化核心模块单测 (BD-STAR-CANVAS-GAME-001 落地)
import { describe, it, expect } from "vitest";
import {
  createRobot,
  directionFromVector,
  enemyAimedBullet,
  enemyRadialBullets,
  fireBullet,
  stepRobot,
  GAME_WORLD,
} from "./robotController";
import { aabbHit, stepBullets } from "./collision";
import { enemiesFromCanvas, stepEnemies } from "./enemySpawner";
import type { Bullet, Enemy } from "./types";

let seqN = 0;
const seq = (p: string) => `${p}-${++seqN}`;

describe("robotController (per BD §5.1.1)", () => {
  it("WASD 移动: idle → moving, 方向编码正确 (8 方向)", () => {
    const robot = stepRobot({
      robot: createRobot("p", 100, 100),
      input: { up: false, down: true, left: false, right: false },
      attackPressed: false,
      rollPressed: false,
      now: 0,
      dtMs: 16,
    });
    expect(robot.state).toBe("moving");
    expect(robot.direction).toBe(4); // 向下 = S
    expect(robot.y).toBeGreaterThan(100);
  });

  it("方向向量: 0=N 顺时针 45° 扇区", () => {
    expect(directionFromVector(0, -1)).toBe(0); // 上
    expect(directionFromVector(1, 0)).toBe(2); // 右
    expect(directionFromVector(0, 1)).toBe(4); // 下
    expect(directionFromVector(-1, 0)).toBe(6); // 左
  });

  it("移动边界钳制在世界 1200x800 内", () => {
    let robot = createRobot("p", 1195, 10);
    for (let i = 0; i < 60; i++) {
      robot = stepRobot({
        robot,
        input: { up: false, down: false, left: false, right: true },
        attackPressed: false,
        rollPressed: false,
        now: i * 16,
        dtMs: 16,
      });
    }
    expect(robot.x).toBeLessThanOrEqual(GAME_WORLD.width);
  });

  it("空格攻击: 转 attacking + 记录出招时间, 冷却内不重复", () => {
    const base = createRobot("p", 100, 100);
    const attacking = stepRobot({
      robot: base,
      input: { up: false, down: false, left: false, right: false },
      attackPressed: true,
      rollPressed: false,
      now: 1000,
      dtMs: 16,
    });
    expect(attacking.state).toBe("attacking");
    expect(attacking.lastAttackAt).toBe(1000);

    // 冷却内再次攻击不生效
    const cooldown = stepRobot({
      robot: attacking,
      input: { up: false, down: false, left: false, right: false },
      attackPressed: true,
      rollPressed: false,
      now: 1100,
      dtMs: 16,
    });
    expect(cooldown.lastAttackAt).toBe(1000);
  });

  it("Shift 翻滚: 消耗 SP + 无敌帧 0.3s", () => {
    const base = createRobot("p", 100, 100);
    const rolling = stepRobot({
      robot: base,
      input: { up: false, down: false, left: false, right: false },
      attackPressed: false,
      rollPressed: true,
      now: 500,
      dtMs: 16,
    });
    expect(rolling.state).toBe("rolling");
    expect(rolling.sp).toBeCloseTo(80, 0);
    expect(rolling.rollInvulnUntil).toBe(500 + 300);
  });

  it("开火: 子弹从角色位置沿朝向射出", () => {
    const robot = { ...createRobot("p", 200, 200), direction: 2 as const }; // 朝右
    const b = fireBullet(robot, 0, () => seq("b"));
    expect(b.kind).toBe("player");
    expect(b.vx).toBeGreaterThan(0);
    expect(b.vy).toBe(0);
  });
});

describe("collision (per BD §3.1.4)", () => {
  const makeEnemy = (id: string, x: number, y: number, hp = 30): Enemy => ({
    id,
    x,
    y,
    hp,
    maxHp: hp,
    variant: 0,
    speed: 20,
    fireIntervalFrames: 120,
    framesSinceFire: 0,
    sourceElementId: "el-1",
    kind: "work_item",
  });

  it("AABB: 重叠命中 / 远离不命中", () => {
    expect(aabbHit({ x: 0, y: 0, r: 10 }, { x: 5, y: 5, r: 10 })).toBe(true);
    expect(aabbHit({ x: 0, y: 0, r: 10 }, { x: 100, y: 100, r: 10 })).toBe(false);
  });

  it("玩家子弹命中敌人: 扣血, 击杀产生爆炸 + killsDelta", () => {
    const enemies = [makeEnemy("e1", 300, 300, 10)];
    const bullets: Bullet[] = [{
      id: "b1",
      kind: "player",
      x: 290,
      y: 300,
      vx: 8,
      vy: 0,
      damage: 10,
      ttlFrames: 50,
      radius: 4,
    }];
    const robot = createRobot("p", 100, 100);
    const result = stepBullets(bullets, enemies, robot, 0, () => seq("x"));
    expect(result.killsDelta).toBe(1);
    expect(result.enemies).toHaveLength(0);
    expect(result.explosions).toHaveLength(1);
    expect(result.robot.kills).toBe(0); // kills 由调用方累计
  });

  it("敌方弹幕命中玩家扣 HP; 翻滚无敌帧免疫", () => {
    const bullets: Bullet[] = [{
      id: "b1",
      kind: "enemy_aimed",
      x: 90,
      y: 100,
      vx: 8,
      vy: 0,
      damage: 8,
      ttlFrames: 50,
      radius: 5,
    }];
    const robot = createRobot("p", 100, 100);
    const hit = stepBullets(bullets, [], robot, 0, () => seq("x"));
    expect(hit.robot.hp).toBe(92);

    const invuln = { ...robot, rollInvulnUntil: 100 }; // 无敌帧内
    const dodged = stepBullets(bullets, [], invuln, 0, () => seq("x"));
    expect(dodged.robot.hp).toBe(100);
  });

  it("HP 归零 → dying 状态", () => {
    const bullets: Bullet[] = Array.from({ length: 15 }, (_, i) => ({
      id: `b${i}`,
      kind: "enemy_aimed" as const,
      x: 90 + i,
      y: 100,
      vx: 8,
      vy: 0,
      damage: 8,
      ttlFrames: 50,
      radius: 5,
    }));
    const robot = createRobot("p", 100, 100);
    const result = stepBullets(bullets, [], robot, 0, () => seq("x"));
    expect(result.robot.hp).toBe(0);
    expect(result.robot.state).toBe("dying");
  });
});

describe("enemySpawner (per WBS §23.36)", () => {
  it("从 canvas 种子生成敌人: agent 类更强", () => {
    const enemies = enemiesFromCanvas(
      [
        { id: "el-1", x: 100, y: 100, kind: "work_item", difficulty: 1 },
        { id: "el-2", x: 200, y: 200, kind: "agent", difficulty: 2 },
      ],
      () => seq("e"),
    );
    expect(enemies).toHaveLength(2);
    const agent = enemies.find((e) => e.kind === "agent");
    const item = enemies.find((e) => e.kind === "work_item");
    expect(agent?.hp).toBe(60);
    expect(item?.hp).toBe(30);
  });

  it("敌人追踪玩家 + 定时弹幕 (work_item aimed / agent radial)", () => {
    const enemies = enemiesFromCanvas(
      [{ id: "el-1", x: 500, y: 500, kind: "agent", difficulty: 1 }],
      () => seq("e"),
    );
    const nearFire = enemies.map((e) => ({ ...e, framesSinceFire: e.fireIntervalFrames - 1 }));
    const result = stepEnemies(nearFire, { x: 600, y: 600 }, [], () => seq("b"));
    expect(result.enemies[0].x).toBeGreaterThan(500); // 向玩家移动
    expect(result.newBullets).toHaveLength(8); // radial 8 颗
    expect(result.newBullets.every((b) => b.kind === "enemy_radial")).toBe(true);
  });

  it("aimed 弹幕朝玩家预判方向", () => {
    const b = enemyAimedBullet({ x: 0, y: 0 }, { x: 100, y: 0 }, 100, () => seq("b"));
    expect(b.vx).toBeGreaterThan(0);
    expect(b.vy).toBe(0);
    expect(b.kind).toBe("enemy_aimed");
  });

  it("radial 弹幕 360° 均匀分布", () => {
    const bullets = enemyRadialBullets({ x: 0, y: 0 }, 8, 100, () => seq("b"));
    expect(bullets).toHaveLength(8);
    const angles = bullets.map((b) => Math.atan2(b.vy, b.vx));
    const sorted = [...angles].sort((a, b) => a - b);
    expect(sorted[7] - sorted[0]).toBeGreaterThan(Math.PI); // 覆盖全圆
  });
});
