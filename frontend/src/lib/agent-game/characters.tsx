// =====================================================================
// Agent Game — Characters (3渲2 日漫战术机灵 / Manga Mecha Spirit SVG)
// =====================================================================
// Per 2026-09-06 用户发令:
//   - 增强游戏界面感 (HUD / Game UI), 保持低认知负荷的简洁风格
//   - 3渲2 (3D-to-2D NPR Cel-Shaded) 日漫战术美学
//   - Agent 图标全面升级为日系游戏/热血漫画风格 (Manga Mecha Spirit)
//   - 6 段 tier 视觉渐进 (Lv 1, 2, 3, 5, 7, 10)
//   - 硬墨黑描边 + 二阶赛璐珞阴影阶梯 + 战术机灵耳羽 + 发光动漫战术目镜
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
 * Agent 角色 SVG (3渲2 动漫游戏风格)
 *   - 64x64 基准 (scale 倍)
 *   - 硬墨外边框 (Manga Cel Inking)
 *   - 日漫战术机灵天线耳羽 + 流线型战术头盔
 *   - 动态战术目镜 (Visor Glint / Anime Crosshair Eyes)
 *   - 战术光刃 Katana (Lv 3+)
 *   - 赛璐珞披风 / 破风战术围巾 (Lv 3+)
 *   - 重装多面体战甲与胸口矩阵核心 (Lv 5+)
 *   - 黄金战术冠冕与悬浮浮游刃环 (Lv 7+)
 *   - 神圣金色天脉羽翼 (Lv 10)
 */
export function AgentCharacterSVG({
  level,
  scale = 1,
  dead = false,
  stampText,
  showDivineHalo,
}: CharacterSVGProps): ReactNode {
  const visual = visualForLevel(level);
  const tier = characterTierForLevel(level);
  const s = scale;

  // 基础色与阴影色阶 (Cel 2-Tone Shading)
  const baseColor = dead ? COLORS.ash : tier.color;
  const accentColor = dead ? COLORS.ashLight : tier.accent;
  const shadowColor = dead ? "#374151" : "#0d1117";
  const glowColor = dead ? "#6b7280" : tier.accent;
  const fillOpacity = dead ? 0.5 : 1;

  // 1. 神圣光环与浮游刃 (Lv 7+ 剑圣/神侠)
  const halo = (showDivineHalo || tier.hasHalo) && tier.hasHalo ? (
    <g opacity={dead ? 0.3 : 0.9}>
      {/* 战术几何外环 */}
      <circle
        cx={32 * s}
        cy={32 * s}
        r={28 * s}
        fill="none"
        stroke={accentColor}
        strokeWidth={1.8 * s}
        strokeDasharray={`${6 * s} ${4 * s}`}
        opacity={0.7}
      >
        <animateTransform
          attributeName="transform"
          type="rotate"
          from={`0 ${32 * s} ${32 * s}`}
          to={`360 ${32 * s} ${32 * s}`}
          dur="14s"
          repeatCount="indefinite"
        />
      </circle>
      <circle
        cx={32 * s}
        cy={32 * s}
        r={24 * s}
        fill="none"
        stroke="#000000"
        strokeWidth={1.2 * s}
        opacity={0.9}
      />
      {/* 4 方向浮游结晶菱形刃 (Bits) */}
      {[0, 90, 180, 270].map((deg) => {
        const rad = (deg * Math.PI) / 180;
        const bx = 32 * s + Math.cos(rad) * 28 * s;
        const by = 32 * s + Math.sin(rad) * 28 * s;
        return (
          <g key={deg} transform={`translate(${bx}, ${by}) rotate(${deg + 45})`}>
            <rect
              x={-3 * s}
              y={-3 * s}
              width={6 * s}
              height={6 * s}
              fill={accentColor}
              stroke="#000000"
              strokeWidth={1.2 * s}
            />
          </g>
        );
      })}
      {/* Lv 10 究极神格金翼 */}
      {level >= 10 && (
        <g opacity={0.85}>
          {/* 左金翼 */}
          <path
            d={`M ${18 * s} ${28 * s} L ${2 * s} ${14 * s} L ${6 * s} ${28 * s} L ${0 * s} ${22 * s} L ${6 * s} ${36 * s} L ${18 * s} ${34 * s} Z`}
            fill={COLORS.gold}
            stroke="#000000"
            strokeWidth={1.5 * s}
          />
          {/* 右金翼 */}
          <path
            d={`M ${46 * s} ${28 * s} L ${62 * s} ${14 * s} L ${58 * s} ${28 * s} L ${64 * s} ${22 * s} L ${58 * s} ${36 * s} L ${46 * s} ${34 * s} Z`}
            fill={COLORS.gold}
            stroke="#000000"
            strokeWidth={1.5 * s}
          />
        </g>
      )}
    </g>
  ) : null;

  // 2. 战术披风 / 破风围巾 (Lv 3+)
  const cloak = tier.hasCloak ? (
    <g opacity={fillOpacity}>
      {/* 披风主体: 带有日漫折痕与黑色硬墨描边 */}
      <path
        d={`M ${20 * s} ${27 * s} L ${8 * s} ${52 * s} L ${18 * s} ${50 * s} L ${24 * s} ${56 * s} L ${27 * s} ${33 * s} Z`}
        fill={shadowColor}
        stroke="#000000"
        strokeWidth={1.8 * s}
      />
      <path
        d={`M ${21 * s} ${28 * s} L ${11 * s} ${50 * s} L ${18 * s} ${48 * s} L ${22 * s} ${34 * s} Z`}
        fill={baseColor}
        opacity={0.85}
      />
      {/* 右侧飘拂褶皱 */}
      <path
        d={`M ${44 * s} ${27 * s} L ${56 * s} ${52 * s} L ${46 * s} ${50 * s} L ${40 * s} ${56 * s} L ${37 * s} ${33 * s} Z`}
        fill={shadowColor}
        stroke="#000000"
        strokeWidth={1.8 * s}
      />
      <path
        d={`M ${43 * s} ${28 * s} L ${53 * s} ${50 * s} L ${46 * s} ${48 * s} L ${42 * s} ${34 * s} Z`}
        fill={baseColor}
        opacity={0.85}
      />
    </g>
  ) : null;

  // 3. 武士刀 / 战术高频刃 (Lv 3+)
  const sword = tier.hasSword ? (
    <g opacity={fillOpacity}>
      {/* 刀身/鞘: 斜跨背部 */}
      <line
        x1={14 * s}
        y1={48 * s}
        x2={50 * s}
        y2={12 * s}
        stroke="#000000"
        strokeWidth={3.2 * s}
        strokeLinecap="round"
      />
      <line
        x1={14 * s}
        y1={48 * s}
        x2={43 * s}
        y2={19 * s}
        stroke={dead ? COLORS.ash : "#1f2937"}
        strokeWidth={2 * s}
      />
      {/* 刀镡 Tsuba */}
      <rect
        x={41 * s}
        y={18 * s}
        width={5 * s}
        height={5 * s}
        fill={accentColor}
        stroke="#000000"
        strokeWidth={1.2 * s}
        transform={`rotate(45 ${43.5 * s} ${20.5 * s})`}
      />
      {/* 刀柄 Tsuka & 荧光封刃 */}
      <line
        x1={44 * s}
        y1={18 * s}
        x2={50 * s}
        y2={12 * s}
        stroke={accentColor}
        strokeWidth={2 * s}
      />
    </g>
  ) : null;

  // 4. 机体下肢与足部装甲 (Legs / Thrusters)
  const feet = (
    <g opacity={fillOpacity}>
      {/* 左腿装甲 */}
      <path
        d={`M ${24 * s} ${46 * s} L ${23 * s} ${55 * s} L ${28 * s} ${57 * s} L ${29 * s} ${46 * s} Z`}
        fill={shadowColor}
        stroke="#000000"
        strokeWidth={1.8 * s}
      />
      <rect x={24.5 * s} y={47 * s} width={3.5 * s} height={6 * s} fill={baseColor} />
      {/* 右腿装甲 */}
      <path
        d={`M ${35 * s} ${46 * s} L ${36 * s} ${55 * s} L ${41 * s} ${57 * s} L ${40 * s} ${46 * s} Z`}
        fill={shadowColor}
        stroke="#000000"
        strokeWidth={1.8 * s}
      />
      <rect x={36 * s} y={47 * s} width={3.5 * s} height={6 * s} fill={baseColor} />
      {/* 足尖战术喷口高光 */}
      <line x1={23 * s} y1={56 * s} x2={28 * s} y2={58 * s} stroke={accentColor} strokeWidth={1.2 * s} />
      <line x1={36 * s} y1={56 * s} x2={41 * s} y2={58 * s} stroke={accentColor} strokeWidth={1.2 * s} />
    </g>
  );

  // 5. 机体躯干与装甲胸腔 (Mecha Torso & Reactor Core)
  const body = (
    <g opacity={fillOpacity}>
      {/* 胸部基础装甲外框 (倒梯形切角) */}
      <path
        d={`M ${22 * s} ${27 * s} L ${42 * s} ${27 * s} L ${39 * s} ${47 * s} L ${25 * s} ${47 * s} Z`}
        fill={shadowColor}
        stroke="#000000"
        strokeWidth={2 * s}
      />
      {/* 胸腹主色涂装 (阶梯半色调) */}
      <path
        d={`M ${23.5 * s} ${28.5 * s} L ${40.5 * s} ${28.5 * s} L ${38 * s} ${45.5 * s} L ${26 * s} ${45.5 * s} Z`}
        fill={tier.hasArmor ? baseColor : "#1c2333"}
      />
      {/* 重装护胸板刻线 (Lv 5+) */}
      {tier.hasArmor && (
        <>
          <path
            d={`M ${25 * s} ${32 * s} L ${32 * s} ${35 * s} L ${39 * s} ${32 * s}`}
            fill="none"
            stroke="#000000"
            strokeWidth={1.5 * s}
          />
          {/* 左肩甲 */}
          <polygon
            points={`${18 * s},${26 * s} ${24 * s},${24 * s} ${23 * s},${34 * s} ${17 * s},${31 * s}`}
            fill={baseColor}
            stroke="#000000"
            strokeWidth={1.8 * s}
          />
          {/* 右肩甲 */}
          <polygon
            points={`${46 * s},${26 * s} ${40 * s},${24 * s} ${41 * s},${34 * s} ${47 * s},${31 * s}`}
            fill={baseColor}
            stroke="#000000"
            strokeWidth={1.8 * s}
          />
        </>
      )}
      {/* 胸口矩阵核心 (Arc Reactor Core) */}
      <circle
        cx={32 * s}
        cy={37 * s}
        r={4 * s}
        fill="#000000"
        stroke={accentColor}
        strokeWidth={1.5 * s}
      />
      <circle
        cx={32 * s}
        cy={37 * s}
        r={2.2 * s}
        fill={accentColor}
      />
      <circle
        cx={32 * s}
        cy={37 * s}
        r={1 * s}
        fill="#ffffff"
        opacity={dead ? 0.2 : 0.9}
      />
    </g>
  );

  // 6. 3渲2 战术头盔与漫画机灵面容 (Head, Visor & Antennae)
  const head = (
    <g opacity={fillOpacity}>
      {/* 战术耳羽 / 机动天线 (Anime Mecha Ear Fins) */}
      {/* 左天线 */}
      <polygon
        points={`${22 * s},${17 * s} ${13 * s},${10 * s} ${19 * s},${13 * s} ${21 * s},${22 * s}`}
        fill={accentColor}
        stroke="#000000"
        strokeWidth={1.8 * s}
      />
      {/* 右天线 */}
      <polygon
        points={`${42 * s},${17 * s} ${51 * s},${10 * s} ${45 * s},${13 * s} ${43 * s},${22 * s}`}
        fill={accentColor}
        stroke="#000000"
        strokeWidth={1.8 * s}
      />

      {/* 头盔外轮廓: 流线多面体造型 + 2px 硬墨描边 */}
      <path
        d={`M ${22 * s} ${15 * s} Q ${32 * s} ${8 * s} ${42 * s} ${15 * s} L ${43 * s} ${24 * s} L ${37 * s} ${28 * s} L ${27 * s} ${28 * s} L ${21 * s} ${24 * s} Z`}
        fill={shadowColor}
        stroke="#000000"
        strokeWidth={2 * s}
      />
      {/* 头部顶盔固有色 */}
      <path
        d={`M ${23.5 * s} ${16 * s} Q ${32 * s} ${10 * s} ${40.5 * s} ${16 * s} L ${41 * s} ${21 * s} L ${23 * s} ${21 * s} Z`}
        fill={baseColor}
      />
      {/* 头顶高光反射 (Cel Highlight) */}
      <path
        d={`M ${27 * s} ${13 * s} Q ${32 * s} ${10.5 * s} ${37 * s} ${13 * s}`}
        fill="none"
        stroke="#ffffff"
        strokeWidth={1.2 * s}
        opacity={0.7}
      />

      {/* 额前机能晶片 / 战术额带 (Lv 2+) */}
      {level >= 2 && (
        <rect
          x={28 * s}
          y={15 * s}
          width={8 * s}
          height={2.5 * s}
          fill={accentColor}
          stroke="#000000"
          strokeWidth={1 * s}
          rx={0.5 * s}
        />
      )}

      {/* 战术目镜 / 动漫眼部 (Anime Visor) */}
      {/* 目镜底框 */}
      <polygon
        points={`${23 * s},${20 * s} ${41 * s},${20 * s} ${38 * s},${25.5 * s} ${26 * s},${25.5 * s}`}
        fill="#050811"
        stroke="#000000"
        strokeWidth={1.2 * s}
      />

      {/* 眼睛呈现: 活着的战术锐眸 vs 战损 KO */}
      {!dead ? (
        <g>
          {/* 左眼: 帅气动漫机甲锐眼 */}
          <polygon
            points={`${25.5 * s},${21.5 * s} ${30 * s},${21.5 * s} ${28.5 * s},${24.5 * s} ${26.5 * s},${24.5 * s}`}
            fill={accentColor}
          />
          <circle cx={27.5 * s} cy={22.5 * s} r={0.7 * s} fill="#ffffff" />

          {/* 右眼: 帅气动漫机甲锐眼 */}
          <polygon
            points={`${34 * s},${21.5 * s} ${38.5 * s},${21.5 * s} ${37.5 * s},${24.5 * s} ${35.5 * s},${24.5 * s}`}
            fill={accentColor}
          />
          <circle cx={36.5 * s} cy={22.5 * s} r={0.7 * s} fill="#ffffff" />

          {/* 战术准星斜向反光 (Anime Eye Glint Slash) */}
          <line
            x1={24 * s}
            y1={24 * s}
            x2={40 * s}
            y2={21 * s}
            stroke="#ffffff"
            strokeWidth={0.7 * s}
            opacity={0.65}
          />
        </g>
      ) : (
        /* 死亡态: 经典的漫画战损 × × 符号 */
        <g stroke={COLORS.vermilion} strokeWidth={1.4 * s} strokeLinecap="round">
          <line x1={26 * s} y1={21.5 * s} x2={29 * s} y2={24.5 * s} />
          <line x1={29 * s} y1={21.5 * s} x2={26 * s} y2={24.5 * s} />
          <line x1={35 * s} y1={21.5 * s} x2={38 * s} y2={24.5 * s} />
          <line x1={38 * s} y1={21.5 * s} x2={35 * s} y2={24.5 * s} />
        </g>
      )}

      {/* 下颌装甲面罩刻线 (Chin Guard) */}
      <polygon
        points={`${28 * s},${25.5 * s} ${36 * s},${25.5 * s} ${34 * s},${27.5 * s} ${30 * s},${27.5 * s}`}
        fill={shadowColor}
        stroke="#000000"
        strokeWidth={1 * s}
      />
    </g>
  );

  // 7. 武士天冠 (Lv 7+ 剑圣/神侠)
  const crown = tier.hasCrown ? (
    <g opacity={fillOpacity}>
      {/* V-Fin / 日系武士金冠 */}
      <path
        d={`M ${23 * s} ${13 * s} L ${18 * s} ${3 * s} L ${26 * s} ${8 * s} L ${32 * s} ${2 * s} L ${38 * s} ${8 * s} L ${46 * s} ${3 * s} L ${41 * s} ${13 * s} Z`}
        fill={accentColor}
        stroke="#000000"
        strokeWidth={1.8 * s}
      />
      {/* 冠心红宝石菱镜 */}
      <polygon
        points={`${32 * s},${4 * s} ${34.5 * s},${7.5 * s} ${32 * s},${11 * s} ${29.5 * s},${7.5 * s}`}
        fill={COLORS.vermilion}
        stroke="#000000"
        strokeWidth={1.2 * s}
      />
    </g>
  ) : null;

  // 8. 战术部队印章 (右上角 Stamp)
  const stamp = stampText ? (
    <g transform={`translate(${48 * s}, ${2 * s})`} opacity={0.95}>
      <rect
        width={14 * s}
        height={14 * s}
        fill={COLORS.vermilion}
        stroke="#000000"
        strokeWidth={1.5 * s}
      />
      <rect
        x={1.5 * s}
        y={1.5 * s}
        width={11 * s}
        height={11 * s}
        fill="none"
        stroke={COLORS.paper}
        strokeWidth={0.8 * s}
      />
      <text
        x={7 * s}
        y={10.5 * s}
        textAnchor="middle"
        fontSize={7.5 * s}
        fill={COLORS.paper}
        fontFamily='"Hiragino Mincho ProN", "Yu Mincho", "MS Mincho", serif'
        fontWeight="900"
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
      {feet}
      {body}
      {head}
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

