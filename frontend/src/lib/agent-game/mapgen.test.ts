// =====================================================================
// mapgen.test.ts — 程序生成 + 4-邻接 + BFS 路径
// =====================================================================

import { describe, it, expect } from "vitest";
import { generateMap, isAdjacent, getCell, getNeighbors, type GameMap } from "./mapgen";
import type { WorkItem } from "@/types/ids";

const makeWi = (id: string, status: WorkItem["status"] = "todo"): WorkItem => ({
  id,
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  key: `PHYSIS-${id.slice(3)}`,
  title: id,
  description: "",
  kind: "task",
  status,
  priority: "p2",
  reporter_id: "usr-001",
  labels: [],
  workflow_id: "wf-default",
  created_at: "2026-09-05T08:00:00Z",
  updated_at: "2026-09-05T08:00:00Z",
});

const wis: WorkItem[] = [
  makeWi("wi-001"),
  makeWi("wi-002"),
  makeWi("wi-003"),
  makeWi("wi-004"),
  makeWi("wi-005"),
];

describe("generateMap", () => {
  it("8x6 map with seed=42", () => {
    const m = generateMap({ width: 8, height: 6, seed: 42, workItems: wis });
    expect(m.width).toBe(8);
    expect(m.height).toBe(6);
    expect(m.cells).toHaveLength(6);
    expect(m.cells[0]).toHaveLength(8);
    expect(m.startPos).toEqual({ x: 0, y: 0 });
    expect(m.bossPos).toEqual({ x: 7, y: 5 });
  });

  it("起点 = start, 终点 = boss", () => {
    const m = generateMap({ width: 8, height: 6, seed: 42, workItems: wis });
    expect(m.cells[0][0].type).toBe("start");
    expect(m.cells[5][7].type).toBe("boss");
  });

  it("起点到 boss 一定有路径 (BFS 校验)", () => {
    for (let seed = 1; seed <= 50; seed++) {
      const m = generateMap({ width: 8, height: 6, seed, workItems: wis });
      // BFS 简单校验: 至少存在一条路径 (不绕 trap)
      let visited = new Set<string>();
      const queue: Array<{ x: number; y: number }> = [m.startPos];
      visited.add(`${m.startPos.x},${m.startPos.y}`);
      let reached = false;
      while (queue.length > 0) {
        const cur = queue.shift()!;
        if (cur.x === m.bossPos.x && cur.y === m.bossPos.y) {
          reached = true;
          break;
        }
        for (const [dx, dy] of [[0, 1], [0, -1], [1, 0], [-1, 0]] as const) {
          const nx = cur.x + dx;
          const ny = cur.y + dy;
          if (nx < 0 || ny < 0 || nx >= m.width || ny >= m.height) continue;
          const cell = m.cells[ny][nx];
          if (cell.type === "trap") continue;
          const k = `${nx},${ny}`;
          if (visited.has(k)) continue;
          visited.add(k);
          queue.push({ x: nx, y: ny });
        }
      }
      expect(reached).toBe(true);
    }
  });

  it("同样 seed 同样 map (deterministic)", () => {
    const a = generateMap({ width: 8, height: 6, seed: 42, workItems: wis });
    const b = generateMap({ width: 8, height: 6, seed: 42, workItems: wis });
    expect(JSON.stringify(a.cells)).toBe(JSON.stringify(b.cells));
  });

  it("不同 seed 产出不同 map", () => {
    const a = generateMap({ width: 8, height: 6, seed: 1, workItems: wis });
    const b = generateMap({ width: 8, height: 6, seed: 2, workItems: wis });
    expect(JSON.stringify(a.cells)).not.toBe(JSON.stringify(b.cells));
  });

  it("enemy cell 填入 workItemId", () => {
    let foundEnemy = false;
    for (let seed = 1; seed <= 30 && !foundEnemy; seed++) {
      const m = generateMap({ width: 8, height: 6, seed, workItems: wis });
      for (const row of m.cells) {
        for (const cell of row) {
          if (cell.type === "enemy" && cell.workItemId) {
            expect(wis.find((w) => w.id === cell.workItemId)).toBeTruthy();
            foundEnemy = true;
            break;
          }
        }
        if (foundEnemy) break;
      }
    }
    expect(foundEnemy).toBe(true);  // 至少 1 个 enemy cell
  });

  it("小 map 抛错 (< 4x4)", () => {
    expect(() => generateMap({ width: 3, height: 3, seed: 1, workItems: wis })).toThrow();
  });

  it("无 workItem 也能生成 (enemy cell workItemId = undefined)", () => {
    const m = generateMap({ width: 8, height: 6, seed: 42, workItems: [] });
    for (const row of m.cells) {
      for (const cell of row) {
        if (cell.type === "enemy") {
          expect(cell.workItemId).toBeUndefined();
        }
      }
    }
  });
});

describe("isAdjacent", () => {
  it("4-邻接 (上下左右) 算", () => {
    expect(isAdjacent({ x: 0, y: 0 }, { x: 1, y: 0 })).toBe(true);
    expect(isAdjacent({ x: 0, y: 0 }, { x: 0, y: 1 })).toBe(true);
    expect(isAdjacent({ x: 0, y: 0 }, { x: -1, y: 0 })).toBe(true);
    expect(isAdjacent({ x: 0, y: 0 }, { x: 0, y: -1 })).toBe(true);
  });

  it("对角线不算", () => {
    expect(isAdjacent({ x: 0, y: 0 }, { x: 1, y: 1 })).toBe(false);
  });

  it("自身不算", () => {
    expect(isAdjacent({ x: 0, y: 0 }, { x: 0, y: 0 })).toBe(false);
  });

  it("距离 2 算远邻", () => {
    expect(isAdjacent({ x: 0, y: 0 }, { x: 2, y: 0 })).toBe(false);
  });
});

describe("getCell / getNeighbors", () => {
  const map: GameMap = generateMap({ width: 6, height: 4, seed: 7, workItems: wis });
  it("getCell 越界返回 null", () => {
    expect(getCell(map, -1, 0)).toBeNull();
    expect(getCell(map, 0, -1)).toBeNull();
    expect(getCell(map, map.width, 0)).toBeNull();
    expect(getCell(map, 0, map.height)).toBeNull();
  });

  it("getNeighbors 起点 (0,0) 返回 2 个 (右边 + 下边)", () => {
    const ns = getNeighbors(map, { x: 0, y: 0 });
    expect(ns).toHaveLength(2);
    expect(ns.map((c) => `${c.x},${c.y}`).sort()).toEqual(["0,1", "1,0"]);
  });

  it("getNeighbors 中心 cell 返回 4 个", () => {
    const ns = getNeighbors(map, { x: 2, y: 1 });
    expect(ns).toHaveLength(4);
  });
});
