/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/canvas/coords.ts"})
MERGE (letterboxTransform:Symbol {id:"frontend/src/lib/canvas/coords.ts::letterboxTransform",kind:"function"})
MERGE (clientToSvg:Symbol {id:"frontend/src/lib/canvas/coords.ts::clientToSvg",kind:"function"})
MERGE (svgToWorld:Symbol {id:"frontend/src/lib/canvas/coords.ts::svgToWorld",kind:"function"})
MERGE (worldToSvg:Symbol {id:"frontend/src/lib/canvas/coords.ts::worldToSvg",kind:"function"})
MERGE (clientToWorld:Symbol {id:"frontend/src/lib/canvas/coords.ts::clientToWorld",kind:"function"})
MERGE (clientDeltaToWorld:Symbol {id:"frontend/src/lib/canvas/coords.ts::clientDeltaToWorld",kind:"function"})
MERGE (self)-[:DEFINES]->(letterboxTransform)
MERGE (self)-[:DEFINES]->(clientToSvg)
MERGE (self)-[:DEFINES]->(svgToWorld)
MERGE (self)-[:DEFINES]->(worldToSvg)
MERGE (self)-[:DEFINES]->(clientToWorld)
MERGE (self)-[:DEFINES]->(clientDeltaToWorld)
@endcypher
*/

/**
 * 无限画布坐标换算 — 单一真源 (per canvas-ux 任务 §A)
 *
 * 修复「画笔/鼠标偏离」根因: CanvasView 主 SVG `viewBox="0 0 1200 800"` + 弹性布局下
 * rect 实际尺寸 ≠ viewBox 尺寸, 旧公式 `(clientX - rect.left) / zoom + viewport.x`
 * 缺失 viewBox↔rect 的比例与 letterbox 偏移 (preserveAspectRatio="xMidYMid meet")。
 *
 * 换算链 (客户端像素 → 世界坐标):
 *   1. sx = viewBox.width / rect.width, sy = viewBox.height / rect.height
 *   2. meet letterbox: s = max(sx, sy);
 *      ox = (rect.width  - viewBox.width  / s) / 2
 *      oy = (rect.height - viewBox.height / s) / 2
 *   3. svgX = (clientX - rect.left - ox) * s + viewBox.x   (SVG user 坐标)
 *   4. worldX = svgX / zoom + viewport.x                    (世界坐标)
 *
 * 所有 client→world 的换算路径 (绘制起点 / 拖拽 delta / createElementAt / wheel 缩放
 * 锚点) 必须走这里的函数, 禁止在组件里再写第二套公式。
 */

/** 鼠标事件的最小形状 (兼容 React.MouseEvent / DOM MouseEvent)。 */
export interface ClientPointLike {
  clientX: number;
  clientY: number;
}

/** getBoundingClientRect() 的最小形状。 */
export interface RectLike {
  left: number;
  top: number;
  width: number;
  height: number;
}

/** SVG viewBox。 */
export interface ViewBoxLike {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** 世界坐标视口 (viewport.x/y 世界偏移, zoom 缩放)。 */
export interface ViewportLike {
  x: number;
  y: number;
  zoom: number;
}

/** 目前只实现 meet 语义 (CanvasView 主 SVG 默认 preserveAspectRatio)。 */
export type PreserveAspectRatio = "xMidYMid meet";

/** meet letterbox 换算参数: s 为 viewBox→rect 的逆缩放, ox/oy 为居中留白。 */
export interface LetterboxTransform {
  s: number;
  ox: number;
  oy: number;
}

/**
 * 计算 viewBox 在 rect 内的 meet 缩放与 letterbox 偏移。
 * rect 宽高非法 (<=0) 时回落 s=1, 避免除零。
 */
export function letterboxTransform(
  rect: RectLike,
  viewBox: ViewBoxLike,
  _preserveAspectRatio: PreserveAspectRatio = "xMidYMid meet",
): LetterboxTransform {
  const rw = rect.width > 0 ? rect.width : 1;
  const rh = rect.height > 0 ? rect.height : 1;
  const sx = viewBox.width / rw;
  const sy = viewBox.height / rh;
  // meet: viewBox 整体等比缩放进 rect (留白侧由 ox/oy 表达); s = max(sx, sy)
  const s = Math.max(sx, sy);
  return {
    s,
    ox: (rw - viewBox.width / s) / 2,
    oy: (rh - viewBox.height / s) / 2,
  };
}

/** 客户端像素坐标 → SVG user 坐标 (含 letterbox 与 viewBox 原点)。 */
export function clientToSvg(
  client: ClientPointLike,
  rect: RectLike,
  viewBox: ViewBoxLike,
  preserveAspectRatio: PreserveAspectRatio = "xMidYMid meet",
): { x: number; y: number } {
  const { s, ox, oy } = letterboxTransform(rect, viewBox, preserveAspectRatio);
  return {
    x: (client.clientX - rect.left - ox) * s + viewBox.x,
    y: (client.clientY - rect.top - oy) * s + viewBox.y,
  };
}

/** SVG user 坐标 → 世界坐标 (viewport 偏移 + zoom)。 */
export function svgToWorld(
  svg: { x: number; y: number },
  viewport: ViewportLike,
): { x: number; y: number } {
  const zoom = viewport.zoom > 0 ? viewport.zoom : 1;
  return {
    x: svg.x / zoom + viewport.x,
    y: svg.y / zoom + viewport.y,
  };
}

/** 世界坐标 → SVG user 坐标 (渲染用, 与旧 worldToScreen 同义)。 */
export function worldToSvg(
  wx: number,
  wy: number,
  viewport: ViewportLike,
): { x: number; y: number } {
  return {
    x: (wx - viewport.x) * viewport.zoom,
    y: (wy - viewport.y) * viewport.zoom,
  };
}

/** 客户端像素坐标 → 世界坐标 (完整链路, 单一真源)。 */
export function clientToWorld(
  client: ClientPointLike,
  rect: RectLike,
  viewBox: ViewBoxLike,
  viewport: ViewportLike,
  preserveAspectRatio: PreserveAspectRatio = "xMidYMid meet",
): { x: number; y: number } {
  return svgToWorld(clientToSvg(client, rect, viewBox, preserveAspectRatio), viewport);
}

/**
 * 客户端像素位移 → 世界位移 (拖拽 delta / 缩放锚点用, 与 clientToWorld 同一比例)。
 * letterbox 偏移是常量, 在差值中消去, 故只用 s/zoom。
 */
export function clientDeltaToWorld(
  dx: number,
  dy: number,
  rect: RectLike,
  viewBox: ViewBoxLike,
  viewport: ViewportLike,
  preserveAspectRatio: PreserveAspectRatio = "xMidYMid meet",
): { x: number; y: number } {
  const { s } = letterboxTransform(rect, viewBox, preserveAspectRatio);
  const zoom = viewport.zoom > 0 ? viewport.zoom : 1;
  return {
    x: (dx * s) / zoom,
    y: (dy * s) / zoom,
  };
}
