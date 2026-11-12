/*
@cypher schema=1 source_sha256=2026-11-11-workflow-builder-v1
MERGE (self:File {path:"frontend/src/components/agent-workflow/WorkflowBuilder.tsx"})
MERGE (builder:Function {name:"WorkflowBuilder", language:"typescript", visibility:"public"})
MERGE (canvas:Function {name:"WorkflowCanvas", language:"typescript", visibility:"private"})
MERGE (palette:Function {name:"WorkflowPalette", language:"typescript", visibility:"private"})
MERGE (panel:Function {name:"WorkflowEditorPanel", language:"typescript", visibility:"private"})
MERGE (nodeCard:Function {name:"WorkflowNodeCard", language:"typescript", visibility:"private"})
MERGE (self)-[:DEFINES]->(builder)
MERGE (self)-[:DEFINES]->(canvas)
MERGE (self)-[:DEFINES]->(palette)
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:DEFINES]->(nodeCard)
@endcypher
*/
"use client";

// =====================================================================
// WorkflowBuilder — /agent-view 的 n8n 风格工作流构建器 (per 2026-11-11 用户发令)
// 每个节点卡片必带输入/输出端口; 控制流组件完备: trigger / if-else / for / switch /
// merge / code / http / wait; IPA 技能节点按 IPA-Atomic-Skills 目录生成端口契约。
// =====================================================================

import { useCallback, useMemo, useRef, useState } from "react";
import {
  GitBranch,
  Repeat,
  Split,
  Merge,
  Code2,
  Globe,
  Clock,
  Play,
  FileText,
  Trash2,
  ZoomIn,
  ZoomOut,
  Plus,
} from "lucide-react";
import type {
  WorkflowNode,
  WorkflowPort,
  WorkflowWire,
  WorkflowConditionRule,
} from "@/lib/agent-workflow/types";
import { CONDITION_OPERATORS } from "@/lib/agent-workflow/types";
import {
  CONTROL_NODE_LIBRARY,
  makeControlNode,
  makeIpaSkillNode,
  canConnect,
  nextWireId,
  createExampleWorkflow,
} from "@/lib/agent-workflow/workflowLib";
import { IPA_PHASE_COLUMNS, IPA_SKILL_CATALOG, type IpaSkillEntry } from "@/lib/ipa-skills/catalog";

const KIND_ICON: Record<string, React.ReactNode> = {
  trigger: <Play size={12} />,
  "ipa-skill": <FileText size={12} />,
  "if-else": <GitBranch size={12} />,
  "for-loop": <Repeat size={12} />,
  switch: <Split size={12} />,
  merge: <Merge size={12} />,
  code: <Code2 size={12} />,
  http: <Globe size={12} />,
  wait: <Clock size={12} />,
};

const KIND_TONE: Record<string, string> = {
  trigger: "border-lime-400/60 bg-lime-400/10 text-lime-300",
  "ipa-skill": "border-cyan-400/60 bg-cyan-400/10 text-cyan-300",
  "if-else": "border-amber-400/60 bg-amber-400/10 text-amber-300",
  "for-loop": "border-violet-400/60 bg-violet-400/10 text-violet-300",
  switch: "border-pink-400/60 bg-pink-400/10 text-pink-300",
  merge: "border-sky-400/60 bg-sky-400/10 text-sky-300",
  code: "border-emerald-400/60 bg-emerald-400/10 text-emerald-300",
  http: "border-blue-400/60 bg-blue-400/10 text-blue-300",
  wait: "border-slate-400/60 bg-slate-400/10 text-slate-300",
};

const NODE_WIDTH = 220;
const PORT_RADIUS = 7;

function portY(node: WorkflowNode, direction: "in" | "out", index: number): number {
  const ports = direction === "in" ? node.inputs : node.outputs;
  const header = 46;
  const spacing = 26;
  const total = ports.length * spacing;
  const startY = node.position.y + header + 10;
  const offset = total > 0 ? (Math.max(total, 34) - total) / 2 + index * spacing : 0;
  return startY + offset;
}

function portAnchor(
  node: WorkflowNode,
  portId: string,
  direction: "in" | "out",
): { x: number; y: number } {
  const ports = direction === "in" ? node.inputs : node.outputs;
  const index = Math.max(0, ports.findIndex((p) => p.id === portId));
  return {
    x: node.position.x + (direction === "in" ? 0 : NODE_WIDTH),
    y: portY(node, direction, index) + PORT_RADIUS / 2,
  };
}

function wirePath(
  nodes: WorkflowNode[],
  wire: WorkflowWire,
): string | null {
  const fromNode = nodes.find((n) => n.id === wire.fromNode);
  const toNode = nodes.find((n) => n.id === wire.toNode);
  if (!fromNode || !toNode) return null;
  const from = portAnchor(fromNode, wire.fromPort, "out");
  const to = portAnchor(toNode, wire.toPort, "in");
  const dx = Math.max(60, Math.abs(to.x - from.x) * 0.5);
  return `M ${from.x} ${from.y} C ${from.x + dx} ${from.y}, ${to.x - dx} ${to.y}, ${to.x} ${to.y}`;
}

interface DragState {
  kind: "node" | "wire" | "pan";
  nodeId?: string;
  startX: number;
  startY: number;
  originX?: number;
  originY?: number;
  fromNode?: string;
  fromPort?: string;
}

export function WorkflowBuilder() {
  const [workflow, setWorkflow] = useState<{ nodes: WorkflowNode[]; wires: WorkflowWire[] }>(
    createExampleWorkflow,
  );
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [viewport, setViewport] = useState({ x: 0, y: 0, zoom: 0.85 });
  const [paletteOpen, setPaletteOpen] = useState(true);
  const [paletteQuery, setPaletteQuery] = useState("");
  const dragRef = useRef<DragState | null>(null);
  const [tempWire, setTempWire] = useState<{ fromX: number; fromY: number; toX: number; toY: number } | null>(null);

  const selectedNode = useMemo(
    () => workflow.nodes.find((n) => n.id === selectedNodeId) ?? null,
    [workflow.nodes, selectedNodeId],
  );

  const toCanvas = useCallback(
    (clientX: number, clientY: number, svg: SVGSVGElement) => {
      const rect = svg.getBoundingClientRect();
      return {
        x: (clientX - rect.left - viewport.x) / viewport.zoom,
        y: (clientY - rect.top - viewport.y) / viewport.zoom,
      };
    },
    [viewport],
  );

  const onPointerDown = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      const svg = event.currentTarget;
      const target = event.target as Element;
      const nodeHit = target.closest("[data-node-id]");
      const portHit = target.closest("[data-port]");
      if (portHit) {
        const nodeId = portHit.getAttribute("data-node")!;
        const portId = portHit.getAttribute("data-port")!;
        const direction = portHit.getAttribute("data-direction")! as "in" | "out";
        if (direction === "out") {
          const node = workflow.nodes.find((n) => n.id === nodeId)!;
          const anchor = portAnchor(node, portId, "out");
          dragRef.current = { kind: "wire", fromNode: nodeId, fromPort: portId, startX: anchor.x, startY: anchor.y };
          setTempWire({ fromX: anchor.x, fromY: anchor.y, toX: anchor.x, toY: anchor.y });
        } else {
          // 拖到已有输入端口: 起点用它的线反向接? n8n 支持反向重接; 简化: 仅支持从输出发起
          dragRef.current = null;
        }
        return;
      }
      if (nodeHit) {
        const nodeId = nodeHit.getAttribute("data-node-id")!;
        setSelectedNodeId(nodeId);
        const node = workflow.nodes.find((n) => n.id === nodeId)!;
        const point = toCanvas(event.clientX, event.clientY, svg);
        dragRef.current = {
          kind: "node",
          nodeId,
          startX: point.x,
          startY: point.y,
          originX: node.position.x,
          originY: node.position.y,
        };
        return;
      }
      dragRef.current = { kind: "pan", startX: event.clientX - viewport.x, startY: event.clientY - viewport.y };
      setSelectedNodeId(null);
    },
    [workflow.nodes, toCanvas, viewport],
  );

  const onPointerMove = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      const drag = dragRef.current;
      if (!drag) return;
      const svg = event.currentTarget;
      if (drag.kind === "pan") {
        setViewport((v) => ({ ...v, x: event.clientX - drag.startX, y: event.clientY - drag.startY }));
        return;
      }
      const point = toCanvas(event.clientX, event.clientY, svg);
      if (drag.kind === "node" && drag.nodeId) {
        setWorkflow((w) => ({
          ...w,
          nodes: w.nodes.map((n) =>
            n.id === drag.nodeId
              ? {
                  ...n,
                  position: {
                    x: (drag.originX ?? 0) + point.x - drag.startX,
                    y: (drag.originY ?? 0) + point.y - drag.startY,
                  },
                }
              : n,
          ),
        }));
      } else if (drag.kind === "wire") {
        setTempWire((t) => (t ? { ...t, toX: point.x, toY: point.y } : t));
      }
    },
    [toCanvas],
  );

  const onPointerUp = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      const drag = dragRef.current;
      dragRef.current = null;
      if (drag?.kind === "wire" && drag.fromNode && drag.fromPort) {
        const target = event.target as Element;
        const portHit = target.closest("[data-port]");
        if (portHit) {
          const toNode = portHit.getAttribute("data-node")!;
          const toPort = portHit.getAttribute("data-port")!;
          const direction = portHit.getAttribute("data-direction")!;
          const fromNodeObj = workflow.nodes.find((n) => n.id === drag.fromNode)!;
          const toNodeObj = workflow.nodes.find((n) => n.id === toNode)!;
          const fromPortObj = fromNodeObj.outputs.find((p) => p.id === drag.fromPort)!;
          const toPortObj = toNodeObj.inputs.find((p) => p.id === toPort)!;
          if (
            direction === "in" &&
            fromNodeObj.id !== toNodeObj.id &&
            canConnect(workflow.wires, fromPortObj, toPortObj)
          ) {
            setWorkflow((w) => ({
              ...w,
              wires: [
                ...w.wires,
                {
                  id: nextWireId(),
                  fromNode: drag.fromNode!,
                  fromPort: drag.fromPort!,
                  toNode,
                  toPort,
                },
              ],
            }));
          }
        }
      }
      setTempWire(null);
    },
    [workflow.nodes, workflow.wires],
  );

  const addNode = useCallback(
    (node: WorkflowNode) => {
      setWorkflow((w) => ({ ...w, nodes: [...w.nodes, node] }));
      setSelectedNodeId(node.id);
    },
    [],
  );

  const deleteNode = useCallback(
    (nodeId: string) => {
      setWorkflow((w) => ({
        nodes: w.nodes.filter((n) => n.id !== nodeId),
        wires: w.wires.filter((wire) => wire.fromNode !== nodeId && wire.toNode !== nodeId),
      }));
      setSelectedNodeId(null);
    },
    [],
  );

  const deleteWire = useCallback((wireId: string) => {
    setWorkflow((w) => ({ ...w, wires: w.wires.filter((wire) => wire.id !== wireId) }));
  }, []);

  const updateNodeConfig = useCallback((nodeId: string, patch: Record<string, unknown>) => {
    setWorkflow((w) => ({
      ...w,
      nodes: w.nodes.map((n) => (n.id === nodeId ? { ...n, config: { ...n.config, ...patch } } : n)),
    }));
  }, []);

  const addSkillNode = useCallback((skill: IpaSkillEntry) => {
    addNode(makeIpaSkillNode(skill, { x: 140 + Math.random() * 120, y: 120 + Math.random() * 160 }));
  }, [addNode]);

  const filteredSkills = useMemo(() => {
    const query = paletteQuery.trim().toLowerCase();
    if (!query) return null;
    return IPA_SKILL_CATALOG.filter(
      (s) =>
        s.id.includes(query) ||
        s.titleJa.toLowerCase().includes(query) ||
        s.phaseJa.toLowerCase().includes(query),
    ).slice(0, 40);
  }, [paletteQuery]);

  const zoomBy = useCallback((delta: number) => {
    setViewport((v) => ({ ...v, zoom: Math.min(2, Math.max(0.4, v.zoom + delta)) }));
  }, []);

  return (
    <div className="flex h-[calc(100vh-140px)] min-h-[520px] rounded-lg border border-line bg-[var(--cel-surface-stage,#0b0d10)] overflow-hidden">
      {/* ── 左侧节点面板 ── */}
      <div className="w-64 shrink-0 border-r border-line bg-[var(--cel-surface-card,#0f1422)] flex flex-col">
        <div className="p-3 border-b border-line flex items-center justify-between">
          <span className="text-sm font-semibold text-ink">组件面板</span>
          <button
            data-testid="workflow-palette-toggle"
            className="btn-ghost p-1 rounded"
            onClick={() => setPaletteOpen((v) => !v)}
            aria-label="toggle palette"
          >
            {paletteOpen ? <ZoomOut size={14} /> : <ZoomIn size={14} />}
          </button>
        </div>
        {paletteOpen && (
          <div className="flex-1 overflow-y-auto p-2 space-y-4">
            <input
              data-testid="workflow-palette-search"
              value={paletteQuery}
              onChange={(e) => setPaletteQuery(e.target.value)}
              placeholder="搜索技能 (id / 名称 / 工程)…"
              className="input w-full text-xs"
            />
            <div>
              <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-2">
                控制流组件
              </div>
              <div className="grid grid-cols-2 gap-1.5">
                {CONTROL_NODE_LIBRARY.map((def) => (
                  <button
                    key={def.kind}
                    data-testid={`workflow-add-${def.kind}`}
                    className="flex items-center gap-1.5 rounded border border-line bg-bg-soft/40 hover:border-accent/50 px-2 py-1.5 text-left"
                    onClick={() => addNode(makeControlNode(def, { x: 140 + Math.random() * 120, y: 120 + Math.random() * 160 }))}
                  >
                    <span className={`flex items-center rounded px-1 py-0.5 border ${KIND_TONE[def.kind]}`}>
                      {KIND_ICON[def.kind]}
                    </span>
                    <span className="text-[11px] leading-tight text-ink">{def.title.split(" ")[0]}</span>
                  </button>
                ))}
              </div>
            </div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-1">
              IPA 工程技能 ({IPA_SKILL_CATALOG.length})
            </div>
            {filteredSkills
              ? filteredSkills.map((skill) => (
                  <SkillSlot key={skill.id} skill={skill} onAdd={addSkillNode} />
                ))
              : IPA_PHASE_COLUMNS.map((column) => (
                  <div key={column.phase}>
                    <div className="text-[11px] font-semibold text-accent/80 mb-1">
                      {column.phase} {column.ja} · {column.skillCount}
                    </div>
                    <div className="space-y-1">
                      {IPA_SKILL_CATALOG.filter((s) => s.phase === column.phase)
                        .slice(0, 5)
                        .map((skill) => (
                          <SkillSlot key={skill.id} skill={skill} onAdd={addSkillNode} />
                        ))}
                    </div>
                  </div>
                ))}
          </div>
        )}
      </div>

      {/* ── 中间画布 ── */}
      <div className="flex-1 relative">
        <svg
          data-testid="workflow-canvas"
          className="w-full h-full touch-none"
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerLeave={() => {
            dragRef.current = null;
            setTempWire(null);
          }}
        >
          <defs>
            <pattern id="wf-grid" width="24" height="24" patternUnits="userSpaceOnUse">
              <path d="M 24 0 L 0 0 0 24" fill="none" stroke="rgba(120,140,160,0.10)" strokeWidth="1" />
            </pattern>
          </defs>
          <g transform={`translate(${viewport.x},${viewport.y}) scale(${viewport.zoom})`}>
            <rect x={-4000} y={-4000} width={8000} height={8000} fill="url(#wf-grid)" />
            {workflow.wires.map((wire) => {
              const path = wirePath(workflow.nodes, wire);
              if (!path) return null;
              return (
                <g
                  key={wire.id}
                  data-testid="workflow-wire"
                  className="cursor-pointer"
                  onClick={() => deleteWire(wire.id)}
                >
                  <path d={path} fill="none" stroke="rgba(0,0,0,0.9)" strokeWidth={5} />
                  <path d={path} fill="none" stroke="#38bdf8" strokeWidth={2.2} />
                </g>
              );
            })}
            {tempWire && (
              <path
                d={`M ${tempWire.fromX} ${tempWire.fromY} C ${tempWire.fromX + 60} ${tempWire.fromY}, ${tempWire.toX - 60} ${tempWire.toY}, ${tempWire.toX} ${tempWire.toY}`}
                fill="none"
                stroke="#38bdf8"
                strokeWidth={2}
                strokeDasharray="6 4"
              />
            )}
            {workflow.nodes.map((node) => (
              <WorkflowNodeCard
                key={node.id}
                node={node}
                selected={node.id === selectedNodeId}
              />
            ))}
          </g>
        </svg>
        <div className="absolute bottom-3 right-3 flex gap-1.5">
          <button className="btn-ghost p-1.5 rounded border border-line" onClick={() => zoomBy(0.1)} aria-label="zoom in">
            <ZoomIn size={14} />
          </button>
          <button className="btn-ghost p-1.5 rounded border border-line" onClick={() => zoomBy(-0.1)} aria-label="zoom out">
            <ZoomOut size={14} />
          </button>
        </div>
      </div>

      {/* ── 右侧编辑面板 ── */}
      {selectedNode && (
        <WorkflowEditorPanel
          node={selectedNode}
          onClose={() => setSelectedNodeId(null)}
          onDelete={() => deleteNode(selectedNode.id)}
          onConfigChange={(patch) => updateNodeConfig(selectedNode.id, patch)}
        />
      )}
    </div>
  );
}

function SkillSlot({ skill, onAdd }: { skill: IpaSkillEntry; onAdd: (skill: IpaSkillEntry) => void }) {
  return (
    <button
      data-testid={`workflow-add-skill-${skill.id}`}
      className="w-full flex items-center gap-2 rounded border border-line/60 bg-bg-soft/30 hover:border-accent/50 px-2 py-1 text-left"
      onClick={() => onAdd(skill)}
      title={skill.purpose}
    >
      <span className="font-mono text-[10px] text-ink-dim shrink-0">{skill.id}</span>
      <span className="text-[11px] text-ink truncate">{skill.titleJa}</span>
      <span className="ml-auto flex items-center gap-0.5 shrink-0">
        <span className="text-[9px] font-mono text-accent">in{skill.inputs.length}</span>
        <span className="text-[9px] font-mono text-emerald-400/80">out{skill.outputs.length}</span>
        {skill.hasHumanGate && <span className="text-[9px] text-amber-400">👤</span>}
      </span>
    </button>
  );
}

function WorkflowNodeCard({ node, selected }: { node: WorkflowNode; selected: boolean }) {
  const height = 46 + 10 + Math.max(node.inputs.length, node.outputs.length, 1) * 26 + 8;
  return (
    <g data-node-id={node.id} data-testid={`workflow-node-${node.id}`} className="cursor-move">
      {/* 卡片体 */}
      <rect
        x={node.position.x}
        y={node.position.y}
        width={NODE_WIDTH}
        height={height}
        rx={8}
        fill="var(--cel-surface-card,#0f1422)"
        stroke={selected ? "#38bdf8" : "rgba(120,140,160,0.35)"}
        strokeWidth={selected ? 2 : 1}
      />
      {/* 头部 */}
      <rect
        x={node.position.x}
        y={node.position.y}
        width={NODE_WIDTH}
        height={46}
        rx={8}
        fill="rgba(56,189,248,0.08)"
        stroke="none"
      />
      <rect
        x={node.position.x}
        y={node.position.y + 38}
        width={NODE_WIDTH}
        height={8}
        fill="rgba(56,189,248,0.08)"
      />
      <text
        x={node.position.x + 34}
        y={node.position.y + 22}
        fontSize={12}
        fontWeight={600}
        fill="var(--cel-text-primary,#fff)"
      >
        {node.title.length > 24 ? `${node.title.slice(0, 24)}…` : node.title}
      </text>
      {node.subtitle && (
        <text x={node.position.x + 34} y={node.position.y + 37} fontSize={9} fill="#94a3b8">
          {node.subtitle}
        </text>
      )}
      {/* 头部图标占位 */}
      <g transform={`translate(${node.position.x + 12}, ${node.position.y + 12})`}>
        <rect width={16} height={16} rx={3} fill="rgba(15,20,34,0.9)" stroke="rgba(56,189,248,0.5)" strokeWidth={1} />
      </g>

      {/* 输入端口 (左侧) */}
      {node.inputs.map((port, index) => (
        <g
          key={port.id}
          data-port={port.id}
          data-node={node.id}
          data-direction="in"
          data-testid={`workflow-port-in-${node.id}-${port.id}`}
          className="cursor-crosshair"
        >
          <circle
            cx={node.position.x}
            cy={portY(node, "in", index) + PORT_RADIUS / 2}
            r={PORT_RADIUS}
            fill="#0f1422"
            stroke="#38bdf8"
            strokeWidth={1.6}
          />
          <text
            x={node.position.x + 12}
            y={portY(node, "in", index) + PORT_RADIUS / 2 + 3}
            fontSize={8.5}
            fill="#94a3b8"
          >
            {port.label.length > 14 ? `${port.label.slice(0, 14)}…` : port.label}
          </text>
        </g>
      ))}

      {/* 输出端口 (右侧) */}
      {node.outputs.map((port, index) => {
        const y = portY(node, "out", index) + PORT_RADIUS / 2;
        const label = port.label.length > 12 ? `${port.label.slice(0, 12)}…` : port.label;
        return (
          <g
            key={port.id}
            data-port={port.id}
            data-node={node.id}
            data-direction="out"
            data-testid={`workflow-port-out-${node.id}-${port.id}`}
            className="cursor-crosshair"
          >
            <circle cx={node.position.x + NODE_WIDTH} cy={y} r={PORT_RADIUS} fill="#0f1422" stroke="#34d399" strokeWidth={1.6} />
            <text x={node.position.x + NODE_WIDTH - 12} y={y + 3} fontSize={8.5} fill="#94a3b8" textAnchor="end">
              {label}
            </text>
          </g>
        );
      })}
    </g>
  );
}

function WorkflowEditorPanel({
  node,
  onClose,
  onDelete,
  onConfigChange,
}: {
  node: WorkflowNode;
  onClose: () => void;
  onDelete: () => void;
  onConfigChange: (patch: Record<string, unknown>) => void;
}) {
  const config = node.config;
  return (
    <div className="w-80 shrink-0 border-l border-line bg-[var(--cel-surface-card,#0f1422)] p-3 overflow-y-auto">
      <div className="flex items-center justify-between mb-3">
        <span className={`flex items-center gap-1.5 rounded border px-2 py-0.5 text-[11px] ${KIND_TONE[node.kind]}`}>
          {KIND_ICON[node.kind]} {node.title}
        </span>
        <div className="flex gap-1">
          <button data-testid="workflow-node-delete" className="btn-ghost p-1 rounded hover:text-red-400" onClick={onDelete} aria-label="delete node">
            <Trash2 size={14} />
          </button>
          <button className="btn-ghost p-1 rounded" onClick={onClose} aria-label="close panel">
            ✕
          </button>
        </div>
      </div>

      {node.kind === "if-else" && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-warn">条件规则 (全部满足才走 True)</div>
          {(config.rules as WorkflowConditionRule[]).map((rule, index) => (
            <div key={index} className="card !p-2 space-y-1.5">
              <div className="grid grid-cols-3 gap-1.5">
                <input
                  data-testid="workflow-if-field"
                  className="input text-[11px] col-span-1"
                  value={rule.field}
                  placeholder="字段"
                  onChange={(e) => {
                    const rules = (config.rules as WorkflowConditionRule[]).map((r, i) =>
                      i === index ? { ...r, field: e.target.value } : r,
                    );
                    onConfigChange({ rules });
                  }}
                />
                <select
                  data-testid="workflow-if-operator"
                  className="input text-[11px] col-span-1"
                  value={rule.operator}
                  onChange={(e) => {
                    const rules = (config.rules as WorkflowConditionRule[]).map((r, i) =>
                      i === index
                        ? { ...r, operator: e.target.value as WorkflowConditionRule["operator"] }
                        : r,
                    );
                    onConfigChange({ rules });
                  }}
                >
                  {CONDITION_OPERATORS.map((op) => (
                    <option key={op.id} value={op.id}>
                      {op.label}
                    </option>
                  ))}
                </select>
                <input
                  data-testid="workflow-if-value"
                  className="input text-[11px] col-span-1"
                  value={rule.value}
                  placeholder="值"
                  onChange={(e) => {
                    const rules = (config.rules as WorkflowConditionRule[]).map((r, i) =>
                      i === index ? { ...r, value: e.target.value } : r,
                    );
                    onConfigChange({ rules });
                  }}
                />
              </div>
              <button
                className="text-[10px] text-red-400/80 hover:text-red-300"
                onClick={() =>
                  onConfigChange({ rules: (config.rules as WorkflowConditionRule[]).filter((_, i) => i !== index) })
                }
              >
                删除此规则
              </button>
            </div>
          ))}
          <button
            data-testid="workflow-if-add-rule"
            className="btn-secondary w-full text-xs"
            onClick={() =>
              onConfigChange({
                rules: [...(config.rules as WorkflowConditionRule[]), { field: "", operator: "equals", value: "" }],
              })
            }
          >
            <Plus size={12} /> 添加规则
          </button>
        </div>
      )}

      {node.kind === "for-loop" && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-violet-300">循环配置</div>
          <label className="text-[11px] text-ink-dim">迭代方式</label>
          <select
            data-testid="workflow-for-mode"
            className="input text-xs"
            value={(config.iterateMode as string) ?? "count"}
            onChange={(e) => onConfigChange({ iterateMode: e.target.value })}
          >
            <option value="count">固定次数</option>
            <option value="expression">遍历表达式 ($items)</option>
          </select>
          {config.iterateMode === "count" ? (
            <div className="space-y-1">
              <label className="text-[11px] text-ink-dim">次数</label>
              <input
                data-testid="workflow-for-count"
                type="number"
                min={1}
                className="input text-xs"
                value={(config.count as number) ?? 3}
                onChange={(e) => onConfigChange({ count: Math.max(1, Number(e.target.value)) })}
              />
            </div>
          ) : (
            <div className="space-y-1">
              <label className="text-[11px] text-ink-dim">表达式</label>
              <input
                className="input text-xs font-mono"
                value={(config.expression as string) ?? "$items"}
                onChange={(e) => onConfigChange({ expression: e.target.value })}
              />
            </div>
          )}
          <div className="text-[10px] text-ink-mute">
            输出端口: <span className="text-emerald-400">Loop</span> 循环体 ·{" "}
            <span className="text-emerald-400">Items</span> 结果集 ·{" "}
            <span className="text-emerald-400">Done</span> 结束
          </div>
        </div>
      )}

      {node.kind === "switch" && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-pink-300">路由规则 (顺序匹配)</div>
          {(config.rules as WorkflowConditionRule[]).map((rule, index) => (
            <div key={index} className="card !p-2 space-y-1.5">
              <div className="grid grid-cols-3 gap-1.5">
                <input
                  className="input text-[11px]"
                  value={rule.field}
                  placeholder="字段"
                  onChange={(e) => {
                    const rules = (config.rules as WorkflowConditionRule[]).map((r, i) =>
                      i === index ? { ...r, field: e.target.value } : r,
                    );
                    onConfigChange({ rules });
                  }}
                />
                <select
                  className="input text-[11px]"
                  value={rule.operator}
                  onChange={(e) => {
                    const rules = (config.rules as WorkflowConditionRule[]).map((r, i) =>
                      i === index ? { ...r, operator: e.target.value as WorkflowConditionRule["operator"] } : r,
                    );
                    onConfigChange({ rules });
                  }}
                >
                  {CONDITION_OPERATORS.map((op) => (
                    <option key={op.id} value={op.id}>
                      {op.label}
                    </option>
                  ))}
                </select>
                <input
                  className="input text-[11px]"
                  value={rule.value}
                  placeholder="值"
                  onChange={(e) => {
                    const rules = (config.rules as WorkflowConditionRule[]).map((r, i) =>
                      i === index ? { ...r, value: e.target.value } : r,
                    );
                    onConfigChange({ rules });
                  }}
                />
              </div>
            </div>
          ))}
          <button
            className="btn-secondary w-full text-xs"
            onClick={() =>
              onConfigChange({
                rules: [...(config.rules as WorkflowConditionRule[]), { field: "", operator: "equals", value: "" }],
              })
            }
          >
            <Plus size={12} /> 添加分支
          </button>
        </div>
      )}

      {node.kind === "http" && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-blue-300">HTTP 请求</div>
          <div className="grid grid-cols-3 gap-1.5">
            <select
              className="input text-xs col-span-1"
              value={(config.method as string) ?? "GET"}
              onChange={(e) => onConfigChange({ method: e.target.value })}
            >
              {["GET", "POST", "PUT", "DELETE", "PATCH"].map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
            <input
              className="input text-xs col-span-2 font-mono"
              value={(config.url as string) ?? ""}
              placeholder="https://…"
              onChange={(e) => onConfigChange({ url: e.target.value })}
            />
          </div>
        </div>
      )}

      {node.kind === "code" && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-ok">代码节点</div>
          <select
            className="input text-xs"
            value={(config.language as string) ?? "javascript"}
            onChange={(e) => onConfigChange({ language: e.target.value })}
          >
            <option value="javascript">JavaScript</option>
            <option value="python">Python</option>
          </select>
          <textarea
            className="input font-mono text-[11px] min-h-[120px]"
            value={(config.code as string) ?? ""}
            onChange={(e) => onConfigChange({ code: e.target.value })}
          />
        </div>
      )}

      {node.kind === "wait" && (
        <div className="space-y-2">
          <div className="text-xs font-semibold text-slate-300">等待时间</div>
          <input
            type="number"
            min={0}
            className="input text-xs"
            value={(config.seconds as number) ?? 5}
            onChange={(e) => onConfigChange({ seconds: Math.max(0, Number(e.target.value)) })}
          />
          <div className="text-[10px] text-ink-mute">单位: 秒</div>
        </div>
      )}

      {node.kind === "ipa-skill" && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-accent">IPA 工序节点</div>
          <div className="text-[11px] text-ink-dim font-mono">{node.skillId}</div>
          <div className="text-[10px] text-ink-mute">mode: {(config.mode as string) ?? "-"}</div>
          {config.hasHumanGate ? (
            <div className="text-[10px] text-warn border border-warn/40 rounded p-2">
              👤 人工审批门: 签名与批准不可由 AI 代行
            </div>
          ) : null}
          <div className="space-y-1">
            <label className="text-[11px] text-ink-dim">project_uid</label>
            <input
              className="input text-xs font-mono"
              value={(config.projectUid as string) ?? ""}
              placeholder="项目 UID"
              onChange={(e) => onConfigChange({ projectUid: e.target.value })}
            />
            <label className="text-[11px] text-ink-dim">task_uid</label>
            <input
              className="input text-xs font-mono"
              value={(config.taskUid as string) ?? ""}
              placeholder="任务 UID"
              onChange={(e) => onConfigChange({ taskUid: e.target.value })}
            />
          </div>
          <div className="text-[10px] text-ink-mute">
            输入端口来自 workflow-contract required_inputs; 输出端口来自 required_outputs。
          </div>
        </div>
      )}

      {node.kind === "trigger" && (
        <div className="text-[11px] text-ink-dim">
          手动触发节点: 工作流从这里开始, 输出 <span className="font-mono text-lime-300">Start</span> 信号。
        </div>
      )}

      {node.kind === "merge" && (
        <div className="text-[11px] text-ink-dim">
          合并两个输入为一个输出 (<span className="font-mono">append</span> 模式)。
        </div>
      )}
    </div>
  );
}
