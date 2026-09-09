"use client";

// =====================================================================
// RelationshipView — 图谱浏览 + 节点详情 (per brief v0.50 §2.1 A)
// =====================================================================
// Per docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §1.1
// + docs/design/BD-AGENT-RELATIONSHIP-001.md v0.1 §1.1.4
// + docs/frontend-canvas-design.md v0.1 无限画布
//
// SVG 绘制 nodes + edges; 节点 click → NodeDetail; WS events 高亮 (per doc 14 §2.3).
// =====================================================================

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useARGStore } from "@/lib/arg/store";
import type { Agent, Edge, Uuid, RelationshipType } from "@/lib/arg/types";
import { NodeDetail } from "./NodeDetail";

export interface RelationshipViewProps {
  /** Optional tenant id (RLS 13 類). */
  tenantId?: Uuid;
}

// ---- layout helpers ----

function layoutAgents(agents: Agent[], w: number, h: number) {
  const positions = new Map<Uuid, { x: number; y: number }>();
  if (agents.length === 0) return positions;
  const cols = Math.max(1, Math.ceil(Math.sqrt(agents.length * (w / h))));
  const rows = Math.ceil(agents.length / cols);
  const cellW = w / cols;
  const cellH = h / rows;
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

const EDGE_COLOR: Partial<Record<RelationshipType, string>> = {
  DELEGATES_TO: "#00f0ff",
  CONSULTS: "#ffc400",
  COLLABORATES_WITH: "#22c55e",
  REPORTS_TO: "#f97316",
  MENTORS: "#a855f7",
  PEER_REVIEWS: "#06b6d4",
  STAND_IN_FOR: "#ec4899",
  SHADOWS: "#94a3b8",
  CHALLENGES: "#ef4444",
  TRUSTS: "#10b981",
};

// =====================================================================
// Component
// =====================================================================

export function RelationshipView({ tenantId }: RelationshipViewProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [size, setSize] = useState({ width: 800, height: 500 });
  const [selectedId, setSelectedId] = useState<Uuid | null>(null);
  const [highlightEventId, setHighlightEventId] = useState<string | null>(null);

  const agentsMap = useARGStore((s) => s.agents);
  const edgesMap = useARGStore((s) => s.edges);
  const argEvents = useARGStore((s) => s.argEvents);
  const wsStatus = useARGStore((s) => s.wsStatus);
  const loadAgents = useARGStore((s) => s.loadAgents);
  const loadEdges = useARGStore((s) => s.loadEdges);
  const subscribeEvents = useARGStore((s) => s.subscribeEvents);
  const unsubscribeEvents = useARGStore((s) => s.unsubscribeEvents);

  // Initial load
  useEffect(() => {
    void loadAgents(tenantId);
    void loadEdges({}, tenantId);
    subscribeEvents(tenantId);
    return () => unsubscribeEvents();
  }, [loadAgents, loadEdges, subscribeEvents, unsubscribeEvents, tenantId]);

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

  const agents = useMemo(() => Array.from(agentsMap.values()), [agentsMap]);
  const edges = useMemo(() => Array.from(edgesMap.values()), [edgesMap]);
  const positions = useMemo(
    () => layoutAgents(agents, size.width, size.height),
    [agents, size.width, size.height],
  );
  const selectedAgent = selectedId ? agentsMap.get(selectedId) : null;

  // Pulse highlight when a new edge event arrives
  useEffect(() => {
    if (argEvents.length === 0) return;
    const latest = argEvents[argEvents.length - 1];
    const id = `evt-${argEvents.length}`;
    setHighlightEventId(id);
    const t = setTimeout(() => setHighlightEventId(null), 1500);
    return () => clearTimeout(t);
  }, [argEvents.length]);

  const onNodeClick = useCallback(
    (e: React.MouseEvent, id: Uuid) => {
      e.stopPropagation();
      setSelectedId(id);
    },
    [],
  );

  const onBackgroundClick = useCallback(() => setSelectedId(null), []);

  return (
    <div
      className="grid grid-cols-1 md:grid-cols-[1fr_320px] gap-3 h-full"
      data-testid="relationship-view"
    >
      <div className="relative border border-line rounded-md bg-[var(--cel-surface-card,#0f1422)] min-h-[400px]">
        <div className="absolute top-2 left-2 right-2 flex items-center justify-between text-[10px] font-mono text-ink-dim z-10">
          <span data-testid="relationship-view-summary">
            {agents.length} agents · {edges.length} edges
          </span>
          <span
            data-testid="relationship-view-ws-status"
            className={`pill border text-[10px] px-1.5 py-0.5 ${
              wsStatus === "connected"
                ? "border-ok/40 text-ok bg-ok/10"
                : wsStatus === "mock"
                ? "border-warn/40 text-warn bg-warn/10"
                : wsStatus === "connecting"
                ? "border-info/40 text-info bg-info/10"
                : "border-ink-mute/40 text-ink-mute bg-bg-soft"
            }`}
          >
            WS: {wsStatus}
          </span>
        </div>
        <svg
          ref={svgRef}
          className="w-full h-full"
          viewBox={`0 0 ${size.width} ${size.height}`}
          preserveAspectRatio="xMidYMid meet"
          onClick={onBackgroundClick}
          data-testid="relationship-view-svg"
        >
          {/* Edges */}
          <g data-testid="relationship-view-edges">
            {edges.map((e) => {
              const from = positions.get(e.from_agent);
              const to = positions.get(e.to_agent);
              if (!from || !to) return null;
              const color = EDGE_COLOR[e.edge_type] ?? "#94a3b8";
              const isUndirected = e.direction === "undirected";
              // Bezier connector (per frontend-canvas-design.md v0.1)
              const midX = (from.x + to.x) / 2;
              const midY = (from.y + to.y) / 2;
              const dx = to.x - from.x;
              const dy = to.y - from.y;
              const ctrlOffset = Math.min(80, Math.sqrt(dx * dx + dy * dy) * 0.3);
              const cx = midX + (dy / Math.sqrt(dx * dx + dy * dy || 1)) * -ctrlOffset;
              const cy = midY + (dx / Math.sqrt(dx * dx + dy * dy || 1)) * ctrlOffset;
              return (
                <g key={e.id} data-testid={`edge-${e.id}`}>
                  <path
                    d={`M ${from.x} ${from.y} Q ${cx} ${cy} ${to.x} ${to.y}`}
                    fill="none"
                    stroke={color}
                    strokeWidth={1 + e.weight * 2}
                    opacity={e.archived ? 0.3 : 0.8}
                    strokeDasharray={e.archived ? "4 3" : undefined}
                    markerEnd={isUndirected ? undefined : "url(#arrow)"}
                  />
                  <text
                    x={midX}
                    y={midY - 4}
                    fill="#94a3b8"
                    fontSize={9}
                    fontFamily="monospace"
                    textAnchor="middle"
                  >
                    {e.edge_type}
                  </text>
                </g>
              );
            })}
            <defs>
              <marker
                id="arrow"
                viewBox="0 0 10 10"
                refX={9}
                refY={5}
                markerWidth={6}
                markerHeight={6}
                orient="auto-start-reverse"
              >
                <path d="M 0 0 L 10 5 L 0 10 z" fill="#00f0ff" />
              </marker>
            </defs>
          </g>
          {/* Nodes */}
          <g data-testid="relationship-view-nodes">
            {agents.map((a) => {
              const p = positions.get(a.id);
              if (!p) return null;
              const isLead = a.archetype.startsWith("LEAD_");
              const isHighlighted =
                highlightEventId !== null && selectedId === a.id;
              return (
                <g
                  key={a.id}
                  transform={`translate(${p.x},${p.y})`}
                  onClick={(e) => onNodeClick(e, a.id)}
                  data-testid={`view-node-${a.id}`}
                  style={{ cursor: "pointer" }}
                >
                  <circle
                    r={isLead ? 26 : 18}
                    fill={isLead ? "#1e293b" : "#0f172a"}
                    stroke={isHighlighted ? "#22c55e" : isLead ? "#ffc400" : "#00f0ff"}
                    strokeWidth={isHighlighted ? 3 : 2}
                  />
                  <text
                    textAnchor="middle"
                    y={4}
                    fill="#e2e8f0"
                    fontSize={10}
                    fontFamily="monospace"
                  >
                    {a.name.length > 8 ? `${a.name.slice(0, 7)}…` : a.name}
                  </text>
                </g>
              );
            })}
          </g>
        </svg>
      </div>
      <div className="space-y-2">
        {selectedAgent ? (
          <NodeDetail agent={selectedAgent} onClose={() => setSelectedId(null)} />
        ) : (
          <div
            className="card text-xs text-ink-dim p-3"
            data-testid="relationship-view-empty"
          >
            Click a node to see its details.
          </div>
        )}
        <div
          className="card text-[10px] font-mono text-ink-dim p-2 max-h-40 overflow-auto"
          data-testid="relationship-view-event-log"
        >
          <div className="text-ink-mute mb-1 uppercase">recent events ({argEvents.length})</div>
          {argEvents.length === 0 ? (
            <div className="italic">no events yet</div>
          ) : (
            argEvents.slice(-8).map((ev, i) => (
              <div key={`${argEvents.length - i}-${ev.type}`} className="truncate">
                {ev.type}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
