"use client";

// =====================================================================
// RelationshipEditor — 画布拖拽建关系 (per DD §4.13 + brief v0.50 §2.1 A)
// =====================================================================
// Per docs/design/DD-AGENT-RELATIONSHIP-001.md v0.1.1 §4.13
// Per docs/design/BD-AGENT-RELATIONSHIP-001.md v0.1 §1.1.4
// Per docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §1.1
//
// 复用 frontend-canvas-design.md v0.1 无限画布概念 (但 ARG.5 范围不引入 react-flow
// 等新依赖 — 用 SVG 自绘轻量画布, 节点拖拽 → 释放到另一节点上 → 弹 EdgeTypeSelector).
// 落库走 useARGStore.createEdge, 不直接 fetch.
// =====================================================================

import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useARGStore } from "@/lib/arg/store";
import type { Agent, Uuid } from "@/lib/arg/types";
import { EdgeTypeSelector } from "./EdgeTypeSelector";
import type { RelationshipType } from "@/lib/arg/types";

export interface RelationshipEditorProps {
  /** Optional initial agents (overrides store). */
  initialAgents?: Agent[];
  /** Optional tenant id (RLS 13 類). */
  tenantId?: Uuid;
  /** Optional creator id (per 守门 #10). */
  createdBy?: Uuid;
}

// =====================================================================
// Internal: simple 5×N force-free layout
// =====================================================================

function layoutAgents(
  agents: Agent[],
  width: number,
  height: number,
): Map<Uuid, { x: number; y: number }> {
  const positions = new Map<Uuid, { x: number; y: number }>();
  if (agents.length === 0) return positions;
  const cols = Math.max(1, Math.ceil(Math.sqrt(agents.length * (width / height))));
  const rows = Math.ceil(agents.length / cols);
  const cellW = width / cols;
  const cellH = height / rows;
  agents.forEach((a, idx) => {
    const col = idx % cols;
    const row = Math.floor(idx / cols);
    positions.set(a.id, {
      x: cellW * (col + 0.5),
      y: cellH * (row + 0.5),
    });
  });
  return positions;
}

// =====================================================================
// Component
// =====================================================================

export function RelationshipEditor({
  initialAgents,
  tenantId,
  createdBy,
}: RelationshipEditorProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [draggingFrom, setDraggingFrom] = useState<Uuid | null>(null);
  const [dragPos, setDragPos] = useState<{ x: number; y: number } | null>(null);
  const [pendingEdge, setPendingEdge] = useState<{
    from: Uuid;
    to: Uuid;
  } | null>(null);
  const [size, setSize] = useState({ width: 800, height: 500 });

  const agentsMap = useARGStore((s) => s.agents);
  const agentsLoading = useARGStore((s) => s.agentsLoading);
  const agents = useMemo<Agent[]>(() => {
    if (initialAgents) return initialAgents;
    return Array.from(agentsMap.values());
  }, [initialAgents, agentsMap]);

  const loadAgents = useARGStore((s) => s.loadAgents);
  const createEdge = useARGStore((s) => s.createEdge);

  // Initial load (per 守门 #9 不调外部 fetch 在 SSG; 实际通过 store action 走)
  useEffect(() => {
    if (!initialAgents && agentsMap.size === 0 && !agentsLoading) {
      void loadAgents(tenantId);
    }
  }, [initialAgents, agentsMap.size, agentsLoading, loadAgents, tenantId]);

  // Resize observer
  useEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    const el = svgRef.current?.parentElement;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) {
        const cr = e.contentRect;
        setSize({ width: cr.width, height: cr.height });
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const positions = useMemo(
    () => layoutAgents(agents, size.width, size.height),
    [agents, size.width, size.height],
  );

  // --- mouse handlers (SVG coordinate space) ---
  const getSvgCoords = useCallback(
    (clientX: number, clientY: number): { x: number; y: number } | null => {
      const svg = svgRef.current;
      if (!svg) return null;
      const rect = svg.getBoundingClientRect();
      return {
        x: ((clientX - rect.left) / rect.width) * size.width,
        y: ((clientY - rect.top) / rect.height) * size.height,
      };
    },
    [size.width, size.height],
  );

  const onNodeMouseDown = useCallback(
    (e: React.MouseEvent, agentId: Uuid) => {
      e.preventDefault();
      const pt = getSvgCoords(e.clientX, e.clientY);
      if (!pt) return;
      setDraggingFrom(agentId);
      setDragPos(pt);
    },
    [getSvgCoords],
  );

  const onMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!draggingFrom) return;
      const pt = getSvgCoords(e.clientX, e.clientY);
      if (pt) setDragPos(pt);
    },
    [draggingFrom, getSvgCoords],
  );

  const onMouseUp = useCallback(() => {
    if (draggingFrom) {
      setDraggingFrom(null);
      setDragPos(null);
    }
  }, [draggingFrom]);

  const onNodeMouseUp = useCallback(
    (e: React.MouseEvent, targetId: Uuid) => {
      if (draggingFrom && draggingFrom !== targetId) {
        setPendingEdge({ from: draggingFrom, to: targetId });
        setDraggingFrom(null);
        setDragPos(null);
      }
    },
    [draggingFrom],
  );

  const handleEdgeTypeSelect = useCallback(
    async (type: RelationshipType, weight: number) => {
      if (!pendingEdge) return;
      const isUndirected = type === "COLLABORATES_WITH" || type === "PEER_REVIEWS";
      try {
        await createEdge(
          {
            from_agent: pendingEdge.from,
            to_agent: pendingEdge.to,
            edge_type: type,
            weight,
            direction: isUndirected ? "undirected" : "directed",
            tenant_id: tenantId ?? "00000000-0000-0000-0000-000000000000",
            created_by: createdBy ?? "00000000-0000-0000-0000-000000000000",
          },
          tenantId,
        );
      } catch (err) {
        // store records edgesError; surface in UI via banner
        console.warn("[RelationshipEditor] createEdge failed", err);
      } finally {
        setPendingEdge(null);
      }
    },
    [pendingEdge, createEdge, tenantId, createdBy],
  );

  return (
    <div
      className="relative w-full h-full min-h-[400px] border border-line rounded-md bg-[var(--cel-surface-card,#0f1422)]"
      data-testid="relationship-editor"
    >
      <div className="absolute top-2 left-2 text-[10px] font-mono text-ink-dim z-10">
        Drag from one agent node to another to create a relationship.
        {agents.length === 0 && !agentsLoading && (
          <span className="text-warn ml-2">No agents loaded.</span>
        )}
      </div>
      <svg
        ref={svgRef}
        className="w-full h-full"
        viewBox={`0 0 ${size.width} ${size.height}`}
        preserveAspectRatio="xMidYMid meet"
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        data-testid="relationship-editor-svg"
      >
        {/* Edges (visual placeholder; full view lives in RelationshipView) */}
        <g data-testid="relationship-editor-edges" />
        {/* Dragging preview */}
        {draggingFrom && dragPos && positions.get(draggingFrom) && (
          <line
            x1={positions.get(draggingFrom)!.x}
            y1={positions.get(draggingFrom)!.y}
            x2={dragPos.x}
            y2={dragPos.y}
            stroke="#00f0ff"
            strokeWidth={2}
            strokeDasharray="6 4"
            data-testid="drag-preview"
          />
        )}
        {/* Agent nodes */}
        <g data-testid="relationship-editor-nodes">
          {agents.map((a) => {
            const p = positions.get(a.id);
            if (!p) return null;
            const isDomainLead = a.archetype.startsWith("LEAD_");
            return (
              <g
                key={a.id}
                transform={`translate(${p.x},${p.y})`}
                onMouseDown={(e) => onNodeMouseDown(e, a.id)}
                onMouseUp={(e) => onNodeMouseUp(e, a.id)}
                data-testid={`agent-node-${a.id}`}
                style={{ cursor: "grab" }}
              >
                <circle
                  r={isDomainLead ? 28 : 20}
                  fill={isDomainLead ? "#1e293b" : "#0f172a"}
                  stroke={isDomainLead ? "#ffc400" : "#00f0ff"}
                  strokeWidth={2}
                />
                <text
                  textAnchor="middle"
                  y={4}
                  fill="#e2e8f0"
                  fontSize={11}
                  fontFamily="monospace"
                >
                  {a.name.length > 8 ? `${a.name.slice(0, 7)}…` : a.name}
                </text>
                <text
                  textAnchor="middle"
                  y={isDomainLead ? 44 : 36}
                  fill="#94a3b8"
                  fontSize={9}
                  fontFamily="monospace"
                >
                  {a.archetype}
                </text>
              </g>
            );
          })}
        </g>
      </svg>
      {pendingEdge && (
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-20">
          <EdgeTypeSelector
            onSelect={handleEdgeTypeSelect}
            onCancel={() => setPendingEdge(null)}
          />
        </div>
      )}
    </div>
  );
}
