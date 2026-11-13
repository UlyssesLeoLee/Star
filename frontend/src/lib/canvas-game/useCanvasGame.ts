/*
@cypher schema=1 source_sha256=1511164e9e191e8326c48f2f8b71c60b6a8d81cd05dfa03c08416c76edff3792
MERGE (self:File {path:"frontend/src/lib/canvas-game/useCanvasGame.ts"})
MERGE (hook:Function {name:"canvas-game::useCanvasGame",language:"typescript"})
MERGE (types:File {path:"frontend/src/lib/canvas-game/types.ts"})
MERGE (self)-[:DEFINES]->(hook)
MERGE (self)-[:IMPORTS]->(types)
@endcypher
*/
// =====================================================================
// useCanvasGame — rAF 游戏循环 + 键盘输入 + 状态编排 (per WBS §23.36)
// 编排: stepRobot → stepEnemies → stepBullets → 爆炸衰减
// =====================================================================

import { useCallback, useEffect, useRef, useState } from "react";
import { stepBullets } from "./collision";
import { stepEnemies, enemiesFromCanvas, rollPowerUpDrop } from "./enemySpawner";
import {
  createRobot,
  fireBullet,
  fireUltimate,
  stepRobot,
  STATE_DURATIONS,
} from "./robotController";
import type { EnemySpawnSeed, GameState } from "./types";

const FRAME_MS = 1000 / 60;

export interface UseCanvasGameOptions {
  seeds: EnemySpawnSeed[];
  enabled: boolean;
}

let seqCounter = 0;
const seq = (prefix: string) => `${prefix}-${++seqCounter}-${Date.now().toString(36)}`;

/** 初始 GameState (per BD: 机器人 + 空战场 + 升级等级 0) */
function makeInitialState(seeds: EnemySpawnSeed[]): GameState {
  return {
    robot: createRobot("player", 600, 400),
    bullets: [],
    enemies: enemiesFromCanvas(seeds, () => seq("enemy")),
    explosions: [],
    powerUps: [],
    input: { up: false, down: false, left: false, right: false },
    kills: 0,
    damageLevel: 0,
    firerateLevel: 0,
    ultimateCooldownUntil: 0,
    gameOver: false,
    startedAt: Date.now(),
  };
}

export function useCanvasGame({ seeds, enabled }: UseCanvasGameOptions) {
  const [gameState, setGameState] = useState<GameState | null>(() =>
    enabled
      ? makeInitialState(seeds)
      : null,
  );

  // 键盘输入 (WASD + 空格攻击 + Shift 翻滚 + Q 大招, per BD §1.3)
  // 攻击/翻滚/大招是按住型输入: keydown 置 ref, rAF 帧消费后复位 (避免连发失控)
  const attackHeldRef = useRef(false);
  const rollHeldRef = useRef(false);
  const ultimateHeldRef = useRef(false);

  useEffect(() => {
    if (!enabled) return;
    const down = (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      if ([" ", "arrowup", "arrowdown", "arrowleft", "arrowright"].includes(key)) {
        e.preventDefault();
      }
      if (key === " " || key === "j") {
        attackHeldRef.current = true;
        return;
      }
      if (key === "shift" || key === "k") {
        rollHeldRef.current = true;
        return;
      }
      if (key === "q") {
        ultimateHeldRef.current = true;
        return;
      }
      setGameState((prev) => {
        if (!prev) return prev;
        if (key === "w" || key === "arrowup") return { ...prev, input: { ...prev.input, up: true } };
        if (key === "s" || key === "arrowdown") return { ...prev, input: { ...prev.input, down: true } };
        if (key === "a" || key === "arrowleft") return { ...prev, input: { ...prev.input, left: true } };
        if (key === "d" || key === "arrowright") return { ...prev, input: { ...prev.input, right: true } };
        return prev;
      });
    };
    const up = (e: KeyboardEvent) => {
      setGameState((prev) => {
        if (!prev) return prev;
        const key = e.key.toLowerCase();
        if (key === "w" || key === "arrowup") return { ...prev, input: { ...prev.input, up: false } };
        if (key === "s" || key === "arrowdown") return { ...prev, input: { ...prev.input, down: false } };
        if (key === "a" || key === "arrowleft") return { ...prev, input: { ...prev.input, left: false } };
        if (key === "d" || key === "arrowright") return { ...prev, input: { ...prev.input, right: false } };
        return prev;
      });
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, [enabled]);

  const frame = useCallback(
    (nowMs: number) => {
      setGameState((prev) => {
        if (!prev || prev.gameOver) return prev;
        const dtMs = Math.min(50, FRAME_MS);
        const now = nowMs;

        // 1. 机器人步进 (输入 + 状态机)
        const wasAttacking = prev.robot.state === "attacking";
        const attackPressed = attackHeldRef.current;
        const rollPressed = rollHeldRef.current;
        attackHeldRef.current = false;
        rollHeldRef.current = false;
        const robot = stepRobot({ robot: prev.robot, input: prev.input, attackPressed, rollPressed, now, dtMs });

        // 2. 攻击开火 (进入 attacking 状态的首帧出弹, per BD §5.2.1)
        //    伤害/射速受 Roguelike 升级影响 (per BD §3.1.2 FR-GAME-110~112)
        let bullets = prev.bullets;
        if (robot.state === "attacking" && !wasAttacking) {
          bullets = [...bullets, fireBullet(robot, now, () => seq("bullet"))];
          if (prev.damageLevel > 0 || prev.firerateLevel > 0) {
            const last = bullets[bullets.length - 1];
            bullets[bullets.length - 1] = {
              ...last,
              damage: last.damage + prev.damageLevel * 4,
              ttlFrames: last.ttlFrames + prev.firerateLevel * 15,
            };
          }
        }

        // 2b. 大招 (Q 键, per BD §3.1.7 FR-GAME-602): 冷却 8s, 16 向爆发
        let ultimateCooldownUntil = prev.ultimateCooldownUntil;
        if (ultimateHeldRef.current) {
          ultimateHeldRef.current = false;
          if (now >= ultimateCooldownUntil) {
            bullets = [...bullets, ...fireUltimate(robot, prev.damageLevel, () => seq("bullet"))];
            ultimateCooldownUntil = now + 8000;
          }
        }

        // 3. 敌人步进 + 弹幕
        const enemyStep = stepEnemies(prev.enemies, robot, bullets, () => seq("bullet"));
        bullets = enemyStep.newBullets;
        const enemies = enemyStep.enemies;

        // 4. 子弹碰撞结算
        const collided = stepBullets(bullets, enemies, robot, now, () => seq("expl"));
        let explosions = [...prev.explosions, ...collided.explosions];
        const kills = prev.kills + collided.killsDelta;

        // 4b. 击杀掉落 Roguelike 道具 (per BD §3.1.6 FR-GAME-505)
        let powerUps = prev.powerUps;
        if (collided.killsDelta > 0) {
          for (const ex of collided.explosions) {
            const source = prev.enemies.find((en) => Math.abs(en.x - ex.x) < 20 && Math.abs(en.y - ex.y) < 20);
            if (source) {
              const drop = rollPowerUpDrop(source, () => seq("power"));
              if (drop) powerUps = [...powerUps, drop];
            }
          }
        }

        // 4c. 道具拾取 (接触即拾取, per BD §3.1.6)
        let { damageLevel, firerateLevel } = prev;
        let robotHp = collided.robot.hp;
        powerUps = powerUps.filter((p) => {
          if (Math.hypot(p.x - robot.x, p.y - robot.y) < 28) {
            if (p.kind === "damage") damageLevel += 1;
            else if (p.kind === "firerate") firerateLevel += 1;
            else robotHp = Math.min(collided.robot.maxHp, robotHp + 25);
            return false;
          }
          return true;
        });

        // 5. 爆炸 + 道具衰减
        explosions = explosions
          .map((ex) => ({ ...ex, ttlFrames: ex.ttlFrames - 1 }))
          .filter((ex) => ex.ttlFrames > 0);
        powerUps = powerUps
          .map((p) => ({ ...p, ttlFrames: p.ttlFrames - 1 }))
          .filter((p) => p.ttlFrames > 0);

        // 6. 死亡判定 (per BD §5.1.1: HP=0 → dying; P0 简化: 立即 game over)
        const finalRobot = { ...collided.robot, hp: robotHp };
        const gameOver = prev.gameOver || (finalRobot.state === "dying" && finalRobot.hp <= 0);

        return {
          ...prev,
          robot: finalRobot,
          bullets: collided.bullets,
          enemies: collided.enemies,
          explosions,
          powerUps,
          kills,
          damageLevel,
          firerateLevel,
          ultimateCooldownUntil,
          gameOver,
        };
      });
    },
    [],
  );

  useEffect(() => {
    if (!enabled) return;
    let raf = 0;
    const loop = (t: number) => {
      frame(t);
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, [enabled, frame]);

  /** 重开一局 */
  const restart = useCallback(() => {
    setGameState(makeInitialState(seeds));
  }, [seeds]);

  return { gameState, restart };
}
