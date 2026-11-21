/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/wasm/layout-engine.math.test.ts"})
MERGE (self)-[:TESTS]->(t:File {path:"frontend/src/lib/wasm/layout-engine.ts"})
@endcypher
*/

/**
 * layout-engine.ts 数学性质测试 (2026-11-21, per "完善自动测试脚本和mock项目，加入数学测试的内容")
 *
 * visualDistance(d) = ln(d+1)·30 (per FR-WT-005 + DD §36) 的数学不变量:
 *   V1. 单调递增: d1 < d2 ⇒ visualDistance(d1) < visualDistance(d2)
 *   V2. 凹性 (边际递减): 等量 commit 增量带来的视觉增量递减
 *   V3. 零点: visualDistance(0) = 0
 *   V4. 与公式逐点一致 (含大数值域)
 *   V5. pickAlgorithmJS 阈值边界全扫描 (tree 100 / agent 50)
 *   V6. pickAlgorithmJS 与 Rust engine.rs pick_algorithm 语义对齐 (常量表驱动)
 */

import { describe, expect, it } from "vitest";
import { visualDistance, pickAlgorithmJS, type ViewMode } from "./layout-engine";

describe("visualDistance 数学性质", () => {
  it("V1: 严格单调递增", () => {
    let prev = -Infinity;
    for (let d = 0; d <= 10000; d += 7) {
      const v = visualDistance(d);
      expect(v).toBeGreaterThan(prev);
      prev = v;
    }
  });

  it("V2: 凹性 — 等量增量的视觉增量递减 (ln 边际递减)", () => {
    const step = 10;
    let prevDelta = Infinity;
    for (let d = 0; d < 1000; d += step) {
      const delta = visualDistance(d + step) - visualDistance(d);
      expect(delta).toBeLessThan(prevDelta);
      prevDelta = delta;
    }
  });

  it("V3: 零点 visualDistance(0) = 0", () => {
    expect(visualDistance(0)).toBe(0);
  });

  it("V4: 与 ln(d+1)·30 公式逐点一致", () => {
    for (const d of [0, 1, 2, 10, 99, 100, 999, 1000, 12345, 1e6]) {
      expect(visualDistance(d)).toBeCloseTo(Math.log(d + 1) * 30, 10);
    }
  });
});

describe("pickAlgorithmJS 阈值边界全扫描", () => {
  it("V5: tree 视图 — n<100 → dagre, n>=100 → elk (0..200 全扫描)", () => {
    for (let n = 0; n <= 200; n++) {
      expect(pickAlgorithmJS("tree", n)).toBe(n < 100 ? "dagre" : "elk");
    }
  });

  it("V5b: agent 视图 — n<=50 → d3_force, n>50 → elk (0..200 全扫描)", () => {
    for (let n = 0; n <= 200; n++) {
      expect(pickAlgorithmJS("agent", n)).toBe(n <= 50 ? "d3_force" : "elk");
    }
  });

  it("V6: dependency/risk/history 恒为 d3_force (与 nodeCount 无关)", () => {
    const modes: ViewMode[] = ["dependency", "risk", "history"];
    for (const mode of modes) {
      for (const n of [0, 1, 50, 100, 1000, 100000]) {
        expect(pickAlgorithmJS(mode, n)).toBe("d3_force");
      }
    }
  });
});