/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/components/CanvasView.tsx",type:"file",language:"tsx"}),
  (props:Class {name:"CanvasViewProps",type:"class",visibility:"private"}),
  (view:Function {name:"CanvasView",type:"function",signature:"CanvasView(props: CanvasViewProps): JSX.Element",visibility:"public",complexity:"complex"}),
  (updateViewport:Function {name:"updateViewport",type:"function",signature:"updateViewport(nextViewport: CanvasViewport): void",visibility:"private",complexity:"simple"}),
  (onMouseMove:Function {name:"onMouseMove",type:"function",signature:"onMouseMove(event: React.MouseEvent): void",visibility:"private",complexity:"moderate"}),
  (onMouseUp:Function {name:"onMouseUp",type:"function",signature:"onMouseUp(): void",visibility:"private",complexity:"moderate"}),
  (displayElements:Variable {name:"displayElements",type:"variable",language:"typescript"}),
  (dragPreview:Variable {name:"dragPreview",type:"variable",language:"typescript"}),
  (dragPreviewRef:Variable {name:"dragPreviewRef",type:"variable",language:"typescript"}),
  (positionChange:Variable {name:"onElementPositionChange",type:"variable",language:"typescript"}),
  (selectionChange:Variable {name:"onSelectionChange",type:"variable",language:"typescript"}),
  (deleteElements:Variable {name:"onDeleteElements",type:"variable",language:"typescript"}),
  (deleteSelection:Function {name:"deleteSelection",type:"function",signature:"deleteSelection(): Promise<void>",visibility:"private",complexity:"moderate"}),
  (viewport:Variable {name:"viewport",type:"variable",language:"typescript"}),
  (onViewportChange:Variable {name:"onViewportChange",type:"variable",language:"typescript"}),
  (file)-[:CONTAINS]->(props),(file)-[:CONTAINS]->(view),(view)-[:CONTAINS]->(updateViewport),(view)-[:CONTAINS]->(onMouseMove),(view)-[:CONTAINS]->(onMouseUp),(view)-[:CONTAINS]->(displayElements),
  (view)-[:CONTAINS]->(deleteSelection),(view)-[:USES]->(viewport),(view)-[:USES]->(onViewportChange),(view)-[:USES]->(positionChange),(view)-[:USES]->(selectionChange),(view)-[:USES]->(deleteElements),(view)-[:USES]->(dragPreview),(view)-[:USES]->(dragPreviewRef),(view)-[:CALLS]->(updateViewport),(view)-[:CALLS]->(onMouseMove),(view)-[:CALLS]->(onMouseUp),(view)-[:CALLS]->(displayElements),(view)-[:CALLS]->(deleteSelection),(onMouseMove)-[:USES]->(positionChange),(onMouseMove)-[:USES]->(dragPreviewRef),(onMouseUp)-[:USES]->(dragPreviewRef),(onMouseUp)-[:CALLS]->(positionChange),(displayElements)-[:USES]->(dragPreview),(updateViewport)-[:USES]->(viewport),(updateViewport)-[:USES]->(onViewportChange),(deleteSelection)-[:USES]->(deleteElements);
*/

"use client";

/**
 * CanvasView - Miro 模式无限画布
 *
 * 继承 frontend-canvas-design.md v0.1:
 * - 无限世界坐标,viewport 转换
 * - 7 种 element 渲染
 * - Bezier connector(复用 StateMachineDiagram 算法)
 * - worktree_node 状态色码走 StatusPill 60+ 同步(联动 3)
 * - work_item_card 双击跳详情(联动 2)
 * - frame 分区(可作 slide)
 */

import type {
  Canvas, CanvasElement, CanvasConnector, CanvasViewport, Worktree, WorkItem, AgentSession, AutomationRule, Feedback,
} from "@/types/ids";
import { useMemo, useState, useRef, useEffect, useCallback } from "react";
import { useStore } from "@/lib/store";
import { StatusPill } from "./StatusPill";
import { MousePointer2, Hand, Plus, Trash2, ZoomIn, ZoomOut, Maximize2, StickyNote, Type, Square, Frame, Brush } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

interface CanvasViewProps {
  canvas: Pick<Canvas, "id" | "viewport" | "frames">;
  elements: CanvasElementView[];
  connectors: CanvasConnectorView[];
  highlightElementId?: string;
  readOnly?: boolean;
  groupWorkItems?: Array<Pick<WorkItem, "id" | "key" | "title" | "status">>;
  groupWorktrees?: Array<Pick<Worktree, "id" | "branch" | "status">>;
  onOpenWorkItem?: (workItemId: string, worktreeId?: string) => void;
  onViewportChange?: (viewport: CanvasViewport) => void;
  onElementPositionChange?: (position: { elementId: string; x: number; y: number; expectedVersion: number }) => Promise<void>;
  onSelectionChange?: (elementIds: string[]) => void;
  onDeleteElements?: (elementIds: string[]) => Promise<void>;
  /**
   * 新增元素回调 (per 2026-10-01 OOB: 恢复丢失的画笔/创建功能).
   * - element 字段按 CanvasElement 形状 (id/created_at/created_by 由后端生成)
   * - worktree-id 通过 groupApi 上下文获取
   */
  onCreateElement?: (body: {
    kind: CanvasElement["kind"];
    x: number;
    y: number;
    width: number;
    height: number;
    z_index: number;
    content: CanvasElement["content"];
  }) => Promise<{ id?: string } | void>;
}

type CanvasElementView = Pick<CanvasElement,
  "id" | "canvas_id" | "kind" | "x" | "y" | "width" | "height" | "rotation" | "z_index" | "entity_ref" | "content" | "locked" | "hidden"
> & { version?: number };
type CanvasConnectorView = Pick<CanvasConnector,
  "id" | "canvas_id" | "kind" | "from_element_id" | "to_element_id" | "routing" | "arrow_start" | "arrow_end" | "color" | "width" | "label"
>;

const STICKY_PALETTE = ["#f9d77e", "#ffb3c1", "#a3d9ff", "#b8f0c4", "#d4b3ff"];
// per 2026-10-01 OOB 自由画笔 (path element) - 主题感知色 + 亮色 fallback
const BRUSH_PALETTE = ["#e6edf3", "#79c0ff", "#ff7b72", "#a5d6ff", "#ffa657"];
const BRUSH_SIZES = [2, 4, 8, 12];

export function CanvasView({ canvas, elements, connectors, highlightElementId, readOnly = false, groupWorkItems, groupWorktrees, onOpenWorkItem, onViewportChange, onElementPositionChange, onSelectionChange, onDeleteElements, onCreateElement }: CanvasViewProps) {
  const { t } = useTranslation();
  // viewport: 世界坐标
  const [viewport, setViewport] = useState(canvas.viewport);
  const [selected, setSelected] = useState<string[]>([]);
  const [tool, setTool] = useState<"select" | "pan" | "sticky" | "text" | "shape" | "frame-select" | "brush">("select");
  const [pendingFrameId, setPendingFrameId] = useState<string | null>(null);
  const [createPending, setCreatePending] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);
  // 自由画笔 (per 2026-10-01 OOB 恢复丢失的画笔功能 - 鼠标轨迹 SVG path)
  const [brushColor, setBrushColor] = useState<string>(BRUSH_PALETTE[0]);
  const [brushSize, setBrushSize] = useState<number>(BRUSH_SIZES[1]);
  const [drawingPath, setDrawingPath] = useState<{
    points: Array<{ x: number; y: number }>;
    /** 包围盒 (用于 element width/height) */
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
  } | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const [dragPreview, setDragPreview] = useState<{ elementId: string; x: number; y: number } | null>(null);
  const dragPreviewRef = useRef<{ elementId: string; x: number; y: number } | null>(null);
  const dragState = useRef<{ type: "pan" | "element" | null; startX: number; startY: number; elX: number; elY: number; elId: string | null; expectedVersion: number }>({
    type: null, startX: 0, startY: 0, elX: 0, elY: 0, elId: null, expectedVersion: 0,
  });

  const worktrees = useStore((s) => s.worktrees);
  const agentSessions = useStore((s) => s.agentSessions);
  const automationRules = useStore((s) => s.automationRules);
  const feedbacks = useStore((s) => s.feedbacks);
  const moveCanvasElement = useStore((s) => s.moveCanvasElement);
  const deleteCanvasElement = useStore((s) => s.deleteCanvasElement);
  const availableWorkItems = groupWorkItems ?? useStore.getState().workItems;
  const availableWorktrees = groupWorktrees ?? worktrees;

  useEffect(() => {
    onSelectionChange?.(selected);
  }, [onSelectionChange, selected]);

  // 屏幕坐标 → 世界坐标
  const screenToWorld = useCallback((sx: number, sy: number) => ({
    x: sx / viewport.zoom + viewport.x,
    y: sy / viewport.zoom + viewport.y,
  }), [viewport]);

  // 世界坐标 → 屏幕坐标
  const worldToScreen = useCallback((wx: number, wy: number) => ({
    x: (wx - viewport.x) * viewport.zoom,
    y: (wy - viewport.y) * viewport.zoom,
  }), [viewport]);

  const updateViewport = useCallback((nextViewport: CanvasViewport) => {
    setViewport(nextViewport);
    onViewportChange?.(nextViewport);
  }, [onViewportChange]);

  // 自动滚动到高亮 element
  useEffect(() => {
    if (!highlightElementId) return;
    const el = elements.find((e) => e.id === highlightElementId);
    if (!el) return;
    // 计算 fit to element
    const targetX = el.x + el.width / 2;
    const targetY = el.y + el.height / 2;
    setViewport({ x: targetX - 600 / viewport.zoom / 2, y: targetY - 400 / viewport.zoom / 2, zoom: viewport.zoom });
  }, [highlightElementId]);  // eslint-disable-line react-hooks/exhaustive-deps

  // pan / drag 处理
  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button === 1 || (e.button === 0 && tool === "pan") || e.shiftKey) {
      // 中键 / pan 工具 / shift = pan viewport
      dragState.current = { type: "pan", startX: e.clientX, startY: e.clientY, elX: viewport.x, elY: viewport.y, elId: null, expectedVersion: 0 };
    } else if (e.button === 0 && tool === "brush" && !readOnly) {
      // 自由画笔 (per 2026-10-01 OOB 恢复丢失的画笔功能)
      // - mousedown 在 SVG 任意位置启动 (不限于空白区, 笔画重叠允许)
      // - 在 SVG 坐标系内取点 (因为 path_data 相对 element bbox)
      const svgPt = svgPointFromClient(e);
      if (!svgPt) return;
      setDrawingPath({ points: [svgPt], minX: svgPt.x, minY: svgPt.y, maxX: svgPt.x, maxY: svgPt.y });
    }
  };

  const onMouseMove = (e: React.MouseEvent) => {
    const ds = dragState.current;
    if (ds.type === "pan") {
      const dx = (e.clientX - ds.startX) / viewport.zoom;
      const dy = (e.clientY - ds.startY) / viewport.zoom;
      updateViewport({ ...viewport, x: ds.elX - dx, y: ds.elY - dy });
    } else if (ds.type === "element" && ds.elId && (!readOnly || onElementPositionChange)) {
      const dx = (e.clientX - ds.startX) / viewport.zoom;
      const dy = (e.clientY - ds.startY) / viewport.zoom;
      const nextX = ds.elX + dx;
      const nextY = ds.elY + dy;
      if (onElementPositionChange) {
        const preview = { elementId: ds.elId, x: nextX, y: nextY };
        dragPreviewRef.current = preview;
        setDragPreview(preview);
      }
      else moveCanvasElement(ds.elId, nextX, nextY);
    } else if (drawingPath) {
      // 自由画笔 (per 2026-10-01 OOB 恢复丢失的画笔功能)
      // - 累计路径点 (按距离阈值采样, 避免冗余点)
      // - 更新包围盒
      const svgPt = svgPointFromClient(e);
      if (!svgPt) return;
      const lastPt = drawingPath.points[drawingPath.points.length - 1];
      const dist = Math.hypot(svgPt.x - lastPt.x, svgPt.y - lastPt.y);
      if (dist < 1.5) return; // 过滤掉抖动冗余点
      setDrawingPath({
        points: [...drawingPath.points, svgPt],
        minX: Math.min(drawingPath.minX, svgPt.x),
        minY: Math.min(drawingPath.minY, svgPt.y),
        maxX: Math.max(drawingPath.maxX, svgPt.x),
        maxY: Math.max(drawingPath.maxY, svgPt.y),
      });
    }
  };

  const onMouseUp = () => {
    const ds = dragState.current;
    dragState.current = { type: null, startX: 0, startY: 0, elX: 0, elY: 0, elId: null, expectedVersion: 0 };
    const preview = dragPreviewRef.current;
    dragPreviewRef.current = null;
    if (ds.type === "element" && ds.elId && ds.expectedVersion > 0 && onElementPositionChange && preview?.elementId === ds.elId) {
      const position = { elementId: ds.elId, x: preview.x, y: preview.y, expectedVersion: ds.expectedVersion };
      if (position.x !== ds.elX || position.y !== ds.elY) {
        void onElementPositionChange(position).catch(() => undefined).finally(() => setDragPreview(null));
        return;
      }
    }
    // 自由画笔 (per 2026-10-01 OOB) - mouseup 提交当前 path 为 element
    if (drawingPath) {
      void createPathElement(drawingPath);
      // 不切回 select tool (用户连续画); 仅 setDrawingPath 在 createPathElement finally 里清空
      return;
    }
    setDragPreview(null);
  };

  // 滚轮 zoom
  const onWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.1, Math.min(4, viewport.zoom * delta));
    // 以光标为中心
    if (svgRef.current) {
      const rect = svgRef.current.getBoundingClientRect();
      const sx = e.clientX - rect.left;
      const sy = e.clientY - rect.top;
      const wx = sx / viewport.zoom + viewport.x;
      const wy = sy / viewport.zoom + viewport.y;
      updateViewport({ x: wx - sx / newZoom, y: wy - sy / newZoom, zoom: newZoom });
    }
  };

  const onElementMouseDown = (e: React.MouseEvent, el: CanvasElementView) => {
    e.stopPropagation();
    if (tool === "pan") return;
    if (e.shiftKey) {
      setSelected((current) => current.includes(el.id)
        ? current.filter((selectedId) => selectedId !== el.id)
        : [...current, el.id]);
      return;
    }
    setSelected([el.id]);
    dragPreviewRef.current = null;
    setDragPreview(null);
    if (!el.locked && (!readOnly || onElementPositionChange)) {
      dragState.current = {
        type: "element",
        startX: e.clientX,
        startY: e.clientY,
        elX: el.x,
        elY: el.y,
        elId: el.id,
        expectedVersion: el.version ?? 0,
      };
    }
  };

  const deleteSelection = async () => {
    if (selected.length === 0) return;
    try {
      if (onDeleteElements) await onDeleteElements(selected);
      else selected.forEach((id) => deleteCanvasElement(id));
      setSelected([]);
    } catch {
      // The owning Group page reports the API failure and keeps this selection available for retry.
    }
  };

  const onElementDoubleClick = (el: CanvasElementView) => {
    const workItemId = el.entity_ref
      ? el.entity_ref.ref_type === "work_item" ? el.entity_ref.ref_id : undefined
      : el.content.work_item_id;
    if (workItemId && onOpenWorkItem) {
      onOpenWorkItem(workItemId, el.entity_ref?.worktree_id);
      return;
    }

    // 联动 2:work_item_card / worktree_node / agent_cursor / automation_node → 跳详情
    const ref = el.entity_ref
      ? el.entity_ref.ref_id
      : el.content.work_item_id || el.content.worktree_id || el.content.agent_session_id || el.content.automation_id;
    const kind = el.entity_ref?.ref_type === "work_item" ? "work-item"
      : el.entity_ref?.ref_type === "worktree" ? "worktree"
      : el.entity_ref?.ref_type === "agent_session" ? "agent"
      : el.entity_ref?.ref_type === "automation" ? "automation"
      : el.entity_ref ? null
      : el.content.work_item_id ? "work-item"
      : el.content.worktree_id ? "worktree"
      : el.content.agent_session_id ? "agent"
      : el.content.automation_id ? "automation"
      : null;
    if (ref && kind) {
      window.location.href = `/${kind}?selected=${ref}`;
    }
  };

  /**
   * SVG client coords -> SVG user-space coords (考虑 viewport.zoom + viewport.x/y)
   * - 不依赖 onMouseDown only target=svg (svgPtFromClient 也适用于 element 上的事件)
   * - 不调用 React.hooks (helper function)
   */
  const svgPointFromClient = (e: React.MouseEvent): { x: number; y: number } | null => {
    if (!svgRef.current) return null;
    const rect = svgRef.current.getBoundingClientRect();
    // svg viewBox = 0 0 1200 800 (固定); 我们用 viewport.x/y/zoom 推导世界坐标
    return {
      x: (e.clientX - rect.left) / viewport.zoom + viewport.x,
      y: (e.clientY - rect.top) / viewport.zoom + viewport.y,
    };
  };

  const displayElements = dragPreview
    ? elements.map((element) => element.id === dragPreview.elementId ? { ...element, x: dragPreview.x, y: dragPreview.y } : element)
    : elements;

  /**
   * 新增元素 (per 2026-10-01 OOB: 恢复丢失的画笔/创建功能)
   * - sticky_note / text / shape 工具在 SVG 空白区点击时触发
   * - worktree canvas 走 onCreateElement (group page 接到 groupApi.createCanvasElement)
   * - 本地 store canvas (preview mode) 走 addCanvasElement 直接入 store
   * - frame-select mode 已确定目标 frame 后创建
   */
  const createElementAt = async (
    e: React.MouseEvent,
    kind: "sticky_note" | "text" | "shape",
    width: number,
    height: number,
    content: CanvasElement["content"] = {},
  ) => {
    if (readOnly || !onCreateElement || createPending) return;
    const world = { x: e.clientX / viewport.zoom + viewport.x, y: e.clientY / viewport.zoom + viewport.y };
    if (!world) return;
    setCreateError(null);
    setCreatePending(true);
    try {
      const result = await onCreateElement({
        kind,
        x: Math.round(world.x - width / 2),
        y: Math.round(world.y - height / 2),
        width,
        height,
        z_index: elements.length + 1,
        content,
      });
      // 切回 select tool, 选中新建元素 (按 id 反馈)
      if (result?.id) setSelected([result.id]);
      setTool("select");
    } catch (err) {
      setCreateError(err instanceof Error ? err.message : "Failed to create element");
    } finally {
      setCreatePending(false);
    }
  };

  /**
   * 自由画笔创建 (per 2026-10-01 OOB 恢复丢失的画笔功能)
   * - 鼠标按下记录起点, 拖动累积 path points, 抬起 → 创建 element
   * - path_data 用 SVG d 属性 (M x0 y0 L x1 y1 ...); 坐标相对 element bbox 起点
   * - bbox 留 4px padding (避免 stroke 半截)
   */
  const createPathElement = async (path: NonNullable<typeof drawingPath>) => {
    if (readOnly || !onCreateElement || createPending) return;
    if (path.points.length < 2) {
      // 误触 (单击未拖), 丢弃
      setDrawingPath(null);
      return;
    }
    const PAD = Math.max(2, brushSize);
    const x = path.minX - PAD;
    const y = path.minY - PAD;
    const width = Math.max(2, path.maxX - path.minX + PAD * 2);
    const height = Math.max(2, path.maxY - path.minY + PAD * 2);
    const pathData =
      "M " + path.points.map((p) => `${(p.x - x).toFixed(1)} ${(p.y - y).toFixed(1)}`).join(" L ");
    setCreateError(null);
    setCreatePending(true);
    try {
      const result = await onCreateElement({
        kind: "path",
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(width),
        height: Math.round(height),
        z_index: elements.length + 1,
        content: {
          path_data: pathData,
          brush_size: brushSize,
          brush_color: brushColor,
        },
      });
      if (result?.id) setSelected([result.id]);
    } catch (err) {
      setCreateError(err instanceof Error ? err.message : "Failed to save brush stroke");
    } finally {
      setDrawingPath(null);
      setCreatePending(false);
    }
  };

  // render element
  const renderElement = (el: CanvasElementView) => {
    const isHighlighted = el.id === highlightElementId;
    const isSelected = selected.includes(el.id);
    const stroke = isHighlighted ? "var(--cel-cyan, #2f81f7)" : isSelected ? "var(--cel-cyan, #79c0ff)" : "var(--cel-ink, #30363d)";
    const strokeWidth = isHighlighted || isSelected ? 2 : 1;
    const pos = worldToScreen(el.x, el.y);
    const w = el.width * viewport.zoom;
    const h = el.height * viewport.zoom;

    switch (el.kind) {
      case "path": {
        // 自由画笔 (per 2026-10-01 OOB 恢复丢失的画笔功能)
        // path_data 用 SVG d 属性, 坐标相对 element x/y origin
        // stroke width 也按 viewport.zoom 缩放 (避免远处看不清/近处不锐利)
        const pathData = el.content.path_data;
        if (!pathData) return null;
        const bw = (el.content.brush_size ?? 4) * viewport.zoom;
        const bc = el.content.brush_color ?? "var(--cel-text-primary, #e6edf3)";
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <rect width={w} height={h} fill="transparent" />
            <path
              d={pathData}
              stroke={bc}
              strokeWidth={bw}
              fill="none"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            {(isHighlighted || isSelected) && (
              <rect width={w} height={h} fill="none" stroke={stroke} strokeWidth={1} strokeDasharray="4 4" />
            )}
          </g>
        );
      }
      case "sticky_note": {
        const color = el.content.color || STICKY_PALETTE[0];
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <rect width={w} height={h} fill={color} stroke={stroke} strokeWidth={strokeWidth} rx={4} />
            <foreignObject x={6} y={6} width={w - 12} height={h - 12}>
          <div style={{ fontSize: 11 * viewport.zoom, color: "var(--cel-surface-stage, #0b0d10)", lineHeight: 1.3, fontFamily: "system-ui", wordBreak: "break-word", overflow: "hidden" }}>
            {el.content.text}
          </div>
            </foreignObject>
          </g>
        );
      }
      case "text": {
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <foreignObject width={w} height={h}>
          <div style={{ fontSize: 12 * viewport.zoom, color: "var(--cel-text-primary, #e6edf3)", lineHeight: 1.4, fontFamily: "system-ui" }}>
            {el.content.text}
          </div>
            </foreignObject>
          </g>
        );
      }
      case "work_item_card": {
        const workItemId = el.entity_ref
          ? el.entity_ref.ref_type === "work_item" ? el.entity_ref.ref_id : undefined
          : el.content.work_item_id;
        const wi = availableWorkItems.find((w) => w.id === workItemId);
        if (!wi) return null;
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <rect width={w} height={h} fill="var(--cel-surface-card, #161b22)" stroke={stroke} strokeWidth={strokeWidth} rx={4} />
            <text x={8} y={16 * viewport.zoom} fontSize={10 * viewport.zoom} fill="var(--cel-text-secondary, #8b949e)" fontFamily="ui-monospace, monospace">
          {wi.key}
            </text>
            <foreignObject x={8} y={20 * viewport.zoom} width={w - 16} height={h - 30 * viewport.zoom}>
          <div style={{ fontSize: 11 * viewport.zoom, color: "var(--cel-text-primary, #e6edf3)", lineHeight: 1.3, fontFamily: "system-ui", overflow: "hidden" }}>
            {wi.title}
          </div>
            </foreignObject>
            <g transform={`translate(8, ${h - 24 * viewport.zoom})`}>
          <StatusPill value={wi.status} size="xs" />
            </g>
          </g>
        );
      }
      case "worktree_node": {
        const worktreeId = el.entity_ref
          ? el.entity_ref.ref_type === "worktree" ? el.entity_ref.ref_id : undefined
          : el.content.worktree_id;
        const wt = availableWorktrees.find((w) => w.id === worktreeId);
        if (!wt) return null;
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <rect width={w} height={h} fill="var(--cel-surface-card, #161b22)" stroke={stroke} strokeWidth={strokeWidth} rx={20} />
            <text x={12 * viewport.zoom} y={18 * viewport.zoom} fontSize={10 * viewport.zoom} fill="var(--cel-text-secondary, #8b949e)" fontFamily="ui-monospace, monospace">
          worktree
            </text>
            <text x={12 * viewport.zoom} y={34 * viewport.zoom} fontSize={12 * viewport.zoom} fill="var(--cel-text-primary, #e6edf3)" fontFamily="ui-monospace, monospace">
          {wt.branch}
            </text>
            <g transform={`translate(${w - 90 * viewport.zoom}, ${h - 22 * viewport.zoom})`}>
          <StatusPill value={wt.status} size="xs" />
            </g>
          </g>
        );
      }
      case "agent_cursor": {
        const ag = agentSessions.find((a) => a.id === el.content.agent_session_id);
        if (!ag) return null;
        // 像素风机器人 (per 2026-09-06 12:34 JST 用户发令)
        // 32x32 内部 pixel grid, 用 SVG <rect> 绘制保证缩放清晰
        // 状态色码: running/active → LED 绿, paused/awaiting → LED 黄, failed/cancelled → LED 红 + 报警底色
        const ledColor =
          ag.status === "failed" || ag.status === "cancelled" ? "#ff4757"
          : ag.status === "paused" || ag.status === "awaiting_feedback" || ag.status === "awaiting_human" || ag.status === "awaiting_tool" ? "#fbbf24"
          : ag.status === "completed" ? "#94a3b8"
          : "#51cf66";
        const bodyColor =
          ag.agent_kind === "claude-sonnet" ? "#a78bfa"
          : ag.agent_kind === "gpt-4o" ? "#10b981"
          : ag.agent_kind === "codex" ? "#22d3ee"
          : "#3b5bdb";
        const bodyDark =
          ag.agent_kind === "claude-sonnet" ? "#7c3aed"
          : ag.agent_kind === "gpt-4o" ? "#047857"
          : ag.agent_kind === "codex" ? "#0e7490"
          : "#1c2e6e";
        const isAlert = ag.status === "failed" || ag.status === "cancelled";
        // 16x16 像素精灵, 占 element 短边的 60% (per 2026-10-01 OOB: 角色外观不能太小)
        const targetSize = Math.floor(Math.min(w, h) * 0.6);
        const cellSize = Math.max(3, Math.floor(targetSize / 16));  // 每 pixel cell 实际渲染大小
        const spriteSize = cellSize * 16;  // sprite 总尺寸 (16 unit grid)
        const gridOriginX = (w - spriteSize) / 2;
        const gridOriginY = (h - spriteSize) / 2;
        // 简化像素精灵 (16x16)
        // 0 = 透明, 1 = outline, 2 = body, 3 = bodyDark, 4 = bodyLight, 5 = LED
        // 造型: 双天线 + 头部-眼带 + 躯干-胸灯 + 腰甲 + 双脚
        const sprite: number[][] = [
          [0,0,0,0,0,1,1,1,1,1,0,0,0,0,0,0],
          [0,0,0,0,1,2,2,1,2,2,1,0,0,0,0,0],
          [0,0,0,0,1,2,5,1,5,2,1,0,0,0,0,0],
          [0,0,0,0,1,2,2,2,2,2,1,0,0,0,0,0],
          [0,0,0,0,1,2,5,5,5,2,1,0,0,0,0,0],
          [0,0,0,0,1,2,5,2,5,2,1,0,0,0,0,0],
          [0,1,1,1,1,2,2,2,2,2,1,1,1,1,0,0],
          [1,2,2,1,2,2,2,2,2,2,2,1,2,2,1,0],
          [1,2,4,1,2,4,4,2,4,4,2,1,4,2,1,0],
          [1,2,2,2,2,2,2,5,2,2,2,2,2,2,1,0],
          [1,2,2,2,2,2,2,2,2,2,2,2,2,2,1,0],
          [1,2,2,2,2,1,2,2,2,1,2,2,2,2,1,0],
          [0,1,2,2,2,1,1,2,1,1,2,2,2,1,0,0],
          [0,1,2,2,1,0,1,2,1,0,1,2,2,1,0,0],
          [0,0,1,1,0,0,1,2,1,0,0,1,1,0,0,0],
          [0,0,1,1,0,0,1,1,1,0,0,1,1,0,0,0],
        ];
        const colorFor = (cell: number): string | null => {
          switch (cell) {
            case 1: return "#0a0e1a";
            case 2: return bodyColor;
            case 3: return bodyDark;
            case 4: return "#748ffc";
            case 5: return ledColor;
            default: return null;
          }
        };
        const spriteRectX = gridOriginX;
        const spriteRectY = gridOriginY;
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            {isAlert && <rect x={0} y={0} width={w} height={h} fill="#ff475722" rx={6} />}
            {/* 阴影 */}
            <ellipse cx={w / 2} cy={h - 4} rx={w / 2 - 6} ry={3} fill="rgba(0,0,0,0.25)" />
            {/* 像素精灵 */}
            <g transform={`translate(${spriteRectX}, ${spriteRectY}) scale(${cellSize})`} shapeRendering="crispEdges">
          {sprite.map((row, y) =>
            row.map((cell, x) => {
              const c = colorFor(cell);
              return c ? <rect key={`px-${x}-${y}`} x={x} y={y} width={1} height={1} fill={c} /> : null;
            }),
          )}
            </g>
            {/* name label */}
            <text x={w / 2} y={h - Math.max(8, cellSize * 2)} textAnchor="middle" fontSize={Math.max(8, cellSize * 1.2)} fill="var(--cel-text-primary, #e6edf3)" fontFamily="ui-monospace, monospace" style={{ paintOrder: "stroke", stroke: "var(--cel-surface-stage, #0b0d10)", strokeWidth: 2 }}>
          {ag.name}
            </text>
          </g>
        );
      }
      case "automation_node": {
        const au = automationRules.find((a) => a.id === el.content.automation_id);
        if (!au) return null;
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <polygon points={`${w / 2},0 ${w},${h / 3} ${w},${(2 * h) / 3} ${w / 2},${h} 0,${(2 * h) / 3} 0,${h / 3}`} fill="#d2992222" stroke={stroke} strokeWidth={strokeWidth} />
            <text x={w / 2} y={h / 2 - 4} textAnchor="middle" fontSize={10 * viewport.zoom} fill="#d29922" fontFamily="ui-monospace, monospace">
          rule
            </text>
            <foreignObject x={8} y={h / 2 - 2} width={w - 16} height={h / 2}>
          <div style={{ fontSize: 10 * viewport.zoom, color: "var(--cel-text-primary, #e6edf3)", lineHeight: 1.2, fontFamily: "system-ui", textAlign: "center", overflow: "hidden" }}>
            {au.name}
          </div>
            </foreignObject>
          </g>
        );
      }
      case "comment_pin": {
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)}>
            <circle cx={w / 2} cy={h / 2} r={Math.min(w, h) / 2} fill="rgba(121, 192, 255, 0.2)" stroke={stroke} strokeWidth={strokeWidth} />
            <text x={w / 2} y={h / 2 + 4} textAnchor="middle" fontSize={10 * viewport.zoom} fill="var(--cel-cyan, #79c0ff)" fontFamily="ui-monospace, monospace">
          💬
            </text>
          </g>
        );
      }
      case "shape":
      case "image":
      case "embed":
      default:
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`}>
            <rect width={w} height={h} fill="var(--cel-ink, #21262d)" stroke={stroke} strokeWidth={strokeWidth} rx={4} />
          </g>
        );
    }
  };

  // render frame(画布分区)
  const renderFrame = (frame: typeof canvas.frames[number]) => {
    const pos = worldToScreen(frame.x, frame.y);
    const w = frame.width * viewport.zoom;
    const h = frame.height * viewport.zoom;
    return (
      <g key={frame.id} transform={`translate(${pos.x}, ${pos.y})`}>
        <rect width={w} height={h} fill="color-mix(in srgb, var(--cel-surface-stage, #0b0d10) 10%, transparent)" stroke="var(--cel-ink, #21262d)" strokeWidth={1} strokeDasharray="4 4" rx={6} />
        <text x={10} y={16 * viewport.zoom} fontSize={11 * viewport.zoom} fill="var(--cel-text-secondary, #8b949e)" fontFamily="system-ui">
          {frame.title}
        </text>
        {frame.is_slide && (
          <text x={w - 30 * viewport.zoom} y={16 * viewport.zoom} fontSize={9 * viewport.zoom} fill="var(--cel-text-mute, #6e7681)" fontFamily="system-ui">
            [slide]
          </text>
        )}
      </g>
    );
  };

  // render connector(bezier 复用 SmView 算法)
  const renderConnector = (c: CanvasConnectorView) => {
    const from = displayElements.find((e) => e.id === c.from_element_id);
    const to = displayElements.find((e) => e.id === c.to_element_id);
    if (!from || !to) return null;
    const fx = from.x + from.width / 2;
    const fy = from.y + from.height / 2;
    const tx = to.x + to.width / 2;
    const ty = to.y + to.height / 2;
    const dx = tx - fx;
    const dy = ty - fy;
    let path: string;
    if (c.routing === "straight") {
      path = `M ${fx} ${fy} L ${tx} ${ty}`;
    } else if (c.routing === "orthogonal") {
      const midX = fx + dx / 2;
      path = `M ${fx} ${fy} L ${midX} ${fy} L ${midX} ${ty} L ${tx} ${ty}`;
    } else {
      // curved (bezier, 复用 SmView 算法)
      const c1x = fx + dx * 0.25;
      const c1y = fy + dy * 0.1;
      const c2x = tx - dx * 0.25;
      const c2y = ty - dy * 0.1;
      path = `M ${fx} ${fy} C ${c1x} ${c1y}, ${c2x} ${c2y}, ${tx} ${ty}`;
    }
    // 转为屏幕坐标
    const fromScreen = worldToScreen(0, 0);
    const fx_s = (fx - viewport.x) * viewport.zoom;
    const fy_s = (fy - viewport.y) * viewport.zoom;
    const tx_s = (tx - viewport.x) * viewport.zoom;
    const ty_s = (ty - viewport.y) * viewport.zoom;
    let screenPath: string;
    if (c.routing === "straight") {
      screenPath = `M ${fx_s} ${fy_s} L ${tx_s} ${ty_s}`;
    } else if (c.routing === "orthogonal") {
      const midX = fx_s + (tx_s - fx_s) / 2;
      screenPath = `M ${fx_s} ${fy_s} L ${midX} ${fy_s} L ${midX} ${ty_s} L ${tx_s} ${ty_s}`;
    } else {
      const dx_s = tx_s - fx_s;
      const dy_s = ty_s - fy_s;
      const c1x = fx_s + dx_s * 0.25;
      const c1y = fy_s + dy_s * 0.1;
      const c2x = tx_s - dx_s * 0.25;
      const c2y = ty_s - dy_s * 0.1;
      screenPath = `M ${fx_s} ${fy_s} C ${c1x} ${c1y}, ${c2x} ${c2y}, ${tx_s} ${ty_s}`;
    }
    const midScreenX = (fx_s + tx_s) / 2;
    const midScreenY = (fy_s + ty_s) / 2;
    return (
      <g key={c.id}>
        <path
          d={screenPath}
          fill="none"
          stroke={c.color}
          strokeWidth={c.width * viewport.zoom}
          markerEnd={c.arrow_end ? "url(#canvas-arrow)" : undefined}
          markerStart={c.arrow_start ? "url(#canvas-arrow-start)" : undefined}
        />
        {c.label && (
          <g transform={`translate(${midScreenX}, ${midScreenY})`}>
            <rect x={-c.label.length * 3.5} y={-8} width={c.label.length * 7} height={14} fill="var(--cel-surface-stage, #0b0d10)" stroke={c.color} rx={3} />
            <text textAnchor="middle" y={3} fontSize={9} fill={c.color} fontFamily="ui-monospace, monospace">
          {c.label}
            </text>
          </g>
        )}
      </g>
    );
  };

  // minimap(右下角,显示 viewport 范围)
  const allX = displayElements.map((e) => e.x);
  const allY = displayElements.map((e) => e.y);
  const minX = allX.length > 0 ? Math.min(...allX) - 100 : 0;
  const minY = allY.length > 0 ? Math.min(...allY) - 100 : 0;
  const maxX = allX.length > 0 ? Math.max(...allX.map((x, i) => x + displayElements[i].width)) + 100 : 1200;
  const maxY = allY.length > 0 ? Math.max(...allY.map((y, i) => y + displayElements[i].height)) + 100 : 800;

  return (
    <div data-testid="canvas-container" className="relative w-full h-full bg-bg overflow-hidden">
      {/* Toolbar */}
      <div data-testid="canvas-toolbar" className="absolute top-3 left-1/2 -translate-x-1/2 z-20 flex items-center gap-1 bg-bg-card border-2 border-[var(--cel-ink)] rounded-md p-1 cel-shadow">
        <button onClick={() => setTool("select")} className={`btn p-1.5 ${tool === "select" ? "border-accent text-accent" : ""}`} title={t.ariaLabels.canvasSelect}>
          <MousePointer2 size={14} />
        </button>
        <button onClick={() => setTool("pan")} className={`btn p-1.5 ${tool === "pan" ? "border-accent text-accent" : ""}`} title={t.ariaLabels.canvasPan}>
          <Hand size={14} />
        </button>
        {/* 新增元素工具 (per 2026-10-01 OOB 恢复丢失的画笔/创建功能) - 仅 write mode 显示 */}
        {!readOnly && onCreateElement && (
          <>
            <div className="w-px h-5 bg-line" />
            <button
          onClick={() => setTool("sticky")}
          className={`btn p-1.5 ${tool === "sticky" ? "border-accent text-accent" : ""}`}
          title="新增便利贴 (sticky note)"
          data-testid="canvas-tool-sticky"
            >
          <StickyNote size={14} />
            </button>
            <button
          onClick={() => setTool("text")}
          className={`btn p-1.5 ${tool === "text" ? "border-accent text-accent" : ""}`}
          title="新增文字 (text)"
          data-testid="canvas-tool-text"
            >
          <Type size={14} />
            </button>
            <button
          onClick={() => setTool("shape")}
          className={`btn p-1.5 ${tool === "shape" ? "border-accent text-accent" : ""}`}
          title="新增图形 (shape)"
          data-testid="canvas-tool-shape"
            >
          <Square size={14} />
            </button>
            <button onClick={() => setTool("frame-select")} className={`btn p-1.5 ${tool === "frame-select" ? "border-accent text-accent" : ""}`} title="选中 frame 后插入 (下一阶段)" data-testid="canvas-tool-frame-select">
              <Frame size={14} />
            </button>
            {/* 自由画笔 (per 2026-10-01 OOB 恢复丢失的画笔功能) */}
            <button
              onClick={() => setTool("brush")}
              className={`btn p-1.5 ${tool === "brush" ? "border-accent text-accent" : ""}`}
              title="自由画笔 (brush)"
              data-testid="canvas-tool-brush"
            >
              <Brush size={14} />
            </button>
            {/* 画笔调色盘 + 笔刷大小 - 选 brush tool 后展开 */}
            {tool === "brush" && (
              <>
                <div className="w-px h-5 bg-line" />
                <div className="flex items-center gap-1" data-testid="canvas-brush-palette">
                  {BRUSH_PALETTE.map((c) => (
                    <button
                      key={c}
                      onClick={() => setBrushColor(c)}
                      className={`w-4 h-4 rounded-sm border ${brushColor === c ? "border-accent" : "border-line"}`}
                      style={{ background: c }}
                      aria-label={`brush color ${c}`}
                    />
                  ))}
                </div>
                <div className="flex items-center gap-1" data-testid="canvas-brush-size">
                  {BRUSH_SIZES.map((s) => (
                    <button
                      key={s}
                      onClick={() => setBrushSize(s)}
                      className={`btn px-1.5 py-0.5 text-[10px] ${brushSize === s ? "border-accent text-accent" : ""}`}
                      aria-label={`brush size ${s}px`}
                    >
                      {s}
                    </button>
                  ))}
                </div>
              </>
            )}
          </>
        )}
        <div className="w-px h-5 bg-line" />
        <button onClick={() => updateViewport({ ...viewport, zoom: Math.min(4, viewport.zoom * 1.2) })} className="btn p-1.5" title={t.ariaLabels.canvasZoomIn}>
          <ZoomIn size={14} />
        </button>
        <button onClick={() => updateViewport({ ...viewport, zoom: Math.max(0.1, viewport.zoom / 1.2) })} className="btn p-1.5" title={t.ariaLabels.canvasZoomOut}>
          <ZoomOut size={14} />
        </button>
        <button onClick={() => {
          // fit to content
          updateViewport({ x: minX, y: minY, zoom: Math.min(1200 / (maxX - minX), 800 / (maxY - minY), 1) });
        }} className="btn p-1.5" title={t.ariaLabels.canvasFit}>
          <Maximize2 size={14} />
        </button>
        <span className="text-[10px] text-ink-dim font-mono px-2">{Math.round(viewport.zoom * 100)}%</span>
        <div className="w-px h-5 bg-line" />
        {selected.length > 0 && (!readOnly || onDeleteElements) && (
          <button onClick={() => void deleteSelection()} disabled={selected.some((id) => elements.some((element) => element.id === id && element.locked))} className="btn p-1.5 text-err disabled:opacity-40" title={t.ariaLabels.canvasDelete}>
            <Trash2 size={14} />
          </button>
        )}
      </div>

      {/* SVG Canvas */}
      <svg
        ref={svgRef}
        data-testid="canvas-svg"
        viewBox="0 0 1200 800"
        className="w-full h-full"
        style={{ cursor: tool === "pan" ? "grab" : "default", backgroundColor: "var(--cel-surface-stage, #0b0d10)", backgroundImage: "radial-gradient(circle, var(--cel-ink, #21262d) 1px, transparent 1px)", backgroundSize: "20px 20px" }}
        onMouseDown={onMouseDown}
            onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        onWheel={onWheel}
        onClick={(e) => {
          // 新增元素工具 (sticky / text / shape) 在 SVG 空白区点击触发
          // - 仅 write mode (非 readOnly + 有 onCreateElement)
          // - 仅 target=SVG 本身 (避免在元素上误触)
          if (readOnly || !onCreateElement) return;
          if ((e.target as Element).tagName?.toLowerCase() !== "svg") return;
          if (tool === "sticky") {
            void createElementAt(e, "sticky_note", 180, 100, { color: STICKY_PALETTE[0], text: "" });
          } else if (tool === "text") {
            void createElementAt(e, "text", 200, 60, { text: "" });
          } else if (tool === "shape") {
            void createElementAt(e, "shape", 120, 120, {});
          }
        }}
      >
        <defs>
          <marker id="canvas-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="context-stroke" />
          </marker>
          <marker id="canvas-arrow-start" viewBox="0 0 10 10" refX="1" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 10 0 L 0 5 L 10 10 z" fill="context-stroke" />
          </marker>
        </defs>

        {/* Frame (画布分区) */}
        {canvas.frames.map((f) => (
          <g key={f.id} data-testid={`canvas-frame-${f.id}`}>
            {renderFrame(f)}
          </g>
        ))}

        {/* Connector(在 element 下面) */}
        {connectors.map(renderConnector)}

        {/* 自由画笔 in-progress path (per 2026-10-01 OOB 恢复丢失的画笔功能) */}
        {drawingPath && drawingPath.points.length >= 2 && (() => {
          const PAD = Math.max(2, brushSize);
          const x = drawingPath.minX - PAD;
          const y = drawingPath.minY - PAD;
          const d = "M " + drawingPath.points.map((p) => `${(p.x - x).toFixed(1)} ${(p.y - y).toFixed(1)}`).join(" L ");
          return (
            <g transform={`translate(${worldToScreen(x, y).x}, ${worldToScreen(x, y).y})`} data-testid="canvas-drawing-path" pointerEvents="none">
              <path d={d} stroke={brushColor} strokeWidth={brushSize * viewport.zoom} fill="none" strokeLinecap="round" strokeLinejoin="round" opacity={0.85} />
            </g>
          );
        })()}

        {/* Element */}
        {displayElements.map((el) => (
          <g key={`wrapper-${el.id}`} data-testid={`canvas-element-${el.id}`}>
            {renderElement(el)}
          </g>
        ))}
      </svg>

      {/* Minimap */}
      <div data-testid="canvas-minimap" className="absolute bottom-3 right-3 z-20 w-40 h-28 bg-bg-card border-2 border-[var(--cel-ink)] rounded-md overflow-hidden cel-shadow">
        <svg viewBox={`${minX} ${minY} ${maxX - minX} ${maxY - minY}`} className="w-full h-full">
          {/* viewport rect */}
          <rect
            x={viewport.x}
            y={viewport.y}
            width={1200 / viewport.zoom}
            height={800 / viewport.zoom}
            fill="none"
            stroke="var(--cel-cyan, #2f81f7)"
            strokeWidth={2}
          />
          {/* elements dots */}
          {displayElements.map((e) => (
            <rect key={e.id} x={e.x} y={e.y} width={e.width} height={e.height} fill="#3fb950" opacity={0.6} />
          ))}
        </svg>
      </div>

      {/* Status bar */}
      <div className="absolute bottom-3 left-3 z-20 text-[10px] text-ink-mute font-mono flex gap-3">
        <span>zoom {Math.round(viewport.zoom * 100)}%</span>
        <span>elements {elements.length}</span>
        <span>connectors {connectors.length}</span>
        <span>selected {selected.length}</span>
      </div>
    </div>
  );
}
