/*
@cypher schema=1 source_sha256=cfff0eae34e846e50f649709a3c43b071068c6c1bf4d9c9a8c18dea742a62c68
MERGE (self:File {path:"frontend/src/app/canvas/worktree-graph/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/worktree-graph/page.tsx::WorktreeGraphPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /canvas/worktree-graph — Worktree 图 (per 0·页面合并图: 画布子页 关系图核心)
// 5 视图: 树 / 依赖 / 风险 / Agent / 历史 + 6 级缩放
// =====================================================================
"use client";

import { useState } from "react";
import { Network, GitBranch, AlertTriangle, Bot, History } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";

const GRAPH_VIEWS = ["树", "依赖", "风险", "Agent", "历史"] as const;

export default function WorktreeGraphPage() {
  const { t } = useTranslation();
  const [activeView, setActiveView] = useState<typeof GRAPH_VIEWS[number]>("树");
  const [zoom, setZoom] = useState(1);

  return (
    <div className="flex h-full flex-col bg-bg">
      {/* 顶部视图切换 */}
      <div className="flex items-center gap-4 border-b border-line bg-bg-card px-4 py-2">
        <div className="flex items-center gap-1 border-b border-line">
          {GRAPH_VIEWS.map((v) => (
            <button
              key={v}
              type="button"
              onClick={() => setActiveView(v)}
              className={clsx(
                "px-3 py-1.5 text-sm border-b-2 -mb-px transition-colors",
                activeView === v
                  ? "border-accent text-accent font-semibold"
                  : "border-transparent text-ink-dim hover:text-ink"
              )}
            >
              {v}
            </button>
          ))}
        </div>
        <div className="flex-1" />
        <div className="flex items-center gap-2 text-xs text-ink-dim">
          <span>缩放: {Math.round(zoom * 100)}%</span>
          <button type="button" onClick={() => setZoom(Math.max(0.1, zoom - 0.2))} className="px-2 py-1 border border-line rounded hover:bg-bg-soft">−</button>
          <button type="button" onClick={() => setZoom(Math.min(3, zoom + 0.2))} className="px-2 py-1 border border-line rounded hover:bg-bg-soft">+</button>
          <button type="button" onClick={() => setZoom(1)} className="px-2 py-1 border border-line rounded hover:bg-bg-soft">100%</button>
        </div>
      </div>

      {/* 图区域 */}
      <main className="flex-1 overflow-auto bg-bg relative">
        <div
          className="absolute inset-0"
          style={{ transform: `scale(${zoom})`, transformOrigin: "center center" }}
        >
          <div className="h-full w-full flex items-center justify-center">
            <div className="text-center text-ink-dim">
              <Network size={48} className="mx-auto mb-3 text-ink-mute" />
              <p className="text-sm">Worktree {activeView} 视图 (待实装 6 级缩放 + 5 视图切换)</p>
              <p className="text-xs text-ink-mute mt-1">树 / 依赖 / 风险 / Agent / 历史</p>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
