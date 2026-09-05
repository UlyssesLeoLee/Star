"use client";

// =====================================================================
// Decorations — 日漫 + 赛博朋克 装饰组件 (per 9/5 12:33 JST 拍板)
// =====================================================================
// 全装饰 (per 拍板 #3):
//   - EnergyRing 能量光环 (脉动圆环, 赛博朋克风)
//   - Stamp 印章 (右上角 "Mavis" / 字符 印章)
//   - InkTrail 墨汁拖尾 (移动残影, 武侠风)
//   - HaloArc 神侠光环 (Lv 7+ 弧线装饰)
// =====================================================================

import { COLORS, DECORATION } from "@/lib/agent-game/theme";
import { characterTierForLevel } from "@/lib/agent-game/characters";
import type { ReactNode } from "react";

interface EnergyRingProps {
  cx: number;
  cy: number;
  /** 主色 (默认 cyan) */
  color?: string;
  /** 半径 (默认 40) */
  radius?: number;
  /** 脉动动画 (默认 true) */
  pulse?: boolean;
}

/** 能量光环: 2 圈脉动虚线圆环 (赛博朋克) */
export function EnergyRing({ cx, cy, color = COLORS.neonCyan, radius = DECORATION.energyRingRadius, pulse = true }: EnergyRingProps) {
  return (
    <g data-testid="decoration-energy-ring" opacity={0.5}>
      <circle cx={cx} cy={cy} r={radius} fill="none" stroke={color} strokeWidth={DECORATION.energyRingStroke} opacity={0.4} strokeDasharray="4 4">
        {pulse && <animateTransform attributeName="transform" type="rotate" from={`0 ${cx} ${cy}`} to={`360 ${cx} ${cy}`} dur="8s" repeatCount="indefinite" />}
      </circle>
      <circle cx={cx} cy={cy} r={radius - 4} fill="none" stroke={color} strokeWidth={1} opacity={0.7}>
        {pulse && <animateTransform attributeName="transform" type="rotate" from={`360 ${cx} ${cy}`} to={`0 ${cx} ${cy}`} dur="6s" repeatCount="indefinite" />}
      </circle>
    </g>
  );
}

interface StampProps {
  /** 印章字符 (1-2 字) */
  text: string;
  cx: number;
  cy: number;
  /** 印章颜色 (默认朱红) */
  color?: string;
  /** 印章尺寸 (默认 12) */
  size?: number;
}

/** 印章: 红色方块 + 字符 (日式风格) */
export function Stamp({ text, cx, cy, color = COLORS.vermilion, size = DECORATION.stampSize }: StampProps) {
  return (
    <g
      data-testid="decoration-stamp"
      transform={`translate(${cx - size / 2}, ${cy - size / 2})`}
      opacity={0.9}
    >
      <rect
        width={size}
        height={size}
        fill={color}
        stroke={COLORS.vermilionGlow}
        strokeWidth={DECORATION.stampBorder}
        rx={1}
      />
      <text
        x={size / 2}
        y={size * 0.7}
        textAnchor="middle"
        fontSize={size * 0.55}
        fill={COLORS.paper}
        fontFamily='"Hiragino Mincho ProN", "Yu Mincho", "MS Mincho", serif'
        fontWeight="bold"
      >
        {text.slice(0, 1)}
      </text>
    </g>
  );
}

interface InkTrailProps {
  cx: number;
  cy: number;
  /** 残影位置数组 (从老到新) */
  trail: Array<{ x: number; y: number }>;
  color?: string;
}

/** 墨汁拖尾: 一串残影 (从老到新透明度递增) */
export function InkTrail({ cx, cy, trail, color = COLORS.paper }: InkTrailProps) {
  const len = trail.length;
  if (len === 0) return null;
  return (
    <g data-testid="decoration-ink-trail" opacity={0.6}>
      {trail.map((pos, i) => {
        const opacity = (i + 1) / (len + 1) * DECORATION.inkTrailOpacityDecay * 4;
        return (
          <circle
            key={i}
            cx={pos.x}
            cy={pos.y}
            r={(4 - i * 0.5)}
            fill={color}
            opacity={opacity}
          />
        );
      })}
      {/* 当前位置高亮 */}
      <circle cx={cx} cy={cy} r={3} fill={color} opacity={0.9} />
    </g>
  );
}

interface HaloArcProps {
  level: number;
  cx: number;
  cy: number;
  size?: number;
}

/** 神侠光环: Lv 7+ 圆弧装饰 (per CHARACTER_TIERS) */
export function HaloArc({ level, cx, cy, size = DECORATION.divineHaloRadius }: HaloArcProps) {
  const tier = characterTierForLevel(level);
  if (!tier.hasHalo) return null;
  return (
    <g data-testid="decoration-halo-arc" opacity={0.7}>
      <circle cx={cx} cy={cy} r={size} fill="none" stroke={tier.accent} strokeWidth={1} strokeDasharray="3 5" opacity={0.6}>
        <animateTransform attributeName="transform" type="rotate" from={`0 ${cx} ${cy}`} to={`360 ${cx} ${cy}`} dur="10s" repeatCount="indefinite" />
      </circle>
      {/* 4 角小金点 */}
      {[0, 90, 180, 270].map((deg) => {
        const rad = (deg * Math.PI) / 180;
        const x = cx + Math.cos(rad) * (size + 4);
        const y = cy + Math.sin(rad) * (size + 4);
        return <circle key={deg} cx={x} cy={y} r={2} fill={tier.accent} />;
      })}
    </g>
  );
}

interface GodSealProps {
  level: number;
  cx: number;
  cy: number;
  size?: number;
}

/** 神印: 满级 Lv 10 才显示, "Mavis" 大印 */
export function GodSeal({ level, cx, cy, size = 18 }: GodSealProps) {
  if (level < 10) return null;
  return (
    <g data-testid="decoration-god-seal" transform={`translate(${cx - size / 2}, ${cy - size / 2})`} opacity={0.85}>
      <rect width={size} height={size} fill={COLORS.gold} stroke={COLORS.goldGlow} strokeWidth={2} rx={1} />
      <text
        x={size / 2}
        y={size * 0.7}
        textAnchor="middle"
        fontSize={size * 0.55}
        fill={COLORS.inkBlack}
        fontFamily='"Hiragino Mincho ProN", "Yu Mincho", "MS Mincho", serif'
        fontWeight="bold"
      >
        神
      </text>
    </g>
  );
}

/** 组合: 完整装饰套件 (per level) — 给中心节点用 */
export function FullDecoration({
  level,
  cx,
  cy,
  size = 64,
  showStamp = true,
  showHalo = true,
  stampText = "M",
}: {
  level: number;
  cx: number;
  cy: number;
  size?: number;
  showStamp?: boolean;
  showHalo?: boolean;
  stampText?: string;
}): ReactNode {
  const tier = characterTierForLevel(level);
  return (
    <g data-testid="full-decoration" data-level={level}>
      {showHalo && tier.hasHalo && <HaloArc level={level} cx={cx} cy={cy} size={size * 0.6} />}
      <EnergyRing cx={cx} cy={cy} color={tier.accent} radius={size * 0.7} />
      {showStamp && level >= 5 && <Stamp text={stampText} cx={cx + size * 0.5} cy={cy - size * 0.5} color={tier.color} />}
      {level >= 10 && <GodSeal level={level} cx={cx - size * 0.5} cy={cy - size * 0.5} />}
    </g>
  );
}
