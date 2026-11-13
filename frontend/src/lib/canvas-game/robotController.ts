/*
@cypher schema=1 source_sha256=d70bc8ecc7d3535013a0d8221ad4ce4cb55c70417a02cb1741e52ca1ff797b88
MERGE (self:File {path:"frontend/src/lib/canvas-game/robotController.ts"})
MERGE (step:Function {name:"robotController::stepRobot",language:"typescript"})
MERGE (fire:Function {name:"robotController::fireBullet",language:"typescript"})
MERGE (types:File {path:"frontend/src/lib/canvas-game/types.ts"})
MERGE (self)-[:DEFINES]->(step)
MERGE (self)-[:DEFINES]->(fire)
MERGE (self)-[:IMPORTS]->(types)
@endcypher
*/
// =====================================================================
// robotController — WASD 输入 + 5 状态机 + 32 帧动画调度 (P0: idle/move 2 帧)
// per BD-STAR-CANVAS-GAME-001 §3.2.1 FR-GAME-101/102 + §5.1.1
// =====================================================================

import type { Bullet, PlayerRobot, RobotDirection } from "./types";

export const GAME_WORLD = { width: 1200, height: 800 };

/** 状态持续时间 (ms, per BD §5.1.1) */
export const STATE_DURATIONS = {
  attacking: 500, // 出招前摇 + 攻击 + 后摇
  rolling: 300,   // 无敌帧 0.3s
  dying: 600,     // 倒地
} as const;

/** 翻滚无敌帧 (per BD: 0.3s) */
export const ROLL_INVULN_MS = 300;
export const ATTACK_COOLDOWN_MS = 350;

export function createRobot(id: string, x: number, y: number): PlayerRobot {
  return {
    id,
    x,
    y,
    hp: 100,
    maxHp: 100,
    sp: 100,
    maxSp: 100,
    state: "idle",
    direction: 4,
    speed: 220, // px/s (per BD §4.3 speed 200)
    attackCooldownMs: ATTACK_COOLDOWN_MS,
    lastAttackAt: 0,
    rollInvulnUntil: 0,
    kills: 0,
  };
}

/** 8 方向编码 (per FR-GAME-703): 从输入向量 → 方向索引 (0=N 顺时针) */
export function directionFromVector(dx: number, dy: number): RobotDirection {
  if (dx === 0 && dy === 0) return 0; // 静止保持上次方向, 由调用方决定
  const angle = Math.atan2(dy, dx); // 0=E, +90=S (y 向下)
  const deg = ((angle * 180) / Math.PI + 360) % 360;
  // 0=N(上) 顺时针 45° 扇区: N=0, E=2, S=4, W=6 (atan2 0°=E 需 +90° 偏移)
  const index = Math.round(((deg + 90) % 360) / 45) % 8;
  return index as RobotDirection;
}

/** 方向索引 → 单位向量 (y 向下坐标系, 0=N) */
export function vectorFromDirection(d: RobotDirection): { x: number; y: number } {
  const angle = (-90 + d * 45) * (Math.PI / 180);
  return { x: Math.cos(angle), y: Math.sin(angle) };
}

interface StepInput {
  robot: PlayerRobot;
  input: { up: boolean; down: boolean; left: boolean; right: boolean };
  /** 本帧是否按下攻击键 */
  attackPressed: boolean;
  /** 本帧是否按下翻滚键 */
  rollPressed: boolean;
  now: number;
  dtMs: number;
}

/**
 * 推进机器人一帧: 输入 → 移动 / 状态机转换 / 边界钳制。
 * 纯函数 (per 守门 #1 可测性), 返回新对象不突变。
 */
export function stepRobot(input: StepInput): PlayerRobot {
  const { robot, input: keys, attackPressed, rollPressed, now, dtMs } = input;
  let { state } = robot;
  let { x, y, sp } = robot;
  let rollInvulnUntil = robot.rollInvulnUntil;

  const dt = dtMs / 1000;
  const dx = (keys.right ? 1 : 0) - (keys.left ? 1 : 0);
  const dy = (keys.down ? 1 : 0) - (keys.up ? 1 : 0);

  // 状态机 (per BD §5.1.1): dying 不可中断; 其余状态按时长退出
  if (state === "dying") {
    return robot;
  }
  if (state === "attacking" && now - robot.lastAttackAt >= STATE_DURATIONS.attacking) {
    state = "idle";
  }
  if (state === "rolling" && now >= rollInvulnUntil) {
    state = "idle";
  }

  // 翻滚 (Shift): 消耗 SP, 进入 rolling + 无敌帧 (per BD: 无敌帧 0.3s)
  if (rollPressed && sp >= 20 && state !== "rolling") {
    sp = Math.max(0, sp - 20);
    rollInvulnUntil = now + ROLL_INVULN_MS;
    state = "rolling";
  }

  // 移动: idle/moving 互转; rolling 期间可滑动 (速度 1.6x), attacking 定身
  let direction = robot.direction;
  if (state !== "attacking" && (dx !== 0 || dy !== 0)) {
    const rollBoost = state === "rolling" ? 1.6 : 1.0;
    const len = Math.hypot(dx, dy);
    x += (dx / len) * robot.speed * rollBoost * dt;
    y += (dy / len) * robot.speed * rollBoost * dt;
    if (state === "idle") state = "moving";
    direction = directionFromVector(dx, dy);
  } else if (state === "moving") {
    state = "idle";
  }

  // 攻击 (空格): 状态机 idle/moving → attacking, 记录出招时间 (per BD §5.1.1)
  let lastAttackAt = robot.lastAttackAt;
  if (attackPressed && now - robot.lastAttackAt >= robot.attackCooldownMs && state !== "attacking" && state !== "rolling") {
    state = "attacking";
    lastAttackAt = now;
  }

  // SP 缓慢回复
  sp = Math.min(robot.maxSp, sp + dtMs * 0.01);

  return {
    ...robot,
    x: clamp(x, 0, GAME_WORLD.width),
    y: clamp(y, 0, GAME_WORLD.height),
    state,
    direction,
    sp,
    lastAttackAt,
    rollInvulnUntil,
  };
}

function clamp(v: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, v));
}

/** 玩家开火: 生成一颗玩家子弹 (per BD §5.2.1 场景 1) */
export function fireBullet(robot: PlayerRobot, now: number, bulletSeq: () => string): Bullet {
  const dir = vectorFromDirection(robot.direction);
  const speed = 420;
  return {
    id: bulletSeq(),
    kind: "player",
    x: robot.x + dir.x * 18,
    y: robot.y + dir.y * 18,
    vx: dir.x * speed,
    vy: dir.y * speed,
    damage: 10,
    ttlFrames: 90,
    radius: 4,
  };
}

/** 敌人弹幕: aimed (朝玩家预判 0.3s, per BD §5.1.2) */
export function enemyAimedBullet(
  enemy: { x: number; y: number },
  target: { x: number; y: number; vx?: number; vy?: number },
  speed: number,
  bulletSeq: () => string,
): Bullet {
  const predictX = target.x + (target.vx ?? 0) * 0.3;
  const predictY = target.y + (target.vy ?? 0) * 0.3;
  const dx = predictX - enemy.x;
  const dy = predictY - enemy.y;
  const len = Math.hypot(dx, dy) || 1;
  return {
    id: bulletSeq(),
    kind: "enemy_aimed",
    x: enemy.x,
    y: enemy.y,
    vx: (dx / len) * speed,
    vy: (dy / len) * speed,
    damage: 8,
    ttlFrames: 180,
    radius: 5,
  };
}

/** 敌人弹幕: radial (N 颗 360° 均匀, per BD §5.1.2) */
export function enemyRadialBullets(
  enemy: { x: number; y: number },
  count: number,
  speed: number,
  bulletSeq: () => string,
): Bullet[] {
  const bullets: Bullet[] = [];
  for (let i = 0; i < count; i++) {
    const angle = (i / count) * Math.PI * 2;
    bullets.push({
      id: bulletSeq(),
      kind: "enemy_radial",
      x: enemy.x,
      y: enemy.y,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed,
      damage: 6,
      ttlFrames: 150,
      radius: 5,
    });
  }
  return bullets;
}
