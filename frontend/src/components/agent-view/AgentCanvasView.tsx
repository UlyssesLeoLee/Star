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
import type { AgentCanvas, AgentCanvasNode, AgentCanvasConnector } from "@/lib/agent-view/types";
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
  Crosshair, Shield, CheckCircle2,
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
}

export function AgentCanvasView({ canvas, agent, worktree, gameState, onClaim }: AgentCanvasViewProps) {
  const { t } = useTranslation();
  const workItems = useStore((s) => s.workItems);
  const workItemById = useMemo(
    () => new Map(workItems.map((w) => [w.id, w] as const)),
    [workItems],
  );
  const { colors, mode } = useAgentGameTheme();
  const [viewport, setViewport] = useState(canvas.viewport);
  const [tool, setTool] = useState<"select" | "pan">("pan");
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  const dragState = useRef<{ type: "pan" | null; startX: number; startY: number; elX: number; elY: number }>({
    type: null, startX: 0, startY: 0, elX: 0, elY: 0,
  });

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

  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button === 1 || (e.button === 0 && tool === "pan") || e.shiftKey) {
      dragState.current = { type: "pan", startX: e.clientX, startY: e.clientY, elX: viewport.x, elY: viewport.y };
    } else if (e.button === 0 && tool === "select") {
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

        {/* connectors */}
        {canvas.connectors.map(renderConnector)}
        {/* nodes */}
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
  const c = Math.max(6 * zoom, 8); // 切角大小
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
      <foreignObject x={14 * zoom} y={(h - 26) * zoom} width={(w - 28)} height={20 * zoom}>
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
