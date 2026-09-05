// =====================================================================
// Agent Game — Enemies (光球 SVG, 6 种类型)
// =====================================================================
// Per 2026-09-05 12:33 JST 拍板 (ask_635d0b81cfd9b1dfc63fd70f):
//   - 敌人 = 各种光球 (6 种)
//   - SVG 手工画: 外晕 + 内核 + 脉冲动画 (CSS animation via animateTransform)
//   - 颜色: 青/朱/金/紫/白/神 (per ENEMY_TYPES)
//   - 等级越高的 wi 敌人颜色越强 (p0 朱火光球 > p1 金雷 > p2 青 > p3 白纸)
//
// 设计: 纯函数, 接受 (type, scale, dead) → JSX
// =====================================================================

import { COLORS, ENEMY_TYPES } from "./theme";
import type { ReactNode } from "react";
import { enemyTypeForPriority, pickRandomEnemyType } from "./theme";

type EnemyTypeKey = typeof ENEMY_TYPES[number]["key"];

interface EnemyOrbSVGProps {
  type: EnemyTypeKey | string;
  scale?: number;
  dead?: boolean;
  /** 可选: 在 orb 中间叠加一个字符 (e.g. "!" for boss) */
  glyph?: string;
}

/** 由 type 找 enemy definition */
function getEnemyDef(type: string): typeof ENEMY_TYPES[number] {
  return ENEMY_TYPES.find((e) => e.key === type) ?? ENEMY_TYPES[0]!;
}

/**
 * 光球 SVG (3渲2 漫画战术核心 / Enemy Orb)
 *   - 64x64 基准 (scale 倍)
 *   - 战术瞄准括号与刻度 (Tactical Brackets)
 *   - 二阶赛璐珞阴影硬黑墨线 (Manga Inked Outline)
 *   - 旋转粒子与十字高光
 */
export function EnemyOrbSVG({ type, scale = 1, dead = false, glyph }: EnemyOrbSVGProps): ReactNode {
  const def = getEnemyDef(type);
  const s = scale;
  const fillOpacity = dead ? 0.35 : 1;
  const cx = 32 * s;
  const cy = 32 * s;

  const innerR = 10 * s;
  const outerR = 22 * s;
  const ringR = 17 * s;

  return (
    <g data-testid={`enemy-orb-${type}`} data-enemy-tier={def.name} opacity={fillOpacity}>
      <defs>
        <radialGradient id={`orb-grad-${type}`}>
          <stop offset="0%" stopColor={def.glow} stopOpacity={0.8} />
          <stop offset="60%" stopColor={def.color} stopOpacity={0.35} />
          <stop offset="100%" stopColor={def.color} stopOpacity={0} />
        </radialGradient>
      </defs>

      {/* 外晕 */}
      <circle cx={cx} cy={cy} r={outerR} fill={`url(#orb-grad-${type})`} />

      {/* 战术瞄准十字线 (微透明) */}
      <g stroke={def.glow} strokeWidth={1 * s} opacity={0.4}>
        <line x1={cx - 24 * s} y1={cy} x2={cx - 14 * s} y2={cy} />
        <line x1={cx + 14 * s} y1={cy} x2={cx + 24 * s} y2={cy} />
        <line x1={cx} y1={cy - 24 * s} x2={cx} y2={cy - 14 * s} />
        <line x1={cx} y1={cy + 14 * s} x2={cx} y2={cy + 24 * s} />
      </g>

      {/* 战术外环 (旋转刻度) */}
      <circle
        cx={cx}
        cy={cy}
        r={ringR}
        fill="none"
        stroke={def.glow}
        strokeWidth={1.2 * s}
        opacity={0.7}
        strokeDasharray={`${4 * s} ${3 * s}`}
      >
        <animateTransform
          attributeName="transform"
          type="rotate"
          from={`0 ${cx} ${cy}`}
          to={`360 ${cx} ${cy}`}
          dur="7s"
          repeatCount="indefinite"
        />
      </circle>

      {/* 硬黑描边底层 (Cel 墨线) */}
      <circle cx={cx} cy={cy} r={innerR + 1.5 * s} fill="#000000" />

      {/* 内核主色 */}
      <circle cx={cx} cy={cy} r={innerR} fill={def.color} stroke="#000000" strokeWidth={1.5 * s} />

      {/* 赛璐珞高光弧 */}
      <path
        d={`M ${cx - 7 * s} ${cy - 2 * s} A ${7 * s} ${7 * s} 0 0 1 ${cx + 2 * s} ${cy - 7 * s}`}
        fill="none"
        stroke="#ffffff"
        strokeWidth={1.8 * s}
        strokeLinecap="round"
        opacity={0.8}
      />

      {/* 核心光点 */}
      <circle cx={cx - 3 * s} cy={cy - 3 * s} r={2 * s} fill="#ffffff" opacity={0.9} />

      {/* 字符 (e.g. "!" for boss) */}
      {glyph && (
        <text
          x={cx}
          y={cy + 4.5 * s}
          textAnchor="middle"
          fontSize={13 * s}
          fill="#000000"
          fontWeight="900"
          fontFamily='"JetBrains Mono", ui-monospace, monospace'
        >
          {glyph}
        </text>
      )}
    </g>
  );
}

/** Helper: 接受 priority 字符串返回 orb SVG (JSX component 形式) */
export function EnemyOrbForPrioritySVG({ priority, scale, dead }: { priority: string; scale?: number; dead?: boolean }): ReactNode {
  const def = enemyTypeForPriority(priority);
  return <EnemyOrbSVG type={def.key} scale={scale} dead={dead} />;
}

/** Helper: 接受 seed 挑 1 个光球 (JSX component 形式) */
export function RandomEnemyOrbSVG({ seed, scale, dead }: { seed: number; scale?: number; dead?: boolean }): ReactNode {
  const def = pickRandomEnemyType(seed);
  return <EnemyOrbSVG type={def.key} scale={scale} dead={dead} />;
}

/** Boss 光球 (神光球, 用神侠光环包) (JSX component 形式) */
export function BossOrbSVG({ scale = 1, dead = false }: { scale?: number; dead?: boolean }): ReactNode {
  return (
    <g data-testid="enemy-orb-boss" opacity={dead ? 0.3 : 1}>
      <EnemyOrbSVG type="boss_divine" scale={scale} dead={dead} glyph="!" />
      {/* 神光球额外加 4 角 (赛博) */}
      <g stroke={COLORS.gold} strokeWidth={1.5} fill="none" opacity={0.6}>
        <line x1={32 * scale} y1={4 * scale} x2={32 * scale} y2={14 * scale} />
        <line x1={32 * scale} y1={50 * scale} x2={32 * scale} y2={60 * scale} />
        <line x1={4 * scale} y1={32 * scale} x2={14 * scale} y2={32 * scale} />
        <line x1={50 * scale} y1={32 * scale} x2={60 * scale} y2={32 * scale} />
      </g>
    </g>
  );
}
