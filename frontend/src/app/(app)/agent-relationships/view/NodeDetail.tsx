"use client";

// =====================================================================
// NodeDetail — 节点详情侧栏 (per brief v0.50 §2.1 A + DD §3.2.1)
// =====================================================================
// Per docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §1.1
// 显示节点的 name + archetype + trust_score + edge 数 (in/out) + 域信息.
// =====================================================================

import { useMemo } from "react";
import { useARGStore, selectEdgesByAgent } from "@/lib/arg/store";
import type { Agent } from "@/lib/arg/types";

export interface NodeDetailProps {
  /** Agent to display. */
  agent: Agent;
  /** Close handler. */
  onClose: () => void;
}

const RARITY_DOT: Record<"active" | "standby" | "archived", string> = {
  active: "bg-ok",
  standby: "bg-warn",
  archived: "bg-ink-mute",
};

export function NodeDetail({ agent, onClose }: NodeDetailProps) {
  const adjacent = useARGStore(selectEdgesByAgent(agent.id));
  const inCount = useMemo(
    () => adjacent.filter((e) => e.to_agent === agent.id && !e.archived).length,
    [adjacent, agent.id],
  );
  const outCount = useMemo(
    () => adjacent.filter((e) => e.from_agent === agent.id && !e.archived).length,
    [adjacent, agent.id],
  );
  const trustPct = Math.round(agent.trust_score * 100);

  return (
    <aside
      data-testid={`node-detail-${agent.id}`}
      className="card w-80 p-4 space-y-3"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold flex items-center gap-2">
          <span
            className={`inline-block w-2 h-2 rounded-full ${RARITY_DOT[agent.status]}`}
            data-testid="node-detail-status-dot"
          />
          {agent.name}
        </h3>
        <button
          type="button"
          onClick={onClose}
          className="text-xs text-ink-dim hover:text-ink"
          data-testid="node-detail-close"
        >
          ✕
        </button>
      </div>
      <dl className="space-y-1.5 text-xs font-mono">
        <div className="flex items-center justify-between">
          <dt className="text-ink-dim">id</dt>
          <dd className="text-ink truncate ml-2" title={agent.id}>
            {agent.id.slice(0, 8)}…
          </dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-ink-dim">archetype</dt>
          <dd>
            <span className="pill border-info/40 text-info bg-info/10 text-[10px]">
              {agent.archetype}
            </span>
          </dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-ink-dim">domain</dt>
          <dd className="text-ink">{agent.domain ?? "—"}</dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-ink-dim">status</dt>
          <dd className="text-ink">{agent.status}</dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-ink-dim">trust</dt>
          <dd className="text-ink">
            <span data-testid="node-detail-trust">{trustPct}%</span>
            <span className="ml-2 text-ink-mute">v{agent.version}</span>
          </dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-ink-dim">edges</dt>
          <dd className="text-ink" data-testid="node-detail-edge-count">
            in {inCount} · out {outCount}
          </dd>
        </div>
      </dl>
      <div>
        <div className="text-[10px] uppercase text-ink-dim mb-1">
          trust bar
        </div>
        <div className="h-1.5 w-full bg-bg rounded overflow-hidden border border-line">
          <div
            className="h-full bg-accent"
            style={{ width: `${trustPct}%` }}
            data-testid="node-detail-trust-bar"
          />
        </div>
      </div>
    </aside>
  );
}
