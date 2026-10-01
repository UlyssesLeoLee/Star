"use client";

// =====================================================================
// AgentCanvasView — 3渲2 战术全息无限画布 (Miro / Tactical Holo-Board)
// =====================================================================
// Per 2026-09-06 用户发令:
//   - 增强游戏界面感 (Game UI / Tactical Holo-Canvas)
//   - 保持低认知负荷的极简克制排版规范
//   - 3渲2 (3D-to-2D NPR Cel-Shaded) 日漫战术科技风格
//   - 切角装甲节点 + 阶梯分段血槽 + 流动战术数据链路
//   - 任务悬赏卡 (Bounty Card) + 撃破 CLEARED 漫画印记
// =====================================================================

import type {
  AgentSession, Worktree, WorkItem, AgentStatus, WorkItemStatus,
} from "@/types/ids";
import type {
  AgentCanvas, AgentCanvasNode, AgentCanvasConnector,
  AgentCanvasAnnotation, AgentCanvasFreeConnector,
} from "@/lib/agent-view/types";
import type { AgentGameState } from "@/lib/agent-game/types";
import { visualForLevel, MAX_HP } from "@/lib/agent-game/types";
import { AgentCharacterSVG } from "@/lib/agent-game/characters";
import { EnemyOrbForPrioritySVG } from "@/lib/agent-game/enemies";
import { useAgentGameTheme } from "@/lib/agent-game/theme-tokens";
import { EnergyRing, HaloArc, Stamp, GodSeal } from "@/components/agent-game/Decorations";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { StatusPill } from "@/components/StatusPill";
import { useStore } from "@/lib/store";
import {
  Hand, MousePointer2, ZoomIn, ZoomOut, Maximize2, GitBranch, Skull, Coins,
  StickyNote, Type, Square, Brush, Plus, Trash2,
} from "lucide-react";
import { useTranslation } from "@/lib/i18n";

interface AgentCanvasViewProps {
  canvas: AgentCanvas;
  agent: AgentSession;
  worktree: Worktree | null;
  /** 拟人化游戏化 (per 2026-09-05 11:42 JST 拍板) */
  gameState: AgentGameState | null;
  /** 领奖回调 (work-item done + 未领奖时) */
  onClaim?: (workItemId: string) => void;
  /** 用户注释 (per 2026-10-01 OOB 恢复无限画布画笔 - sticky/text/shape/path/connector */
  annotations?: AgentCanvasAnnotation[];
  freeConnectors?: AgentCanvasFreeConnector[];
  /** 注释回调 (默认 read-only: 节点 + 注释都不编辑) */
  readOnly?: boolean;
  onCreateAnnotation?: (body: AgentCanvasAnnotation) => Promise<string | void>;
  onDeleteAnnotation?: (id: string) => Promise<void>;
  onAnnotationPositionChange?: (body: { id: string; x: number; y: number }) => Promise<void>;
  onCreateFreeConnector?: (body: Omit<AgentCanvasFreeConnector, "id">) => Promise<string | void>;
  onDeleteFreeConnector?: (id: string) => Promise<void>;
  /** 注释更新回调 (per 任务 #2 — text edit mode) */
  onUpdateAnnotation?: (id: string, body: Partial<AgentCanvasAnnotation>) => Promise<void>;
  /** 注释批量删除回调 (per 任务 #4 — 多选/框选) */
  onBulkDeleteAnnotation?: (ids: string[]) => Promise<void>;
}

export function AgentCanvasView({
  canvas, agent, worktree, gameState, onClaim,
  annotations = [], freeConnectors = [],
  readOnly = true,
  onCreateAnnotation, onDeleteAnnotation, onAnnotationPositionChange,
  onCreateFreeConnector, onDeleteFreeConnector, onUpdateAnnotation, onBulkDeleteAnnotation,
}: AgentCanvasViewProps) {
  const { t } = useTranslation();
  const workItems = useStore((s) => s.workItems);
  const workItemById = useMemo(
    () => new Map(workItems.map((w) => [w.id, w] as const)),
    [workItems],
  );
  const { colors, mode } = useAgentGameTheme();
  const [viewport, setViewport] = useState(canvas.viewport);
  const [tool, setTool] = useState<"select" | "pan" | "sticky" | "text" | "shape" | "brush" | "connector" | "connect-source">("pan");
  const [connectSourceId, setConnectSourceId] = useState<string | null>(null);
  // 自由画笔 (per 2026-10-01 OOB 恢复无限画布画笔)
  const AGENT_BRUSH_PALETTE = ["#e6edf3", "#00f0ff", "#ffc400", "#ff184c", "#a5d6ff"]; // 主题色 (cyan/gold/red) + 中性
  const AGENT_BRUSH_SIZES = [2, 4, 8, 12];
  const AGENT_STICKY_PALETTE = ["#f9d77e", "#ffb3c1", "#a3d9ff", "#b8f0c4", "#d4b3ff"];
  const [brushColor, setBrushColor] = useState<string>(AGENT_BRUSH_PALETTE[0]);
  const [brushSize, setBrushSize] = useState<number>(AGENT_BRUSH_SIZES[1]);
  const [drawingPath, setDrawingPath] = useState<{
    points: Array<{ x: number; y: number }>;
    minX: number; minY: number; maxX: number; maxY: number;
  } | null>(null);
  const [annotationError, setAnnotationError] = useState<string | null>(null);
  const [annotationPending, setAnnotationPending] = useState(false);
  const [selectedAnnotationId, setSelectedAnnotationId] = useState<string | null>(null);
  // 多选 (per 任务 #4): shift-click 加选 / marquee 框选
  const [multiSelected, setMultiSelected] = useState<Set<string>>(new Set());
  // marquee 框选 (per 任务 #4)
  const [marquee, setMarquee] = useState<{
    startWorldX: number;
    startWorldY: number;
    endWorldX: number;
    endWorldY: number;
  } | null>(null);
  // 编辑模式 — 点选 sticky_note/text 2 次进入 (per 任务 #2); 第一次=select, 第二次=edit
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editText, setEditText] = useState<string>("");

  // 编辑生命周期 (per 任务 #2): commit 写在 onBlur / Enter, exit 写在 Esc / outside-click
  const beginEditAnnotation = (a: AgentCanvasAnnotation) => {
    if (a.kind !== "sticky_note" && a.kind !== "text") return;
    const text = (a.content as { text?: string }).text ?? "";
    setEditingId(a.id);
    setEditText(text);
  };
  const commitEditAnnotation = async () => {
    if (!editingId || !onUpdateAnnotation) return;
    await onUpdateAnnotation(editingId, { content: { text: editText } } as Partial<AgentCanvasAnnotation>);
    setEditingId(null);
  };
  const cancelEditAnnotation = () => {
    setEditingId(null);
    setEditText("");
  };
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  const dragState = useRef<{
    type: "pan" | "annotation" | null;
    startX: number;
    startY: number;
    /** pan: viewport.x at start; annotation: annotation.x at start */
    elX: number;
    /** pan: viewport.y at start; annotation: annotation.y at start */
    elY: number;
    /** annotation drag 专用 — annotation id */
    annId: string | null;
  }>({
    type: null, startX: 0, startY: 0, elX: 0, elY: 0, annId: null,
  });
  /** 拖拽中 — preview position (per 任务 #3: drag annotation) */
  const [dragAnnPreview, setDragAnnPreview] = useState<{ id: string; x: number; y: number } | null>(null);

  useEffect(() => {
    setViewport(canvas.viewport);
  }, [canvas.viewport, canvas.derivedAt]);

  // 键盘快捷键: V=select, H=pan, +=zoom in, --=zoom out, 1=fit
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
      if (e.key === "v" || e.key === "V") setTool("select");
      else if (e.key === "h" || e.key === "H") setTool("pan");
      else if (e.key === "+" || e.key === "=") {
        setViewport((v) => ({ ...v, zoom: Math.min(4, v.zoom * 1.2) }));
      } else if (e.key === "-") {
        setViewport((v) => ({ ...v, zoom: Math.max(0.1, v.zoom / 1.2) }));
      } else if (e.key === "1") {
        setViewport(canvas.viewport);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [canvas.viewport]);

  
  /**
   * 屏幕坐标 → 世界坐标 (考虑 viewport.zoom + viewport.x/y)
   * per 2026-10-01 OOB 自由画笔/创建功能 — 跟独立 CanvasView 公式一致
   */
  const svgPointFromClient = (e: React.MouseEvent | MouseEvent): { x: number; y: number } | null => {
    if (!svgRef.current) return null;
    const rect = svgRef.current.getBoundingClientRect();
    return {
      x: (e.clientX - rect.left) / viewport.zoom + viewport.x,
      y: (e.clientY - rect.top) / viewport.zoom + viewport.y,
    };
  };

  /**
   * 创建 sticky / text / shape annotation (per 2026-10-01 OOB 恢复无限画布画笔)
   * - 鼠标在 SVG 空白区点击触发
   * - 仅 write mode (非 readOnly + 有 onCreateAnnotation)
   */
  const createAnnotationAt = async (
    e: React.MouseEvent,
    kind: "sticky_note" | "text" | "shape",
    width: number,
    height: number,
    content: AgentCanvasAnnotation["content"],
  ) => {
    if (readOnly || !onCreateAnnotation || annotationPending) return;
    const world = svgPointFromClient(e);
    if (!world) return;
    setAnnotationError(null);
    setAnnotationPending(true);
    try {
      const id = await onCreateAnnotation({
        id: `ann-local-${Math.random().toString(36).slice(2, 10)}`,
        kind,
        x: Math.round(world.x - width / 2),
        y: Math.round(world.y - height / 2),
        width,
        height,
        created_at: new Date().toISOString(),
        created_by: "usr-001",
        content,
      } as AgentCanvasAnnotation);
      if (id) setSelectedAnnotationId(id);
      setTool("select");
    } catch (err) {
      setAnnotationError(err instanceof Error ? err.message : "Failed to create annotation");
    } finally {
      setAnnotationPending(false);
    }
  };

  /**
   * 自由画笔 — 创建 path annotation
   * - 鼠标按下记录起点, 拖动累积, 抬起 → 创建
   */
  const createPathAnnotation = async (path: NonNullable<typeof drawingPath>) => {
    if (readOnly || !onCreateAnnotation || annotationPending) return;
    if (path.points.length < 2) {
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
    setAnnotationError(null);
    setAnnotationPending(true);
    try {
      await onCreateAnnotation({
        id: `ann-local-${Math.random().toString(36).slice(2, 10)}`,
        kind: "path",
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(width),
        height: Math.round(height),
        created_at: new Date().toISOString(),
        created_by: "usr-001",
        content: {
          path_data: pathData,
          brush_size: brushSize,
          brush_color: brushColor,
        },
      } as AgentCanvasAnnotation);
    } catch (err) {
      setAnnotationError(err instanceof Error ? err.message : "Failed to save brush stroke");
    } finally {
      setDrawingPath(null);
      setAnnotationPending(false);
    }
  };

  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button === 1 || (e.button === 0 && tool === "pan") || e.shiftKey) {
      dragState.current = { type: "pan", startX: e.clientX, startY: e.clientY, elX: viewport.x, elY: viewport.y, annId: null };
      return;
    }
    if (e.button === 0 && tool === "select") {
      // 拖拽已选 annotation (per 任务 #3: drag annotation)
      if (selectedAnnotationId && !readOnly && onAnnotationPositionChange) {
        const target = annotations.find((a) => a.id === selectedAnnotationId);
        if (target) {
          dragState.current = { type: "annotation", startX: e.clientX, startY: e.clientY, elX: target.x, elY: target.y, annId: target.id };
          setDragAnnPreview({ id: target.id, x: target.x, y: target.y });
          return;
        }
      }
      // 没有点中 annotation → 进入 marquee 框选 (per 任务 #4)
      if (!readOnly && (e.target as Element).tagName?.toLowerCase() === "svg") {
        const wp = svgPointFromClient(e);
        if (wp) {
          setSelectedNodeId(null);
          setSelectedAnnotationId(null);
          if (!e.shiftKey) setMultiSelected(new Set());
          setMarquee({ startWorldX: wp.x, startWorldY: wp.y, endWorldX: wp.x, endWorldY: wp.y });
          return;
        }
      }
      setSelectedNodeId(null);
      setSelectedAnnotationId(null);
      return;
    }
    if (e.button === 0 && tool === "brush" && !readOnly) {
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
      setViewport({ ...viewport, x: ds.elX - dx, y: ds.elY - dy });
    } else if (ds.type === "annotation" && ds.annId && dragAnnPreview) {
      // 拖拽 annotation — 实时更新 preview (per 任务 #3)
      const dx = (e.clientX - ds.startX) / viewport.zoom;
      const dy = (e.clientY - ds.startY) / viewport.zoom;
      setDragAnnPreview({ id: ds.annId, x: ds.elX + dx, y: ds.elY + dy });
    } else if (marquee) {
      // marquee 框选 (per 任务 #4)
      const wp = svgPointFromClient(e);
      if (wp) setMarquee({ ...marquee, endWorldX: wp.x, endWorldY: wp.y });
    } else if (drawingPath) {
      // 自由画笔 - 累积 path points (per 2026-10-01 OOB)
      const svgPt = svgPointFromClient(e);
      if (!svgPt) return;
      const last = drawingPath.points[drawingPath.points.length - 1];
      const dist = Math.hypot(svgPt.x - last.x, svgPt.y - last.y);
      if (dist < 1.5) return;
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
    dragState.current = { type: null, startX: 0, startY: 0, elX: 0, elY: 0, annId: null };
    // marquee 提交 (per 任务 #4)
    if (marquee) {
      const minX = Math.min(marquee.startWorldX, marquee.endWorldX);
      const maxX = Math.max(marquee.startWorldX, marquee.endWorldX);
      const minY = Math.min(marquee.startWorldY, marquee.endWorldY);
      const maxY = Math.max(marquee.startWorldY, marquee.endWorldY);
      const hits = annotations
        .filter((a) => a.x + a.width >= minX && a.x <= maxX && a.y + a.height >= minY && a.y <= maxY)
        .map((a) => a.id);
      setMultiSelected((prev) => {
        const next = new Set(prev);
        hits.forEach((id) => next.add(id));
        return next;
      });
      setMarquee(null);
      return;
    }
    // 拖拽 annotation 提交 (per 任务 #3)
    if (ds.type === "annotation" && ds.annId && dragAnnPreview && onAnnotationPositionChange) {
      if (dragAnnPreview.x !== ds.elX || dragAnnPreview.y !== ds.elY) {
        void onAnnotationPositionChange({ id: ds.annId, x: dragAnnPreview.x, y: dragAnnPreview.y });
      }
      setDragAnnPreview(null);
      return;
    }
    setDragAnnPreview(null);
    // 自由画笔 - 抬起提交 (per 2026-10-01 OOB)
    if (drawingPath) {
      void createPathAnnotation(drawingPath);
      return;
    }
  };

  const onWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.1, Math.min(4, viewport.zoom * delta));
    if (svgRef.current) {
      const rect = svgRef.current.getBoundingClientRect();
      const sx = e.clientX - rect.left;
      const sy = e.clientY - rect.top;
      const wx = sx / viewport.zoom + viewport.x;
      const wy = sy / viewport.zoom + viewport.y;
      setViewport({ x: wx - sx / newZoom, y: wy - sy / newZoom, zoom: newZoom });
    }
  };

  const onNodeDoubleClick = (ref: AgentCanvasNode["ref"]) => {
    if (ref.kind === "agent") {
      window.location.href = `/agent?selected=${ref.agentId}`;
    } else if (ref.kind === "worktree") {
      window.location.href = `/worktree?selected=${ref.worktreeId}`;
    } else if (ref.kind === "work_item") {
      window.location.href = `/work-item?selected=${ref.workItemId}`;
    }
  };

  const onNodeClick = (e: React.MouseEvent, nodeId: string) => {
    e.stopPropagation();
    if (tool === "pan") return;
    setSelectedNodeId(nodeId);
  };

  // 渲染战术数据流 connector (带硬黑描边 + 内部高亮 + 脉动粒子)
  const renderConnector = (c: AgentCanvasConnector) => {
    const from = canvas.nodes.find((n) => n.id === c.fromNodeId);
    const to = canvas.nodes.find((n) => n.id === c.toNodeId);
    if (!from || !to) return null;
    const fx = from.x + from.width / 2;
    const fy = from.y + from.height / 2;
    const tx = to.x + to.width / 2;
    const ty = to.y + to.height / 2;

    const fx_s = (fx - viewport.x) * viewport.zoom;
    const fy_s = (fy - viewport.y) * viewport.zoom;
    const tx_s = (tx - viewport.x) * viewport.zoom;
    const ty_s = (ty - viewport.y) * viewport.zoom;

    const dx_s = tx_s - fx_s;
    const dy_s = ty_s - fy_s;
    const c1x_s = fx_s + dx_s * 0.3;
    const c1y_s = fy_s + dy_s * 0.05;
    const c2x_s = tx_s - dx_s * 0.3;
    const c2y_s = ty_s - dy_s * 0.05;
    const path = `M ${fx_s} ${fy_s} C ${c1x_s} ${c1y_s}, ${c2x_s} ${c2y_s}, ${tx_s} ${ty_s}`;

    const midX = (fx_s + tx_s) / 2;
    const midY = (fy_s + ty_s) / 2;

    return (
      <g key={c.id} data-testid={`agent-canvas-connector-${c.id}`}>
        {/* 1. 硬墨底层描边 */}
        <path
          d={path}
          fill="none"
          stroke="#000000"
          strokeWidth={3.5 * viewport.zoom}
          opacity={0.8}
        />
        {/* 2. 主体彩色导线 */}
        <path
          d={path}
          fill="none"
          stroke={c.color}
          strokeWidth={1.8 * viewport.zoom}
          opacity={0.75}
          markerEnd="url(#tactical-arrow)"
        />
        {/* 3. 战术流动数据脉冲粒子 */}
        <path
          d={path}
          fill="none"
          stroke="#ffffff"
          strokeWidth={1.5 * viewport.zoom}
          strokeDasharray={`${6 * viewport.zoom} ${18 * viewport.zoom}`}
          opacity={0.8}
        >
          <animate
            attributeName="stroke-dashoffset"
            from="48"
            to="0"
            dur="2s"
            repeatCount="indefinite"
          />
        </path>
        {/* 4. 战术标签胶囊 */}
        {c.label && (
          <g transform={`translate(${midX}, ${midY - 6})`}>
            <rect
              x={-((c.label.length * 5.5 + 8) * viewport.zoom) / 2}
              y={-7 * viewport.zoom}
              width={(c.label.length * 5.5 + 8) * viewport.zoom}
              height={14 * viewport.zoom}
              fill="#080c14"
              stroke="#000000"
              strokeWidth={1 * viewport.zoom}
              rx={2 * viewport.zoom}
            />
            <text
              x={0}
              y={3 * viewport.zoom}
              textAnchor="middle"
              fontSize={8.5 * viewport.zoom}
              fill={c.color}
              fontFamily='"JetBrains Mono", monospace'
              fontWeight="bold"
            >
              {c.label}
            </text>
          </g>
        )}
      </g>
    );
  };

  // 渲染节点
  const renderNode = (node: AgentCanvasNode) => {
    const posX = (node.x - viewport.x) * viewport.zoom;
    const posY = (node.y - viewport.y) * viewport.zoom;
    const w = node.width * viewport.zoom;
    const h = node.height * viewport.zoom;
    const isSelected = selectedNodeId === node.id;
    const isHovered = hoveredNodeId === node.id;

    if (node.ref.kind === "agent") {
      const tier = gameState ? visualForLevel(gameState.level) : null;
      const alive = gameState?.alive ?? true;
      const scale = tier?.scale ?? 1;
      const nodeW = w * scale;
      const nodeH = h * scale;
      const offsetX = (w - nodeW) / 2;
      const offsetY = (h - nodeH) / 2;
      const tierColor = alive ? (tier?.color ?? "#00f0ff") : "#6b7280";

      // 10段阶梯血槽
      const totalPips = 8;
      const hpRatio = gameState ? gameState.hp / MAX_HP : 1;
      const filledPips = Math.ceil(hpRatio * totalPips);

      return (
        <g
          key={node.id}
          data-testid={`agent-canvas-node-${node.id}`}
          transform={`translate(${posX + offsetX}, ${posY + offsetY})`}
          style={{ cursor: "pointer" }}
          onMouseDown={(e) => onNodeClick(e, node.id)}
          onMouseEnter={() => setHoveredNodeId(node.id)}
          onMouseLeave={() => setHoveredNodeId(null)}
          onDoubleClick={() => onNodeDoubleClick(node.ref)}
        >
          {/* 神侠战术光环 (Lv 7+) */}
          {tier && tier.level >= 7 && (
            <HaloArc level={gameState?.level ?? 1} cx={nodeW / 2} cy={nodeH / 2} size={nodeW * 0.55} />
          )}
          {/* 能量脉冲环 */}
          <EnergyRing cx={nodeW / 2} cy={nodeH / 2} color={tierColor} radius={nodeW * 0.52} />

          {/* 战术切角装甲外框 (3渲2 Cel Card) */}
          <TacticalChamferNode
            w={nodeW}
            h={nodeH}
            zoom={viewport.zoom}
            stroke={isSelected ? "#ffc400" : isHovered ? "#00f0ff" : "#000000"}
            strokeWidth={2 * viewport.zoom}
            fill={alive ? "#0f1422" : "#131720"}
            glowColor={isSelected ? "#ffc400" : isHovered ? "#00f0ff" : undefined}
          />

          {/* 顶部战术微标头 [SEC:01 // COMMAND UNIT] */}
          <g transform={`translate(${10 * viewport.zoom}, ${12 * viewport.zoom})`}>
            <circle
              cx={4 * viewport.zoom}
              cy={-1 * viewport.zoom}
              r={2.5 * viewport.zoom}
              fill={alive ? "#00ff9d" : "#ff184c"}
            />
            <text
              x={10 * viewport.zoom}
              y={2 * viewport.zoom}
              fontSize={8 * viewport.zoom}
              fill="#94a3b8"
              fontFamily='"JetBrains Mono", monospace'
              fontWeight="bold"
              letterSpacing={0.5}
            >
              COMMAND UNIT // {agent.agent_kind.toUpperCase()}
            </text>
          </g>

          {/* 3渲2 日漫战术角色 SVG */}
          {gameState && (
            <g transform={`translate(${nodeW * 0.5 - 32 * viewport.zoom}, ${nodeH * 0.44 - 32 * viewport.zoom})`}>
              <AgentCharacterSVG
                level={gameState.level}
                scale={viewport.zoom * 1.05}
                dead={!alive}
                stampText="侠"
                showDivineHalo={gameState.level >= 7}
              />
            </g>
          )}

          {/* 印章 (Lv 5+) */}
          {gameState && gameState.level >= 5 && (
            <Stamp text="M" cx={nodeW - 16 * viewport.zoom} cy={16 * viewport.zoom} color={tierColor} size={16 * viewport.zoom} />
          )}

          {/* 神印 (Lv 10) */}
          {gameState && gameState.level >= 10 && (
            <GodSeal level={gameState.level} cx={16 * viewport.zoom} cy={16 * viewport.zoom} size={20 * viewport.zoom} />
          )}

          {/* 战术阶梯血槽 (Stepped Segmented HP Gauge) */}
          {gameState && (
            <g transform={`translate(${14 * viewport.zoom}, ${nodeH - 34 * viewport.zoom})`}>
              <text
                x={0}
                y={-4 * viewport.zoom}
                fontSize={7.5 * viewport.zoom}
                fill="#64748b"
                fontFamily='"JetBrains Mono", monospace'
                fontWeight="bold"
              >
                HP {gameState.hp}/{MAX_HP}
              </text>
              <g transform={`translate(0, 0)`}>
                {Array.from({ length: totalPips }).map((_, i) => {
                  const isFilled = i < filledPips;
                  const isDanger = gameState.hp <= 30;
                  return (
                    <rect
                      key={i}
                      x={i * 8 * viewport.zoom}
                      y={0}
                      width={6 * viewport.zoom}
                      height={4 * viewport.zoom}
                      fill={
                        isFilled
                          ? isDanger
                            ? "#ff184c"
                            : i > 5
                            ? "#00f0ff"
                            : tierColor
                          : "#1e293b"
                      }
                      stroke="#000000"
                      strokeWidth={0.8 * viewport.zoom}
                      transform={`skewX(-15)`}
                    />
                  );
                })}
              </g>
            </g>
          )}

          {/* 等级勋章 (右上角) */}
          {tier && (
            <g transform={`translate(${nodeW - 46 * viewport.zoom}, ${-10 * viewport.zoom})`}>
              <rect
                width={38 * viewport.zoom}
                height={19 * viewport.zoom}
                fill={tierColor}
                stroke="#000000"
                strokeWidth={1.5 * viewport.zoom}
                rx={0}
              />
              <text
                x={19 * viewport.zoom}
                y={13 * viewport.zoom}
                textAnchor="middle"
                fontSize={10.5 * viewport.zoom}
                fill="#000000"
                fontWeight="900"
                fontFamily='"JetBrains Mono", monospace'
              >
                Lv.{tier.level}
              </text>
            </g>
          )}

          {/* Agent ID (底部居中) */}
          {gameState && (
            <text
              x={nodeW * 0.5}
              y={nodeH - 12 * viewport.zoom}
              textAnchor="middle"
              fontSize={8.5 * viewport.zoom}
              fill="#cbd5e1"
              fontFamily='"JetBrains Mono", monospace'
              fontWeight="bold"
              style={{ pointerEvents: "none" }}
            >
              {agent.id}
            </text>
          )}

          {/* 死亡态 Skull Overlay */}
          {!alive && (
            <g transform={`translate(${nodeW / 2 - 16 * viewport.zoom}, ${nodeH / 2 - 16 * viewport.zoom})`}>
              <circle cx={16 * viewport.zoom} cy={16 * viewport.zoom} r={20 * viewport.zoom} fill="rgba(0,0,0,0.85)" stroke="#ff184c" strokeWidth={1.5 * viewport.zoom} />
              <Skull size={32 * viewport.zoom} color="#ff184c" strokeWidth={2} />
            </g>
          )}
        </g>
      );
    }

    if (node.ref.kind === "worktree" && worktree) {
      return (
        <g
          key={node.id}
          data-testid={`agent-canvas-node-${node.id}`}
          transform={`translate(${posX}, ${posY})`}
          style={{ cursor: "pointer" }}
          onMouseDown={(e) => onNodeClick(e, node.id)}
          onMouseEnter={() => setHoveredNodeId(node.id)}
          onMouseLeave={() => setHoveredNodeId(null)}
          onDoubleClick={() => onNodeDoubleClick(node.ref)}
        >
          <TacticalChamferNode
            w={w}
            h={h}
            zoom={viewport.zoom}
            stroke={isSelected ? "#ffc400" : isHovered ? "#00f0ff" : "#000000"}
            strokeWidth={1.8 * viewport.zoom}
            fill="#0f1422"
            glowColor={isSelected ? "#ffc400" : isHovered ? "#00f0ff" : undefined}
          />
          <WorktreeNodeBody worktree={worktree} w={w} h={h} zoom={viewport.zoom} />
        </g>
      );
    }

    if (node.ref.kind === "work_item") {
      const wi = workItemById.get(node.ref.workItemId);
      if (!wi) return null;
      const canClaim = onClaim && wi.status === "done" && gameState?.alive && !gameState.lastClaimAt[wi.id];

      return (
        <g
          key={node.id}
          data-testid={`agent-canvas-node-${node.id}`}
          transform={`translate(${posX}, ${posY})`}
          style={{ cursor: "pointer" }}
          onMouseDown={(e) => onNodeClick(e, node.id)}
          onMouseEnter={() => setHoveredNodeId(node.id)}
          onMouseLeave={() => setHoveredNodeId(null)}
          onDoubleClick={() => onNodeDoubleClick(node.ref)}
        >
          <TacticalChamferNode
            w={w}
            h={h}
            zoom={viewport.zoom}
            stroke={isSelected ? "#ffc400" : isHovered ? "#00f0ff" : "#000000"}
            strokeWidth={1.8 * viewport.zoom}
            fill="#0b0f19"
            glowColor={isSelected ? "#ffc400" : isHovered ? "#00f0ff" : undefined}
          />

          {/* 3渲2 敌人光球 */}
          {wi.status !== "done" && (
            <g transform={`translate(${w * 0.5 - 24 * viewport.zoom}, ${4 * viewport.zoom}) scale(${0.72 * viewport.zoom})`}>
              <EnemyOrbForPrioritySVG priority={wi.priority} />
            </g>
          )}

          {/* 完成态: 动漫战术盖章 [撃破 CLEARED] */}
          {wi.status === "done" && (
            <g transform={`translate(${w * 0.5}, ${20 * viewport.zoom})`}>
              <rect
                x={-28 * viewport.zoom}
                y={-8 * viewport.zoom}
                width={56 * viewport.zoom}
                height={16 * viewport.zoom}
                fill="rgba(0, 255, 157, 0.15)"
                stroke="#00ff9d"
                strokeWidth={1.5 * viewport.zoom}
                transform={`rotate(-5)`}
              />
              <text
                x={0}
                y={4 * viewport.zoom}
                textAnchor="middle"
                fontSize={9 * viewport.zoom}
                fill="#00ff9d"
                fontFamily='"JetBrains Mono", monospace'
                fontWeight="900"
                letterSpacing={1}
                transform={`rotate(-5)`}
              >
                撃破 CLEARED
              </text>
            </g>
          )}

          <WorkItemNodeBody wi={wi} w={w} h={h} zoom={viewport.zoom} />

          {/* Claim 领赏按钮 */}
          {canClaim && (
            <foreignObject
              x={4 * viewport.zoom}
              y={h - 22 * viewport.zoom}
              width={w - 8 * viewport.zoom}
              height={18 * viewport.zoom}
            >
              <button
                data-testid={`claim-btn-${wi.id}`}
                onClick={(e) => {
                  e.stopPropagation();
                  onClaim!(wi.id);
                }}
                className="w-full h-full text-[10px] font-black font-mono uppercase flex items-center justify-center gap-1 bg-[var(--cel-gold,#ffc400)] text-black border border-black hover:brightness-110 active:translate-y-0.5 shadow-[0_0_6px_#ffc400] transition-all"
                style={{ pointerEvents: "all" }}
              >
                <Coins size={10} /> CLAIM +50🪙
              </button>
            </foreignObject>
          )}
        </g>
      );
    }
    return null;
  };

  /**
   * 渲染用户 annotation (per 2026-10-01 OOB 恢复无限画布画笔)
   * - 节点坐标 → 屏幕坐标: (x - viewport.x) * viewport.zoom
   * - sticky/text: SVG rect + text
   * - shape: SVG rect
   * - path: SVG path (相对 bbox 起点)
   */
  const renderAnnotation = (a: AgentCanvasAnnotation) => {
    const isDragging = dragAnnPreview?.id === a.id;
    const ax = isDragging ? dragAnnPreview.x : a.x;
    const ay = isDragging ? dragAnnPreview.y : a.y;
    const sx = (ax - viewport.x) * viewport.zoom;
    const sy = (ay - viewport.y) * viewport.zoom;
    const w = a.width * viewport.zoom;
    const h = a.height * viewport.zoom;
    const isSelected = a.id === selectedAnnotationId;
    const isMultiSelected = multiSelected.has(a.id);
    const isConnectSource = a.id === connectSourceId;
    const outline = isConnectSource
      ? "var(--cel-gold,#ffc400)"
      : isSelected || isMultiSelected
        ? "var(--cel-cyan,#00f0ff)"
        : "var(--cel-ink,#30363d)";
    const baseTestId = `annotation-${a.id}`;
    const onAnnotationClick = (e: React.MouseEvent) => {
      e.stopPropagation();
      if (tool === "connector" && onCreateFreeConnector) {
        if (!connectSourceId) {
          setConnectSourceId(a.id);
        } else if (connectSourceId !== a.id) {
          void onCreateFreeConnector({ fromAnnotationId: connectSourceId, toAnnotationId: a.id, color: "#00f0ff" }).finally(() => {
            setConnectSourceId(null);
            setTool("select");
          });
        }
      } else {
        setSelectedAnnotationId(a.id);
      }
    };
    const baseCursor = tool === "connector" ? "crosshair" : isDragging ? "grabbing" : (tool === "select" && selectedAnnotationId === a.id) ? "grab" : "default";
  const onAnnotationDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (a.kind === "sticky_note" || a.kind === "text") {
      beginEditAnnotation(a);
    }
  };

    switch (a.kind) {
      case "sticky_note": {
        const fill = (a as { content: { color: string; text: string } }).content.color ?? "#f9d77e";
        const text = (a as { content: { color: string; text: string } }).content.text ?? "";
        const isEditing = editingId === a.id;
        return (
          <g key={a.id} data-testid={baseTestId} transform={`translate(${sx}, ${sy})`} style={{ cursor: isEditing ? "text" : baseCursor }} onClick={onAnnotationClick} onDoubleClick={onAnnotationDoubleClick}>
            <rect width={w} height={h} fill={fill} stroke={outline} strokeWidth={isSelected || isConnectSource || isEditing ? 2 : 1} rx={2} />
            <foreignObject x={4} y={4} width={w - 8} height={h - 8}>
              {isEditing ? (
                <textarea
                  data-testid={`annotation-edit-${a.id}`}
                  autoFocus
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  onBlur={() => void commitEditAnnotation()}
                  onKeyDown={(e) => {
                    if (e.key === "Escape") {
                      e.preventDefault();
                      cancelEditAnnotation();
                    } else if (e.key === "Enter" && !e.shiftKey) {
                      e.preventDefault();
                      void commitEditAnnotation();
                    }
                  }}
                  style={{
                    width: "100%",
                    height: "100%",
                    fontSize: 11 * viewport.zoom,
                    color: "#0b0d10",
                    lineHeight: 1.3,
                    fontFamily: "system-ui",
                    background: "transparent",
                    border: "none",
                    outline: "none",
                    resize: "none",
                  }}
                />
              ) : (
                <div style={{ fontSize: 11 * viewport.zoom, color: "#0b0d10", lineHeight: 1.3, fontFamily: "system-ui", wordBreak: "break-word", overflow: "hidden", whiteSpace: "pre-wrap" }}>
                  {text || (isSelected ? "(双击编辑)" : "")}
                </div>
              )}
            </foreignObject>
          </g>
        );
      }
      case "text": {
        const text = (a as { content: { text: string } }).content.text ?? "";
        const isEditing = editingId === a.id;
        return (
          <g key={a.id} data-testid={baseTestId} transform={`translate(${sx}, ${sy})`} style={{ cursor: isEditing ? "text" : baseCursor }} onClick={onAnnotationClick} onDoubleClick={onAnnotationDoubleClick}>
            <foreignObject width={w} height={h}>
              {isEditing ? (
                <textarea
                  data-testid={`annotation-edit-${a.id}`}
                  autoFocus
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  onBlur={() => void commitEditAnnotation()}
                  onKeyDown={(e) => {
                    if (e.key === "Escape") {
                      e.preventDefault();
                      cancelEditAnnotation();
                    } else if (e.key === "Enter" && !e.shiftKey) {
                      e.preventDefault();
                      void commitEditAnnotation();
                    }
                  }}
                  style={{
                    width: "100%",
                    height: "100%",
                    fontSize: 14 * viewport.zoom,
                    color: "var(--cel-text-primary,#e6edf3)",
                    lineHeight: 1.3,
                    fontFamily: "system-ui",
                    background: "rgba(0,0,0,0.4)",
                    border: "1px solid var(--cel-cyan,#00f0ff)",
                    outline: "none",
                    padding: 4,
                    resize: "none",
                  }}
                />
              ) : (
                <div style={{ fontSize: 14 * viewport.zoom, color: "var(--cel-text-primary,#e6edf3)", lineHeight: 1.3, fontFamily: "system-ui", wordBreak: "break-word", overflow: "hidden", textShadow: "0 0 4px #000", whiteSpace: "pre-wrap" }}>
                  {text || (isSelected ? "(双击编辑)" : "")}
                </div>
              )}
            </foreignObject>
          </g>
        );
      }
      case "shape": {
        const shape = (a as { content: { shape: "rect" | "ellipse" } }).content.shape ?? "rect";
        return (
          <g key={a.id} data-testid={baseTestId} transform={`translate(${sx}, ${sy})`} style={{ cursor: baseCursor }} onClick={onAnnotationClick}>
            {shape === "rect" ? (
              <rect width={w} height={h} fill="none" stroke={outline} strokeWidth={2} />
            ) : (
              <ellipse cx={w / 2} cy={h / 2} rx={w / 2} ry={h / 2} fill="none" stroke={outline} strokeWidth={2} />
            )}
          </g>
        );
      }
      case "path": {
        const c = a.content;
        const bw = c.brush_size * viewport.zoom;
        return (
          <g key={a.id} data-testid={baseTestId} transform={`translate(${sx}, ${sy})`} style={{ cursor: baseCursor }} onClick={onAnnotationClick}>
            <rect width={w} height={h} fill="transparent" />
            <path
              d={c.path_data}
              stroke={c.brush_color}
              strokeWidth={bw}
              fill="none"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            {(isSelected || isConnectSource) && (
              <rect width={w} height={h} fill="none" stroke={outline} strokeWidth={1} strokeDasharray="4 4" />
            )}
          </g>
        );
      }
      default:
        return null;
    }
  };

  /**
   * 用户连接 annotation 之间的 connector (per 2026-10-01 OOB 恢复无限画布 connector)
   * - 跟 nodes 派生 connector 区分: 用户连用 onCreateFreeConnector / onDeleteFreeConnector 管理
   * - 配色沿用节点 connector 风格 (硬黑描边 + 内部高亮)
   */
  const renderFreeConnector = (c: AgentCanvasFreeConnector) => {
    const from = annotations.find((a) => a.id === c.fromAnnotationId);
    const to = annotations.find((a) => a.id === c.toAnnotationId);
    if (!from || !to) return null;
    const fx = (from.x + from.width / 2 - viewport.x) * viewport.zoom;
    const fy = (from.y + from.height / 2 - viewport.y) * viewport.zoom;
    const tx = (to.x + to.width / 2 - viewport.x) * viewport.zoom;
    const ty = (to.y + to.height / 2 - viewport.y) * viewport.zoom;
    const stroke = c.color ?? "#00f0ff";
    return (
      <g key={c.id} data-testid={`agent-canvas-free-connector-${c.id}`}>
        <line x1={fx} y1={fy} x2={tx} y2={ty} stroke="#000" strokeWidth={4} />
        <line x1={fx} y1={fy} x2={tx} y2={ty} stroke={stroke} strokeWidth={2} />
        {c.label && (
          <text
            x={(fx + tx) / 2}
            y={(fy + ty) / 2 - 4}
            fill="#fff"
            fontSize={10}
            fontFamily="monospace"
            textAnchor="middle"
            stroke="#000"
            strokeWidth={3}
            paintOrder="stroke"
          >
            {c.label}
          </text>
        )}
      </g>
    );
  };

  // minimap 计算
  const { bbox } = useMemo(() => {
    const xs = canvas.nodes.map((n) => n.x);
    const ys = canvas.nodes.map((n) => n.y);
    const xe = canvas.nodes.map((n) => n.x + n.width);
    const ye = canvas.nodes.map((n) => n.y + n.height);
    return {
      bbox: {
        minX: xs.length ? Math.min(...xs) - 60 : 0,
        minY: ys.length ? Math.min(...ys) - 60 : 0,
        maxX: xe.length ? Math.max(...xe) + 60 : 1200,
        maxY: ye.length ? Math.max(...ye) + 60 : 800,
      },
    };
  }, [canvas.nodes]);

  return (
    <div data-testid="agent-canvas-container" className="relative w-full h-full bg-[#080c14] overflow-hidden select-none">
      {/* 战术浮动 HUD 控制栏 (Tactical Floating Bar) */}
      <div
        data-testid="agent-canvas-toolbar"
        className="absolute top-3 left-1/2 -translate-x-1/2 z-20 flex items-center gap-1 bg-[var(--cel-surface-card,#0f1422)] border-2 border-black p-1 cel-shadow"
      >
        <button
          onClick={() => setTool("select")}
          className={`px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black transition-all ${
            tool === "select"
              ? "bg-[var(--cel-cyan,#00f0ff)] text-black"
              : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"
          }`}
          title={t.ariaLabels.canvasSelect}
        >
          <MousePointer2 size={13} />
        </button>
        <button
          onClick={() => setTool("pan")}
          className={`px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black transition-all ${
            tool === "pan"
              ? "bg-[var(--cel-gold,#ffc400)] text-black"
              : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"
          }`}
          title={t.ariaLabels.canvasPan}
        >
          <Hand size={13} />
        </button>
        <div className="w-[1.5px] h-5 bg-black mx-0.5" />
        <button
          onClick={() => setViewport((v) => ({ ...v, zoom: Math.min(4, v.zoom * 1.2) }))}
          className="p-1.5 bg-[var(--cel-surface-sub,#151c2c)] border border-black text-ink-dim hover:text-white transition-colors"
          title={t.ariaLabels.canvasZoomIn}
        >
          <ZoomIn size={13} />
        </button>
        <button
          onClick={() => setViewport((v) => ({ ...v, zoom: Math.max(0.1, v.zoom / 1.2) }))}
          className="p-1.5 bg-[var(--cel-surface-sub,#151c2c)] border border-black text-ink-dim hover:text-white transition-colors"
          title={t.ariaLabels.canvasZoomOut}
        >
          <ZoomOut size={13} />
        </button>
        <button
          onClick={() => setViewport(canvas.viewport)}
          className="p-1.5 bg-[var(--cel-surface-sub,#151c2c)] border border-black text-ink-dim hover:text-white transition-colors"
          title={t.ariaLabels.canvasFit}
        >
          <Maximize2 size={13} />
        </button>
        <div className="w-[1.5px] h-5 bg-black mx-0.5" />
        {!readOnly && onCreateAnnotation && (
          <>
            <button
              onClick={() => setTool("sticky")}
              className={`px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black transition-all ${tool === "sticky" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"}`}
              title="新增便利贴"
              data-testid="agent-canvas-tool-sticky"
            >
              <StickyNote size={13} />
            </button>
            <button
              onClick={() => setTool("text")}
              className={`px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black transition-all ${tool === "text" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"}`}
              title="新增文字"
              data-testid="agent-canvas-tool-text"
            >
              <Type size={13} />
            </button>
            <button
              onClick={() => setTool("shape")}
              className={`px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black transition-all ${tool === "shape" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"}`}
              title="新增图形"
              data-testid="agent-canvas-tool-shape"
            >
              <Square size={13} />
            </button>
            <button
              onClick={() => setTool("brush")}
              className={`px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black transition-all ${tool === "brush" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"}`}
              title="自由画笔"
              data-testid="agent-canvas-tool-brush"
            >
              <Brush size={13} />
            </button>
            {tool === "brush" && (
              <>
                <div className="w-[1.5px] h-5 bg-black mx-0.5" />
                <div className="flex items-center gap-1" data-testid="agent-canvas-brush-palette">
                  {AGENT_BRUSH_PALETTE.map((c) => (
                    <button
                      key={c}
                      onClick={() => setBrushColor(c)}
                      className={`w-4 h-4 border border-black ${brushColor === c ? "ring-2 ring-[var(--cel-cyan,#00f0ff)]" : ""}`}
                      style={{ background: c }}
                      aria-label={`brush color ${c}`}
                    />
                  ))}
                </div>
                <div className="flex items-center gap-1" data-testid="agent-canvas-brush-size">
                  {AGENT_BRUSH_SIZES.map((s) => (
                    <button
                      key={s}
                      onClick={() => setBrushSize(s)}
                      className={`px-1.5 py-0.5 text-[10px] font-mono border border-black ${brushSize === s ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"}`}
                      aria-label={`brush size ${s}px`}
                    >
                      {s}
                    </button>
                  ))}
                </div>
              </>
            )}
            <button
              onClick={() => { setTool("connector"); setConnectSourceId(null); }}
              className={`px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black transition-all ${tool === "connector" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"}`}
              title="连线 — 选 annotation A → 选 annotation B"
              data-testid="agent-canvas-tool-connector"
            >
              <GitBranch size={13} />
            </button>
            {(selectedAnnotationId || multiSelected.size > 0) && (
              <button
                onClick={() => {
                  const ids = new Set<string>(multiSelected);
                  if (selectedAnnotationId) ids.add(selectedAnnotationId);
                  if (ids.size === 0) return;
                  if (ids.size === 1) {
                    void onDeleteAnnotation?.([...ids][0]);
                  } else {
                    void onBulkDeleteAnnotation?.([...ids]);
                  }
                  setSelectedAnnotationId(null);
                  setMultiSelected(new Set());
                }}
                className="px-2 py-1 text-xs font-mono font-bold flex items-center gap-1 border border-black bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-danger,#ff184c)] hover:bg-[#ff184c] hover:text-white transition-all"
                title={`删除选中 (${multiSelected.size + (selectedAnnotationId ? 1 : 0)}) annotation`}
                data-testid="agent-canvas-tool-delete"
              >
                <Trash2 size={13} />
                {(multiSelected.size + (selectedAnnotationId ? 1 : 0)) > 1 && (
                  <span className="text-[10px] font-mono">×{multiSelected.size + (selectedAnnotationId ? 1 : 0)}</span>
                )}
              </button>
            )}
          </>
        )}
        <span
          className="text-[11px] text-[var(--cel-cyan,#00f0ff)] font-mono font-bold px-2 tabular-nums"
          data-testid="agent-canvas-zoom"
        >
          {Math.round(viewport.zoom * 100)}%
        </span>
      </div>

      {/* SVG Canvas (全息机战沙盘) */}
      <svg
        ref={svgRef}
        data-testid="agent-canvas-svg"
        viewBox="0 0 1200 800"
        className="w-full h-full"
        style={{
          cursor: tool === "pan" ? "grab" : "default",
          backgroundColor: "#070a10",
        }}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        onWheel={onWheel}
        onClick={(e) => {
          // 注释创建 (per 2026-10-01 OOB 恢复无限画布画笔)
          // - sticky/text/shape: SVG 空白区点击
          // - connect-source mode: 点已有 annotation 设置连接源端
          if ((e.target as Element).tagName?.toLowerCase() !== "svg") return;
          if (tool === "sticky") {
            void createAnnotationAt(e, "sticky_note", 180, 100, { color: AGENT_STICKY_PALETTE[0], text: "" });
          } else if (tool === "text") {
            void createAnnotationAt(e, "text", 200, 60, { text: "" });
          } else if (tool === "shape") {
            void createAnnotationAt(e, "shape", 120, 120, { shape: "rect" });
          }
        }}
      >
        <defs>
          {/* 战术箭头标记 */}
          <marker
            id="tactical-arrow"
            viewBox="0 0 12 12"
            refX="10"
            refY="6"
            markerWidth="8"
            markerHeight="8"
            orient="auto"
          >
            <polygon points="0,2 10,6 0,10 3,6" fill="context-stroke" stroke="#000000" strokeWidth="1" />
          </marker>

          {/* 战术机甲坐标网格 Pattern */}
          <pattern id="tactical-grid-pattern" width="60" height="60" patternUnits="userSpaceOnUse">
            {/* 细线次网格 */}
            <path d="M 60 0 L 0 0 0 60" fill="none" stroke="#162032" strokeWidth="0.8" opacity="0.6" />
            {/* 顶点十字刻度 */}
            <path d="M 0 4 L 0 -4 M -4 0 L 4 0" fill="none" stroke="#00f0ff" strokeWidth="0.8" opacity="0.4" />
          </pattern>
        </defs>

        {/* 战术全息网格铺底 */}
        <rect width="100%" height="100%" fill="url(#tactical-grid-pattern)" />

        {/* 边缘战术 HUD 极弱水印标 (低认知负荷, 不遮挡内容) */}
        <g opacity="0.2" className="pointer-events-none select-none">
          <text x="24" y="32" fontSize="10" fill="#00f0ff" fontFamily='"JetBrains Mono", monospace' fontWeight="bold">
            [SECTOR-01 // TACTICAL HOLO-CANVAS // LIVE MATRIX]
          </text>
          <text x="1176" y="32" textAnchor="end" fontSize="10" fill="#ffc400" fontFamily='"JetBrains Mono", monospace' fontWeight="bold">
            [SYS_STATUS: OPTIMAL · LATENCY &lt; 5MS]
          </text>
          <text x="24" y="780" fontSize="10" fill="#94a3b8" fontFamily='"JetBrains Mono", monospace'>
            + [GRID 00:00:X]
          </text>
        </g>

        {/* marquee 框选 (per 任务 #4) */}
        {marquee && (() => {
          const minX = Math.min(marquee.startWorldX, marquee.endWorldX);
          const maxX = Math.max(marquee.startWorldX, marquee.endWorldX);
          const minY = Math.min(marquee.startWorldY, marquee.endWorldY);
          const maxY = Math.max(marquee.startWorldY, marquee.endWorldY);
          const sx = (minX - viewport.x) * viewport.zoom;
          const sy = (minY - viewport.y) * viewport.zoom;
          const w = (maxX - minX) * viewport.zoom;
          const h = (maxY - minY) * viewport.zoom;
          return (
            <rect data-testid="agent-canvas-marquee"
              x={sx} y={sy} width={w} height={h}
              fill="rgba(0, 240, 255, 0.1)"
              stroke="var(--cel-cyan,#00f0ff)"
              strokeWidth={1}
              strokeDasharray="4 4"
              pointerEvents="none"
            />
          );
        })()}

        {/* connectors (auto-laid, 节点之间的派生连线) */}
        {canvas.connectors.map(renderConnector)}

        {/* 用户注释 annotation (per 2026-10-01 OOB 恢复无限画布画笔 - sticky/text/shape/path + connector 用户连) */}
        {annotations.map(renderAnnotation)}
        {freeConnectors.map(renderFreeConnector)}

        {/* 自由画笔 in-progress path (per 2026-10-01 OOB) */}
        {drawingPath && drawingPath.points.length >= 2 && (() => {
          const PAD = Math.max(2, brushSize);
          const x = drawingPath.minX - PAD;
          const y = drawingPath.minY - PAD;
          const sx = (x - viewport.x) * viewport.zoom;
          const sy = (y - viewport.y) * viewport.zoom;
          const d = "M " + drawingPath.points.map((p) => `${(p.x - x).toFixed(1)} ${(p.y - y).toFixed(1)}`).join(" L ");
          return (
            <g transform={`translate(${sx}, ${sy})`} data-testid="agent-canvas-drawing-path" pointerEvents="none">
              <path d={d} stroke={brushColor} strokeWidth={brushSize * viewport.zoom} fill="none" strokeLinecap="round" strokeLinejoin="round" opacity={0.85} />
            </g>
          );
        })()}

        {/* nodes (auto-laid, 只读派生 — 不可拖动, 不可删, 不可编辑) */}
        {canvas.nodes.map(renderNode)}
      </svg>

      {/* 战术雷达 Minimap */}
      <div
        data-testid="agent-canvas-minimap"
        className="absolute bottom-3 right-3 z-20 w-44 h-32 bg-[var(--cel-surface-card,#0f1422)] border-2 border-black cel-shadow overflow-hidden"
      >
        <div className="absolute top-1 left-2 text-[9px] font-mono font-bold text-[var(--cel-cyan,#00f0ff)] uppercase tracking-wider z-10">
          RADAR // MINI-MAP
        </div>
        <svg viewBox={`${bbox.minX} ${bbox.minY} ${bbox.maxX - bbox.minX} ${bbox.maxY - bbox.minY}`} className="w-full h-full bg-[#050811]">
          {/* 视口框 */}
          <rect
            x={viewport.x}
            y={viewport.y}
            width={1200 / viewport.zoom}
            height={800 / viewport.zoom}
            fill="rgba(0, 240, 255, 0.08)"
            stroke="#00f0ff"
            strokeWidth={1.5}
          />
          {/* 节点信标 */}
          {canvas.nodes.map((n) => (
            <rect
              key={n.id}
              x={n.x}
              y={n.y}
              width={n.width}
              height={n.height}
              fill={n.kind === "agent" ? "#00f0ff" : "#ffc400"}
              stroke="#000000"
              strokeWidth={1}
              opacity={0.8}
            />
          ))}
        </svg>
      </div>

      {/* 底部状态条 */}
      <div
        className="absolute bottom-3 left-3 z-20 text-[11px] text-ink-dim font-mono flex items-center gap-3 bg-[var(--cel-surface-card,#0f1422)] border-2 border-black px-3 py-1 cel-shadow"
        data-testid="agent-canvas-statusbar"
      >
        <span className="flex items-center gap-1">
          <span className="size-2 rounded-full bg-[var(--cel-cyan,#00f0ff)] animate-pulse" />
          zoom <strong className="text-white">{Math.round(viewport.zoom * 100)}%</strong>
        </span>
        <span className="text-ink-mute">|</span>
        <span>nodes <strong className="text-white">{canvas.nodes.length}</strong></span>
        <span className="text-ink-mute">|</span>
        <span>connectors <strong className="text-white">{canvas.connectors.length}</strong></span>
        <span className="text-ink-mute">|</span>
        <span>selected <strong className="text-[var(--cel-gold,#ffc400)]">{selectedNodeId ?? "—"}</strong></span>
      </div>
    </div>
  );
}

// 战术切角卡片基座 (3渲2 Cel Chamfer Polygon with Ink Outline)
function TacticalChamferNode({
  w,
  h,
  zoom,
  fill,
  stroke,
  strokeWidth = 2,
  glowColor,
}: {
  w: number;
  h: number;
  zoom: number;
  fill: string;
  stroke: string;
  strokeWidth?: number;
  glowColor?: string;
}) {
  const maxChamfer = Math.min(w, h) * 0.25;
  const c = Math.max(3, Math.min(6 * zoom, maxChamfer)); // 切角大小 (按 zoom 等比缩放, 不超 25%)
  const path = `M 0 ${c} L ${c} 0 L ${w - c} 0 L ${w} ${c} L ${w} ${h - c} L ${w - c} ${h} L ${c} ${h} L 0 ${h - c} Z`;

  return (
    <g>
      {/* 3渲2 漫画像素投影 (4px hard black shadow) */}
      <path
        d={path}
        transform="translate(4, 4)"
        fill="#000000"
        opacity={0.8}
      />
      {/* 节点装甲本体 */}
      <path
        d={path}
        fill={fill}
        stroke={stroke}
        strokeWidth={strokeWidth}
      />
      {/* 4 角战术括号瞄准标 (Tactical HUD Corner Brackets) */}
      <g stroke={glowColor ?? "#2a374d"} strokeWidth={1.2 * zoom} fill="none">
        {/* 左上 */}
        <path d={`M ${c + 4} 3 L 3 3 L 3 ${c + 4}`} />
        {/* 右上 */}
        <path d={`M ${w - c - 4} 3 L ${w - 3} 3 L ${w - 3} ${c + 4}`} />
        {/* 左下 */}
        <path d={`M 3 ${h - c - 4} L 3 ${h - 3} L ${c + 4} ${h - 3}`} />
        {/* 右下 */}
        <path d={`M ${w - 3} ${h - c - 4} L ${w - 3} ${h - 3} L ${w - c - 4} ${h - 3}`} />
      </g>
    </g>
  );
}

function WorktreeNodeBody({ worktree, w, h, zoom }: { worktree: Worktree; w: number; h: number; zoom: number }) {
  return (
    <g>
      <g transform={`translate(${12 * zoom}, ${12 * zoom})`}>
        <GitBranch size={13 * zoom} color="#00f0ff" strokeWidth={2} />
      </g>
      <text
        x={30 * zoom}
        y={21 * zoom}
        fontSize={9 * zoom}
        fill="#94a3b8"
        fontFamily='"JetBrains Mono", monospace'
        fontWeight="bold"
      >
        DATA BASE // WORKTREE
      </text>
      <text
        x={14 * zoom}
        y={40 * zoom}
        fontSize={12 * zoom}
        fill="#ffffff"
        fontFamily='"JetBrains Mono", monospace'
        fontWeight="800"
      >
        {worktree.branch}
      </text>
      <foreignObject x={14 * zoom} y={(h - 26) * zoom} width={(w - 28) * zoom} height={20 * zoom}>
        <div>
          <StatusPill value={worktree.status} size="xs" />
        </div>
      </foreignObject>
    </g>
  );
}

function WorkItemNodeBody({ wi, w, h, zoom }: { wi: WorkItem; w: number; h: number; zoom: number }) {
  const maxChars = Math.max(8, Math.floor((w - 24) / (6.5 * zoom)));
  const titleTrunc = wi.title.length > maxChars ? `${wi.title.slice(0, Math.max(1, maxChars - 1))}…` : wi.title;

  return (
    <g>
      <text
        x={12 * zoom}
        y={16 * zoom}
        fontSize={9.5 * zoom}
        fill="#94a3b8"
        fontFamily='"JetBrains Mono", monospace'
        fontWeight="bold"
      >
        {wi.key}
      </text>
      <text
        x={12 * zoom}
        y={31 * zoom}
        fontSize={11 * zoom}
        fill="#f8fafc"
        fontFamily="system-ui, sans-serif"
        fontWeight="600"
        style={{ pointerEvents: "none" }}
      >
        {titleTrunc}
      </text>
      <foreignObject x={12 * zoom} y={(h - 24) * zoom} width={(w - 24)} height={20 * zoom}>
        <div style={{ display: "flex", alignItems: "center", gap: 4 * zoom, justifyContent: "space-between" }}>
          <StatusPill value={wi.status as WorkItemStatus} size="xs" translateAs="workItem" />
          <span
            style={{
              fontSize: 9.5 * zoom,
              fontWeight: 800,
              color: wi.priority === "p0" ? "#ff184c" : wi.priority === "p1" ? "#ffc400" : "#94a3b8",
              fontFamily: '"JetBrains Mono", monospace',
            }}
          >
            {wi.priority.toUpperCase()}
          </span>
        </div>
      </foreignObject>
    </g>
  );
}
