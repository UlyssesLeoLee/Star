"use client";

// =====================================================================
// /agent-view — Agent 视图 (无限画布, 当前工作 agent 筛选)
// =====================================================================
// Per 2026-09-05 11:25 JST 用户发令 + 拍板:
//   1. 形式 = 无限画布 (Miro 风格, 自由散开)
//   2. 筛选 = 当前工作 agent (auto 选最近 active, 用户可手动覆盖)
//   3. 数据 = 跟 kanban 等界面共享 store (workItems / worktrees / agentSessions)
//   4. 路由 = /agent-view
//   5. 界面名 = "Agent"
//
// 数据流:
//   store.agentSessions
//     → resolveCurrentAgent (URL ?agent= > auto pick)
//     → pickAgentWorktree (1:1 worktree)
//     → pickAgentWorkItems (worktree_id 关联)
//     → layoutAgentCanvas (派生 nodes + connectors + viewport)
//     → AgentCanvasView 渲染
//     → AgentFilter 顶部覆盖, onChange 更新 URL (?agent=ag-XXX)
//
// 已知缺口 (per 缺标比错标, AGENTS.md §1.2 #3):
//   - 节点只读, 不能拖动 (派生视图; 拖动会跟 store 同步冲突)
//   - 不存到 store (避免污染 canvasElements 持久化); 只进 derivedAt 时间戳
//   - mock 数据, 真实后端 D.6+ 接入时改 store 即可, 组件不动
// =====================================================================

import { useMemo, useCallback, useEffect } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import { useStore } from "@/lib/store";
import {
  resolveCurrentAgent,
  pickAgentWorktree,
  pickAgentWorkItems,
} from "@/lib/agent-view/selectors";
import { layoutAgentCanvas, fitToContentViewport } from "@/lib/agent-view/layout";
import type { AgentCanvas } from "@/lib/agent-view/types";
import { AgentCanvasView } from "@/components/agent-view/AgentCanvasView";
import { AgentFilter } from "@/components/agent-view/AgentFilter";
import { PageHeader } from "@/components/PageHeader";
import { Bot, AlertTriangle, Maximize2 } from "lucide-react";

export default function AgentViewPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // store (顶层订阅, 避免 hooks 违规)
  const agents = useStore((s) => s.agentSessions);
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);

  // URL 参数: ?agent=ag-XXX
  const urlAgentId = searchParams.get("agent");

  // 解析 current agent
  const resolution = useMemo(
    () => resolveCurrentAgent(agents, urlAgentId),
    [agents, urlAgentId],
  );

  // 取 worktree + work-items
  const worktree = useMemo(
    () => (resolution ? pickAgentWorktree(worktrees, resolution.agent) : null),
    [worktrees, resolution],
  );
  const relatedWorkItems = useMemo(
    () => (resolution ? pickAgentWorkItems(workItems, resolution.agent, worktree) : []),
    [workItems, resolution, worktree],
  );

  // 派生 canvas (layout + fit-to-content viewport)
  const canvas: AgentCanvas | null = useMemo(() => {
    if (!resolution) return null;
    const layout = layoutAgentCanvas({
      agent: resolution.agent,
      worktree,
      workItems: relatedWorkItems,
    });
    return {
      agentId: resolution.agent.id,
      nodes: layout.nodes,
      connectors: layout.connectors,
      viewport: fitToContentViewport(layout.bbox, 1200, 800, 60),
      derivedAt: new Date().toISOString(),
    };
  }, [resolution, worktree, relatedWorkItems]);

  // dropdown 切换: 更新 URL (per 拍板 #2)
  const handleAgentChange = useCallback(
    (agentId: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set("agent", agentId);
      router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    },
    [router, pathname, searchParams],
  );

  // 清空 store 触发 layout 重派生 (per 守门 #9: 不偷偷 commit 子代理产出,
  //   这里 derivedAt 让 AgentCanvasView 在 useEffect 里重置 viewport)
  useEffect(() => {
    // no-op, 仅占位触发依赖追踪 (canvas?.derivedAt 在 deps 即可)
  }, [canvas?.derivedAt]);

  // ---- 空状态: 没有任何 agent ----
  if (agents.length === 0) {
    return (
      <div className="max-w-3xl">
        <PageHeader
          title="Agent"
          subtitle="无限画布 + 当前工作 agent 筛选 · 数据对应 kanban / worktree 视图"
          icon={<Bot className="text-accent" size={20} />}
          track="F"
        />
        <div className="card text-center py-12" data-testid="agent-view-empty">
          <AlertTriangle size={32} className="text-warn mx-auto mb-3" />
          <div className="text-base font-semibold mb-1">No agent sessions</div>
          <div className="text-xs text-ink-dim">
            请先在 <a href="/agents" className="text-info underline">Agents</a> 启动一个 agent session, 再回到此视图.
          </div>
        </div>
      </div>
    );
  }

  // ---- 空状态: 找不到 agent 但 store 有 ----
  if (!resolution) {
    return (
      <div className="max-w-3xl">
        <PageHeader
          title="Agent"
          subtitle="无限画布 + 当前工作 agent 筛选 · 数据对应 kanban / worktree 视图"
          icon={<Bot className="text-accent" size={20} />}
          track="F"
        />
        <div className="card text-center py-12" data-testid="agent-view-empty">
          <AlertTriangle size={32} className="text-warn mx-auto mb-3" />
          <div className="text-base font-semibold mb-1">No resolvable agent</div>
        </div>
      </div>
    );
  }

  // ---- 主渲染 ----
  const { agent } = resolution;
  return (
    <div className="-mx-6 -mt-5 h-[calc(100vh-3.5rem)] flex flex-col">
      {/* Header */}
      <div className="border-b border-line bg-bg-soft/40 px-6 py-3 flex items-center justify-between gap-4">
        <div className="flex items-center gap-3 min-w-0">
          <div className="text-sm font-semibold shrink-0" data-testid="agent-view-title">Agent</div>
          <div className="text-[10px] text-ink-mute font-mono truncate hidden md:block">
            {agent.id} · {agent.agent_kind} · {agent.current_step} · {relatedWorkItems.length} task{relatedWorkItems.length === 1 ? "" : "s"}
            {worktree && <> · worktree <span className="text-info">{worktree.branch}</span></>}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <AgentFilter
            agents={agents}
            selectedId={resolution.agentId}
            auto={resolution.auto}
            onChange={handleAgentChange}
          />
          <a
            href={`/board?assignee_id=&worktree_id=${agent.worktree_id}`}
            className="btn text-[10px] py-1 px-2 hidden md:inline-flex"
            data-testid="agent-view-jump-board"
            title="在 Kanban Board 视图查看关联 work-items"
          >
            <Maximize2 size={10} /> Kanban
          </a>
        </div>
      </div>

      {/* Canvas */}
      {canvas && (
        <div className="flex-1 relative">
          <AgentCanvasView
            canvas={canvas}
            agent={agent}
            worktree={worktree}
          />
        </div>
      )}

      {/* 说明 footer (per 缺标比错标 #1) */}
      <div className="border-t border-line bg-bg-soft/40 px-6 py-1.5 text-[10px] text-ink-mute font-mono flex items-center justify-between">
        <span>
          V/H 切换 select/pan · +/- 缩放 · 1 适配 · 双击节点跳详情 · 数据 = kanban / worktree 共享 store
        </span>
        <span>
          nodes {canvas?.nodes.length ?? 0} · connectors {canvas?.connectors.length ?? 0} · derived {canvas?.derivedAt.slice(11, 19) ?? "—"}
        </span>
      </div>
    </div>
  );
}
