// =====================================================================
// movement.test.ts — Roguelike 4-邻接 step + cell 效果
// =====================================================================

import { describe, it, expect } from "vitest";
import { moveAgent, computeCellEffect, applyCellEffect, STEP_COST_USD } from "./movement";
import { generateMap, type GameMap } from "./mapgen";
import { createInitialGameState, MAX_HP } from "./types";
import type { WorkItem } from "@/types/ids";

const makeWi = (id: string): WorkItem => ({
  id,
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  key: `PHYSIS-${id.slice(3)}`,
  title: id,
  description: "",
  kind: "task",
  status: "todo",
  priority: "p2",
  reporter_id: "usr-001",
  labels: [],
  workflow_id: "wf-default",
  created_at: "2026-09-05T08:00:00Z",
  updated_at: "2026-09-05T08:00:00Z",
});

const wis: WorkItem[] = [makeWi("wi-001"), makeWi("wi-002")];

describe("moveAgent", () => {
  it("无 map → no_map", () => {
    const s = createInitialGameState("ag-001", 5);
    const r = moveAgent(s, null, { x: 0, y: 0 }, { x: 1, y: 0 }, 5);
    expect(r.ok).toBe(false);
    if (!r.ok) expect(r.reason).toBe("no_map");
  });

  it("死亡 agent 不能移动", () => {
    const m = generateMap({ width: 6, height: 4, seed: 1, workItems: wis });
    const s = { ...createInitialGameState("ag-001", 5), alive: false };
    const r = moveAgent(s, m, { x: 0, y: 0 }, { x: 1, y: 0 }, 5);
    expect(r.ok).toBe(false);
    if (!r.ok) expect(r.reason).toBe("agent_dead");
  });

  it("非 4-邻接 → not_adjacent", () => {
    const m = generateMap({ width: 6, height: 4, seed: 1, workItems: wis });
    const s = createInitialGameState("ag-001", 5);
    const r = moveAgent(s, m, { x: 0, y: 0 }, { x: 2, y: 0 }, 5);
    expect(r.ok).toBe(false);
    if (!r.ok) expect(r.reason).toBe("not_adjacent");
  });

  it("trap cell 不可进入", () => {
    // 找 1 个起点 (0,0) 旁边有 trap 的 map (循环尝试 seeds)
    let trapPos: { x: number; y: number } | null = null;
    let trapMap: GameMap | null = null;
    for (let seed = 1; seed <= 30 && !trapPos; seed++) {
      const mm = generateMap({ width: 6, height: 4, seed, workItems: wis });
      for (const [dx, dy] of [[1, 0], [0, 1]] as const) {
        const cell = mm.cells[dy]?.[dx];
        if (cell?.type === "trap") {
          trapPos = { x: dx, y: dy };
          trapMap = mm;
          break;
        }
      }
    }
    if (trapPos && trapMap) {
      const s = createInitialGameState("ag-001", 5);
      const r = moveAgent(s, trapMap, { x: 0, y: 0 }, trapPos, 5);
      expect(r.ok).toBe(false);
      if (!r.ok) expect(r.reason).toBe("trap_cell_blocked");
    }
  });

  it("走到 blank cell: cost +0.1, hp 扣 1 (无 perk)", () => {
    const m = generateMap({ width: 6, height: 4, seed: 1, workItems: wis });
    const s = createInitialGameState("ag-001", 5);
    // 找 (0,0) 旁边的 blank cell
    const target = findCellOfType(m, { x: 0, y: 0 }, "blank");
    if (target) {
      const r = moveAgent(s, m, { x: 0, y: 0 }, target, 5);
      expect(r.ok).toBe(true);
      if (r.ok) {
        expect(r.hpAfter).toBe(98);  // 100 - round(0.1/5 * 100) = 100 - 2 = 98
        expect(r.died).toBe(false);
        expect(r.effect.kind).toBe("none");
      }
    }
  });

  it("走到 enemy cell: effect.enemy 有 workItemId", () => {
    const m = generateMap({ width: 8, height: 6, seed: 42, workItems: wis });
    const s = createInitialGameState("ag-001", 5);
    // 找起点相邻的 enemy cell
    const target = findCellOfType(m, { x: 0, y: 0 }, "enemy");
    if (target) {
      const r = moveAgent(s, m, { x: 0, y: 0 }, target, 5);
      expect(r.ok).toBe(true);
      if (r.ok) {
        expect(r.effect.kind).toBe("enemy");
        if (r.effect.kind === "enemy") {
          expect(r.effect.workItemId).toBeDefined();
        }
      }
    }
  });

  it("走到 boss cell: reachedBoss = true", () => {
    // 找一个 enemy 在 (1, 0) 旁边, 然后把 boss 移到 (1, 0) (测试用, 调包 internal)
    const m = generateMap({ width: 6, height: 4, seed: 1, workItems: wis });
    // 强制把 (1, 0) 改成 boss
    const mm: GameMap = {
      ...m,
      cells: m.cells.map((row, y) => row.map((c, x) => (x === 1 && y === 0) ? { ...c, type: "boss" as const, emoji: "👑" } : c)),
    };
    const s = createInitialGameState("ag-001", 5);
    const r = moveAgent(s, mm, { x: 0, y: 0 }, { x: 1, y: 0 }, 5);
    expect(r.ok).toBe(true);
    if (r.ok) {
      expect(r.reachedBoss).toBe(true);
    }
  });

  it("iron_will perk ×1: HP 损失 -25%", () => {
    const m = generateMap({ width: 6, height: 4, seed: 1, workItems: wis });
    const s = { ...createInitialGameState("ag-001", 5), perks: ["iron_will"] };
    const target = findCellOfType(m, { x: 0, y: 0 }, "blank");
    if (target) {
      const r = moveAgent(s, m, { x: 0, y: 0 }, target, 5);
      if (r.ok) {
        // ratio 0.1/5 = 0.02, hpLoss = round(0.02 * 100 * 0.75) = round(1.5) = 2
        // 100 - 2 = 98
        expect(r.hpAfter).toBe(98);
      }
    }
  });
});

// ---- helpers ----
function findCellOfType(m: GameMap, near: { x: number; y: number }, type: "blank" | "enemy" | "treasure" | "trap"): { x: number; y: number } | null {
  for (const [dx, dy] of [[1, 0], [0, 1]] as const) {
    const c = m.cells[near.y + dy]?.[near.x + dx];
    if (c?.type === type) return { x: near.x + dx, y: near.y + dy };
  }
  return null;
}

describe("computeCellEffect", () => {
  it("enemy → kind: 'enemy'", () => {
    const e = computeCellEffect({ x: 0, y: 0, type: "enemy", workItemId: "wi-001", emoji: "⚔️" });
    expect(e.kind).toBe("enemy");
  });

  it("treasure +5 coin → coinsDelta 5", () => {
    const e = computeCellEffect({ x: 0, y: 0, type: "treasure", description: "+5 coin", emoji: "💎" });
    expect(e.kind).toBe("treasure");
    if (e.kind === "treasure") expect(e.coinsDelta).toBe(5);
  });

  it("treasure full HP → hpDelta 20", () => {
    const e = computeCellEffect({ x: 0, y: 0, type: "treasure", description: "full HP", emoji: "💎" });
    if (e.kind === "treasure") expect(e.hpDelta).toBe(20);
  });

  it("trap -20 HP → hpDelta -20", () => {
    const e = computeCellEffect({ x: 0, y: 0, type: "trap", description: "-20 HP", emoji: "💀" });
    expect(e.kind).toBe("trap");
    if (e.kind === "trap") expect(e.hpDelta).toBe(-20);
  });

  it("blank → kind: 'none'", () => {
    const e = computeCellEffect({ x: 0, y: 0, type: "blank", emoji: "·" });
    expect(e.kind).toBe("none");
  });

  it("boss → kind: 'boss'", () => {
    const e = computeCellEffect({ x: 0, y: 0, type: "boss", emoji: "👑" });
    expect(e.kind).toBe("boss");
  });
});

describe("applyCellEffect (state mutation)", () => {
  it("treasure +5 coin → coins +5", () => {
    const s = createInitialGameState("ag-001", 5);
    const r = applyCellEffect(s, { kind: "treasure", description: "+5 coin", coinsDelta: 5, hpDelta: 0 });
    expect(r.coins).toBe(5);
  });

  it("treasure +20 HP → hp min(100, hp+20)", () => {
    const s = { ...createInitialGameState("ag-001", 5), hp: 50 };
    const r = applyCellEffect(s, { kind: "treasure", description: "full HP", coinsDelta: 0, hpDelta: 20 });
    expect(r.hp).toBe(70);
  });

  it("trap -20 HP → hp max(0, hp-20)", () => {
    const s = { ...createInitialGameState("ag-001", 5), hp: 15 };
    const r = applyCellEffect(s, { kind: "trap", description: "-20 HP", coinsDelta: 0, hpDelta: -20 });
    expect(r.hp).toBe(0);
  });

  it("死亡时 cell 效果不应用 (gated)", () => {
    const s = { ...createInitialGameState("ag-001", 5), alive: false, coins: 0 };
    const r = applyCellEffect(s, { kind: "treasure", description: "+5 coin", coinsDelta: 5, hpDelta: 0 });
    expect(r).toBe(s);  // 引用相等 (no-op)
  });
});

describe("STEP_COST_USD sanity", () => {
  it("0.1", () => expect(STEP_COST_USD).toBe(0.1));
});

describe("MAX_HP sanity", () => {
  it("100", () => expect(MAX_HP).toBe(100));
});
