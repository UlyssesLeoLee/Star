/*
@cypher schema=1 source_sha256=61a6484fed307a83a34834f8f131d8ce236238960cab68e89bd7f66972c2f708
MERGE (self:File {path:"frontend/src/components/canvas-game/CanvasGameOverlay.tsx"})
MERGE (overlay:Symbol {id:"frontend/src/components/canvas-game/CanvasGameOverlay.tsx::CanvasGameOverlay",kind:"component"})
MERGE (hook:Symbol {id:"frontend/src/lib/canvas-game/useCanvasGame.ts::useCanvasGame",kind:"hook"})
MERGE (sprites:File {path:"frontend/src/lib/canvas-game/sprites.ts"})
MERGE (self)-[:DEFINES]->(overlay)
MERGE (self)-[:IMPORTS]->(hook)
MERGE (self)-[:IMPORTS]->(sprites)
@endcypher
*/
"use client";

// =====================================================================
// CanvasGameOverlay — 像素风游戏渲染层 (per WBS §23.36)
// 全部 SVG rect 矩阵渲染 (crispEdges), HUD: HP/SP/击杀数 + 操作提示
// 设计依据: BD-STAR-CANVAS-GAME-001 §2.1 <HUD> + <BulletPool> + <Robot3D>
// =====================================================================

import { memo } from "react";
import { Gamepad2, Heart, Zap, Skull, RotateCcw } from "lucide-react";
import { useCanvasGame } from "@/lib/canvas-game/useCanvasGame";
import {
  ENEMY_PALETTES,
  ENEMY_SPRITES,
  ROBOT_SPRITE,
  ROBOT_SPRITE_MOVE,
  SPRITE_PALETTE,
} from "@/lib/canvas-game/sprites";
import type { EnemySpawnSeed, GameState } from "@/lib/canvas-game/types";
import { useTranslation } from "@/lib/i18n";

interface CanvasGameOverlayProps {
  seeds: EnemySpawnSeed[];
  enabled: boolean;
  onExit: () => void;
}

const EXPLOSION_COLORS = ["#ffd43b", "#ff922b"];

/** 道具色表 (per BD §3.1.6): damage=红 / firerate=蓝 / heal=绿 */
const POWERUP_COLORS: Record<string, string> = {
  damage: "#ff6b6b",
  firerate: "#4dabf7",
  heal: "#51cf66",
};

function SpriteMatrix({ matrix, palette, cellSize }: { matrix: number[][]; palette: Record<number, string>; cellSize: number }) {
  const rects: React.ReactNode[] = [];
  matrix.forEach((row, y) => {
    row.forEach((cell, x) => {
      const color = palette[cell];
      if (color) rects.push(<rect key={`${x}-${y}`} x={x} y={y} width={1} height={1} fill={color} />);
    });
  });
  return (
    <g transform={`scale(${cellSize})`} shapeRendering="crispEdges">
      {rects}
    </g>
  );
}

function GameCanvasSvg({ state }: { state: GameState }) {
  const { robot, bullets, enemies, explosions, powerUps } = state;
  const moving = robot.state === "moving" || robot.state === "rolling";
  const robotMatrix = moving ? ROBOT_SPRITE_MOVE : ROBOT_SPRITE;
  const robotSize = 40; // 世界单位

  return (
    <svg
      data-testid="canvas-game-svg"
      viewBox="0 0 1200 800"
      className="h-full w-full"
      style={{ imageRendering: "pixelated", backgroundColor: "rgba(11,13,16,0.92)" }}
    >
      {/* 网格 (像素风地板) */}
      <defs>
        <pattern id="game-grid" width="40" height="40" patternUnits="userSpaceOnUse">
          <rect width="40" height="40" fill="none" stroke="#1a2030" strokeWidth="1" />
        </pattern>
      </defs>
      <rect width="1200" height="800" fill="url(#game-grid)" />

      {/* 爆炸粒子 (8x8 像素矩阵) */}
      {explosions.map((ex) => (
        <g
          key={ex.id}
          data-testid={`game-explosion-${ex.id}`}
          transform={`translate(${ex.x - 4}, ${ex.y - 4})`}
          opacity={ex.ttlFrames / 18}
        >
          <SpriteMatrix matrix={ex.cells} palette={{ 1: EXPLOSION_COLORS[1], 2: EXPLOSION_COLORS[0] }} cellSize={2} />
        </g>
      ))}

      {/* 子弹 (像素方形, per kind 分色: spiral 紫 / wave 青 / laser 亮红) */}
      {bullets.map((b) => (
        <rect
          key={b.id}
          data-testid={`game-bullet-${b.id}`}
          x={b.x - b.radius}
          y={b.y - b.radius}
          width={b.radius * 2}
          height={b.radius * 2}
          fill={
            b.kind === "player" ? "#51cf66"
            : b.kind === "enemy_radial" ? "#ff6b6b"
            : b.kind === "enemy_spiral" ? "#cc5de8"
            : b.kind === "enemy_wave" ? "#3bc9db"
            : b.kind === "enemy_laser" ? "#ffa8a8"
            : "#ffd43b"
          }
          shapeRendering="crispEdges"
        />
      ))}

      {/* Roguelike 升级道具 (per BD §3.1.6): 红=伤害 蓝=射速 绿=回血 */}
      {powerUps.map((p) => (
        <g key={p.id} data-testid={`game-powerup-${p.id}`} transform={`translate(${p.x - 8}, ${p.y - 8})`} opacity={p.ttlFrames < 90 ? 0.5 : 1}>
          <rect width={16} height={16} fill="#111" stroke={POWERUP_COLORS[p.kind]} strokeWidth={2} shapeRendering="crispEdges" />
          <rect x={4} y={4} width={8} height={8} fill={POWERUP_COLORS[p.kind]} shapeRendering="crispEdges" />
        </g>
      ))}

      {/* 敌人 (16x16 像素精灵, 3 色变体) */}
      {enemies.map((e) => (
        <g
          key={e.id}
          data-testid={`game-enemy-${e.id}`}
          transform={`translate(${e.x - 20}, ${e.y - 20})`}
        >
          <SpriteMatrix matrix={ENEMY_SPRITES[e.variant]} palette={ENEMY_PALETTES[e.variant]} cellSize={2.5} />
          {/* HP 条 */}
          <rect x={0} y={-6} width={40} height={4} fill="#111" shapeRendering="crispEdges" />
          <rect
            x={0}
            y={-6}
            width={(e.hp / e.maxHp) * 40}
            height={4}
            fill="#ff6b6b"
            shapeRendering="crispEdges"
          />
        </g>
      ))}

      {/* 玩家机器人 (16x16 像素精灵, 翻滚时闪烁) */}
      <g
        data-testid="game-robot"
        transform={`translate(${robot.x - robotSize / 2}, ${robot.y - robotSize / 2})`}
        opacity={robot.state === "rolling" ? 0.6 : 1}
      >
        <SpriteMatrix matrix={robotMatrix} palette={SPRITE_PALETTE} cellSize={robotSize / 16} />
      </g>
    </svg>
  );
}

function Hud({ state, onExit, onRestart }: { state: GameState; onExit: () => void; onRestart: () => void }) {
  const { t } = useTranslation();
  const { robot } = state;
  return (
    <>
      {/* HUD (per BD §2.1 <HUD>) */}
      <div className="absolute left-3 top-3 z-10 flex items-center gap-3 rounded border-2 border-[var(--cel-ink)] bg-[rgba(13,17,23,0.9)] px-3 py-2 font-mono text-xs">
        <span className="flex items-center gap-1 text-[#51cf66]" data-testid="game-hud-hp">
          <Heart size={12} /> {robot.hp}/{robot.maxHp}
        </span>
        <span className="flex items-center gap-1 text-[#748ffc]" data-testid="game-hud-sp">
          <Zap size={12} /> {Math.floor(robot.sp)}
        </span>
        <span className="flex items-center gap-1 text-[#ffd43b]" data-testid="game-hud-kills">
          <Skull size={12} /> {state.kills}
        </span>
        <button type="button" onClick={onExit} className="ml-2 rounded border border-[var(--cel-ink)] px-2 py-0.5 text-[10px] text-ink-dim hover:text-ink" data-testid="game-exit">
          {t.canvasGame.exit}
        </button>
      </div>

      {/* 操作提示 */}
      <div className="absolute bottom-3 left-3 z-10 rounded border border-[var(--cel-ink)] bg-[rgba(13,17,23,0.85)] px-3 py-1.5 font-mono text-[10px] text-ink-dim" data-testid="game-controls-hint">
        WASD {t.canvasGame.move} · SPACE {t.canvasGame.attack} · SHIFT {t.canvasGame.roll}
      </div>

      {/* Game Over */}
      {state.gameOver && (
        <div className="absolute inset-0 z-20 flex flex-col items-center justify-center gap-4 bg-black/70" data-testid="game-over">
          <div className="font-mono text-3xl text-[#ff6b6b]">{t.canvasGame.gameOver}</div>
          <div className="font-mono text-sm text-ink-dim">
            {t.canvasGame.killsLabel}: {state.kills}
          </div>
          <button type="button" onClick={onRestart} className="btn flex items-center gap-2" data-testid="game-restart">
            <RotateCcw size={14} /> {t.canvasGame.restart}
          </button>
        </div>
      )}
    </>
  );
}

export const CanvasGameOverlay = memo(function CanvasGameOverlay({ seeds, enabled, onExit }: CanvasGameOverlayProps) {
  const { gameState, restart } = useCanvasGame({ seeds, enabled });

  if (!enabled || !gameState) return null;

  return (
    <div className="absolute inset-0 z-10" data-testid="canvas-game-overlay">
      <div className="absolute inset-0">
        <GameCanvasSvg state={gameState} />
      </div>
      <Hud state={gameState} onExit={onExit} onRestart={restart} />
    </div>
  );
});

// 导出 sprite 组件给入口页复用 (游戏模式入口按钮图标)
export function GameModeIcon() {
  return <Gamepad2 size={16} />;
}
