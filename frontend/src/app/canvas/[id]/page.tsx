/*
@cypher schema=1 source_sha256=f6db98c4313b4243ae0177af771bc8a1c063fc7bcd4427c344379a1b4ffa22e5
MERGE (self:File {path:"frontend/src/app/canvas/[id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasDetailPage",kind:"component"})
MERGE (canvasView:Symbol {id:"frontend/src/components/CanvasView.tsx::CanvasView",kind:"component"})
MERGE (relationshipView:Symbol {id:"frontend/src/app/(app)/agent-relationships/view/RelationshipView.tsx::RelationshipView",kind:"component"})
MERGE (agentViewPage:Symbol {id:"frontend/src/components/agent-view/AgentViewPage.tsx::AgentViewPage",kind:"component"})
MERGE (scopeSelector:Symbol {id:"frontend/src/components/canvas/CanvasScopeSelector.tsx::CanvasScopeSelector",kind:"component"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (createCanvasLocal:Symbol {id:"frontend/src/lib/canvas-entry.ts::createCanvasLocal",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(canvasView)
MERGE (self)-[:IMPORTS]->(relationshipView)
MERGE (self)-[:IMPORTS]->(agentViewPage)
MERGE (self)-[:IMPORTS]->(scopeSelector)
MERGE (self)-[:IMPORTS]->(createCanvasLocal)
MERGE (page)-[:CALLS]->(navStore)
MERGE (page)-[:CALLS]->(createCanvasLocal)
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
// 2026-11-15 (per TASK-ulys-canvas-ctx): Canvas 作为 Lane 应用入口,
//   通过 ?worktree=<id> 深链携带当前 Worktree 上下文, 顶部渲染上下文条.
// 2026-11-15 (per TASK-ulys-canvas-fine): 关系图 tab RelationshipView
//   tenantId 去固定值, 接 store.projects[0].tenant_id.
// 2026-11-16 (per Canvas 范围选择器需求, 10 条第 4 条): 顶部三级范围选择器
//   (Project → Lane → Worktree, 读写 navStore) + BI/Plugins/Workflow 入口
//   (点击读 navStore 范围传后端做权限复验, 接入点见下方注释).
// 2026-11-16 (per 10 条第 4 条): 画布按路由 id 取用 (不再 fallback
//   seed.canvases[0]); 找不到画布时渲染未接入状态 + 新建画布按钮.
// =====================================================================

import { Suspense, use, useCallback, useEffect, useState } from "react";
import { GitBranch, Network, Plus } from "lucide-react";
import { useStore } from "@/lib/store";
import { CanvasView } from "@/components/CanvasView";
import { RelationshipView } from "@/app/(app)/agent-relationships/view/RelationshipView";
import { AgentViewPage } from "@/components/agent-view/AgentViewPage";
import { WorkflowBuilder } from "@/components/agent-workflow/WorkflowBuilder";
import { GitCanvas } from "../components/GitCanvas";
import { CanvasScopeSelector, type CanvasScope } from "@/components/canvas/CanvasScopeSelector";
import { useNavStore } from "@/lib/nav/navStore";
import { createCanvasLocal } from "@/lib/canvas-entry";

const CANVAS_SUB_TABS = ["画布", "Agent 画布", "关系图", "流程", "Git 画布"] as const;

export default function CanvasDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = use(params);
  const [activeTab, setActiveTab] = useState<typeof CANVAS_SUB_TABS[number]>("画布");

  // ?tab= 深链 (per 2026-11-15 Agent 中心并入: /agent-view → ?tab=agent)
  useEffect(() => {
    if (typeof window === "undefined") return;
    const tab = new URLSearchParams(window.location.search).get("tab");
    const map: Record<string, typeof CANVAS_SUB_TABS[number]> = {
      board: "画布",
      agent: "Agent 画布",
      relation: "关系图",
      flow: "流程",
      git: "Git 画布",
    };
    if (tab && map[tab]) setActiveTab(map[tab]);
  }, []);
  const canvases = useStore((s) => s.canvases);
  const canvasElements = useStore((s) => s.canvasElements);
  const canvasConnectors = useStore((s) => s.canvasConnectors);
  const workItems = useStore((s) => s.workItems);
  const worktrees = useStore((s) => s.worktrees);
  const projects = useStore((s) => s.projects);
  // 画布按路由 id 取用 (per 2026-11-16 10 条第 4 条: 不再 fallback seed.canvases[0]);
  // 找不到 → 画布 tab 渲染未接入状态 + 新建画布按钮 (本地创建, 写 store)
  const canvas = canvases.find((c) => c.id === id);

  // 关系图 tenant 动态化 (per TASK-ulys-canvas-fine):
  //   RelationshipView.tenantId 语义 = RLS 13 類 tenant id (非 project id,
  //   per RelationshipViewProps: "Optional tenant id (RLS 13 類)");
  //   接 store.projects[0].tenant_id (Project.tenant_id 13 類必带 per REQ-SEC-001)。
  //   无 project 时为 undefined → RelationshipView 按未过滤加载 (同 AgentViewTab)。
  const relationTenantId = projects[0]?.tenant_id;

  // ===================================================================
  // Canvas 范围 (Project → Lane → Worktree, per 2026-11-16 范围选择器需求):
  //   - 顶部 CanvasScopeSelector 读写 navStore (selectedProjectId /
  //     selectedLaneId / selectedWorktreeId), onChange 触发画布重新加载
  //   - ?worktree=<id> 深链 (per TASK-ulys-canvas-ctx) → 同步进 navStore
  //     (client-only 解析, 不用 useSearchParams — dev 下与页面级 Suspense
  //     组合会产生双份 DOM, 同 kanban ?view= 模式), 数据源 useStore worktrees
  // ===================================================================
  const [canvasReloadKey, setCanvasReloadKey] = useState(0);
  const [scopeSyncError, setScopeSyncError] = useState<string | null>(null);
  const [scopeNotice, setScopeNotice] = useState<string | null>(null);
  useEffect(() => {
    if (typeof window === "undefined") return;
    const wt = new URLSearchParams(window.location.search).get("worktree");
    if (!wt) return;
    const match = useStore.getState().worktrees.find((w) => w.id === wt);
    if (!match) {
      setScopeSyncError(wt);
      return;
    }
    const nav = useNavStore.getState();
    nav.setSelectedProjectId(match.project_id);
    nav.setLaneContext(match.lane_id ?? "", match.id, match.branch);
  }, []);
  const handleScopeChange = useCallback((scope: CanvasScope) => {
    setScopeSyncError(null);
    // 画布数据按新范围重新加载 (本地预览: 数据源为 store;
    //   按 Worktree 隔离的授权画布投影接入后, 此处替换为按 scope 重拉)
    setCanvasReloadKey((k) => k + 1);
    setScopeNotice(
      scope.worktreeId
        ? `范围已切换: project=${scope.projectId} lane=${scope.laneId} worktree=${scope.worktreeId} (画布已重新加载)`
        : `范围已切换: project=${scope.projectId}${scope.laneId ? ` lane=${scope.laneId}` : ""} (尚未指定 Worktree)`,
    );
  }, []);

  // ===================================================================
  // BI / Plugins / Workflow 入口 (per 2026-11-16 范围选择器需求):
  //   点击时读取 navStore 当前选中范围 → 传给后端做权限复验。
  //   ── 后端接入点 (授权投影接入后替换 TODO) ─────────────────────
  //     POST /api/v1/canvas/{canvas.id}/entries/{entry}/open
  //       body: { scope: { project_id, lane_id, worktree_id } }
  //     后端按 RLS (tenant/project/lane/worktree) 校验后返回授权 URL
  //     或数据; 未授权返回 403, 前端渲染拒绝态 (不打开入口)。
  //   ── 本地预览行为 (无后端) ─────────────────────────────────────
  //     workflow: 直接切"流程"tab; bi/plugins: 仅展示已读取的范围提示。
  // ===================================================================
  const openScopedEntry = useCallback(
    (entry: "bi" | "plugins" | "workflow") => {
      const nav = useNavStore.getState();
      const scope = {
        project_id: nav.selectedProjectId,
        lane_id: nav.selectedLaneId,
        worktree_id: nav.selectedWorktreeId,
      };
      // TODO(backend): await fetch(`/api/v1/canvas/${canvas?.id}/entries/${entry}/open`,
      //   { method: "POST", headers: { "Content-Type": "application/json" },
      //     body: JSON.stringify({ scope }) }) → 按授权结果跳转/渲染拒绝态
      setScopeNotice(
        `${entry.toUpperCase()} 入口 · 已按范围 project=${scope.project_id || "-"} lane=${scope.lane_id || "-"} worktree=${scope.worktree_id || "-"} 请求后端权限复验 (本地预览未接入后端)`,
      );
      if (entry === "workflow") setActiveTab("流程");
    },
    [canvas?.id],
  );

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
              <div
                className="h-full flex flex-col items-center justify-center gap-3"
                data-testid="canvas-unbound"
              >
                <p className="text-sm text-ink-mute" data-testid="canvas-unbound-hint">
                  画布未接入 — 找不到 {id} 对应的画布 (不再回退 seed 默认画布)
                </p>
                <button
                  type="button"
                  data-testid="canvas-unbound-create"
                  onClick={() =>
                    createCanvasLocal({ id, title: `画布 ${id}`, ref_kind: "free" })
                  }
                  className="flex items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-xs font-semibold text-accent transition-colors hover:bg-accent/20"
                >
                  <Plus size={14} /> 新建画布
                </button>
              </div>
            )}
          </div>
        );
      case "Agent 画布":
        return (
          <div className="h-full overflow-hidden" data-testid="canvas-tab-agent-canvas">
            <AgentViewPage variant="canvas-only" />
          </div>
        );
      case "关系图":
        return (
          <div className="h-full overflow-y-auto p-4" data-testid="canvas-tab-relation">
            <div className="mb-3 flex items-center justify-end">
              <a
                href="/canvas/worktree-graph"
                data-testid="canvas-relation-open-wtg"
                className="flex items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/10 px-2.5 py-1.5 text-[11px] font-semibold text-accent hover:bg-accent/20 transition-colors"
              >
                <Network size={12} /> Worktree 图 · 5 视图
              </a>
            </div>
            <RelationshipView tenantId={relationTenantId} />
          </div>
        );
      case "流程":
        return (
          <div className="h-full overflow-y-auto" data-testid="canvas-tab-flow">
            <WorkflowBuilder />
          </div>
        );
      case "Git 画布":
        return (
          <div className="h-full overflow-hidden" data-testid="canvas-tab-git">
            <GitCanvas />
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg" data-testid="canvas-detail-page">
      {/* Canvas 范围条 (per 2026-11-16: 三级范围选择器 + BI/Plugins/Workflow 权限复验入口) */}
      <div
        className="flex flex-wrap items-center gap-2 border-b border-line bg-bg-card px-3 py-1.5"
        data-testid="canvas-worktree-context"
      >
        <GitBranch size={13} className="shrink-0 text-accent" />
        <CanvasScopeSelector onScopeChange={handleScopeChange} />
        <span
          data-testid="canvas-worktree-data-scope"
          title="画布元素按 canvas_id 加载; 按 Worktree 隔离的画布投影接入后, 此处切换为授权数据"
          className="rounded border border-warn/40 bg-warn/10 px-1.5 py-0.5 font-mono text-[9px] text-warn"
        >
          数据未按 Worktree 隔离 (画布投影待接入)
        </span>
        {scopeSyncError && (
          <span className="text-xs text-warn" data-testid="canvas-worktree-context-missing">
            Worktree 上下文未找到 / 未授权读取: {scopeSyncError}
          </span>
        )}
        <span className="ml-auto flex items-center gap-1">
          <button
            type="button"
            data-testid="canvas-entry-bi"
            onClick={() => openScopedEntry("bi")}
            className="rounded border border-line bg-bg-soft px-2 py-0.5 text-[11px] text-ink transition-colors hover:border-accent hover:text-accent"
          >
            BI
          </button>
          <button
            type="button"
            data-testid="canvas-entry-plugins"
            onClick={() => openScopedEntry("plugins")}
            className="rounded border border-line bg-bg-soft px-2 py-0.5 text-[11px] text-ink transition-colors hover:border-accent hover:text-accent"
          >
            Plugins
          </button>
          <button
            type="button"
            data-testid="canvas-entry-workflow"
            onClick={() => openScopedEntry("workflow")}
            className="rounded border border-line bg-bg-soft px-2 py-0.5 text-[11px] text-ink transition-colors hover:border-accent hover:text-accent"
          >
            Workflow
          </button>
        </span>
      </div>
      {scopeNotice && (
        <div
          data-testid="canvas-entry-scope-notice"
          className="border-b border-line bg-bg-soft px-3 py-1 font-mono text-[10px] text-ink-mute"
        >
          {scopeNotice}
        </div>
      )}

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

      <main className="flex-1 overflow-hidden" key={canvasReloadKey}>
        <Suspense fallback={null}>{renderCanvasContent()}</Suspense>
      </main>
    </div>
  );
}
