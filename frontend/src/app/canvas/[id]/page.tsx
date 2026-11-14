/*
@cypher schema=1 source_sha256=2732d153c48a96bc69255ed3df1a953c36c4ed3c3c7ed781b56ac9646dd157e2
MERGE (self:File {path:"frontend/src/app/canvas/[id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /canvas/[id] — 画布实例页 (per 0·页面合并图: Branch 工作区·固定页签 画布)
// 4 子 tab: 白板 / 关系图(Worktree/任务/文件/上下文) / IPA(V字模型) / 代码(低代码流程)
// =====================================================================
"use client";

import { useState, Suspense } from "react";
import { useParams } from "next/navigation";
import { CanvasView } from "@/components/CanvasView";
import { useStore } from "@/lib/store";
import { useTranslation } from "@/lib/i18n";

const CANVAS_SUB_TABS = ["白板", "关系图", "IPA (V字模型)", "代码 (低代码流程)"] as const;

export default function CanvasPage() {
  const params = useParams();
  const canvasId = params.id as string;
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<typeof CANVAS_SUB_TABS[number]>("白板");

  // 真实 store 数据
  const canvases = useStore((s) => s.canvases);
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const canvas = canvases.find((c) => c.id === canvasId);

  if (!canvas) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center p-8">
          <h1 className="text-xl font-bold text-ink mb-2">画布不存在</h1>
          <p className="text-ink-dim">找不到 ID 为 {canvasId} 的画布</p>
        </div>
      </div>
    );
  }

  // 根据 tab 渲染不同视图
  const renderCanvasContent = () => {
    switch (activeTab) {
      case "白板":
        return (
          <Suspense fallback={null}>
            <CanvasView
              canvas={canvas}
              elements={[]}
              connectors={[]}
              readOnly={false}
              groupWorkItems={workItems}
              groupWorktrees={worktrees}
            />
          </Suspense>
        );
      case "关系图":
        return (
          <div className="h-full bg-bg-card rounded-xl border border-line p-4">
            <h2 className="text-sm font-bold text-ink mb-3">关系图视图 (待实装: Worktree/任务/文件/上下文 4 层)</h2>
            <p className="text-xs text-ink-dim">关联 /canvas/worktree-graph, /agent-relationships, /context, /collaboration</p>
          </div>
        );
      case "IPA (V字模型)":
        return (
          <div className="h-full bg-bg-card rounded-xl border border-line p-4">
            <h2 className="text-sm font-bold text-ink mb-3">IPA V字模型视图 (待实装)</h2>
            <p className="text-xs text-ink-dim">需求定义 → 系统设计 → 模块设计 → 编码 → 单测 → 集成测 → 系统测 → 验收</p>
          </div>
        );
      case "代码 (低代码流程)":
        return (
          <div className="h-full bg-bg-card rounded-xl border border-line p-4">
            <h2 className="text-sm font-bold text-ink mb-3">代码低代码流程视图 (待实装)</h2>
            <p className="text-xs text-ink-dim">Agent 与 Workflow 均为节点，可视化编排</p>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg">
      {/* 子 tab 栏 */}
      <div className="flex items-center gap-1 border-b border-line bg-bg-card px-3">
        {CANVAS_SUB_TABS.map((tab) => (
          <button
            key={tab}
            type="button"
            onClick={() => setActiveTab(tab)}
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

      {/* 画布内容区 */}
      <main className="flex-1 overflow-hidden">
        {renderCanvasContent()}
      </main>
    </div>
  );
}
