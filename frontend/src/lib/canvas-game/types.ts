/*
@cypher schema=1 source_sha256=856cc72adb4733409b37405ef85530ce6ff30aedce33efb35465308e26e32a9e
MERGE (self:File {path:"frontend/src/lib/canvas-game/types.ts"})
MERGE (state:Type {id:"canvas-game::GameState",kind:"interface"})
MERGE (robot:Type {id:"canvas-game::PlayerRobot",kind:"interface"})
MERGE (bullet:Type {id:"canvas-game::Bullet",kind:"interface"})
MERGE (enemy:Type {id:"canvas-game::Enemy",kind:"interface"})
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(robot)
MERGE (self)-[:DEFINES]->(bullet)
MERGE (self)-[:DEFINES]->(enemy)
@endcypher
*/
// =====================================================================
// canvas-game types — 弹幕 Roguelike 像素风 (per WBS §23.36, 2026-11-13)
// 设计依据: docs/design/BD-STAR-CANVAS-GAME-001.md (弹幕 Roguelike)
// 命名空间: frontend/src/lib/canvas-game/ (per BD §6.2)
// =====================================================================

/** 机器人 5 状态机 (per BD §5.1.1 / FR-GAME-102) */
export type RobotState = "idle" | "moving" | "attacking" | "rolling" | "dying";

/** 8 方向 (per FR-GAME-703, 0=N 顺时针) */
export type RobotDirection = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7;

export interface PlayerRobot {
  id: string;
  x: number;
  y: number;
  hp: number;
  maxHp: number;
  /** 精力 (翻滚消耗, per BD §4.3 sp 字段) */
  sp: number;
  maxSp: number;
  state: RobotState;
  direction: RobotDirection;
  speed: number;
  /** 攻击冷却 (ms) */
  attackCooldownMs: number;
  lastAttackAt: number;
  /** 翻滚无敌帧 (per BD: 0.3s) */
  rollInvulnUntil: number;
  kills: number;
}

export type BulletKind = "player" | "enemy_aimed" | "enemy_radial" | "enemy_spiral" | "enemy_wave" | "enemy_laser";

export interface Bullet {
  id: string;
  kind: BulletKind;
  x: number;
  y: number;
  vx: number;
  vy: number;
  damage: number;
  /** 存活帧数 (子弹池回收依据) */
  ttlFrames: number;
  radius: number;
  /** wave 模式: 相位偏移 (sin 摆动, per BD §5.1.2) */
  phase?: number;
  /** laser 模式: 目标点 (发射时锁定, 飞行到达后消失) */
  targetX?: number;
  targetY?: number;
}

/** Roguelike 升级道具 (per BD §3.1.6 FR-GAME-505 掉落) */
export type PowerUpKind = "damage" | "firerate" | "heal";

export interface PowerUp {
  id: string;
  kind: PowerUpKind;
  x: number;
  y: number;
  ttlFrames: number;
}

export interface Enemy {
  id: string;
  x: number;
  y: number;
  hp: number;
  maxHp: number;
  /** 3 色变体 (0/1/2) */
  variant: number;
  speed: number;
  /** 弹幕发射间隔 (帧) */
  fireIntervalFrames: number;
  framesSinceFire: number;
  /** 已发射次数 (弹幕模式轮换用, per BD §5.1.2) */
  fireCount: number;
  /** 来源 canvas 元素 id (work_item/agent 节点) */
  sourceElementId: string;
  kind: "work_item" | "agent";
}

/** 爆炸粒子 (8x8 像素矩阵) */
export interface ExplosionParticle {
  id: string;
  x: number;
  y: number;
  /** 粒子像素色值 (0-2 阶) */
  cells: number[][];
  ttlFrames: number;
}

export interface GameState {
  robot: PlayerRobot;
  bullets: Bullet[];
  enemies: Enemy[];
  explosions: ExplosionParticle[];
  powerUps: PowerUp[];
  /** 玩家输入 (rAF 循环消费) */
  input: { up: boolean; down: boolean; left: boolean; right: boolean };
  /** 本局击杀数 */
  kills: number;
  /** Roguelike 升级 (per BD §3.1.2 FR-GAME-110~112): 伤害/射速等级 */
  damageLevel: number;
  firerateLevel: number;
  /** 大招冷却 (Q 键, per BD §3.1.7 FR-GAME-602) */
  ultimateCooldownUntil: number;
  /** game over 标志 */
  gameOver: boolean;
  /** 游戏开始时间 (ms) */
  startedAt: number;
}

/** 关卡素材: 从 canvas 元素生成的敌人种子 (per §23.36 enemySpawner) */
export interface EnemySpawnSeed {
  id: string;
  x: number;
  y: number;
  kind: "work_item" | "agent";
  difficulty: number;
}
