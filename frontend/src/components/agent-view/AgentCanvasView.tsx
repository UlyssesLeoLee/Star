"use client";

// =====================================================================
// AgentCanvasView — 无限画布 (Miro 风格), agent 视图专用
// =====================================================================
// Per 2026-09-05 11:25 JST 拍板 #1: 自由散开布局. 跟通用 CanvasView 不同:
//   - 节点不可拖动 (派生视图, 写不归本组件管)
//   - 节点类型只 3 种: agent / worktree / work_item
//   - 双击跳详情 (跟 work-item / worktree / agent 页面联动)
//   - 默认 fit-to-content; 鼠标拖空白 pan, 滚轮 zoom, 工具栏控制
//   - 节点视觉:
//     - agent: 大圆角矩形, 顶部 status pill + kind icon, 底部 token/cost
//     - worktree: 中圆角矩形, 显示 branch + status pill
//     - work_item: 矩形卡片, key + title + status pill + priority
// =====================================================================

import type {
  AgentSession, Worktree, WorkItem, AgentStatus, WorkItemStatus,
} from "@/types/ids";
import type { AgentCanvas, AgentCanvasNode, AgentCanvasConnector } from "@/lib/agent-view/types";
import type { AgentGameState } from "@/lib/agent-game/types";
import { visualForLevel, MAX_HP } from "@/lib/agent-game/types";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { StatusPill } from "@/components/StatusPill";
import { useStore } from "@/lib/store";
import {
  Hand, MousePointer2, ZoomIn, ZoomOut, Maximize2, Bot, GitBranch, Hash, Cpu, Skull, Coins,
} from "lucide-react";

interface AgentCanvasViewProps {
  canvas: AgentCanvas;
  agent: AgentSession;
  worktree: Worktree | null;
  /** 拟人化游戏化 (per 2026-09-05 11:42 JST 拍板) */
  gameState: AgentGameState | null;
  /** 领奖回调 (work-item done + 未领奖时) */
  onClaim?: (workItemId: string) => void;
}

export function AgentCanvasView({ canvas, agent, worktree, gameState, onClaim }: AgentCanvasViewProps) {
  // 顶层订阅 workItems, 避免 renderNode 内 useStore.getState 触发 hooks 违规
  const workItems = useStore((s) => s.workItems);
  const workItemById = useMemo(
    () => new Map(workItems.map((w) => [w.id, w] as const)),
    [workItems],
  );
  // viewport (世界坐标 + zoom)
  const [viewport, setViewport] = useState(canvas.viewport);
  const [tool, setTool] = useState<"select" | "pan">("pan"); // 默认 pan, 因为本视图只读
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  // 拖动 viewport (中键 / pan tool / shift)
  const dragState = useRef<{ type: "pan" | null; startX: number; startY: number; elX: number; elY: number }>({
    type: null, startX: 0, startY: 0, elX: 0, elY: 0,
  });

  // 屏幕坐标 → 世界坐标
  const screenToWorld = useCallback((sx: number, sy: number) => ({
    x: sx / viewport.zoom + viewport.x,
    y: sy / viewport.zoom + viewport.y,
  }), [viewport]);

  // 首次加载 fit-to-content (拿 store 渲染后 bbox 不变, 但 viewport 仍走 canvas 初始)
  useEffect(() => {
    setViewport(canvas.viewport);
  }, [canvas.viewport, canvas.derivedAt]);

  // 键盘快捷键: V=select, H=pan, +=zoom in, --=zoom out, 1=fit
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // 跳过输入框
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

  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button === 1 || (e.button === 0 && tool === "pan") || e.shiftKey) {
      dragState.current = { type: "pan", startX: e.clientX, startY: e.clientY, elX: viewport.x, elY: viewport.y };
    } else if (e.button === 0 && tool === "select") {
      // 点空白取消选中
      setSelectedNodeId(null);
    }
  };

  const onMouseMove = (e: React.MouseEvent) => {
    const ds = dragState.current;
    if (ds.type === "pan") {
      const dx = (e.clientX - ds.startX) / viewport.zoom;
      const dy = (e.clientY - ds.startY) / viewport.zoom;
      setViewport({ ...viewport, x: ds.elX - dx, y: ds.elY - dy });
    }
  };

  const onMouseUp = () => {
    dragState.current = { type: null, startX: 0, startY: 0, elX: 0, elY: 0 };
  };

  // 滚轮 zoom (以光标为中心)
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

  // 双击节点 → 跳详情
  const onNodeDoubleClick = (ref: AgentCanvasNode["ref"]) => {
    if (ref.kind === "agent") {
      window.location.href = `/agent?selected=${ref.agentId}`;
    } else if (ref.kind === "worktree") {
      window.location.href = `/worktree?selected=${ref.worktreeId}`;
    } else if (ref.kind === "work_item") {
      window.location.href = `/work-item?selected=${ref.workItemId}`;
    }
  };

  // 节点 hover / select
  const onNodeClick = (e: React.MouseEvent, nodeId: string) => {
    e.stopPropagation();
    if (tool === "pan") return;
    setSelectedNodeId(nodeId);
  };

  // 渲染 connector (bezier, 从世界坐标转屏幕)
  const renderConnector = (c: AgentCanvasConnector) => {
    const from = canvas.nodes.find((n) => n.id === c.fromNodeId);
    const to = canvas.nodes.find((n) => n.id === c.toNodeId);
    if (!from || !to) return null;
    const fx = from.x + from.width / 2;
    const fy = from.y + from.height / 2;
    const tx = to.x + to.width / 2;
    const ty = to.y + to.height / 2;
    // bezier
    const dx = tx - fx;
    const dy = ty - fy;
    const c1x = fx + dx * 0.25;
    const c1y = fy + dy * 0.1;
    const c2x = tx - dx * 0.25;
    const c2y = ty - dy * 0.1;
    // 屏幕坐标
    const fx_s = (fx - viewport.x) * viewport.zoom;
    const fy_s = (fy - viewport.y) * viewport.zoom;
    const tx_s = (tx - viewport.x) * viewport.zoom;
    const ty_s = (ty - viewport.y) * viewport.zoom;
    const dx_s = tx_s - fx_s;
    const dy_s = ty_s - fy_s;
    const c1x_s = fx_s + dx_s * 0.25;
    const c1y_s = fy_s + dy_s * 0.1;
    const c2x_s = tx_s - dx_s * 0.25;
    const c2y_s = ty_s - dy_s * 0.1;
    const path = `M ${fx_s} ${fy_s} C ${c1x_s} ${c1y_s}, ${c2x_s} ${c2y_s}, ${tx_s} ${ty_s}`;
    return (
      <g key={c.id} data-testid={`agent-canvas-connector-${c.id}`}>
        <path d={path} fill="none" stroke={c.color} strokeWidth={1.5 * viewport.zoom} opacity={0.55} markerEnd="url(#agent-arrow)" />
        {c.label && (
          <text
            x={(fx_s + tx_s) / 2}
            y={(fy_s + ty_s) / 2 - 4}
            textAnchor="middle"
            fontSize={9 * viewport.zoom}
            fill={c.color}
            fontFamily="ui-monospace, monospace"
            opacity={0.8}
          >
            {c.label}
          </text>
        )}
      </g>
    );
  };

  // 节点渲染 (派发到对应类型)
  const renderNode = (node: AgentCanvasNode) => {
    const posX = (node.x - viewport.x) * viewport.zoom;
    const posY = (node.y - viewport.y) * viewport.zoom;
    const w = node.width * viewport.zoom;
    const h = node.height * viewport.zoom;
    const isSelected = selectedNodeId === node.id;
    const isHovered = hoveredNodeId === node.id;

    if (node.ref.kind === "agent") {
      // 拟人化游戏化视觉 (per 拍板 #3, Lv 1..10 渐进)
      const tier = gameState ? visualForLevel(gameState.level) : null;
      const alive = gameState?.alive ?? true;
      const scale = tier?.scale ?? 1;
      const nodeW = w * scale;
      const nodeH = h * scale;
      // 中心化 (因为 scale 改了, 调整 translate)
      const offsetX = (w - nodeW) / 2;
      const offsetY = (h - nodeH) / 2;
      const borderColor = !alive
        ? "#f85149"
        : isSelected ? "#79c0ff" : isHovered ? "#2f81f7" : (tier?.color ?? "#1f6feb");
      const fillColor = !alive ? "#1a1a1a" : "#0d2849";
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
          {/* Halo ring (Lv 7+ purple, Lv 9+ gold) */}
          {tier && tier.level >= 7 && (
            <rect
              x={-6 * viewport.zoom}
              y={-6 * viewport.zoom}
              width={nodeW + 12 * viewport.zoom}
              height={nodeH + 12 * viewport.zoom}
              fill="none"
              stroke={tier.color}
              strokeWidth={1}
              strokeDasharray={`${4 * viewport.zoom} ${4 * viewport.zoom}`}
              opacity={0.5}
              rx={16 * viewport.zoom}
            />
          )}
          <NodeRect w={nodeW} h={nodeH} stroke={borderColor} strokeWidth={(tier?.borderWidth ?? 1.5) * viewport.zoom} fill={fillColor} radius={12 * viewport.zoom} />
          <AgentNodeBody agent={agent} w={nodeW} h={nodeH} zoom={viewport.zoom} gameState={gameState} />
          {/* Lv 徽章 (右上角) */}
          {tier && (
            <g transform={`translate(${nodeW - 30 * viewport.zoom}, ${-10 * viewport.zoom})`}>
              <rect width={28 * viewport.zoom} height={20 * viewport.zoom} fill={tier.color} rx={4 * viewport.zoom} />
              <text x={14 * viewport.zoom} y={14 * viewport.zoom} textAnchor="middle" fontSize={11 * viewport.zoom} fill="#0b0d10" fontWeight="bold" fontFamily="ui-monospace, monospace">
                Lv{tier.level}
              </text>
            </g>
          )}
          {/* 死亡 skull overlay */}
          {!alive && (
            <g transform={`translate(${nodeW / 2 - 12 * viewport.zoom}, ${nodeH / 2 - 12 * viewport.zoom})`}>
              <Skull size={24 * viewport.zoom} color="#f85149" strokeWidth={1.5} />
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
          <NodeRect w={w} h={h} stroke={isSelected ? "#79c0ff" : isHovered ? "#2f81f7" : "#30363d"} fill="#161b22" radius={8 * viewport.zoom} />
          <WorktreeNodeBody worktree={worktree} w={w} h={h} zoom={viewport.zoom} />
        </g>
      );
    }
    if (node.ref.kind === "work_item") {
      const wi = workItemById.get(node.ref.workItemId);
      if (!wi) return null;
      // 拟人化游戏化: 完成后可领奖 (status=done + 未领过)
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
          <NodeRect w={w} h={h} stroke={isSelected ? "#79c0ff" : isHovered ? "#2f81f7" : "#30363d"} fill="#161b22" radius={6 * viewport.zoom} />
          <WorkItemNodeBody wi={wi} w={w} h={h} zoom={viewport.zoom} />
          {/* Claim button (foreignObject HTML 按钮) */}
          {canClaim && (
            <foreignObject
              x={2 * viewport.zoom}
              y={h - 18 * viewport.zoom}
              width={w - 4 * viewport.zoom}
              height={16 * viewport.zoom}
            >
              <button
                data-testid={`claim-btn-${wi.id}`}
                onClick={(e) => {
                  e.stopPropagation();
                  onClaim!(wi.id);
                }}
                className="w-full h-full text-[9px] flex items-center justify-center gap-1 rounded bg-warn/20 border border-warn/50 text-warn hover:bg-warn/30 font-mono"
                style={{ pointerEvents: "all" }}
              >
                <Coins size={9} /> Claim
              </button>
            </foreignObject>
          )}
        </g>
      );
    }
    return null;
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
    <div data-testid="agent-canvas-container" className="relative w-full h-full bg-bg overflow-hidden">
      {/* 工具栏 */}
      <div data-testid="agent-canvas-toolbar" className="absolute top-3 left-1/2 -translate-x-1/2 z-20 flex items-center gap-1 bg-bg-card border border-line rounded-md p-1 shadow-lg">
        <button onClick={() => setTool("select")} className={`btn p-1.5 ${tool === "select" ? "border-accent text-accent" : ""}`} title="Select (V)">
          <MousePointer2 size={14} />
        </button>
        <button onClick={() => setTool("pan")} className={`btn p-1.5 ${tool === "pan" ? "border-accent text-accent" : ""}`} title="Pan (H)">
          <Hand size={14} />
        </button>
        <div className="w-px h-5 bg-line" />
        <button onClick={() => setViewport((v) => ({ ...v, zoom: Math.min(4, v.zoom * 1.2) }))} className="btn p-1.5" title="Zoom in (+)">
          <ZoomIn size={14} />
        </button>
        <button onClick={() => setViewport((v) => ({ ...v, zoom: Math.max(0.1, v.zoom / 1.2) }))} className="btn p-1.5" title="Zoom out (-)">
          <ZoomOut size={14} />
        </button>
        <button onClick={() => setViewport(canvas.viewport)} className="btn p-1.5" title="Fit to content (1)">
          <Maximize2 size={14} />
        </button>
        <span className="text-[10px] text-ink-dim font-mono px-2" data-testid="agent-canvas-zoom">{Math.round(viewport.zoom * 100)}%</span>
      </div>

      {/* SVG Canvas */}
      <svg
        ref={svgRef}
        data-testid="agent-canvas-svg"
        viewBox="0 0 1200 800"
        className="w-full h-full"
        style={{
          cursor: tool === "pan" ? "grab" : "default",
          backgroundColor: "#0b0d10",
          backgroundImage: "radial-gradient(circle, #21262d 1px, transparent 1px)",
          backgroundSize: "20px 20px",
        }}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        onWheel={onWheel}
      >
        <defs>
          <marker id="agent-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="context-stroke" />
          </marker>
        </defs>

        {/* connectors (在节点下面) */}
        {canvas.connectors.map(renderConnector)}
        {/* nodes */}
        {canvas.nodes.map(renderNode)}
      </svg>

      {/* minimap */}
      <div data-testid="agent-canvas-minimap" className="absolute bottom-3 right-3 z-20 w-40 h-28 bg-bg-card border border-line rounded-md overflow-hidden">
        <svg viewBox={`${bbox.minX} ${bbox.minY} ${bbox.maxX - bbox.minX} ${bbox.maxY - bbox.minY}`} className="w-full h-full">
          <rect x={viewport.x} y={viewport.y} width={1200 / viewport.zoom} height={800 / viewport.zoom} fill="none" stroke="#2f81f7" strokeWidth={2} />
          {canvas.nodes.map((n) => (
            <rect key={n.id} x={n.x} y={n.y} width={n.width} height={n.height} fill="#3fb950" opacity={0.6} />
          ))}
        </svg>
      </div>

      {/* 底部状态条 */}
      <div className="absolute bottom-3 left-3 z-20 text-[10px] text-ink-mute font-mono flex gap-3" data-testid="agent-canvas-statusbar">
        <span>zoom {Math.round(viewport.zoom * 100)}%</span>
        <span>nodes {canvas.nodes.length}</span>
        <span>connectors {canvas.connectors.length}</span>
        <span>selected {selectedNodeId ?? "—"}</span>
      </div>
    </div>
  );
}

// ---- 节点组件 (内部, 不用 export) ----

function NodeRect({ w, h, fill, stroke, strokeWidth = 1.5, radius }: { w: number; h: number; fill: string; stroke: string; strokeWidth?: number; radius: number }) {
  return <rect width={w} height={h} fill={fill} stroke={stroke} strokeWidth={strokeWidth} rx={radius} />;
}

function AgentNodeBody({ agent, w, h, zoom, gameState }: { agent: AgentSession; w: number; h: number; zoom: number; gameState: AgentGameState | null }) {
  // 拟人化游戏化: 在底部 tokens/cost 之上加 HP bar (per 拍板)
  const hp = gameState?.hp ?? MAX_HP;
  const alive = gameState?.alive ?? true;
  const hpPct = alive ? Math.round((hp / MAX_HP) * 100) : 0;
  return (
    <g>
      {/* icon */}
      <g transform={`translate(${12 * zoom}, ${12 * zoom})`}>
        <Bot size={16 * zoom} color={alive ? "#79c0ff" : "#6e7681"} strokeWidth={1.5} />
      </g>
      {/* kind */}
      <text x={(12 + 22) * zoom} y={(12 + 12) * zoom} fontSize={10 * zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
        {agent.agent_kind}
      </text>
      {/* id */}
      <text x={12 * zoom} y={(28 + 6) * zoom} fontSize={11 * zoom} fill={alive ? "#e6edf3" : "#6e7681"} fontFamily="ui-monospace, monospace">
        {agent.id}
      </text>
      {/* status pill (foreignObject) — agent 没有 StatusKind, 走 StatusPill 默认 prettify */}
      <foreignObject x={12 * zoom} y={(40) * zoom} width={(w - 24)} height={20 * zoom}>
        <div style={{ display: "flex", alignItems: "center", gap: 4 * zoom }}>
          <StatusPill value={agent.status as AgentStatus} size="xs" />
        </div>
      </foreignObject>
      {/* HP bar (拟人化游戏化, per 拍板) */}
      {gameState && (
        <g transform={`translate(${12 * zoom}, ${h - 36 * zoom})`}>
          <rect width={(w - 24)} height={5 * zoom} fill="#0b0d10" rx={2 * zoom} />
          <rect width={(w - 24) * (hpPct / 100)} height={5 * zoom} fill={hpPct <= 30 ? "#f85149" : "#3fb950"} rx={2 * zoom} />
        </g>
      )}
      {/* tokens + cost (底部两行) */}
      <g transform={`translate(${12 * zoom}, ${gameState ? h - 24 * zoom : h - 28 * zoom})`}>
        <Hash size={9 * zoom} color="#8b949e" />
      </g>
      <text x={26 * zoom} y={(gameState ? h - 16 * zoom : h - 20 * zoom)} fontSize={9 * zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
        {agent.token_usage.total.toLocaleString()} tokens
      </text>
      <g transform={`translate(${12 * zoom}, ${gameState ? h - 10 * zoom : h - 14 * zoom})`}>
        <Cpu size={9 * zoom} color="#8b949e" />
      </g>
      <text x={26 * zoom} y={(gameState ? h - 2 * zoom : h - 6 * zoom)} fontSize={9 * zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
        ${agent.cost_summary.usd.toFixed(2)} / ${agent.cost_summary.budget_usd.toFixed(2)}
      </text>
    </g>
  );
}

function WorktreeNodeBody({ worktree, w, h, zoom }: { worktree: Worktree; w: number; h: number; zoom: number }) {
  return (
    <g>
      <g transform={`translate(${10 * zoom}, ${10 * zoom})`}>
        <GitBranch size={12 * zoom} color="#8b949e" strokeWidth={1.5} />
      </g>
      <text x={28 * zoom} y={(10 + 9) * zoom} fontSize={9 * zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
        worktree
      </text>
      <text x={12 * zoom} y={(32) * zoom} fontSize={11 * zoom} fill="#e6edf3" fontFamily="ui-monospace, monospace">
        {worktree.branch}
      </text>
      <foreignObject x={12 * zoom} y={(h - 22) * zoom} width={(w - 24)} height={18 * zoom}>
        <div>
          <StatusPill value={worktree.status} size="xs" />
        </div>
      </foreignObject>
    </g>
  );
}

function WorkItemNodeBody({ wi, w, h, zoom }: { wi: WorkItem; w: number; h: number; zoom: number }) {
  // 标题截断
  const maxChars = Math.max(8, Math.floor((w - 24) / (6 * zoom)));
  const titleTrunc = wi.title.length > maxChars ? `${wi.title.slice(0, Math.max(1, maxChars - 1))}…` : wi.title;
  return (
    <g>
      <text x={10 * zoom} y={14 * zoom} fontSize={9 * zoom} fill="#8b949e" fontFamily="ui-monospace, monospace">
        {wi.key}
      </text>
      <text
        x={10 * zoom}
        y={28 * zoom}
        fontSize={10 * zoom}
        fill="#e6edf3"
        fontFamily="system-ui"
        style={{ pointerEvents: "none" }}
      >
        {titleTrunc}
      </text>
      <foreignObject x={10 * zoom} y={(h - 22) * zoom} width={(w - 20)} height={18 * zoom}>
        <div style={{ display: "flex", alignItems: "center", gap: 4 * zoom, justifyContent: "space-between" }}>
          <StatusPill value={wi.status as WorkItemStatus} size="xs" translateAs="workItem" />
          <span style={{ fontSize: 9 * zoom, color: "#6e7681", fontFamily: "ui-monospace, monospace" }}>{wi.priority.toUpperCase()}</span>
        </div>
      </foreignObject>
    </g>
  );
}
