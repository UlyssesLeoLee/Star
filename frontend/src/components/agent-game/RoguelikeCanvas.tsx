"use client";

// =====================================================================
// RoguelikeCanvas — 网格画布 (4-邻接点击移动)
// =====================================================================
// Per 2026-09-05 12:23 JST 拍板 (ask_8a60a3bc90f779308a69be1d):
//   - 程序生成 grid 8x6 (default), 起点 (0,0) + 终点 (右下 boss)
//   - 4-邻接点击移动 (上下左右)
//   - 节点类型: start / enemy / treasure / trap / blank / boss
//   - trap 不可进入 (per 拍板 BFS 限制)
//   - agent 走到节点 = 触发 cell 效果 (战斗/buff/debuff)
//   - 死 = all agents freeze 等玩家重开
// =====================================================================

import { useState, useMemo, useCallback, useEffect } from "react";
import type { GameMap, MapCell } from "@/lib/agent-game/mapgen";
import { isAdjacent } from "@/lib/agent-game/mapgen";
import type { AgentSession, WorkItem } from "@/types/ids";
import { StatusPill } from "@/components/StatusPill";
import { Coins, Skull, Heart, Sword, Gem, MapPin, RotateCcw, RefreshCw, Eye } from "lucide-react";

interface RoguelikeCanvasProps {
  map: GameMap;
  position: { x: number; y: number };
  agent: AgentSession;
  workItems: ReadonlyArray<WorkItem>;
  onMove: (target: { x: number; y: number }) => void;
  onReset: () => void;
  /** 是否可移动 (死亡时 freeze) */
  canMove: boolean;
}

const CELL_SIZE = 72;        // 像素
const GAP = 4;               // cell 间距

export function RoguelikeCanvas({ map, position, agent, workItems, onMove, onReset, canMove }: RoguelikeCanvasProps) {
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
        // trap 不可进 (per 拍板 BFS), 不高亮
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

  // 渲染 1 个 cell
  const renderCell = (cell: MapCell) => {
    const isAgent = cell.x === position.x && cell.y === position.y;
    const isNeighbor = neighborCells.has(`${cell.x},${cell.y}`);
    const isHovered = hovered?.x === cell.x && hovered?.y === cell.y;
    const isSelected = selected?.x === cell.x && selected?.y === cell.y;

    // 背景色 by type
    const bg = (() => {
      if (isAgent) return "#2f81f7";
      if (cell.type === "boss") return "#d29922";
      if (cell.type === "start") return "#3fb950";
      if (cell.type === "enemy") return "#da3633";
      if (cell.type === "treasure") return "#a371f7";
      if (cell.type === "trap") return "#6e7681";
      return "#161b22";  // blank
    })();
    const opacity = cell.type === "trap" ? 0.5 : 1;
    const borderColor = isSelected ? "#f0b429" : isNeighbor ? "#2f81f7" : isHovered ? "#79c0ff" : "#30363d";
    const borderWidth = isSelected ? 3 : isNeighbor ? 2 : 1;
    const cellX = cell.x * (CELL_SIZE + GAP);
    const cellY = cell.y * (CELL_SIZE + GAP);

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
        <rect
          width={CELL_SIZE}
          height={CELL_SIZE}
          fill={bg}
          opacity={opacity}
          stroke={borderColor}
          strokeWidth={borderWidth}
          rx={6}
        />
        {/* emoji icon */}
        <text
          x={CELL_SIZE / 2}
          y={CELL_SIZE / 2 + 4}
          textAnchor="middle"
          fontSize={cell.type === "blank" ? 14 : 28}
          fill="#0b0d10"
          fontFamily="ui-monospace, monospace"
          style={{ pointerEvents: "none" }}
        >
          {isAgent ? "🤖" : cell.emoji}
        </text>
        {/* type label (small) */}
        {cell.type !== "blank" && !isAgent && (
          <text
            x={CELL_SIZE / 2}
            y={CELL_SIZE - 6}
            textAnchor="middle"
            fontSize={8}
            fill={cell.type === "trap" ? "#0b0d10" : "#0b0d10"}
            fontFamily="ui-monospace, monospace"
            opacity={0.7}
            style={{ pointerEvents: "none" }}
          >
            {cell.type}
          </text>
        )}
        {/* agent 标识 */}
        {isAgent && (
          <text
            x={CELL_SIZE / 2}
            y={CELL_SIZE - 6}
            textAnchor="middle"
            fontSize={9}
            fill="#0b0d10"
            fontWeight="bold"
            fontFamily="ui-monospace, monospace"
            style={{ pointerEvents: "none" }}
          >
            YOU
          </text>
        )}
        {/* enemy 的 work-item key (per 拍板, 关联真实数据) */}
        {cell.type === "enemy" && cell.workItemId && (
          <text
            x={CELL_SIZE / 2}
            y={14}
            textAnchor="middle"
            fontSize={8}
            fill="#0b0d10"
            fontFamily="ui-monospace, monospace"
            opacity={0.8}
            style={{ pointerEvents: "none" }}
          >
            {workItemById.get(cell.workItemId)?.key ?? cell.workItemId}
          </text>
        )}
      </g>
    );
  };

  // hover tooltip 信息
  const hoveredCell = hovered ? map.cells[hovered.y]?.[hovered.x] : null;

  return (
    <div data-testid="roguelike-canvas-container" className="flex flex-col h-full">
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
        <div className="flex-1 flex items-center justify-center p-4 overflow-auto bg-bg">
          <svg
            data-testid="roguelike-canvas-svg"
            width={map.width * (CELL_SIZE + GAP)}
            height={map.height * (CELL_SIZE + GAP)}
            viewBox={`0 0 ${map.width * (CELL_SIZE + GAP)} ${map.height * (CELL_SIZE + GAP)}`}
            style={{ backgroundColor: "#0b0d10", borderRadius: 8 }}
          >
            {map.cells.flat().map(renderCell)}
          </svg>
        </div>

        {/* 右侧 info panel */}
        <div className="w-64 border-l border-line bg-bg-soft/40 p-3 text-xs overflow-y-auto">
          <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-2">Agent</div>
          <div className="text-xs font-mono mb-1">{agent.id}</div>
          <StatusPill value={agent.status} size="xs" />
          <div className="mt-3 text-[10px] uppercase tracking-wider text-ink-mute mb-1">Hovered</div>
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

          <div className="mt-3 text-[10px] uppercase tracking-wider text-ink-mute mb-1">图例</div>
          <div className="text-[10px] space-y-0.5 text-ink-dim font-mono">
            <div>🏠 start · 起点</div>
            <div>🤖 you · 当前 agent</div>
            <div>· · blank · 空地</div>
            <div>⚔️ enemy · 敌人 (完成 wi)</div>
            <div>💎 treasure · 宝箱</div>
            <div>💀 trap · 陷阱 (不可入)</div>
            <div>👑 boss · 终点</div>
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
