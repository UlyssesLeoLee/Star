"use client";

// =====================================================================
// RoguelikeCanvas — 网格画布 (4-邻接点击移动, 日漫 + 武侠 + 赛博朋克 风格)
// =====================================================================
// Per 2026-09-05 12:23 JST 拍板 (ask_8a60a3bc90f779308a69be1d):
//   - 程序生成 grid 8x6 (default), 起点 (0,0) + 终点 (右下 boss)
//   - 4-邻接点击移动 (上下左右)
//   - 节点类型: start / enemy / treasure / trap / blank / boss
//   - trap 不可进入 (per 拍板 BFS 限制)
// Per 2026-09-05 12:33 JST 拍板 (ask_635d0b81cfd9b1dfc63fd70f):
//   - 日漫 + 武侠 + 赛博朋克 风格
//   - 角色: AgentCharacterSVG (6 段 tier, 墨黑/朱红/霓虹青/金/紫)
//   - 敌人: EnemyOrbSVG (6 种光球, 按 priority 着色)
//   - 装饰: EnergyRing / HaloArc / Stamp / GodSeal (神侠 Lv 10)
// =====================================================================

import { useState, useMemo, useCallback, useEffect } from "react";
import type { GameMap, MapCell } from "@/lib/agent-game/mapgen";
import { isAdjacent } from "@/lib/agent-game/mapgen";
import type { AgentSession, WorkItem } from "@/types/ids";
import { StatusPill } from "@/components/StatusPill";
import { MapPin, RefreshCw } from "lucide-react";
import { AgentCharacterSVG } from "@/lib/agent-game/characters";
import { EnemyOrbSVG, BossOrbSVG } from "@/lib/agent-game/enemies";
import { enemyTypeForPriority } from "@/lib/agent-game/theme";
import { useAgentGameTheme } from "@/lib/agent-game/theme-tokens";
import { EnergyRing, HaloArc, Stamp, GodSeal } from "@/components/agent-game/Decorations";

interface RoguelikeCanvasProps {
  map: GameMap;
  position: { x: number; y: number };
  agent: AgentSession;
  workItems: ReadonlyArray<WorkItem>;
  onMove: (target: { x: number; y: number }) => void;
  onReset: () => void;
  /** 是否可移动 (死亡时 freeze) */
  canMove: boolean;
  /** agent 等级 (per game state) */
  agentLevel?: number;
}

const CELL_SIZE = 88;        // 像素 (放大以容纳 64x64 character SVG)
const GAP = 4;               // cell 间距

export function RoguelikeCanvas({ map, position, agent, workItems, onMove, onReset, canMove, agentLevel = 1 }: RoguelikeCanvasProps) {
  const { colors, mode } = useAgentGameTheme();
  const [hovered, setHovered] = useState<{ x: number; y: number } | null>(null);
  const [selected, setSelected] = useState<{ x: number; y: number } | null>(null);

  // 当 position 变化时, 自动 focus 到新位置
  useEffect(() => {
    setSelected(position);
  }, [position.x, position.y]);

  // 计算邻居 (4-邻接) — 高亮
  const neighborCells = useMemo(() => {
    const out = new Set<string>();
    for (const [dx, dy] of [[0, 1], [0, -1], [1, 0], [-1, 0]] as const) {
      const x = position.x + dx;
      const y = position.y + dy;
      if (x >= 0 && y >= 0 && x < map.width && y < map.height) {
        const cell = map.cells[y]?.[x];
        if (cell && cell.type !== "trap") {
          out.add(`${x},${y}`);
        }
      }
    }
    return out;
  }, [map, position.x, position.y]);

  const handleCellClick = useCallback((cell: MapCell) => {
    if (!canMove) return;
    if (!isAdjacent(position, cell)) return;
    setSelected(cell);
    onMove({ x: cell.x, y: cell.y });
  }, [canMove, position, onMove]);

  // 计算 workItems by id (快速查)
  const workItemById = useMemo(
    () => new Map(workItems.map((w) => [w.id, w] as const)),
    [workItems],
  );

  // 渲染 1 个 cell (per 9/5 12:33 JST 拍板, 全新美术)
  const renderCell = (cell: MapCell) => {
    const isAgent = cell.x === position.x && cell.y === position.y;
    const isNeighbor = neighborCells.has(`${cell.x},${cell.y}`);
    const isHovered = hovered?.x === cell.x && hovered?.y === cell.y;
    const isSelected = selected?.x === cell.x && selected?.y === cell.y;

    // 背景色 (跟主题切换, per 9/5 23:13 JST 拍板)
    const bg = isAgent ? "transparent" : (() => {
      // 用 hex + opacity 模拟主题切换 (light 模式用 rgba 透明叠加, dark 用实色)
      const c = colors;
      if (cell.type === "boss") return mode === "dark" ? `${c.gold}26` : `${c.gold}40`;        // 金
      if (cell.type === "start") return mode === "dark" ? `${c.neonCyan}26` : `${c.neonCyan}33`;  // 霓虹青
      if (cell.type === "enemy") return mode === "dark" ? `${c.vermilion}1A` : `${c.vermilion}26`; // 朱红
      if (cell.type === "treasure") return mode === "dark" ? `${c.cyberPurple}1A` : `${c.cyberPurple}26`; // 紫
      if (cell.type === "trap") return mode === "dark" ? `${c.ash}33` : `${c.ash}40`;          // 灰
      return colors.inkDark;  // blank (主题切换: dark 暗, light 宣纸)
    })();
    const opacity = cell.type === "trap" ? 0.4 : 1;
    const borderColor = isSelected
      ? colors.goldGlow
      : isNeighbor
        ? colors.neonCyan
        : isHovered
          ? colors.neonCyanGlow
          : colors.inkLight;  // 主题切换
    const borderWidth = isSelected ? 3 : isNeighbor ? 2 : 1;
    const cellX = cell.x * (CELL_SIZE + GAP);
    const cellY = cell.y * (CELL_SIZE + GAP);
    const centerX = CELL_SIZE / 2;
    const centerY = CELL_SIZE / 2;

    return (
      <g
        key={`cell-${cell.x}-${cell.y}`}
        data-testid={`roguelike-cell-${cell.x}-${cell.y}`}
        data-type={cell.type}
        data-is-agent={isAgent}
        data-is-neighbor={isNeighbor}
        transform={`translate(${cellX}, ${cellY})`}
        onMouseEnter={() => setHovered({ x: cell.x, y: cell.y })}
        onMouseLeave={() => setHovered(null)}
        onClick={() => handleCellClick(cell)}
        style={{ cursor: isNeighbor && canMove ? "pointer" : "default" }}
      >
        {/* cell 背景 (墨黑风格) */}
        <rect
          width={CELL_SIZE}
          height={CELL_SIZE}
          fill={bg}
          stroke={borderColor}
          strokeWidth={borderWidth}
          rx={6}
          opacity={opacity}
        />
        {/* 邻居能量环 (4-邻接可走) */}
        {isNeighbor && canMove && (
          <EnergyRing cx={centerX} cy={centerY} color="#22d3ee" radius={CELL_SIZE * 0.4} />
        )}
        {/* cell 内容 (per type) */}
        {!isAgent && cell.type === "enemy" && (() => {
          const wi = cell.workItemId ? workItemById.get(cell.workItemId) : undefined;
          const def = enemyTypeForPriority(wi?.priority ?? "p2");
          return <EnemyOrbSVG type={def.key} scale={1} />;
        })()}
        {!isAgent && cell.type === "boss" && <BossOrbSVG />}
        {!isAgent && cell.type === "treasure" && (
          <text x={centerX} y={centerY + 4} textAnchor="middle" fontSize={32} style={{ pointerEvents: "none" }}>💎</text>
        )}
        {!isAgent && cell.type === "trap" && (
          <text x={centerX} y={centerY + 4} textAnchor="middle" fontSize={32} opacity={0.6} style={{ pointerEvents: "none" }}>💀</text>
        )}
        {!isAgent && cell.type === "start" && (
          <text x={centerX} y={centerY + 4} textAnchor="middle" fontSize={32} style={{ pointerEvents: "none" }}>🏯</text>
        )}
        {!isAgent && cell.type === "blank" && (
          <text x={centerX} y={centerY + 8} textAnchor="middle" fontSize={16} fill="#2a2a35" style={{ pointerEvents: "none" }}>·</text>
        )}
        {/* agent 角色 (per 拍板) */}
        {isAgent && (
          <>
            <AgentCharacterSVG level={agentLevel} scale={CELL_SIZE / 64} dead={!canMove} stampText="侠" showDivineHalo={agentLevel >= 7} />
            {agentLevel >= 5 && <Stamp text="M" cx={CELL_SIZE - 8} cy={10} color="#dc2626" size={14} />}
            {agentLevel >= 10 && <GodSeal level={agentLevel} cx={12} cy={12} size={16} />}
          </>
        )}
        {/* enemy 的 work-item key (上方) */}
        {cell.type === "enemy" && cell.workItemId && (
          <text
            x={centerX}
            y={14}
            textAnchor="middle"
            fontSize={9}
            fill={colors.paper}  // 主题切换: dark=白, light=墨
            fontFamily='"SF Mono", monospace'
            fontWeight="bold"
            style={{ pointerEvents: "none" }}
          >
            {workItemById.get(cell.workItemId)?.key ?? cell.workItemId}
          </text>
        )}
        {/* type label (下方) */}
        {!isAgent && cell.type !== "blank" && (
          <text
            x={centerX}
            y={CELL_SIZE - 6}
            textAnchor="middle"
            fontSize={9}
            fill={colors.ashLight}  // 主题切换
            fontFamily='"Hiragino Sans", system-ui, sans-serif'
            opacity={0.7}
            style={{ pointerEvents: "none" }}
          >
            {cell.type}
          </text>
        )}
        {/* agent 标识 */}
        {isAgent && (
          <text
            x={centerX}
            y={CELL_SIZE - 6}
            textAnchor="middle"
            fontSize={10}
            fill={colors.gold}  // 主题切换
            fontWeight="bold"
            fontFamily='"Hiragino Sans", system-ui, sans-serif'
            style={{ pointerEvents: "none" }}
          >
            YOU
          </text>
        )}
      </g>
    );
  };

  // hover tooltip 信息
  const hoveredCell = hovered ? map.cells[hovered.y]?.[hovered.x] : null;

  return (
    <div data-testid="roguelike-canvas-container" data-theme={mode} className="flex flex-col h-full" style={{ background: colors.inkBlack }}>
      {/* 顶部信息栏 */}
      <div className="flex items-center justify-between gap-3 p-3 border-b border-line bg-bg-soft/40">
        <div className="flex items-center gap-3 text-[10px] font-mono">
          <span className="flex items-center gap-1 text-info">
            <MapPin size={11} /> {position.x},{position.y}
          </span>
          <span className="text-ink-mute">
            seed <span className="text-info">{map.seed}</span>
          </span>
          <span className="text-ink-mute">grid {map.width}×{map.height}</span>
          <span className="text-warn">Lv {agentLevel}</span>
          <span className="text-ink-mute" data-testid="roguelike-theme-mode">[{mode}]</span>
        </div>
        <button
          data-testid="roguelike-reset-btn"
          onClick={onReset}
          className="btn text-[10px] py-1 px-2"
          title="重开一局 (新 map, 回到起点)"
        >
          <RefreshCw size={10} /> Reset Map
        </button>
      </div>

      {/* 画布 + 侧边信息 */}
      <div className="flex flex-1 min-h-0">
        {/* Canvas */}
        <div className="flex-1 flex items-center justify-center p-4 overflow-auto">
          <svg
            data-testid="roguelike-canvas-svg"
            width={map.width * (CELL_SIZE + GAP)}
            height={map.height * (CELL_SIZE + GAP)}
            viewBox={`0 0 ${map.width * (CELL_SIZE + GAP)} ${map.height * (CELL_SIZE + GAP)}`}
            style={{
              backgroundColor: colors.inkBlack,
              backgroundImage: `radial-gradient(circle, ${colors.inkLight} 1px, transparent 1px)`,
              backgroundSize: "20px 20px",
              borderRadius: 8,
            }}
          >
            {/* 画布背板 (主题切换) */}
            <rect width={map.width * (CELL_SIZE + GAP)} height={map.height * (CELL_SIZE + GAP)} fill="transparent" />
            {map.cells.flat().map(renderCell)}
          </svg>
        </div>

        {/* 右侧 info panel */}
        <div className="w-64 border-l border-line bg-bg-soft/40 p-3 text-xs overflow-y-auto">
          <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-2 font-mono">Agent</div>
          <div className="text-xs font-mono mb-1">{agent.id}</div>
          <StatusPill value={agent.status} size="xs" />
          <div className="mt-3 text-[10px] uppercase tracking-wider text-ink-mute mb-1 font-mono">Hovered</div>
          {hoveredCell ? (
            <div className="card p-2 bg-bg-soft">
              <div className="text-xs font-mono">
                {hoveredCell.x},{hoveredCell.y} · {hoveredCell.type}
              </div>
              {hoveredCell.workItemId && (
                <div className="text-[10px] text-ink-mute mt-1">
                  {workItemById.get(hoveredCell.workItemId)?.title ?? hoveredCell.workItemId}
                </div>
              )}
              {hoveredCell.description && (
                <div className="text-[10px] text-warn mt-1">{hoveredCell.description}</div>
              )}
            </div>
          ) : (
            <div className="text-[10px] text-ink-mute">悬停节点查看详情</div>
          )}

          <div className="mt-3 text-[10px] uppercase tracking-wider text-ink-mute mb-1 font-mono">图例 (日漫风)</div>
          <div className="text-[10px] space-y-0.5 text-ink-dim font-mono">
            <div>🏯 起点</div>
            <div>🤖 YOU · 当前 agent</div>
            <div>· 空地</div>
            <div>🔮 光球 · 敌人 (完成 wi)</div>
            <div>💎 宝箱</div>
            <div>💀 陷阱 (不可入)</div>
            <div>👁 神光球 · boss</div>
          </div>

          <div className="mt-3 text-[10px] text-ink-mute">
            {canMove
              ? "💡 点击 4-邻接节点移动 (每步 -2 HP)"
              : "⛔ 死亡中, 点击 [Reset Map] 重开一局"}
          </div>
        </div>
      </div>
    </div>
  );
}
