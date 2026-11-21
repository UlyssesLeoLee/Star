/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/canvas/coords.math.test.ts"})
MERGE (self)-[:TESTS]->(t:File {path:"frontend/src/lib/canvas/coords.ts"})
@endcypher
*/

/**
 * coords.ts 数学性质测试 (2026-11-21, per "完善自动测试脚本和mock项目，加入数学测试的内容")
 *
 * 与 coords.test.ts (示例驱动) 互补: 本文件用确定性伪随机 (mulberry32) 做
 * 性质测试 (property-based), 覆盖:
 *   P1. clientToWorld ∘ worldToClient 往返恒等 (任意 rect/viewBox/viewport)
 *   P2. clientDeltaToWorld = clientToWorld 差值 (letterbox 偏移消去)
 *   P3. svgToWorld ∘ worldToSvg 往返恒等
 *   P4. 平移等变性: rect.left/top 平移 ⇒ 世界坐标不变 (client 同步平移)
 *   P5. zoom 缩放齐次性: zoom×k ⇒ 相对 viewport 的世界偏移 ÷k
 *   P6. letterbox 几何不变量: 内容区居中、不超出 rect、等比
 *   P7. 退化输入守卫: rect 宽高 ≤0 / zoom ≤0 不产生 NaN/Infinity
 *
 * 不引入 fast-check 依赖 (repo 未装), 用固定种子 PRNG 保证可复现。
 */

import { describe, expect, it } from "vitest";
import {
  clientDeltaToWorld,
  clientToWorld,
  letterboxTransform,
  svgToWorld,
  worldToSvg,
  type RectLike,
  type ViewBoxLike,
  type ViewportLike,
} from "./coords";

/** mulberry32 — 确定性 32-bit PRNG, 种子固定 ⇒ 用例可复现。 */
function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const SAMPLES = 200;

function randomRect(rand: () => number): RectLike {
  return {
    left: (rand() - 0.5) * 2000,
    top: (rand() - 0.5) * 2000,
    width: 1 + rand() * 3000,
    height: 1 + rand() * 3000,
  };
}

function randomViewBox(rand: () => number): ViewBoxLike {
  return {
    x: (rand() - 0.5) * 500,
    y: (rand() - 0.5) * 500,
    width: 1 + rand() * 2000,
    height: 1 + rand() * 2000,
  };
}

function randomViewport(rand: () => number): ViewportLike {
  return {
    x: (rand() - 0.5) * 1000,
    y: (rand() - 0.5) * 1000,
    zoom: 0.2 + rand() * 3,
  };
}

/** world → client 逆链路 (测试专用参考实现, 与 coords.ts 正向公式互逆)。 */
function worldToClient(
  world: { x: number; y: number },
  rect: RectLike,
  viewBox: ViewBoxLike,
  viewport: ViewportLike,
): { clientX: number; clientY: number } {
  const svg = worldToSvg(world.x, world.y, viewport);
  const { s, ox, oy } = letterboxTransform(rect, viewBox);
  return {
    clientX: (svg.x - viewBox.x) / s + rect.left + ox,
    clientY: (svg.y - viewBox.y) / s + rect.top + oy,
  };
}

describe("coords 数学性质 (property-based, 固定种子)", () => {
  it("P1: clientToWorld ∘ worldToClient 往返恒等 (200 组随机几何)", () => {
    const rand = mulberry32(0xc00d5);
    for (let i = 0; i < SAMPLES; i++) {
      const rect = randomRect(rand);
      const viewBox = randomViewBox(rand);
      const viewport = randomViewport(rand);
      const world = { x: (rand() - 0.5) * 4000, y: (rand() - 0.5) * 4000 };
      const client = worldToClient(world, rect, viewBox, viewport);
      const back = clientToWorld(client, rect, viewBox, viewport);
      // 相对容差: 坐标量级可达数千, 浮点误差按量级缩放
      const scale = Math.max(1, Math.abs(world.x), Math.abs(world.y));
      expect(Math.abs(back.x - world.x)).toBeLessThan(scale * 1e-9);
      expect(Math.abs(back.y - world.y)).toBeLessThan(scale * 1e-9);
    }
  });

  it("P2: clientDeltaToWorld = 两点 clientToWorld 之差 (letterbox 偏移消去)", () => {
    const rand = mulberry32(0xde17a);
    for (let i = 0; i < SAMPLES; i++) {
      const rect = randomRect(rand);
      const viewBox = randomViewBox(rand);
      const viewport = randomViewport(rand);
      const c1 = { clientX: rand() * 3000, clientY: rand() * 3000 };
      const dx = (rand() - 0.5) * 800;
      const dy = (rand() - 0.5) * 800;
      const c2 = { clientX: c1.clientX + dx, clientY: c1.clientY + dy };
      const w1 = clientToWorld(c1, rect, viewBox, viewport);
      const w2 = clientToWorld(c2, rect, viewBox, viewport);
      const delta = clientDeltaToWorld(dx, dy, rect, viewBox, viewport);
      const scale = Math.max(1, Math.abs(w2.x - w1.x), Math.abs(w2.y - w1.y));
      expect(Math.abs(delta.x - (w2.x - w1.x))).toBeLessThan(scale * 1e-9);
      expect(Math.abs(delta.y - (w2.y - w1.y))).toBeLessThan(scale * 1e-9);
    }
  });

  it("P3: svgToWorld ∘ worldToSvg 往返恒等", () => {
    const rand = mulberry32(0x5b6);
    for (let i = 0; i < SAMPLES; i++) {
      const viewport = randomViewport(rand);
      const world = { x: (rand() - 0.5) * 4000, y: (rand() - 0.5) * 4000 };
      const svg = worldToSvg(world.x, world.y, viewport);
      const back = svgToWorld(svg, viewport);
      const scale = Math.max(1, Math.abs(world.x), Math.abs(world.y));
      expect(Math.abs(back.x - world.x)).toBeLessThan(scale * 1e-9);
      expect(Math.abs(back.y - world.y)).toBeLessThan(scale * 1e-9);
    }
  });

  it("P4: 平移等变性 — rect 与 client 同步平移, 世界坐标不变", () => {
    const rand = mulberry32(0x7a15);
    for (let i = 0; i < SAMPLES; i++) {
      const rect = randomRect(rand);
      const viewBox = randomViewBox(rand);
      const viewport = randomViewport(rand);
      const client = { clientX: rand() * 3000, clientY: rand() * 3000 };
      const shiftX = (rand() - 0.5) * 1000;
      const shiftY = (rand() - 0.5) * 1000;
      const w1 = clientToWorld(client, rect, viewBox, viewport);
      const w2 = clientToWorld(
        { clientX: client.clientX + shiftX, clientY: client.clientY + shiftY },
        { ...rect, left: rect.left + shiftX, top: rect.top + shiftY },
        viewBox,
        viewport,
      );
      expect(Math.abs(w2.x - w1.x)).toBeLessThan(1e-9);
      expect(Math.abs(w2.y - w1.y)).toBeLessThan(1e-9);
    }
  });

  it("P5: zoom 齐次性 — zoom×k ⇒ 相对 viewport 原点的世界偏移 ÷k", () => {
    const rand = mulberry32(0x200);
    for (let i = 0; i < SAMPLES; i++) {
      const rect = randomRect(rand);
      const viewBox = randomViewBox(rand);
      const viewport = randomViewport(rand);
      const client = { clientX: rand() * 3000, clientY: rand() * 3000 };
      const k = 0.25 + rand() * 4;
      const w1 = clientToWorld(client, rect, viewBox, viewport);
      const w2 = clientToWorld(client, rect, viewBox, {
        ...viewport,
        zoom: viewport.zoom * k,
      });
      // (w - viewport.origin) 与 zoom 成反比
      const scale = Math.max(1, Math.abs(w1.x - viewport.x), Math.abs(w1.y - viewport.y));
      expect(Math.abs((w2.x - viewport.x) * k - (w1.x - viewport.x))).toBeLessThan(scale * 1e-9);
      expect(Math.abs((w2.y - viewport.y) * k - (w1.y - viewport.y))).toBeLessThan(scale * 1e-9);
    }
  });

  it("P6: letterbox 几何不变量 — 内容区居中、等比、不超出 rect", () => {
    const rand = mulberry32(0x1e77);
    for (let i = 0; i < SAMPLES; i++) {
      const rect = randomRect(rand);
      const viewBox = randomViewBox(rand);
      const { s, ox, oy } = letterboxTransform(rect, viewBox);
      const contentW = viewBox.width / s;
      const contentH = viewBox.height / s;
      // 等比: 内容宽高比 = viewBox 宽高比
      expect(Math.abs(contentW / contentH - viewBox.width / viewBox.height)).toBeLessThan(1e-9);
      // 不超出 rect
      expect(contentW).toBeLessThanOrEqual(rect.width + 1e-9);
      expect(contentH).toBeLessThanOrEqual(rect.height + 1e-9);
      // 居中: 留白对称 (ox 是左留白, 右留白 = rect.width - contentW - ox)
      expect(Math.abs(ox - (rect.width - contentW - ox))).toBeLessThan(1e-9);
      expect(Math.abs(oy - (rect.height - contentH - oy))).toBeLessThan(1e-9);
      // meet: 至少一维撑满 (无额外缩放余地)
      const fillsW = Math.abs(contentW - rect.width) < 1e-6;
      const fillsH = Math.abs(contentH - rect.height) < 1e-6;
      expect(fillsW || fillsH).toBe(true);
    }
  });

  it("P7: 退化输入守卫 — rect 宽高 ≤0 / zoom ≤0 不产生 NaN/Infinity", () => {
    const viewBox: ViewBoxLike = { x: 0, y: 0, width: 1200, height: 800 };
    const client = { clientX: 100, clientY: 100 };
    const badRects: RectLike[] = [
      { left: 0, top: 0, width: 0, height: 800 },
      { left: 0, top: 0, width: 1200, height: 0 },
      { left: 0, top: 0, width: -5, height: -5 },
    ];
    for (const rect of badRects) {
      const { s, ox, oy } = letterboxTransform(rect, viewBox);
      expect(Number.isFinite(s)).toBe(true);
      expect(Number.isFinite(ox)).toBe(true);
      expect(Number.isFinite(oy)).toBe(true);
    }
    for (const zoom of [0, -1, -0.5]) {
      const viewport: ViewportLike = { x: 10, y: 20, zoom };
      const w = clientToWorld(client, { left: 0, top: 0, width: 1200, height: 800 }, viewBox, viewport);
      expect(Number.isFinite(w.x)).toBe(true);
      expect(Number.isFinite(w.y)).toBe(true);
      const d = clientDeltaToWorld(10, 10, { left: 0, top: 0, width: 1200, height: 800 }, viewBox, viewport);
      expect(Number.isFinite(d.x)).toBe(true);
      expect(Number.isFinite(d.y)).toBe(true);
    }
  });
});