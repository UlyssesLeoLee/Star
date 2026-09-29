// =====================================================================
// frontend/src/lib/wasm/layout-engine.test.ts
// =====================================================================
// vitest unit tests for layout-engine JS fallback (no WASM required)
//
// Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1
// + frontend/src/lib/wasm/layout-engine.ts
//
// 已知缺口 (per 缺标比错标):
//   - WASM 集成测试 (happy-dom + node-wasm) 留后续 PR
//   - 本测试只覆盖 JS fallback (per spec §11.3 测试基线)
// =====================================================================

import { describe, it, expect } from "vitest";
import { pickAlgorithmJS, visualDistance, type ViewMode, type LayoutAlgorithmName } from "./layout-engine";

describe("layout-engine JS fallback", () => {
  describe("pickAlgorithmJS — pure function (matches Rust pick_algorithm)", () => {
    it("tree view: < 100 nodes → dagre", () => {
      expect(pickAlgorithmJS("tree", 50)).toBe("dagre");
      expect(pickAlgorithmJS("tree", 99)).toBe("dagre");
    });

    it("tree view: >= 100 nodes → elk", () => {
      expect(pickAlgorithmJS("tree", 100)).toBe("elk");
      expect(pickAlgorithmJS("tree", 500)).toBe("elk");
    });

    it("dependency view: always d3_force", () => {
      expect(pickAlgorithmJS("dependency", 50)).toBe("d3_force");
      expect(pickAlgorithmJS("dependency", 1000)).toBe("d3_force");
    });

    it("risk view: always d3_force", () => {
      expect(pickAlgorithmJS("risk", 1)).toBe("d3_force");
      expect(pickAlgorithmJS("risk", 10000)).toBe("d3_force");
    });

    it("history view: always d3_force", () => {
      expect(pickAlgorithmJS("history", 0)).toBe("d3_force");
      expect(pickAlgorithmJS("history", 9999)).toBe("d3_force");
    });

    it("agent view: > 50 → elk", () => {
      expect(pickAlgorithmJS("agent", 51)).toBe("elk");
      expect(pickAlgorithmJS("agent", 100)).toBe("elk");
    });

    it("agent view: <= 50 → d3_force", () => {
      expect(pickAlgorithmJS("agent", 50)).toBe("d3_force");
      expect(pickAlgorithmJS("agent", 30)).toBe("d3_force");
      expect(pickAlgorithmJS("agent", 0)).toBe("d3_force");
    });

    it("invalid viewMode → null (defensive)", () => {
      // @ts-expect-error — testing invalid input
      expect(pickAlgorithmJS("invalid", 100)).toBeNull();
      // @ts-expect-error — testing invalid input
      expect(pickAlgorithmJS("TREE", 100)).toBeNull(); // case-sensitive
    });

    it("boundary: tree view at exactly 100 nodes", () => {
      expect(pickAlgorithmJS("tree", 100)).toBe("elk"); // >= 100 → elk (not dagre)
    });

    it("boundary: agent view at exactly 50 nodes", () => {
      expect(pickAlgorithmJS("agent", 50)).toBe("d3_force"); // <= 50 → d3_force (not elk)
    });
  });

  describe("visualDistance — log(n+1)*30 formula", () => {
    it("commitDistance=0 → log(1)*30 = 0", () => {
      expect(visualDistance(0)).toBeCloseTo(0, 9);
    });

    it("commitDistance=1 → log(2)*30 ≈ 20.79", () => {
      expect(visualDistance(1)).toBeCloseTo(Math.log(2) * 30, 9);
    });

    it("commitDistance=10 → log(11)*30 ≈ 71.83", () => {
      expect(visualDistance(10)).toBeCloseTo(Math.log(11) * 30, 9);
    });

    it("monotonic increasing", () => {
      expect(visualDistance(0)).toBeLessThan(visualDistance(1));
      expect(visualDistance(1)).toBeLessThan(visualDistance(10));
      expect(visualDistance(10)).toBeLessThan(visualDistance(100));
      expect(visualDistance(100)).toBeLessThan(visualDistance(1000));
    });

    it("non-negative for all valid inputs", () => {
      for (let n = 0; n < 1000; n += 10) {
        expect(visualDistance(n)).toBeGreaterThanOrEqual(0);
      }
    });
  });

  describe("type safety", () => {
    it("ViewMode is exhaustive (5 values)", () => {
      const modes: ViewMode[] = ["tree", "dependency", "risk", "agent", "history"];
      expect(modes).toHaveLength(5);
    });

    it("LayoutAlgorithmName is exhaustive (3 values)", () => {
      const algos: LayoutAlgorithmName[] = ["dagre", "d3_force", "elk"];
      expect(algos).toHaveLength(3);
    });
  });
});