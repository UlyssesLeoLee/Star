"use client";

import type { StateMachine } from "@/types/ids";
import { useMemo, useState } from "react";

/**
 * 状态机可视化 — 极简 SVG 渲染
 * - 节点 = 状态; 边 = 迁移
 * - 颜色编码:initial 蓝 / final 绿 / intermediate 暗灰
 * - 选中节点高亮 + 显示 in/out 边
 */
export function StateMachineDiagram({ sm, highlightState }: { sm: StateMachine; highlightState?: string }) {
  const [hover, setHover] = useState<string | null>(null);
  const active = hover ?? highlightState;

  // 计算节点位置: 简单网格布局
  const layout = useMemo(() => {
    const cols = 5;
    const cellW = 150;
    const cellH = 80;
    return sm.states.map((s, i) => {
      const c = i % cols;
      const r = Math.floor(i / cols);
      return { id: s, x: 30 + c * cellW, y: 30 + r * cellH };
    });
  }, [sm]);

  const posOf = (id: string) => layout.find((p) => p.id === id);

  // 边
  const edges = sm.transitions.map((t) => ({
    fromId: t.from,
    toId: t.to,
    from: posOf(t.from),
    to: posOf(t.to),
  }));

  const stateKind = (s: string) =>
    s === sm.initial ? "initial" : !sm.transitions.some((t) => t.from === s) ? "final" : "intermediate";

  return (
    <div className="card overflow-auto">
      <div className="mb-3 flex items-center gap-2">
        <h3 className="text-sm font-semibold">{sm.name}</h3>
        <div className="flex gap-1.5">
          {sm.invariant_ids.map((id) => (
            <span key={id} className="pill border-line text-ink-dim font-mono text-[10px]">{id}</span>
          ))}
        </div>
      </div>
      <svg
        viewBox="0 0 820 320"
        className="w-full max-w-3xl"
        style={{ minWidth: 800 }}
      >
        <defs>
          <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#6e7681" />
          </marker>
          <marker id="arrow-active" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#2f81f7" />
          </marker>
        </defs>

        {/* edges */}
        {edges.map((e, i) => {
          if (!e.from || !e.to) return null;
          const isActive = active && (active === e.from.id || active === e.to.id);
          const dx = e.to.x - e.from.x;
          const dy = e.to.y - e.from.y;
          const cx1 = e.from.x + dx * 0.25;
          const cy1 = e.from.y + dy * 0.1;
          const cx2 = e.to.x - dx * 0.25;
          const cy2 = e.to.y - dy * 0.1;
          const path = `M ${e.from.x + 60} ${e.from.y + 22} C ${cx1 + 60} ${cy1 + 22}, ${cx2 + 60} ${cy2 + 22}, ${e.to.x + 60} ${e.to.y + 22}`;
          return (
            <g key={i}>
              <path
                d={path}
                fill="none"
                stroke={isActive ? "#2f81f7" : "#30363d"}
                strokeWidth={isActive ? 1.5 : 1}
                markerEnd={isActive ? "url(#arrow-active)" : "url(#arrow)"}
              />
            </g>
          );
        })}

        {/* nodes */}
        {layout.map((p) => {
          const isActive = active === p.id;
          const kind = stateKind(p.id);
          const fill =
            kind === "initial" ? "#1f6feb" :
            kind === "final"   ? "#3fb950" : "#161b22";
          const stroke =
            isActive ? "#2f81f7" :
            kind === "initial" ? "#1f6feb" :
            kind === "final" ? "#3fb950" : "#30363d";
          return (
            <g
              key={p.id}
              onMouseEnter={() => setHover(p.id)}
              onMouseLeave={() => setHover(null)}
              style={{ cursor: "pointer" }}
            >
              <rect
                x={p.x}
                y={p.y}
                width={120}
                height={44}
                rx={6}
                fill={fill}
                stroke={stroke}
                strokeWidth={isActive ? 2 : 1}
                opacity={kind === "intermediate" ? 1 : 0.92}
              />
              <text
                x={p.x + 60}
                y={p.y + 27}
                textAnchor="middle"
                fontSize={11}
                fontFamily="ui-monospace, monospace"
                fill={kind === "intermediate" ? "#e6edf3" : "#0b0d10"}
                fontWeight={kind === "initial" || kind === "final" ? 700 : 500}
              >
                {p.id.replace(/_/g, " ")}
              </text>
            </g>
          );
        })}
      </svg>

      {/* legend */}
      <div className="mt-3 flex flex-wrap gap-3 text-[10px] text-ink-mute">
        <span className="flex items-center gap-1.5">
          <span className="size-3 rounded-sm bg-accent" /> initial
        </span>
        <span className="flex items-center gap-1.5">
          <span className="size-3 rounded-sm bg-bg-card border border-line" /> intermediate
        </span>
        <span className="flex items-center gap-1.5">
          <span className="size-3 rounded-sm bg-ok" /> final
        </span>
        {active && (
          <span className="ml-auto text-ink-dim">
            in/out edges: {sm.transitions.filter((t) => t.from === active || t.to === active).length}
          </span>
        )}
      </div>
    </div>
  );
}
