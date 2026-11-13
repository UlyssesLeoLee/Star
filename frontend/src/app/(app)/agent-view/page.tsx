/*
@cypher schema=1 source_sha256=0bdcf540a108f92567f0d76678b968f401646f4ae8cd130d5e5c39e4a3344c9a
MERGE (self:File {path:"frontend/src/app/(app)/agent-view/page.tsx"})
MERGE (agentSelectors:File {path:"frontend/src/lib/agent-view/selectors.ts"})
MERGE (agentCanvasLayout:File {path:"frontend/src/lib/agent-view/layout.ts"})
MERGE (agentCanvasTypes:File {path:"frontend/src/lib/agent-view/types.ts"})
MERGE (agentAnnotations:File {path:"frontend/src/lib/agent-view/annotationApi.ts"})
MERGE (agentAnnotationHistory:File {path:"frontend/src/lib/agent-view/annotationHistory.ts"})
MERGE (agentCanvasComponent:File {path:"frontend/src/components/agent-view/AgentCanvasView.tsx"})
MERGE (agentFilter:File {path:"frontend/src/components/agent-view/AgentFilter.tsx"})
MERGE (agentSettingsTab:File {path:"frontend/src/components/agent/AgentSettingsTab.tsx"})
MERGE (pageHeader:File {path:"frontend/src/components/PageHeader.tsx"})
MERGE (store:File {path:"frontend/src/lib/store.ts"})
MERGE (agentRelationships:File {path:"frontend/src/app/(app)/agent-relationships/AgentViewTab.tsx"})
MERGE (i18n:File {path:"frontend/src/lib/i18n/index.ts"})
MERGE (agentTypes:File {path:"frontend/src/lib/agent/types.ts"})
MERGE (agentKindDisplayLabel:Function {name:"agentKindDisplayLabel", language:"typescript", visibility:"public"})
MERGE (agentViewPage:Function {name:"AgentViewPage", language:"typescript", visibility:"public"})
MERGE (self)-[:IMPORTS]->(agentTypes)
MERGE (self)-[:IMPORTS]->(agentSelectors)
MERGE (self)-[:IMPORTS]->(agentCanvasLayout)
MERGE (self)-[:IMPORTS]->(agentCanvasTypes)
MERGE (self)-[:IMPORTS]->(agentAnnotations)
MERGE (self)-[:IMPORTS]->(agentAnnotationHistory)
MERGE (self)-[:IMPORTS]->(agentCanvasComponent)
MERGE (self)-[:IMPORTS]->(agentFilter)
MERGE (self)-[:IMPORTS]->(agentSettingsTab)
MERGE (self)-[:IMPORTS]->(pageHeader)
MERGE (self)-[:IMPORTS]->(store)
MERGE (self)-[:IMPORTS]->(agentRelationships)
MERGE (self)-[:IMPORTS]->(i18n)
MERGE (agentViewPage)-[:CALLS]->(agentKindDisplayLabel)
@endcypher
*/
"use client";

import { agentKindDisplayLabel } from "@/lib/agent/types";

// =====================================================================
// /agent-view — Agent 工作区画布、关系和配置视图
// =====================================================================
// Agent、Worktree、Task 和关系视图复用当前共享 store 数据。
// =====================================================================

import { useMemo, useCallback, useEffect, useState, Suspense } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import { useStore } from "@/lib/store";
import {
  resolveCurrentAgent,
  pickAgentWorktree,
  pickAgentWorkItems,
} from "@/lib/agent-view/selectors";
import { layoutAgentCanvas, fitToContentViewport } from "@/lib/agent-view/layout";
import type {
  AgentCanvas,
  AgentCanvasAnnotation,
  AgentCanvasFreeConnector,
} from "@/lib/agent-view/types";
import {
  loadAgentAnnotations,
  saveAgentAnnotations,
} from "@/lib/agent-view/annotationApi";
import {
  initHistory, undo as undoStep, redo as redoStep,
  commit as commitStep, canUndo, canRedo,
  type AnnotationHistoryState,
} from "@/lib/agent-view/annotationHistory";
import { AgentCanvasView } from "@/components/agent-view/AgentCanvasView";
import { AgentFilter } from "@/components/agent-view/AgentFilter";
import { AgentSettingsTab } from "@/components/agent/AgentSettingsTab";
import { PageHeader } from "@/components/PageHeader";
import { Bot, AlertTriangle, Maximize2, LayoutDashboard, Settings, Network, Workflow } from "lucide-react";
import { useTranslation } from "@/lib/i18n";
import { AgentViewTab } from "@/app/(app)/agent-relationships/AgentViewTab";
import { WorkflowBuilder } from "@/components/agent-workflow/WorkflowBuilder";

type ViewMode = "canvas" | "settings" | "relationships" | "workflow";

function AgentViewContent() {
  const { t } = useTranslation();
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // store (顶层订阅, 避免 hooks 违规)
  const agents = useStore((s) => s.agentSessions);
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);

  // URL 参数: ?agent=ag-XXX
  const urlAgentId = searchParams.get("agent");
  const urlView = searchParams.get("view") as ViewMode | null;

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

  // View 模式通过 URL 保留，默认显示工作画布。
  const [viewMode, setViewMode] = useState<ViewMode>(
    urlView === "settings"
      ? "settings"
      : urlView === "relationships"
        ? "relationships"
        : urlView === "workflow"
          ? "workflow"
          : "canvas",
  );

  // 用户注释 / 自由连接 (per 2026-10-01 OOB 恢复无限画布画笔) - local-only state + localStorage 持久化 + undo/redo
  const [localAnns, setLocalAnns] = useState<AgentCanvasAnnotation[]>([]);
  const [localConns, setLocalConns] = useState<AgentCanvasFreeConnector[]>([]);
  const [editMode, setEditMode] = useState(false);
  /** undo/redo 历史栈 (per 任务 #5) */
  const [history, setHistory] = useState<AnnotationHistoryState>(() => initHistory([], []));

  // load on agentId 变化 (per 任务 #1 — localStorage 持久化) + 初始化 history (per 任务 #5)
  const currentAgentId = resolution?.agentId ?? null;
  useEffect(() => {
    if (typeof window === "undefined" || !currentAgentId) return;
    const persisted = loadAgentAnnotations(currentAgentId);
    setLocalAnns(persisted.annotations);
    setLocalConns(persisted.freeConnectors);
    setHistory(initHistory(persisted.annotations, persisted.freeConnectors));
  }, [currentAgentId]);

  // commit wrapper — 更新 localAnns/localConns 同时记录 history (per 任务 #5)
  const commitStateChange = useCallback(
    (nextAnns: AgentCanvasAnnotation[], nextConns: AgentCanvasFreeConnector[]) => {
      setLocalAnns(nextAnns);
      setLocalConns(nextConns);
      setHistory((h) => commitStep(h, { annotations: nextAnns, freeConnectors: nextConns }));
    },
    [],
  );

  // persist on change
  useEffect(() => {
    if (typeof window === "undefined" || !currentAgentId) return;
    saveAgentAnnotations(currentAgentId, localAnns, localConns);
  }, [currentAgentId, localAnns, localConns]);

  // Mount-gate for derivedAt 时间戳 (per 2026-09-06 19:42 JST hydration 修复):
  //   server render 时 derivedAt = null (避免 server t0 vs client t0+1s mismatch)
  //   useEffect mount 后才设 true, 此时 useMemo 重新计算填真实时间
  const [canvasHydrated, setCanvasHydrated] = useState(false);
  useEffect(() => { setCanvasHydrated(true); }, []);

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
      // SSR 阶段 derivedAt = null (避免 hydration mismatch: server t0 vs client t0+1s)
      // client mount 后 useEffect 补真实时间 (per 9/6 19:42 JST 修复)
      derivedAt: canvasHydrated ? new Date().toISOString() : null,
    };
  }, [resolution, worktree, relatedWorkItems, canvasHydrated]);

  // View 模式切换
  const handleViewModeChange = useCallback(
    (mode: ViewMode) => {
      setViewMode(mode);
      const params = new URLSearchParams(searchParams.toString());
      if (mode === "canvas") {
        params.delete("view");
      } else {
        params.set("view", mode);
      }
      router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    },
    [router, pathname, searchParams],
  );

  // dropdown 切换: 更新 URL (per 拍板 #2)
  const handleAgentChange = useCallback(
    (agentId: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set("agent", agentId);
      router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    },
    [router, pathname, searchParams],
  );

  // ---- 空状态 ----
  if (agents.length === 0) {
    return (
      <div className="max-w-3xl">
        <PageHeader
          title={t.pageTitles['/agent-view'].title}
          subtitle="查看当前 Agent、Worktree 与关联任务。此页面不启动 Agent 执行。"
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

  if (!resolution) {
    return (
      <div className="max-w-3xl">
        <PageHeader
          title={t.pageTitles['/agent-view'].title}
          subtitle="查看当前 Agent、Worktree 与关联任务。此页面不启动 Agent 执行。"
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

  const { agent } = resolution;
  return (
    <div className="w-full h-[calc(100vh-5rem)] flex flex-col">
      {/* Header */}
      <div className="border-b border-line bg-bg-soft/40 px-6 py-3 flex items-center justify-between gap-4">
        <div className="flex items-center gap-3 min-w-0">
          <div className="text-sm font-semibold shrink-0" data-testid="agent-view-title">Agent</div>
          <div className="text-xs text-ink-mute font-mono truncate hidden lg:block">
            {agent.id} · {agentKindDisplayLabel(agent.agent_kind)} · {agent.current_step} · {relatedWorkItems.length} task{relatedWorkItems.length === 1 ? "" : "s"}
            {worktree && <> · worktree <span className="text-info font-bold">{worktree.branch}</span></>}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          <AgentFilter
            agents={agents}
            selectedId={resolution.agentId}
            auto={resolution.auto}
            onChange={handleAgentChange}
          />
          <a
            href={`/board?assignee_id=&worktree_id=${agent.worktree_id}`}
            className="btn text-xs py-1.5 px-3 min-h-[34px] hidden md:inline-flex"
            data-testid="agent-view-jump-board"
            title="在 Board 视图查看关联任务"
          >
            <Maximize2 size={12} /> Board
          </a>
        </div>
      </div>

      {/* Agent workspace views */}
      <div className="border-b-2 border-black bg-[var(--cel-surface-card,#0f1422)] px-6 py-2.5 flex items-center gap-2.5 cel-shadow" data-testid="view-mode-tabs">
        <button
          data-testid="view-mode-canvas"
          onClick={() => handleViewModeChange("canvas")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "canvas" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <LayoutDashboard size={13} /> Canvas
        </button>
        <button
          data-testid="view-mode-relationships"
          onClick={() => handleViewModeChange("relationships")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "relationships" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <Network size={13} /> Relationship
        </button>
        <button
          data-testid="view-mode-workflow"
          onClick={() => handleViewModeChange("workflow")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "workflow" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <Workflow size={13} /> Workflow 工作流
        </button>
        <button
          data-testid="view-mode-settings"
          onClick={() => handleViewModeChange("settings")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "settings" ? "bg-[var(--cel-text-primary,#ffffff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <Settings size={13} /> Agent 设置
        </button>
      </div>

      {/* Content (switch by viewMode) */}
      {viewMode === "canvas" ? (
        <>
          <div className="flex items-center gap-2 px-3 py-1.5 bg-[var(--cel-surface-card,#0f1422)] border-b border-black">
            <button
              onClick={() => setEditMode((v) => !v)}
              data-testid="agent-view-edit-toggle"
              className={`px-2 py-1 text-xs font-mono font-bold border border-black transition-all ${editMode ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white"}`}
              title="开启后可加 sticky / 文本 / 图形 / 画笔 / 连线"
            >
              {editMode ? "✏️ Edit Mode" : "👁 View Mode"}
            </button>
            {editMode && (
              <>
                <button
                  onClick={() => {
                    const prev = undoStep(history);
                    if (prev !== history) {
                      setHistory(prev);
                      setLocalAnns(prev.current.annotations);
                      setLocalConns(prev.current.freeConnectors);
                    }
                  }}
                  disabled={!canUndo(history)}
                  data-testid="agent-view-undo"
                  className={`px-2 py-1 text-xs font-mono font-bold border border-black transition-all ${canUndo(history) ? "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white" : "opacity-40 cursor-not-allowed bg-[var(--cel-surface-sub,#151c2c)] text-ink-mute"}`}
                  title="撤销 (per 任务 #5)"
                >
                  ↶ Undo
                </button>
                <button
                  onClick={() => {
                    const next = redoStep(history);
                    if (next !== history) {
                      setHistory(next);
                      setLocalAnns(next.current.annotations);
                      setLocalConns(next.current.freeConnectors);
                    }
                  }}
                  disabled={!canRedo(history)}
                  data-testid="agent-view-redo"
                  className={`px-2 py-1 text-xs font-mono font-bold border border-black transition-all ${canRedo(history) ? "bg-[var(--cel-surface-sub,#151c2c)] text-ink-dim hover:text-white" : "opacity-40 cursor-not-allowed bg-[var(--cel-surface-sub,#151c2c)] text-ink-mute"}`}
                  title="重做 (per 任务 #5)"
                >
                  ↷ Redo
                </button>
              </>
            )}
            {editMode && (
              <span className="text-[10px] text-[var(--cel-cyan,#00f0ff)] font-mono">
                编辑模式 — 选中工具后在画布上画/写。Annotations 暂存本地 (per 2026-10-01 OOB)。
              </span>
            )}
          </div>
          {canvas && (
            <div className="flex-1 relative">
              <AgentCanvasView
                canvas={canvas}
                agent={agent}
                worktree={worktree}
                readOnly={!editMode}
                annotations={localAnns}
                freeConnectors={localConns}
                onCreateAnnotation={async (body) => {
                  const id = body.id;
                  commitStateChange([...localAnns, body], localConns);
                  return id;
                }}
                onDeleteAnnotation={async (id) => {
                  const nextAnns = localAnns.filter((a) => a.id !== id);
                  const nextConns = localConns.filter((c) => c.fromAnnotationId !== id && c.toAnnotationId !== id);
                  commitStateChange(nextAnns, nextConns);
                }}
                onCreateFreeConnector={async (body) => {
                  const id = `fc-local-${Math.random().toString(36).slice(2, 10)}`;
                  commitStateChange(localAnns, [...localConns, { id, ...body }]);
                  return id;
                }}
                onUpdateAnnotation={async (id, body) => {
                  const next = localAnns.map((a) => a.id === id ? { ...a, ...body, content: { ...a.content, ...(body.content || {}) } } as AgentCanvasAnnotation : a);
                  commitStateChange(next, localConns);
                }}
                onAnnotationPositionChange={async (body) => {
                  const next = localAnns.map((a) => a.id === body.id ? { ...a, x: body.x, y: body.y } : a);
                  commitStateChange(next, localConns);
                }}
                onBulkDeleteAnnotation={async (ids) => {
                  const idSet = new Set(ids);
                  const nextAnns = localAnns.filter((a) => !idSet.has(a.id));
                  const nextConns = localConns.filter((c) => !idSet.has(c.fromAnnotationId) && !idSet.has(c.toAnnotationId));
                  commitStateChange(nextAnns, nextConns);
                }}
              />
            </div>
          )}
        </>
      ) : viewMode === "workflow" ? (
        <div className="flex-1 min-h-0 p-2" data-testid="agent-view-workflow">
          <WorkflowBuilder />
        </div>
      ) : viewMode === "relationships" ? (
        <div className="flex-1 min-h-0 overflow-auto p-4">
          <AgentViewTab />
        </div>
      ) : (
        <div className="flex-1 min-h-0">
          <AgentSettingsTab initialAgentId={resolution.agentId} />
        </div>
      )}

      {/* 说明 footer */}
      <div className="border-t border-line bg-bg-soft/40 px-6 py-1.5 text-[10px] text-ink-mute font-mono flex items-center justify-between">
        <span>
          V/H 切换 select/pan · +/- 缩放 · 1 适配 · 双击节点打开关联详情
        </span>
        <span>
          nodes {canvas?.nodes.length ?? 0} · connectors {canvas?.connectors.length ?? 0} · derived {canvas?.derivedAt ? canvas.derivedAt.slice(11, 19) : "—"}
        </span>
      </div>

    </div>
  );
}

export default function AgentViewPage() {
  return (
    <Suspense fallback={<div className="p-8 text-center text-sm font-mono text-ink-dim">Loading Agent workspace…</div>}>
      <AgentViewContent />
    </Suspense>
  );
}
