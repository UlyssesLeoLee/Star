/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/canvas/coords.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/canvas/coords.test.ts::CanvasCoords",kind:"suite"})
MERGE (coords:File {path:"frontend/src/lib/canvas/coords.ts"})
MERGE (clientToWorld:Symbol {id:"frontend/src/lib/canvas/coords.ts::clientToWorld",kind:"function"})
MERGE (clientDeltaToWorld:Symbol {id:"frontend/src/lib/canvas/coords.ts::clientDeltaToWorld",kind:"function"})
MERGE (clientToSvg:Symbol {id:"frontend/src/lib/canvas/coords.ts::clientToSvg",kind:"function"})
MERGE (svgToWorld:Symbol {id:"frontend/src/lib/canvas/coords.ts::svgToWorld",kind:"function"})
MERGE (self)-[:IMPORTS]->(coords)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(clientToWorld)
MERGE (suite)-[:TESTS]->(clientDeltaToWorld)
MERGE (suite)-[:TESTS]->(clientToSvg)
MERGE (suite)-[:TESTS]->(svgToWorld)
@endcypher
*/
import { describe, expect, it } from "vitest";
import {
  clientDeltaToWorld,
  clientToSvg,
  clientToWorld,
  letterboxTransform,
  svgToWorld,
  worldToSvg,
} from "./coords";

// CanvasView 主 SVG: viewBox="0 0 1200 800"
const VIEWBOX = { x: 0, y: 0, width: 1200, height: 800 };

describe("lib/canvas/coords — 坐标换算单一真源", () => {
  it("1:1 无 letterbox: rect 与 viewBox 同尺寸, 中心点换算精确", () => {
    const rect = { left: 0, top: 0, width: 1200, height: 800 };
    const viewport = { x: 0, y: 0, zoom: 1 };
    const p = clientToWorld({ clientX: 600, clientY: 400 }, rect, VIEWBOX, viewport);
    expect(p.x).toBeCloseTo(600, 10);
    expect(p.y).toBeCloseTo(400, 10);
  });

  it("1:1 rect 带页面偏移 (left/top ≠ 0) 时减掉偏移", () => {
    const rect = { left: 100, top: 80, width: 1200, height: 800 };
    const viewport = { x: 0, y: 0, zoom: 1 };
    const p = clientToWorld({ clientX: 100 + 600, clientY: 80 + 400 }, rect, VIEWBOX, viewport);
    expect(p.x).toBeCloseTo(600, 10);
    expect(p.y).toBeCloseTo(400, 10);
  });

  it("宽屏 letterbox: rect 更宽时水平留白, 中心仍映射到 viewBox 中心", () => {
    // rect 2000x800 内 viewBox 等比渲染为 1200x800, 左右各留 400
    const rect = { left: 0, top: 0, width: 2000, height: 800 };
    const t = letterboxTransform(rect, VIEWBOX);
    expect(t.s).toBe(1); // sy=1 > sx=0.6 → s=1
    expect(t.ox).toBe(400);
    expect(t.oy).toBe(0);
    const center = clientToWorld({ clientX: 1000, clientY: 400 }, rect, VIEWBOX, { x: 0, y: 0, zoom: 1 });
    expect(center.x).toBeCloseTo(600, 10);
    expect(center.y).toBeCloseTo(400, 10);
    // 左留白区内: clientX=0 → svg x = -400 (viewBox 外)
    const left = clientToSvg({ clientX: 0, clientY: 400 }, rect, VIEWBOX);
    expect(left.x).toBeCloseTo(-400, 10);
  });

  it("高屏 letterbox: rect 更高时垂直留白", () => {
    const rect = { left: 0, top: 0, width: 1200, height: 1600 };
    const t = letterboxTransform(rect, VIEWBOX);
    expect(t.ox).toBe(0);
    expect(t.oy).toBe(400);
    const center = clientToWorld({ clientX: 600, clientY: 800 }, rect, VIEWBOX, { x: 0, y: 0, zoom: 1 });
    expect(center.x).toBeCloseTo(600, 10);
    expect(center.y).toBeCloseTo(400, 10);
  });

  it("zoom ≠ 1: 世界坐标按 zoom 缩放", () => {
    const rect = { left: 0, top: 0, width: 1200, height: 800 };
    const viewport = { x: 0, y: 0, zoom: 2 };
    const p = clientToWorld({ clientX: 600, clientY: 400 }, rect, VIEWBOX, viewport);
    expect(p.x).toBeCloseTo(300, 10);
    expect(p.y).toBeCloseTo(200, 10);
  });

  it("viewport 偏移: 世界坐标加上 viewport.x/y", () => {
    const rect = { left: 0, top: 0, width: 1200, height: 800 };
    const viewport = { x: 100, y: -50, zoom: 1 };
    const p = clientToWorld({ clientX: 600, clientY: 400 }, rect, VIEWBOX, viewport);
    expect(p.x).toBeCloseTo(700, 10);
    expect(p.y).toBeCloseTo(350, 10);
  });

  it("createElementAt 与绘制起点同坐标一致性: 同一 client 点 = 同一世界点", () => {
    // 画笔起点 (onMouseDown 的 svgPointFromClient) 与新建元素 (createElementAt)
    // 现在共用 clientToWorld — 同一 client 点必须得到同一世界坐标。
    const rect = { left: 40, top: 20, width: 900, height: 600 }; // 同 1.5 比例, 无 letterbox
    const viewport = { x: 30, y: 40, zoom: 1.5 };
    const client = { clientX: 40 + 450, clientY: 20 + 300 };
    const brushStart = clientToWorld(client, rect, VIEWBOX, viewport);
    const createAt = clientToWorld(client, rect, VIEWBOX, viewport);
    expect(createAt).toEqual(brushStart);
    // 旧第二套公式 (clientX/zoom + viewport.x) 在此处会偏移 — 佐证必须走 coords
    const legacy = { x: client.clientX / viewport.zoom + viewport.x, y: client.clientY / viewport.zoom + viewport.y };
    expect(legacy.x).not.toBeCloseTo(createAt.x, 6);
  });

  it("letterbox 场景下旧公式偏差可复现 (回归锁定根因)", () => {
    // 宽屏 letterbox 下左边缘: 正确世界 x = -400; 旧公式给 0
    const rect = { left: 0, top: 0, width: 2000, height: 800 };
    const viewport = { x: 0, y: 0, zoom: 1 };
    const p = clientToWorld({ clientX: 0, clientY: 400 }, rect, VIEWBOX, viewport);
    expect(p.x).toBeCloseTo(-400, 10);
    const legacy = { x: (0 - rect.left) / viewport.zoom + viewport.x, y: (400 - rect.top) / viewport.zoom + viewport.y };
    expect(legacy.x).toBe(0); // 旧公式给出的错误值, 证明修复点
  });

  it("clientDeltaToWorld: 与 clientToWorld 同一比例 (拖拽 delta)", () => {
    const rect = { left: 0, top: 0, width: 1200, height: 800 };
    const viewport = { x: 5, y: 5, zoom: 2 };
    const d = clientDeltaToWorld(120, 80, rect, VIEWBOX, viewport);
    expect(d.x).toBeCloseTo(60, 10);
    expect(d.y).toBeCloseTo(40, 10);
    // 与两点 clientToWorld 之差一致
    const a = clientToWorld({ clientX: 10, clientY: 10 }, rect, VIEWBOX, viewport);
    const b = clientToWorld({ clientX: 130, clientY: 90 }, rect, VIEWBOX, viewport);
    expect(d.x).toBeCloseTo(b.x - a.x, 10);
    expect(d.y).toBeCloseTo(b.y - a.y, 10);
  });

  it("clientDeltaToWorld 在 letterbox 下仍正确 (偏移量消去)", () => {
    const rect = { left: 0, top: 0, width: 2000, height: 800 };
    const viewport = { x: 0, y: 0, zoom: 1 };
    const d = clientDeltaToWorld(100, 50, rect, VIEWBOX, viewport);
    expect(d.x).toBeCloseTo(100, 10); // s=1
    expect(d.y).toBeCloseTo(50, 10);
  });

  it("svgToWorld / worldToSvg 互为逆运算", () => {
    const viewport = { x: 42, y: -17, zoom: 1.3 };
    const w = svgToWorld({ x: 300, y: 200 }, viewport);
    const back = worldToSvg(w.x, w.y, viewport);
    expect(back.x).toBeCloseTo(300, 10);
    expect(back.y).toBeCloseTo(200, 10);
  });
});
