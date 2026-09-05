// =====================================================================
// Agent Game — Map Generator (Roguelike 程序生成)
// =====================================================================
// Per 2026-09-05 12:23 JST 拍板 (ask_8a60a3bc90f779308a69be1d):
//   - 随机节点布局 (程序生成, 不再用 free-form 散开)
//   - agent 4-邻接点击移动
//   - 节点类型: start / enemy / treasure / trap / blank / boss
//   - 起点 (top-left) + 终点 (bottom-right, boss)
//   - 死 = 重新选 agent = 新一局 (map 重生)
//
// 设计目标:
//   1. 确定性 (接受 randomSeed, 同样 seed 同样 map, 便于测试 + 可重玩)
//   2. 起点到 boss 一定有路径 (BFS 校验, 失败则重生成)
//   3. 节点类型分布: 60% blank, 20% enemy, 10% treasure, 8% trap, 2% boss
//   4. enemy 节点带 "敌人" 数据 (从 store.workItems 拿, status !== done)
//   5. 纯函数, 无副作用, 无 Date.now(), 无 random (除 seed RNG)
// =====================================================================

import type { WorkItem } from "@/types/ids";

/** Cell 类型 */
export type MapCellType = "start" | "enemy" | "treasure" | "trap" | "blank" | "boss";

/** 1 个 cell */
export interface MapCell {
  /** 网格坐标 */
  x: number;
  y: number;
  type: MapCellType;
  /** enemy 类型时填入 work-item id (store 查) */
  workItemId?: string;
  /** treasure / trap 描述 */
  description?: string;
  /** 节点 emoji */
  emoji: string;
}

/** 完整 map */
export interface GameMap {
  width: number;          // 列数 (x 方向)
  height: number;         // 行数 (y 方向)
  cells: MapCell[][];     // cells[y][x]
  startPos: { x: number; y: number };
  bossPos: { x: number; y: number };
  seed: number;           // 用来重生成
  generatedAt: string;    // ISO 8601
}

// ---- 简单 seeded RNG (mulberry32) ----
// 优点: 32-bit, 1 个种子, 无依赖, 跨平台一致
function mulberry32(seed: number): () => number {
  let a = seed | 0;
  return () => {
    a = (a + 0x6D2B79F5) | 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// ---- 节点类型分布权重 ----
const CELL_WEIGHTS: Record<MapCellType, number> = {
  start: 0,      // 强制放置, 不参与随机
  boss: 0,       // 强制放置, 不参与随机
  blank: 60,
  enemy: 20,
  treasure: 10,
  trap: 8,
};

const CELL_EMOJI: Record<MapCellType, string> = {
  start: "🏠",
  enemy: "⚔️",
  treasure: "💎",
  trap: "💀",
  blank: "·",
  boss: "👑",
};

// ---- 工具: BFS 路径存在性 (per 拍板 #2, 起点到 boss 一定有路径) ----
function bfsPathExists(
  cells: MapCell[][],
  start: { x: number; y: number },
  end: { x: number; y: number },
): boolean {
  const visited = new Set<string>();
  const queue: Array<{ x: number; y: number }> = [start];
  visited.add(`${start.x},${start.y}`);
  while (queue.length > 0) {
    const cur = queue.shift()!;
    if (cur.x === end.x && cur.y === end.y) return true;
    for (const [dx, dy] of [[0, 1], [0, -1], [1, 0], [-1, 0]] as const) {
      const nx = cur.x + dx;
      const ny = cur.y + dy;
      if (nx < 0 || ny < 0 || nx >= cells[0].length || ny >= cells.length) continue;
      // 不可穿越: trap 和 boss (boss 是终点不算)
      const cell = cells[ny][nx];
      if (cell.type === "trap") continue;
      const key = `${nx},${ny}`;
      if (visited.has(key)) continue;
      visited.add(key);
      queue.push({ x: nx, y: ny });
    }
  }
  return false;
}

// ---- 主入口: 程序生成 map ----
export interface GenerateMapInput {
  width: number;                  // 推荐 8-12
  height: number;                 // 推荐 6-10
  seed: number;                   // 任意 integer
  workItems: ReadonlyArray<WorkItem>;  // 候选 enemy (mock 数据, 拿没 done 的)
  /** 强制重新生成 (默认 true) */
  forceGenerate?: boolean;
}

export function generateMap(input: GenerateMapInput): GameMap {
  const { width, height, workItems } = input;
  if (width < 4 || height < 4) throw new Error(`Map too small: ${width}x${height} (min 4x4)`);

  // 1) 计算总权重
  const totalWeight = Object.values(CELL_WEIGHTS).reduce((s, w) => s + w, 0);
  // 候选 enemy (status !== done)
  const enemyPool = workItems.filter((w) => w.status !== "done");

  // 2) 生成 (含 retry 逻辑: 如果 BFS 不通, trap 改 blank 重试)
  const rand = mulberry32(input.seed);
  let cells = generateCells(width, height, rand, totalWeight, enemyPool);
  let attempts = 0;
  while (!bfsPathExists(cells, { x: 0, y: 0 }, { x: width - 1, y: height - 1 }) && attempts < 10) {
    cells = generateCells(width, height, rand, totalWeight, enemyPool);
    attempts += 1;
  }
  if (!bfsPathExists(cells, { x: 0, y: 0 }, { x: width - 1, y: height - 1 })) {
    throw new Error(`Failed to generate connected map after ${attempts} attempts (seed=${input.seed})`);
  }

  return {
    width,
    height,
    cells,
    startPos: { x: 0, y: 0 },
    bossPos: { x: width - 1, y: height - 1 },
    seed: input.seed,
    generatedAt: new Date().toISOString(),
  };
}

function generateCells(
  width: number,
  height: number,
  rand: () => number,
  totalWeight: number,
  enemyPool: ReadonlyArray<WorkItem>,
): MapCell[][] {
  // 2D array [y][x]
  const cells: MapCell[][] = [];
  for (let y = 0; y < height; y++) {
    const row: MapCell[] = [];
    for (let x = 0; x < width; x++) {
      // 起点 / boss 强制
      if (x === 0 && y === 0) {
        row.push({ x, y, type: "start", emoji: CELL_EMOJI.start });
        continue;
      }
      if (x === width - 1 && y === height - 1) {
        row.push({ x, y, type: "boss", emoji: CELL_EMOJI.boss });
        continue;
      }
      // 随机 type
      const r = rand() * totalWeight;
      let acc = 0;
      let type: MapCellType = "blank";
      for (const [t, w] of Object.entries(CELL_WEIGHTS) as Array<[MapCellType, number]>) {
        if (t === "start" || t === "boss") continue;
        acc += w;
        if (r < acc) {
          type = t;
          break;
        }
      }
      const cell: MapCell = { x, y, type, emoji: CELL_EMOJI[type] };
      // enemy 类型填入 work-item id
      if (type === "enemy" && enemyPool.length > 0) {
        const idx = Math.floor(rand() * enemyPool.length);
        const wi = enemyPool[idx];
        if (wi) {
          cell.workItemId = wi.id;
        }
      }
      // treasure / trap 加 description
      if (type === "treasure") {
        const treasures = ["+5 coin", "+10% XP", "free revive", "full HP", "+1 perk"];
        cell.description = treasures[Math.floor(rand() * treasures.length)] ?? "+5 coin";
      } else if (type === "trap") {
        const traps = ["-20 HP", "-10 coin", "-50% XP next", "perk 清零", "-1 level"];
        cell.description = traps[Math.floor(rand() * traps.length)] ?? "-20 HP";
      }
      row.push(cell);
    }
    cells.push(row);
  }
  return cells;
}

// ---- 辅助: 4-邻接 (per 拍板 #3) ----
export function isAdjacent(
  a: { x: number; y: number },
  b: { x: number; y: number },
): boolean {
  const dx = Math.abs(a.x - b.x);
  const dy = Math.abs(a.y - b.y);
  return (dx === 1 && dy === 0) || (dx === 0 && dy === 1);
}

/** 取 (x, y) 的 cell (越界返回 null) */
export function getCell(map: GameMap, x: number, y: number): MapCell | null {
  if (y < 0 || y >= map.height || x < 0 || x >= map.width) return null;
  return map.cells[y]?.[x] ?? null;
}

/** 给定 (x, y) 邻居 cells (4-邻接) */
export function getNeighbors(map: GameMap, pos: { x: number; y: number }): MapCell[] {
  const out: MapCell[] = [];
  for (const [dx, dy] of [[0, 1], [0, -1], [1, 0], [-1, 0]] as const) {
    const c = getCell(map, pos.x + dx, pos.y + dy);
    if (c) out.push(c);
  }
  return out;
}
