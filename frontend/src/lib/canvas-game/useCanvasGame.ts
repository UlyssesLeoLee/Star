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
import { stepEnemies, enemiesFromCanvas } from "./enemySpawner";
import {
  createRobot,
  fireBullet,
  stepRobot,
  STATE_DURATIONS,
} from "./robotController";
import type { EnemySpawnSeed, GameState, PlayerRobot } from "./types";

const FRAME_MS = 1000 / 60;

export interface UseCanvasGameOptions {
  seeds: EnemySpawnSeed[];
  enabled: boolean;
}

let seqCounter = 0;
const seq = (prefix: string) => `${prefix}-${++seqCounter}-${Date.now().toString(36)}`;

export function useCanvasGame({ seeds, enabled }: UseCanvasGameOptions) {
  const [gameState, setGameState] = useState<GameState | null>(() =>
    enabled
      ? {
          robot: createRobot("player", 600, 400),
          bullets: [],
          enemies: enemiesFromCanvas(seeds, () => seq("enemy")),
          explosions: [],
          input: { up: false, down: false, left: false, right: false },
          kills: 0,
          gameOver: false,
          startedAt: Date.now(),
        }
      : null,
  );

  // 键盘输入 (WASD + 空格攻击 + Shift 翻滚, per BD §1.3)
  // 攻击/翻滚是按住型输入: keydown 置 ref, rAF 帧消费后复位 (避免连发失控)
  const attackHeldRef = useRef(false);
  const rollHeldRef = useRef(false);

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
        let bullets = prev.bullets;
        if (robot.state === "attacking" && !wasAttacking) {
          bullets = [...bullets, fireBullet(robot, now, () => seq("bullet"))];
        }

        // 3. 敌人步进 + 弹幕
        const enemyStep = stepEnemies(prev.enemies, robot, bullets, () => seq("bullet"));
        bullets = enemyStep.newBullets;
        const enemies = enemyStep.enemies;

        // 4. 子弹碰撞结算
        const collided = stepBullets(bullets, enemies, robot, now, () => seq("expl"));
        let explosions = [...prev.explosions, ...collided.explosions];
        const kills = prev.kills + collided.killsDelta;

        // 5. 爆炸衰减
        explosions = explosions
          .map((ex) => ({ ...ex, ttlFrames: ex.ttlFrames - 1 }))
          .filter((ex) => ex.ttlFrames > 0);

        // 6. 死亡判定 (per BD §5.1.1: HP=0 → dying; P0 简化: 立即 game over)
        const finalRobot = collided.robot;
        const gameOver = prev.gameOver || (finalRobot.state === "dying" && finalRobot.hp <= 0);

        return {
          ...prev,
          robot: finalRobot,
          bullets: collided.bullets,
          enemies: collided.enemies,
          explosions,
          kills,
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
    setGameState({
      robot: createRobot("player", 600, 400),
      bullets: [],
      enemies: enemiesFromCanvas(seeds, () => seq("enemy")),
      explosions: [],
      input: { up: false, down: false, left: false, right: false },
      kills: 0,
      gameOver: false,
      startedAt: Date.now(),
    });
  }, [seeds]);

  return { gameState, restart };
}
