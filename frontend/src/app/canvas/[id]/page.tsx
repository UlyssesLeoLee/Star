/*
@cypher schema=1 source_sha256=d5a60069c1b396b12babd40ae03cba5b8c870c1bec3e6c587b68e5c87362b391
MERGE (self:File {path:"frontend/src/app/canvas/[id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasDetailPage",kind:"component"})
MERGE (canvasView:Symbol {id:"frontend/src/components/CanvasView.tsx::CanvasView",kind:"component"})
MERGE (agentCanvas:Symbol {id:"frontend/src/components/agent-view/AgentCanvasView.tsx::AgentCanvasView",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(canvasView)
MERGE (self)-[:IMPORTS]->(agentCanvas)
@endcypher
*/
"use client";

// =====================================================================
// /canvas/[id] — 无限画布统一页面 (per 2026-11-14 用户发令:
//   "无限画布类全部归纳进 canvas, 不再需要点击后三选一")
//   3 标签 = 全部无限画布类:
//     1. 画布 (CanvasView 白板: 帧/端点/连接线/画笔)
//     2. Agent 画布 (AgentCanvasView 拓扑)
//     3. 关系图 (RelationshipView)
//   非无限画布类 (IPA/低代码流程) 移出为单独页面 (/analytics, /workflow)
// =====================================================================

import { Suspense, useState } from "react";
import { Network } from "lucide-react";
import { useStore } from "@/lib/store";
import { CanvasView } from "@/components/CanvasView";
import { RelationshipView } from "@/app/(app)/agent-relationships/view/RelationshipView";
import { AgentViewPage } from "@/app/(app)/agent-view/page";

const CANVAS_SUB_TABS = ["画布", "Agent 画布", "关系图"] as const;

export default function CanvasDetailPage() {
  const [activeTab, setActiveTab] = useState<typeof CANVAS_SUB_TABS[number]>("画布");
  const canvases = useStore((s) => s.canvases);
  const canvasElements = useStore((s) => s.canvasElements);
  const canvasConnectors = useStore((s) => s.canvasConnectors);
  const workItems = useStore((s) => s.workItems);
  const worktrees = useStore((s) => s.worktrees);
  const canvas = canvases[0];

  const renderCanvasContent = () => {
    switch (activeTab) {
      case "画布":
        // 无限画布核心 (白板 CanvasView, store 数据通道)
        return (
          <div className="h-full overflow-hidden" data-testid="canvas-tab-whiteboard">
            {canvas ? (
              <CanvasView
                canvas={canvas}
                elements={canvasElements}
                connectors={canvasConnectors}
                readOnly={false}
                groupWorkItems={workItems}
                groupWorktrees={worktrees}
                onCreateElement={async (input) => {
                  const el = {
                    id: `el-${globalThis.crypto.randomUUID()}`,
                    canvas_id: canvas.id,
                    kind: input.kind,
                    x: input.x,
                    y: input.y,
                    width: input.width,
                    height: input.height,
                    rotation: 0,
                    z_index: input.z_index,
                    content: input.content,
                    locked: false,
                    hidden: false,
                    created_by: "usr-001",
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString(),
                  };
                  useStore.getState().addCanvasElement(el);
                  return { id: el.id };
                }}
              />
            ) : (
              <div className="h-full flex items-center justify-center text-ink-mute text-sm">
                无可用画布 (seed canvas-001 未加载)
              </div>
            )}
          </div>
        );
      case "Agent 画布":
        return (
          <div className="h-full overflow-hidden" data-testid="canvas-tab-agent-canvas">
            <AgentViewPage />
          </div>
        );
      case "关系图":
        return (
          <div className="h-full overflow-y-auto p-4" data-testid="canvas-tab-relation">
            <div className="mb-3 flex items-center justify-between">
              <span className="text-xs text-ink-dim">租户关系图 (RelationshipView)</span>
              <a
                href="/canvas/worktree-graph"
                data-testid="canvas-relation-open-wtg"
                className="flex items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/10 px-2.5 py-1.5 text-[11px] font-semibold text-accent hover:bg-accent/20 transition-colors"
              >
                <Network size={12} /> Worktree 图 · 5 视图
              </a>
            </div>
            <RelationshipView tenantId="tn-001" />
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg" data-testid="canvas-detail-page">
      {/* 无限画布标签切换 (无三选一, 直接呈现) */}
      <div className="flex items-center gap-1 border-b border-line bg-bg-card px-3" data-testid="canvas-merged-tabs">
        {CANVAS_SUB_TABS.map((tab) => (
          <button
            key={tab}
            type="button"
            onClick={() => setActiveTab(tab)}
            data-testid={`canvas-merged-tab-${tab}`}
            className={`px-3 py-2 text-sm border-b-2 -mb-px transition-colors ${
              activeTab === tab
                ? "border-accent text-accent font-semibold"
                : "border-transparent text-ink-dim hover:text-ink"
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      <main className="flex-1 overflow-hidden">
        <Suspense fallback={null}>{renderCanvasContent()}</Suspense>
      </main>
    </div>
  );
}
