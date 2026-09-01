"use client";

// =====================================================================
// ArchGraphModal — Kanban 卡架构查看器 (per ADR-0041-arch-agent-graph-viewer)
// =====================================================================
// 触发:
//   - KanbanCard 第 4 行旁 🕸 Arch icon 按钮 (e.stopPropagation 防冒泡)
//
// 职责:
//   1. modal 居中弹起, 80vw × 80vh
//   2. POST /api/graph/ensure-fresh (幂等+排他, mock 直接返 fresh)
//   3. 失败 → POST /api/graph/cypher (1-hop 查询)
//   4. cytoscape 渲染 + 1-hop 高亮 (current work_item 节点 + 边 cyan 3px,
//      2-hop 代码侧 20% opacity 弱化)
//   5. footer 节点/边统计 + 关闭 (Esc / 背景 / X)
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖, 用 cytoscape + cytoscape-cose-bilkent (package.json 已加)
//   - SSR-safe: portal 只在 client 挂 (typeof window check)
//   - 走 React Query (per @tanstack/react-query 既有, no new dep)
//   - 走 13 类 tenant_id 必带 (per REQ-SEC-001)
//
// 已知缺口 (per 缺标比错标, 8/26 JST 守门 #1):
//   1. 点节点无跳转 (Phase 2+ 接 IDE / 跳 worktree / symbol)
//   2. 节点/边编辑无 (Phase 2+ read-only modal)
//   3. export PNG / SVG / JSON 无 (Phase 2+)
//   4. 多人同时点 = 1 跳并发 (Phase 2 接 coalesce)
// =====================================================================

import { useEffect, useRef, useState, useCallback } from "react";
import { createPortal } from "react-dom";
import { useQuery, useMutation } from "@tanstack/react-query";
import { clsx } from "clsx";
import { X, RefreshCw, AlertTriangle, GitBranch, Loader2 } from "lucide-react";
import type { WorkItem, Identity, Uuid, Iso8601 } from "@/types/ids";
import type {
  GraphPayload, GraphNode, GraphEdge, GraphNodeKind,
  EnsureFreshRequest, EnsureFreshResponse, EnsureFreshPendingResponse,
  GraphCypherRequest, GraphCypherResponse,
} from "@/types/graph";
import { NODE_STYLE, CURRENT_WI_STYLE, EDGE_STYLE } from "@/types/graph";
import { useStore } from "@/lib/store";

// ---- types ----
export interface ArchGraphModalProps {
  open: boolean;
  onClose: () => void;
  /** 当前 work item (per ids.ts) */
  workItem: WorkItem;
  /** tenant_id (per 13 类必带, REQ-SEC-001) */
  tenantId: Uuid;
  /** 解析后的 assignee (供 header 显示) */
  assignee?: Identity | undefined;
  /** 数据源, 默认 "local" (per ActorContext.local_runtime_id 判定) */
  source?: "local" | "git";
}

// ---- size (per ADR-0041 §2.3.2) ----
const MODAL_W_RATIO = 0.8;
const MODAL_H_RATIO = 0.8;
const MIN_W = 800;
const MIN_H = 600;

// =====================================================================
// Cytoscape 动态 import (避免 SSR 拉 cytoscape, per Next 14 client only)
// =====================================================================
type CytoscapeInstance = {
  destroy(): void;
  resize(): void;
  fit(eles?: unknown, padding?: number): void;
  nodes(selector?: string): { length: number };
  png(options?: { scale?: number; bg?: string; full?: boolean }): string;
};

// 3.x cytoscape 的 Stylesheet 类型已重命名为 StylesheetCSS,
// 但 cose-bilkent 4.x 还在用旧的, 这里用 any cast 避免碰撞
type AnyStylesheet = cytoscape.StylesheetCSS | Record<string, unknown>;

let cytoscapeLib: any = null;     // 动态 import, 类型用 any 简化
let coseBilkentExt: ((cy: any) => void) | null = null;

async function loadCytoscape() {
  if (cytoscapeLib && coseBilkentExt) return;
  const [cy, coseBilkent] = await Promise.all([
    import("cytoscape"),
    import("cytoscape-cose-bilkent"),
  ]);
  cytoscapeLib = cy.default;
  coseBilkentExt = coseBilkent.default as unknown as (cy: any) => void;
  // 注册 cose-bilkent 扩展 (per cytoscape-cose-bilkent README)
  coseBilkentExt(cytoscapeLib);
}

// =====================================================================
// Cytoscape 样式生成 (per ADR-0041 §2.3.3)
// =====================================================================
function buildStylesheet(): AnyStylesheet[] {
  const styles: any[] = [
    {
      selector: "node",
      style: {
        "background-color": "data(color)",
        "background-opacity": 1,
        "label": "data(label)",
        "color": "#e2e8f0",
        "font-size": 10,
        "font-family": "ui-monospace, SFMono-Regular, monospace",
        "text-valign": "bottom",
        "text-halign": "center",
        "text-margin-y": 4,
        "text-background-color": "#0a0d12",
        "text-background-opacity": 0.85,
        "text-background-padding": "2",
        "text-border-color": "#1f2937",
        "text-border-width": 1,
        "text-border-opacity": 0.6,
        "text-wrap": "wrap",
        "text-max-width": 100,
        "border-color": "#1f2937",
        "border-width": 1,
        "border-opacity": 0.8,
        "shape": "data(shape)",
        "width": "data(size)",
        "height": "data(size)",
        "opacity": 1,
      } as unknown as cytoscape.Css.Node,
    },
    // 当前 work_item 节点 (主色, 加粗, 全亮)
    {
      selector: 'node[is_current = "true"]',
      style: {
        "background-color": CURRENT_WI_STYLE.color,
        "border-color": CURRENT_WI_STYLE.color,
        "border-width": 3,
        "border-opacity": 1,
        "color": "#0a0d12",
        "font-weight": "bold",
        "font-size": 12,
        "width": CURRENT_WI_STYLE.size,
        "height": CURRENT_WI_STYLE.size,
        "text-background-color": "#00f0ff",
        "text-background-opacity": 0.15,
      } as unknown as cytoscape.Css.Node,
    },
    // 2-hop 弱化
    {
      selector: 'node[hop_level = "2"]',
      style: {
        "opacity": 0.2,
      } as unknown as cytoscape.Css.Node,
    },
    // 1-hop 边 (cyan)
    {
      selector: 'edge[hop_level = "1"]',
      style: {
        "line-color": EDGE_STYLE.hop1.color,
        "target-arrow-color": EDGE_STYLE.hop1.color,
        "width": EDGE_STYLE.hop1.width,
        "line-style": EDGE_STYLE.hop1.dash,
        "curve-style": "bezier",
        "arrow-scale": 0.8,
        "label": "data(kind)",
        "color": "#94a3b8",
        "font-size": 8,
        "text-rotation": "autorotate",
        "text-background-color": "#0a0d12",
        "text-background-opacity": 0.7,
        "text-background-padding": "1",
        "opacity": 0.7,
      } as unknown as cytoscape.Css.Edge,
    },
    // 2-hop 边 (灰, 虚线, 弱化)
    {
      selector: 'edge[hop_level = "2"]',
      style: {
        "line-color": EDGE_STYLE.hop2.color,
        "line-style": EDGE_STYLE.hop2.dash,
        "width": EDGE_STYLE.hop2.width,
        "opacity": 0.3,
        "label": "",
      } as unknown as cytoscape.Css.Edge,
    },
  ];
  return styles;
}

/** 把 GraphNode/Edge 投影成 cytoscape elements */
function buildElements(graph: GraphPayload) {
  const nodes = graph.nodes.map((n) => {
    const style = NODE_STYLE[n.kind];
    return {
      group: "nodes" as const,
      data: {
        id: n.id,
        label: n.label,
        kind: n.kind,
        is_current: String(n.is_current),
        hop_level: String(n.hop_level),
        color: style.color,
        shape: style.shape,
        size: style.size,
        properties: n.properties,
      },
    };
  });
  const edges = graph.edges.map((e) => ({
    group: "edges" as const,
    data: {
      id: e.id,
      source: e.source,
      target: e.target,
      kind: e.kind,
      hop_level: String(e.hop_level),
    },
  }));
  return [...nodes, ...edges];
}

// =====================================================================
// Main component
// =====================================================================
export function ArchGraphModal({
  open, onClose, workItem, tenantId, assignee, source = "local",
}: ArchGraphModalProps) {
  // 1. state
  const containerRef = useRef<HTMLDivElement | null>(null);
  const cyRef = useRef<CytoscapeInstance | null>(null);
  const [phase, setPhase] = useState<"idle" | "ensuring" | "loading" | "rendering" | "ready" | "error">("idle");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  // 记录已渲染的 fingerprint, 防止重复 mount
  const lastFingerprintRef = useRef<string | null>(null);

  // 2. ensure-fresh mutation
  const ensureFresh = useMutation<
    EnsureFreshResponse | EnsureFreshPendingResponse,
    Error,
    EnsureFreshRequest
  >({
    mutationFn: async (req) => {
      const res = await fetch("/api/graph/ensure-fresh", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(req),
      });
      if (!res.ok && res.status !== 202) {
        throw new Error(`ensure-fresh failed: ${res.status}`);
      }
      return res.json();
    },
  });

  // 3. cypher query (当 ensure-fresh 返 fresh 直接用; 202 时 fallback to cypher)
  const cypher = useQuery<GraphCypherResponse, Error>({
    queryKey: ["graph", "cypher", workItem.id, tenantId],
    queryFn: async () => {
      const req: GraphCypherRequest = {
        work_item_id: workItem.id,
        tenant_id: tenantId,
        max_hop: 2,
      };
      const res = await fetch("/api/graph/cypher", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(req),
      });
      if (!res.ok) throw new Error(`cypher failed: ${res.status}`);
      return res.json();
    },
    enabled: open && phase === "loading",
    staleTime: 30_000,
  });

  // 4. ensure-fresh 完成后决定下步
  useEffect(() => {
    if (!open) return;
    if (ensureFresh.isPending) {
      setPhase("ensuring");
      return;
    }
    if (ensureFresh.isError) {
      setPhase("error");
      setErrorMsg(ensureFresh.error?.message ?? "ensure-fresh failed");
      return;
    }
    if (ensureFresh.data) {
      if (ensureFresh.data.status === "fresh") {
        // 已 fresh, 直接渲染
        setPhase("rendering");
      } else if (ensureFresh.data.status === "running") {
        // agent 正在跑, 切到 loading (cypher query 触发)
        setPhase("loading");
      }
    }
  }, [open, ensureFresh.isPending, ensureFresh.isError, ensureFresh.data]);

  // 5. cypher query 完成后切到 rendering
  useEffect(() => {
    if (phase !== "loading") return;
    if (cypher.isError) {
      setPhase("error");
      setErrorMsg(cypher.error?.message ?? "cypher failed");
      return;
    }
    if (cypher.data) {
      setPhase("rendering");
    }
  }, [phase, cypher.isError, cypher.data]);

  // 6. 触发 ensure-fresh (open 时)
  useEffect(() => {
    if (open) {
      setPhase("idle");
      setErrorMsg(null);
      lastFingerprintRef.current = null;
      ensureFresh.mutate({
        work_item_id: workItem.id,
        tenant_id: tenantId,
        source,
      });
    } else {
      // 关闭时 destroy cytoscape
      if (cyRef.current) {
        cyRef.current.destroy();
        cyRef.current = null;
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, workItem.id, tenantId, source]);

  // 7. render cytoscape
  useEffect(() => {
    if (phase !== "rendering") return;
    const graph = ensureFresh.data?.status === "fresh"
      ? ensureFresh.data.graph
      : cypher.data;
    if (!graph || !containerRef.current) return;
    // 防止同 fingerprint 重复 mount
    if (lastFingerprintRef.current === graph.fingerprint) return;
    lastFingerprintRef.current = graph.fingerprint;

    let cy: CytoscapeInstance | null = null;
    let cancelled = false;

    (async () => {
      await loadCytoscape();
      if (cancelled || !containerRef.current || !cytoscapeLib) return;
      // destroy old
      if (cyRef.current) {
        cyRef.current.destroy();
        cyRef.current = null;
      }
      cy = cytoscapeLib({
        container: containerRef.current,
        elements: buildElements(graph),
        style: buildStylesheet() as unknown as any[],
        layout: {
          name: "cose",
          animate: true,
          padding: 30,
          idealEdgeLength: () => 100,
          nodeRepulsion: () => 8000,
          // cose-bilkent 扩展 (loadCytoscape 已注册)
          randomize: true,
        } as unknown as cytoscape.LayoutOptions,
        minZoom: 0.3,
        maxZoom: 3,
        wheelSensitivity: 0.2,
      });
      cyRef.current = cy as unknown as CytoscapeInstance;
      cy?.fit(undefined, 40);
      setPhase("ready");
    })();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [phase]);

  // 8. Esc 关闭
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  // 9. 阻止背景滚动
  useEffect(() => {
    if (!open) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => { document.body.style.overflow = prev; };
  }, [open]);

  // 10. 渲染
  if (!open || typeof window === "undefined") return null;

  const graph = ensureFresh.data?.status === "fresh"
    ? ensureFresh.data.graph
    : cypher.data;

  return createPortal(
    <div
      role="dialog"
      aria-modal="true"
      aria-label={`Architecture graph for ${workItem.key}`}
      data-testid="arch-graph-modal"
      onClick={(e) => {
        // 背景 click 关闭
        if (e.target === e.currentTarget) onClose();
      }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    >
      <div
        data-testid="arch-graph-modal-content"
        className="card flex flex-col shadow-2xl"
        style={{
          width: `min(${MODAL_W_RATIO * 100}vw, 1400px)`,
          height: `min(${MODAL_H_RATIO * 100}vh, 900px)`,
          minWidth: MIN_W, minHeight: MIN_H,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between gap-2 px-4 py-2.5 border-b border-line bg-bg-soft/80">
          <div className="flex items-center gap-2 min-w-0 flex-1">
            <GitBranch size={16} className="text-accent shrink-0" />
            <span className="font-mono text-xs text-info truncate" title={workItem.key}>
              {workItem.key}
            </span>
            <span className="text-sm font-medium text-ink truncate" title={workItem.title}>
              {workItem.title}
            </span>
            {assignee && (
              <span className="text-[10px] text-ink-mute font-mono truncate">
                · {assignee.display_name}
              </span>
            )}
          </div>
          <div className="flex items-center gap-1.5 shrink-0">
            <RefreshButton onClick={() => {
              setPhase("idle");
              ensureFresh.reset();
              cypher.refetch();
              ensureFresh.mutate({
                work_item_id: workItem.id,
                tenant_id: tenantId,
                source,
              });
            }} />
            <button
              type="button"
              data-testid="arch-graph-modal-close"
              onClick={onClose}
              aria-label="Close"
              className="text-ink-mute hover:text-ink transition-colors p-1"
            >
              <X size={16} />
            </button>
          </div>
        </div>

        {/* Body (cytoscape canvas / loading / error) */}
        <div className="flex-1 relative overflow-hidden">
          {/* loading overlay */}
          {(phase === "idle" || phase === "ensuring" || phase === "loading") && (
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-bg-card/80 z-10">
              <Loader2 size={32} className="text-accent animate-spin" />
              <div className="text-sm text-ink-mute font-mono">
                {phase === "ensuring" && "Agent is analyzing code context..."}
                {phase === "loading" && "Agent is running, loading 1-hop graph..."}
                {phase === "idle" && "Initializing..."}
              </div>
              <div className="text-[10px] text-ink-dim font-mono">
                work_item_id = {workItem.id} · source = {source}
              </div>
            </div>
          )}

          {/* error overlay */}
          {phase === "error" && (
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-bg-card/90 z-10 p-6">
              <AlertTriangle size={32} className="text-err" />
              <div className="text-sm text-err font-mono">Failed to load architecture graph</div>
              <div className="text-[10px] text-ink-mute font-mono max-w-md text-center">
                {errorMsg ?? "Unknown error"}
              </div>
              <button
                type="button"
                onClick={() => {
                  setPhase("idle");
                  setErrorMsg(null);
                  ensureFresh.mutate({
                    work_item_id: workItem.id,
                    tenant_id: tenantId,
                    source,
                  });
                }}
                className="mt-2 px-3 py-1.5 rounded-lg border border-line hover:border-accent bg-bg-soft text-xs font-mono text-accent"
              >
                Retry
              </button>
            </div>
          )}

          {/* cytoscape container */}
          <div
            ref={containerRef}
            data-testid="arch-graph-canvas"
            className={clsx(
              "absolute inset-0 bg-bg-card",
              (phase !== "ready" && phase !== "rendering") && "opacity-0",
            )}
          />
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between gap-2 px-4 py-2 border-t border-line bg-bg-soft/80 text-[10px] font-mono text-ink-mute">
          <div className="flex items-center gap-3">
            {graph ? (
              <>
                <span>
                  <span className="text-ink-dim">nodes:</span>{" "}
                  <span data-testid="arch-graph-node-count" className="text-ink">{graph.stats.node_count}</span>
                </span>
                <span>
                  <span className="text-ink-dim">edges:</span>{" "}
                  <span data-testid="arch-graph-edge-count" className="text-ink">{graph.stats.edge_count}</span>
                </span>
                <span>
                  <span className="text-ink-dim">fingerprint:</span>{" "}
                  <span className="text-ink truncate max-w-[200px] inline-block align-middle" title={graph.fingerprint}>
                    {graph.fingerprint}
                  </span>
                </span>
              </>
            ) : (
              <span>no graph</span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <span className="text-[9px]">
              <span className="inline-block w-2 h-2 rounded-full bg-[#00f0ff] mr-1" /> current
            </span>
            <span className="text-[9px]">
              <span className="inline-block w-2 h-2 rounded-full bg-[#7c8499] mr-1" /> 1-hop
            </span>
            <span className="text-[9px]">
              <span className="inline-block w-2 h-2 rounded-full bg-[#475569] opacity-30 mr-1" /> 2-hop code
            </span>
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
}

// =====================================================================
// RefreshButton — 子组件, 避免在主组件内联
// =====================================================================
function RefreshButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      data-testid="arch-graph-modal-refresh"
      onClick={onClick}
      aria-label="Refresh graph"
      className="text-ink-mute hover:text-accent transition-colors p-1"
      title="Refresh from agent"
    >
      <RefreshCw size={14} />
    </button>
  );
}

// =====================================================================
// useArchGraphTrigger — 业务层 hook, 决定何时打开 modal
// =====================================================================
// 用法 (per ProjectsClient.tsx):
//   const arch = useArchGraphTrigger();
//   <ArchGraphModal {...arch.modalProps} />
//   <KanbanCard onArchClick={arch.open} ... />
export function useArchGraphTrigger() {
  const [target, setTarget] = useState<{
    workItem: WorkItem;
    tenantId: Uuid;
    assignee: Identity | undefined;
  } | null>(null);

  const open = useCallback((workItem: WorkItem) => {
    // Phase 1 mock: 用 workItem.tenant_id; Phase 2 接 ActorContext (per 13 类)
    const tenantId = workItem.tenant_id || "tenant-physis-corp";
    const assignee = workItem.assignee_id
      ? useStore.getState().identities.find((i) => i.id === workItem.assignee_id)
      : undefined;
    setTarget({ workItem, tenantId, assignee });
  }, []);

  const close = useCallback(() => setTarget(null), []);

  return {
    open,
    close,
    modalProps: target ? {
      open: true as const,
      onClose: close,
      workItem: target.workItem,
      tenantId: target.tenantId,
      assignee: target.assignee,
    } : null,
  };
}
