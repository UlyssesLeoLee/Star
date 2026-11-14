/*
@cypher schema=1 source_sha256=f836389e64a448b629855fdf081fd626ae115e8ad07d6084255d0f897ec7612f
MERGE (self:File {path:"frontend/src/app/(app)/agent-relationships/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/agent-relationships/page.tsx::AgentRelationshipsPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /agent-relationships — Agent 拓扑 (per 0·页面合并图: 画布子页关系图 -> Agent 拓扑)
// Agent 关联 worktree / work-items 拓扑图
// =====================================================================
"use client";

import { Bot, Network, Link2, GitBranch } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

export default function AgentRelationshipsPage() {
  const { t } = useTranslation();
  return (
    <div className="flex h-full flex-col bg-bg">
      <header className="border-b border-line bg-bg-card px-4 py-3">
        <h1 className="text-base font-bold text-ink flex items-center gap-2">
          <Bot size={16} className="text-accent" /> Agent 拓扑
        </h1>
        <p className="text-xs text-ink-dim mt-0.5">当前工作 Agent 关联 worktree / work-items 拓扑图</p>
      </header>
      <main className="flex-1 overflow-auto bg-bg p-4">
        <div className="h-full flex items-center justify-center">
          <div className="text-center text-ink-dim">
            <Network size={48} className="mx-auto mb-3 text-ink-mute" />
            <p className="text-sm">Agent 拓扑图 (待实装: Agent 关联 worktree / work-items)</p>
            <p className="text-xs text-ink-mute mt-1">关联 /agent-view (Agent 工作区画布、关系和配置)</p>
          </div>
        </div>
      </main>
    </div>
  );
}
