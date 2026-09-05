// =====================================================================
// Agent Game — Characters (武侠机器人 SVG, 6 段 tier)
// =====================================================================
// Per 2026-09-05 12:33 JST 拍板 (ask_635d0b81cfd9b1dfc63fd70f):
//   - SVG 手工画 (Manga + 武侠 + 赛博朋克)
//   - 6 段 tier (per Lv 1, 2, 3, 5, 7, 10)
//   - 圆形头 (机器人) + 武士刀 + 披风 + 战甲 + 头冠
//   - 等级越高, 装饰越丰富 (Lv 7+ 神侠光环, Lv 10 金色+翅膀)
//
// 设计: 纯函数, 接受 (level, scale) → JSX (SVG 节点)
// 不依赖 store, 不读 game state, 只用 theme + visualForLevel
// =====================================================================

import type { AgentVisualTier } from "./types";
import { visualForLevel } from "./types";
import { COLORS, DECORATION, CHARACTER_TIERS } from "./theme";
import type { ReactNode } from "react";

interface CharacterSVGProps {
  level: number;
  /** 缩放 (1.0 = 64x64 base size) */
  scale?: number;
  /** 是否死亡 (灰化) */
  dead?: boolean;
  /** 1 个可选的 id 标签 (印章) */
  stampText?: string;
  /** 是否显示神侠光环 */
  showDivineHalo?: boolean;
}

/** 取 level 对应的 character tier (per CHARACTER_TIERS) */
export function characterTierForLevel(level: number): typeof CHARACTER_TIERS[number] {
  const idx = Math.max(0, Math.min(CHARACTER_TIERS.length - 1, Math.floor(level / 2)));
  return CHARACTER_TIERS[idx]!;
}

/**
 * Agent 角色 SVG (主函数)
 *   - 64x64 基准 (scale 倍)
 *   - 圆形头 (机器人) + 圆形身体
 *   - 武士刀 (Lv 3+)
 *   - 披风 (Lv 3+)
 *   - 战甲 (Lv 5+)
 *   - 头冠 (Lv 7+)
 *   - 神侠光环 (Lv 7+, showDivineHalo=true)
 *   - 印章 (stampText 有值)
 */
export function AgentCharacterSVG({ level, scale = 1, dead = false, stampText, showDivineHalo }: CharacterSVGProps): ReactNode {
  const visual = visualForLevel(level);
  const tier = characterTierForLevel(level);
  const fillOpacity = dead ? 0.4 : 1;
  const baseColor = dead ? COLORS.ash : tier.color;
  const accentColor = dead ? COLORS.ashLight : tier.accent;
  const s = scale; // size multiplier

  // 神侠光环 (Lv 7+ 显示)
  const halo = showDivineHalo && tier.hasHalo ? (
    <g opacity={0.6}>
      <circle cx={32 * s} cy={32 * s} r={DECORATION.divineHaloRadius * s} fill="none" stroke={accentColor} strokeWidth={1.5} opacity={0.4} />
      <circle cx={32 * s} cy={32 * s} r={(DECORATION.divineHaloRadius - 4) * s} fill="none" stroke={accentColor} strokeWidth={1} opacity={0.7} />
    </g>
  ) : null;

  // 头冠 (Lv 7+, 武士冠)
  const crown = tier.hasCrown ? (
    <g>
      {/* 冠顶 */}
      <path
        d={`M ${24 * s} ${8 * s} L ${28 * s} ${4 * s} L ${32 * s} ${8 * s} L ${36 * s} ${4 * s} L ${40 * s} ${8 * s}`}
        fill="none"
        stroke={accentColor}
        strokeWidth={1.5}
      />
      {/* 冠身 */}
      <rect x={22 * s} y={8 * s} width={20 * s} height={4 * s} fill={accentColor} opacity={0.4} />
    </g>
  ) : null;

  // 头 (圆形, 机器人)
  const head = (
    <g>
      <circle cx={32 * s} cy={18 * s} r={8 * s} fill={baseColor} opacity={fillOpacity} stroke={accentColor} strokeWidth={1} />
      {/* 眼睛 (2 圆点) */}
      <circle cx={29 * s} cy={17 * s} r={1.2 * s} fill={COLORS.gold} opacity={dead ? 0.3 : 1} />
      <circle cx={35 * s} cy={17 * s} r={1.2 * s} fill={COLORS.gold} opacity={dead ? 0.3 : 1} />
      {/* 嘴 (一横) */}
      <line x1={28 * s} y1={21 * s} x2={36 * s} y2={21 * s} stroke={accentColor} strokeWidth={0.8} />
      {/* 头带 / 头巾 (Lv 2+) */}
      {level >= 2 && (
        <rect x={24 * s} y={12 * s} width={16 * s} height={3 * s} fill={accentColor} opacity={0.6} />
      )}
    </g>
  );

  // 身体 (矩形, 战甲 Lv 5+)
  const body = (
    <g>
      <rect
        x={24 * s}
        y={26 * s}
        width={16 * s}
        height={20 * s}
        fill={tier.hasArmor ? baseColor : COLORS.inkDark}
        opacity={fillOpacity}
        stroke={accentColor}
        strokeWidth={1.5}
        rx={2}
      />
      {/* 战甲装饰 (Lv 5+) */}
      {tier.hasArmor && (
        <>
          <line x1={24 * s} y1={32 * s} x2={40 * s} y2={32 * s} stroke={accentColor} strokeWidth={0.8} opacity={0.6} />
          <line x1={32 * s} y1={26 * s} x2={32 * s} y2={46 * s} stroke={accentColor} strokeWidth={0.8} opacity={0.6} />
        </>
      )}
      {/* 胸前 core (赛博朋克风) */}
      <circle cx={32 * s} cy={36 * s} r={2.5 * s} fill={accentColor} opacity={0.8} />
      <circle cx={32 * s} cy={36 * s} r={1.5 * s} fill={COLORS.paper} opacity={0.6} />
    </g>
  );

  // 武士刀 (Lv 3+, 斜挎, 鞘 + 柄)
  const sword = tier.hasSword ? (
    <g>
      {/* 鞘 (跨过身体斜) */}
      <line
        x1={20 * s}
        y1={28 * s}
        x2={44 * s}
        y2={44 * s}
        stroke={COLORS.inkBlack}
        strokeWidth={2.5}
        opacity={fillOpacity}
      />
      {/* 鞘口 (装饰) */}
      <circle cx={20 * s} cy={28 * s} r={1.5 * s} fill={accentColor} opacity={0.8} />
      <circle cx={44 * s} cy={44 * s} r={1.5 * s} fill={accentColor} opacity={0.8} />
      {/* 柄 (右下) */}
      <line x1={44 * s} y1={44 * s} x2={48 * s} y2={48 * s} stroke={baseColor} strokeWidth={1.5} opacity={fillOpacity} />
    </g>
  ) : null;

  // 披风 (Lv 3+, 背后飘)
  const cloak = tier.hasCloak ? (
    <g opacity={fillOpacity * 0.7}>
      <path
        d={`M ${20 * s} ${28 * s} L ${14 * s} ${50 * s} L ${20 * s} ${52 * s} L ${26 * s} ${32 * s} Z`}
        fill={tier.hasArmor ? baseColor : accentColor}
        opacity={0.5}
      />
      <path
        d={`M ${44 * s} ${28 * s} L ${50 * s} ${50 * s} L ${44 * s} ${52 * s} L ${38 * s} ${32 * s} Z`}
        fill={tier.hasArmor ? baseColor : accentColor}
        opacity={0.5}
      />
    </g>
  ) : null;

  // 脚 (简单 2 矩形)
  const feet = (
    <g opacity={fillOpacity}>
      <rect x={26 * s} y={46 * s} width={4 * s} height={6 * s} fill={baseColor} />
      <rect x={34 * s} y={46 * s} width={4 * s} height={6 * s} fill={baseColor} />
    </g>
  );

  // 印章 (右上角)
  const stamp = stampText ? (
    <g transform={`translate(${50 * s}, ${2 * s})`} opacity={0.9}>
      <rect width={12 * s} height={12 * s} fill={COLORS.vermilion} stroke={COLORS.vermilionGlow} strokeWidth={DECORATION.stampBorder} rx={1} />
      <text
        x={6 * s}
        y={9 * s}
        textAnchor="middle"
        fontSize={6 * s}
        fill={COLORS.paper}
        fontFamily="Hiragino Mincho ProN, Yu Mincho, MS Mincho, serif"
        fontWeight="bold"
      >
        {stampText.slice(0, 1)}
      </text>
    </g>
  ) : null;

  return (
    <g data-testid={`agent-character-svg-lv${level}`} data-character-tier={tier.name}>
      {halo}
      {cloak}
      {sword}
      {head}
      {body}
      {feet}
      {crown}
      {stamp}
    </g>
  );
}

/** 简化版: 只取主色 (用于 minimap 等小尺寸) */
export function AgentColorSVG(level: number): string {
  const tier = characterTierForLevel(level);
  return tier.color;
}
