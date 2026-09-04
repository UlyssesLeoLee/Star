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
  Canvas, CanvasElement, CanvasConnector, Worktree, AgentSession, AutomationRule, Feedback,
} from "@/types/ids";
import { useMemo, useState, useRef, useEffect, useCallback } from "react";
import { useStore } from "@/lib/store";
import { StatusPill } from "./StatusPill";
import { MousePointer2, Hand, Plus, Trash2, ZoomIn, ZoomOut, Maximize2 } from "lucide-react";

interface CanvasViewProps {
  canvas: Canvas;
  elements: CanvasElement[];
  connectors: CanvasConnector[];
  highlightElementId?: string;
  readOnly?: boolean;
}

const STICKY_PALETTE = ["#f9d77e", "#ffb3c1", "#a3d9ff", "#b8f0c4", "#d4b3ff"];

export function CanvasView({ canvas, elements, connectors, highlightElementId, readOnly = false }: CanvasViewProps) {
  // viewport: 世界坐标
  const [viewport, setViewport] = useState(canvas.viewport);
  const [selected, setSelected] = useState<string[]>([]);
  const [tool, setTool] = useState<"select" | "pan">("select");
  const svgRef = useRef<SVGSVGElement>(null);
  const dragState = useRef<{ type: "pan" | "element" | null; startX: number; startY: number; elX: number; elY: number; elId: string | null }>({
    type: null, startX: 0, startY: 0, elX: 0, elY: 0, elId: null,
  });

  const worktrees = useStore((s) => s.worktrees);
  const agentSessions = useStore((s) => s.agentSessions);
  const automationRules = useStore((s) => s.automationRules);
  const feedbacks = useStore((s) => s.feedbacks);
  const moveCanvasElement = useStore((s) => s.moveCanvasElement);
  const deleteCanvasElement = useStore((s) => s.deleteCanvasElement);

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
      dragState.current = { type: "pan", startX: e.clientX, startY: e.clientY, elX: viewport.x, elY: viewport.y, elId: null };
    }
  };

  const onMouseMove = (e: React.MouseEvent) => {
    const ds = dragState.current;
    if (ds.type === "pan") {
      const dx = (e.clientX - ds.startX) / viewport.zoom;
      const dy = (e.clientY - ds.startY) / viewport.zoom;
      setViewport({ ...viewport, x: ds.elX - dx, y: ds.elY - dy });
    } else if (ds.type === "element" && ds.elId && !readOnly) {
      const dx = (e.clientX - ds.startX) / viewport.zoom;
      const dy = (e.clientY - ds.startY) / viewport.zoom;
      moveCanvasElement(ds.elId, ds.elX + dx, ds.elY + dy);
    }
  };

  const onMouseUp = () => {
    dragState.current = { type: null, startX: 0, startY: 0, elX: 0, elY: 0, elId: null };
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
      setViewport({ x: wx - sx / newZoom, y: wy - sy / newZoom, zoom: newZoom });
    }
  };

  const onElementMouseDown = (e: React.MouseEvent, el: CanvasElement) => {
    e.stopPropagation();
    if (tool === "pan") return;
    setSelected([el.id]);
    if (!readOnly) {
      dragState.current = {
        type: "element",
        startX: e.clientX,
        startY: e.clientY,
        elX: el.x,
        elY: el.y,
        elId: el.id,
      };
    }
  };

  const onElementDoubleClick = (el: CanvasElement) => {
    // 联动 2:work_item_card / worktree_node / agent_cursor / automation_node → 跳详情
    const ref = el.content.work_item_id || el.content.worktree_id || el.content.agent_session_id || el.content.automation_id;
    const kind = el.content.work_item_id ? "work-item"
      : el.content.worktree_id ? "worktree"
      : el.content.agent_session_id ? "agent"
      : el.content.automation_id ? "automation"
      : null;
    if (ref && kind) {
      window.location.href = `/${kind}?selected=${ref}`;
    }
  };

  // render element
  const renderElement = (el: CanvasElement) => {
    const isHighlighted = el.id === highlightElementId;
    const isSelected = selected.includes(el.id);
    const stroke = isHighlighted ? "#2f81f7" : isSelected ? "#79c0ff" : "#30363d";
    const strokeWidth = isHighlighted || isSelected ? 2 : 1;
    const pos = worldToScreen(el.x, el.y);
    const w = el.width * viewport.zoom;
    const h = el.height * viewport.zoom;

    switch (el.kind) {
      case "sticky_note": {
        const color = el.content.color || STICKY_PALETTE[0];
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <rect width={w} height={h} fill={color} stroke={stroke} strokeWidth={strokeWidth} rx={4} />
            <foreignObject x={6} y={6} width={w - 12} height={h - 12}>
              <div style={{ fontSize: 11 * viewport.zoom, color: "#0b0d10", lineHeight: 1.3, fontFamily: "system-ui", wordBreak: "break-word", overflow: "hidden" }}>
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
              <div style={{ fontSize: 12 * viewport.zoom, color: "#e6edf3", lineHeight: 1.4, fontFamily: "system-ui" }}>
                {el.content.text}
              </div>
            </foreignObject>
          </g>
        );
      }
      case "work_item_card": {
        const wi = useStore.getState().workItems.find((w) => w.id === el.content.work_item_id);
        if (!wi) return null;
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <rect width={w} height={h} fill="#161b22" stroke={stroke} strokeWidth={strokeWidth} rx={4} />
            <text x={8} y={16 * viewport.zoom} fontSize={10 * viewport.zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
              {wi.key}
            </text>
            <foreignObject x={8} y={20 * viewport.zoom} width={w - 16} height={h - 30 * viewport.zoom}>
              <div style={{ fontSize: 11 * viewport.zoom, color: "#e6edf3", lineHeight: 1.3, fontFamily: "system-ui", overflow: "hidden" }}>
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
        const wt = worktrees.find((w) => w.id === el.content.worktree_id);
        if (!wt) return null;
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <rect width={w} height={h} fill="#161b22" stroke={stroke} strokeWidth={strokeWidth} rx={20} />
            <text x={12 * viewport.zoom} y={18 * viewport.zoom} fontSize={10 * viewport.zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
              worktree
            </text>
            <text x={12 * viewport.zoom} y={34 * viewport.zoom} fontSize={12 * viewport.zoom} fill="#e6edf3" fontFamily="ui-monospace, monospace">
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
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)} onDoubleClick={() => onElementDoubleClick(el)}>
            <circle cx={w / 2} cy={h / 2} r={Math.min(w, h) / 2 - 2} fill="#1f6feb33" stroke="#2f81f7" strokeWidth={strokeWidth} />
            <text x={w / 2} y={h / 2 - 4} textAnchor="middle" fontSize={10 * viewport.zoom} fill="#79c0ff" fontFamily="ui-monospace, monospace">
              {ag.id}
            </text>
            <text x={w / 2} y={h / 2 + 10} textAnchor="middle" fontSize={8 * viewport.zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
              {ag.agent_kind}
            </text>
            <text x={w / 2} y={h / 2 + 24} textAnchor="middle" fontSize={9 * viewport.zoom} fill="#3fb950" fontFamily="ui-monospace, monospace">
              {ag.status}
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
              <div style={{ fontSize: 10 * viewport.zoom, color: "#e6edf3", lineHeight: 1.2, fontFamily: "system-ui", textAlign: "center", overflow: "hidden" }}>
                {au.name}
              </div>
            </foreignObject>
          </g>
        );
      }
      case "comment_pin": {
        return (
          <g key={el.id} transform={`translate(${pos.x}, ${pos.y})`} style={{ cursor: "pointer" }} onMouseDown={(e) => onElementMouseDown(e, el)}>
            <circle cx={w / 2} cy={h / 2} r={Math.min(w, h) / 2} fill="#79c0ff33" stroke={stroke} strokeWidth={strokeWidth} />
            <text x={w / 2} y={h / 2 + 4} textAnchor="middle" fontSize={10 * viewport.zoom} fill="#79c0ff" fontFamily="ui-monospace, monospace">
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
            <rect width={w} height={h} fill="#21262d" stroke={stroke} strokeWidth={strokeWidth} rx={4} />
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
        <rect width={w} height={h} fill="#11151b1a" stroke="#21262d" strokeWidth={1} strokeDasharray="4 4" rx={6} />
        <text x={10} y={16 * viewport.zoom} fontSize={11 * viewport.zoom} fill="#8b949e" fontFamily="system-ui">
          {frame.title}
        </text>
        {frame.is_slide && (
          <text x={w - 30 * viewport.zoom} y={16 * viewport.zoom} fontSize={9 * viewport.zoom} fill="#6e7681" fontFamily="system-ui">
            [slide]
          </text>
        )}
      </g>
    );
  };

  // render connector(bezier 复用 SmView 算法)
  const renderConnector = (c: CanvasConnector) => {
    const from = elements.find((e) => e.id === c.from_element_id);
    const to = elements.find((e) => e.id === c.to_element_id);
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
            <rect x={-c.label.length * 3.5} y={-8} width={c.label.length * 7} height={14} fill="#0b0d10" stroke={c.color} rx={3} />
            <text textAnchor="middle" y={3} fontSize={9} fill={c.color} fontFamily="ui-monospace, monospace">
              {c.label}
            </text>
          </g>
        )}
      </g>
    );
  };

  // minimap(右下角,显示 viewport 范围)
  const allX = elements.map((e) => e.x);
  const allY = elements.map((e) => e.y);
  const minX = allX.length > 0 ? Math.min(...allX) - 100 : 0;
  const minY = allY.length > 0 ? Math.min(...allY) - 100 : 0;
  const maxX = allX.length > 0 ? Math.max(...allX.map((x, i) => x + elements[i].width)) + 100 : 1200;
  const maxY = allY.length > 0 ? Math.max(...allY.map((y, i) => y + elements[i].height)) + 100 : 800;

  return (
    <div data-testid="canvas-container" className="relative w-full h-full bg-bg overflow-hidden">
      {/* Toolbar */}
      <div data-testid="canvas-toolbar" className="absolute top-3 left-1/2 -translate-x-1/2 z-20 flex items-center gap-1 bg-bg-card border border-line rounded-md p-1 shadow-lg">
        <button onClick={() => setTool("select")} className={`btn p-1.5 ${tool === "select" ? "border-accent text-accent" : ""}`} title="Select (V)">
          <MousePointer2 size={14} />
        </button>
        <button onClick={() => setTool("pan")} className={`btn p-1.5 ${tool === "pan" ? "border-accent text-accent" : ""}`} title="Pan (H)">
          <Hand size={14} />
        </button>
        <div className="w-px h-5 bg-line" />
        <button onClick={() => setViewport({ ...viewport, zoom: Math.min(4, viewport.zoom * 1.2) })} className="btn p-1.5" title="Zoom in (+)">
          <ZoomIn size={14} />
        </button>
        <button onClick={() => setViewport({ ...viewport, zoom: Math.max(0.1, viewport.zoom / 1.2) })} className="btn p-1.5" title="Zoom out (-)">
          <ZoomOut size={14} />
        </button>
        <button onClick={() => {
          // fit to content
          setViewport({ x: minX, y: minY, zoom: Math.min(1200 / (maxX - minX), 800 / (maxY - minY), 1) });
        }} className="btn p-1.5" title="Fit to content (1)">
          <Maximize2 size={14} />
        </button>
        <span className="text-[10px] text-ink-dim font-mono px-2">{Math.round(viewport.zoom * 100)}%</span>
        <div className="w-px h-5 bg-line" />
        {selected.length > 0 && !readOnly && (
          <button onClick={() => { selected.forEach((id) => deleteCanvasElement(id)); setSelected([]); }} className="btn p-1.5 text-err" title="Delete">
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
        style={{ cursor: tool === "pan" ? "grab" : "default", backgroundColor: "#0b0d10", backgroundImage: "radial-gradient(circle, #21262d 1px, transparent 1px)", backgroundSize: "20px 20px" }}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        onWheel={onWheel}
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

        {/* Element */}
        {elements.map((el) => (
          <g key={`wrapper-${el.id}`} data-testid={`canvas-element-${el.id}`}>
            {renderElement(el)}
          </g>
        ))}
      </svg>

      {/* Minimap */}
      <div data-testid="canvas-minimap" className="absolute bottom-3 right-3 z-20 w-40 h-28 bg-bg-card border border-line rounded-md overflow-hidden">
        <svg viewBox={`${minX} ${minY} ${maxX - minX} ${maxY - minY}`} className="w-full h-full">
          {/* viewport rect */}
          <rect
            x={viewport.x}
            y={viewport.y}
            width={1200 / viewport.zoom}
            height={800 / viewport.zoom}
            fill="none"
            stroke="#2f81f7"
            strokeWidth={2}
          />
          {/* elements dots */}
          {elements.map((e) => (
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
