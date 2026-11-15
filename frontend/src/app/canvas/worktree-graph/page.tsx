/*
@cypher schema=1 source_sha256=566c79cb5d55df20b0c16e0d5175a22a4e9a7a89397781724ee7ed861c14e9d4
MERGE (self:File {path:"frontend/src/app/canvas/worktree-graph/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/worktree-graph/page.tsx::WorktreeGraphPage",kind:"component"})
MERGE (groupApiHook:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupApi",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:CALLS]->(groupApiHook)
@endcypher
*/
"use client";

// =====================================================================
// /canvas/worktree-graph — Worktree 图 (per 2026-11-14: 占位 → 实装)
// 5 视图: 树 / 依赖 / 风险 / Agent / 历史 + 缩放, 数据源 store
// 2026-11-15 (per TASK-ulys-canvas-ctx):
//   - 5 视图状态区分: 有数据渲染列表 / 空数据空态 (wtg-empty-<view>) /
//     宿主未接入提示 (wtg-disconnected-<view>), 二选一渲染
//   - 去固定示例: 树视图 Project/LANE 改为 store.projects[0] / store.lanes[0]
// =====================================================================

import { useState } from "react";
import { Network, GitBranch, AlertTriangle, Bot, History, PlugZap } from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";

const GRAPH_VIEWS = ["树", "依赖", "风险", "Agent", "历史"] as const;
/** wtg-view-* / wtg-empty-* / wtg-disconnected-* 的视图 id (与既有 testid 对齐) */
type GraphViewId = "tree" | "deps" | "risk" | "agent" | "history";

export default function WorktreeGraphPage() {
  const [activeView, setActiveView] = useState<typeof GRAPH_VIEWS[number]>("树");
  const [zoom, setZoom] = useState(1);
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const agentSessions = useStore((s) => s.agentSessions);
  const projects = useStore((s) => s.projects);
  const lanes = useStore((s) => s.lanes);

  // 宿主接入信号 (per TASK-ulys-canvas-ctx): Group API 会话存在 = 宿主已接入;
  // 网页预览模式 (无宿主会话) 为 null → 视图渲染「未接入」提示,
  // 不展示无授权的固定示例数据.
  const groupApi = useWorktreeGroupApi();
  const hostConnected = groupApi !== null;

  // 图布局: worktree 为列 hub, 下挂任务; agent 列在右侧
  const cols = Math.min(4, Math.max(1, worktrees.length));
  const colW = 220;

  // 未接入空态 (per TASK-ulys-canvas-ctx: 现有 fallback 文案基础上统一)
  const renderDisconnected = (view: GraphViewId, label: string) => (
    <div
      data-testid={`wtg-disconnected-${view}`}
      className="flex min-h-48 flex-col items-center justify-center gap-2 p-6 text-center"
    >
      <PlugZap size={20} className="text-ink-mute" />
      <p className="text-xs font-semibold text-ink-dim">宿主未接入 — {label}数据通道不可用</p>
      <p className="max-w-md text-[10px] leading-relaxed text-ink-mute">
        Worktree 关系图依赖宿主 Group API 数据通道; 当前会话未装配或无权读取
        (网页预览模式)。请通过宿主会话打开。
      </p>
    </div>
  );

  // 空数据空态 (per TASK-ulys-canvas-ctx)
  const renderEmpty = (view: GraphViewId, text: string) => (
    <div
      data-testid={`wtg-empty-${view}`}
      className="flex min-h-48 flex-col items-center justify-center gap-2 p-6 text-center"
    >
      <div className="rounded-lg border border-dashed border-line bg-bg-soft/40 px-4 py-3">
        <p className="text-xs text-ink-mute">空数据 — {text}</p>
      </div>
    </div>
  );

  const renderView = () => {
    switch (activeView) {
      case "树": {
        // 树视图: Project → LANE → Worktree → Branch 层级
        // (去固定示例: 动态取 store.projects[0] / store.lanes[0])
        const project = projects[0];
        const lane = lanes[0];
        if (!hostConnected || !project || !lane) {
          return renderDisconnected("tree", "Project/LANE 树");
        }
        if (worktrees.length === 0) {
          return renderEmpty("tree", "当前 Project/LANE 下暂无 Worktree");
        }
        return (
          <div data-testid="wtg-view-tree" className="min-w-[640px] p-6">
            <div className="flex items-start gap-4">
              <div className="rounded-lg border-2 border-accent/60 bg-accent/10 px-4 py-3 text-sm font-bold text-accent" data-testid="wtg-root">
                Project · {project.name}
              </div>
              <div className="w-6 border-t-2 border-line mt-5" />
              <div className="flex-1">
                <div className="rounded-lg border border-warn/50 bg-warn/10 px-3 py-2 text-xs font-semibold text-warn inline-block">
                  LANE · {lane.name}
                </div>
                <div className="mt-3 space-y-2">
                  {worktrees.slice(0, 6).map((wt) => (
                    <div key={wt.id} className="flex items-center gap-3">
                      <div className="w-4 border-t border-line" />
                      <div className="flex items-center gap-2 rounded-lg border border-line bg-bg-card px-3 py-2" data-testid={`wtg-node-${wt.id}`}>
                        <GitBranch size={13} className="text-ok" />
                        <div>
                          <div className="text-xs font-semibold text-ink">{wt.name}</div>
                          <div className="text-[9px] font-mono text-ink-mute">{wt.id} · {wt.status}</div>
                        </div>
                      </div>
                      <div className="w-4 border-t border-line" />
                      <span className="text-[10px] font-mono px-1.5 py-0.5 rounded border border-line bg-bg-soft text-ink-dim">
                        {workItems.filter((w) => w.worktree_id === wt.id).length} 任务
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        );
      }
      case "依赖": {
        // 依赖视图: 任务按 worktree 分组 + 依赖关系线 (worktree→任务)
        if (!hostConnected) return renderDisconnected("deps", "依赖");
        if (worktrees.length === 0 || workItems.length === 0) {
          return renderEmpty("deps", "无任务");
        }
        return (
          <div data-testid="wtg-view-deps" className="p-6" style={{ width: cols * colW + 120 }}>
            <div className="flex items-start gap-8">
              {worktrees.slice(0, cols).map((wt, ci) => (
                <div key={wt.id} className="flex-1" style={{ minWidth: colW }}>
                  <div className="rounded-lg border border-ok/50 bg-ok/10 px-3 py-2 text-xs font-bold text-ok text-center" data-testid={`wtg-dep-hub-${wt.id}`}>
                    {wt.name}
                  </div>
                  <div className="mt-2 flex flex-col items-center gap-1.5">
                    {workItems.filter((w) => w.worktree_id === wt.id).slice(0, 8).map((wi) => (
                      <div key={wi.id} className="w-full flex items-center gap-1.5">
                        <div className="w-3 border-t border-line" />
                        <a
                          href={`/work-item/${wi.id}`}
                          className="flex-1 truncate rounded border border-line bg-bg-card px-2 py-1.5 text-[11px] text-ink hover:border-accent/50 font-mono"
                          data-testid={`wtg-dep-task-${wi.id}`}
                        >
                          {wi.key} {wi.title}
                        </a>
                      </div>
                    ))}
                    {workItems.filter((w) => w.worktree_id === wt.id).length === 0 && (
                      <span className="text-[10px] text-ink-mute">无任务</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        );
      }
      case "风险": {
        // 风险视图: 阻塞/冲突任务 + 高优先级
        const risky = workItems.filter((w) => w.status === "blocked" || w.priority === "p0" || w.priority === "p1");
        if (!hostConnected) return renderDisconnected("risk", "风险");
        if (risky.length === 0) return renderEmpty("risk", "无阻塞/高优先风险任务");
        return (
          <div data-testid="wtg-view-risk" className="p-6 space-y-2 max-w-xl">
            <div className="flex items-center gap-2 text-xs text-ink-dim mb-2">
              <AlertTriangle size={13} className="text-err" /> 风险项 ({risky.length})
            </div>
            {risky.map((wi) => (
              <a
                key={wi.id}
                href={`/work-item/${wi.id}`}
                className="flex items-center gap-2 rounded-lg border border-err/40 bg-err/5 px-3 py-2 hover:border-err/70"
                data-testid={`wtg-risk-${wi.id}`}
              >
                <AlertTriangle size={12} className="text-err shrink-0" />
                <span className="font-mono text-[11px] text-err font-bold">{wi.key}</span>
                <span className="flex-1 truncate text-xs text-ink">{wi.title}</span>
                <span className="text-[10px] font-mono text-ink-mute">{wi.status} · {wi.priority}</span>
              </a>
            ))}
          </div>
        );
      }
      case "Agent": {
        // Agent 视图: agent 会话 ↔ worktree 关联
        if (!hostConnected) return renderDisconnected("agent", "Agent 会话");
        if (agentSessions.length === 0) return renderEmpty("agent", "无 Agent 会话");
        return (
          <div data-testid="wtg-view-agent" className="p-6">
            <div className="space-y-2 max-w-xl">
              {agentSessions.slice(0, 8).map((ag) => {
                const wt = worktrees.find((w) => w.id === ag.worktree_id);
                return (
                  <div key={ag.id} className="flex items-center gap-3 rounded-lg border border-line bg-bg-card px-3 py-2">
                    <Bot size={13} className="text-accent shrink-0" />
                    <div className="min-w-0 flex-1">
                      <div className="text-xs font-semibold text-ink truncate">{ag.name}</div>
                      <div className="text-[9px] font-mono text-ink-mute">{ag.id} · {ag.agent_kind} · {ag.status}</div>
                    </div>
                    {wt && (
                      <a href={`/worktree/${wt.id}/group`} className="flex items-center gap-1 text-[10px] font-mono px-1.5 py-0.5 rounded border border-ok/40 bg-ok/10 text-ok shrink-0" data-testid={`wtg-agent-wt-${ag.id}`}>
                        <GitBranch size={9} /> {wt.name}
                      </a>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        );
      }
      case "历史": {
        // 历史视图: 最近更新任务 (按 updated_at 排序)
        const recent = [...workItems].sort((a, b) => (b.updated_at ?? "").localeCompare(a.updated_at ?? "")).slice(0, 10);
        if (!hostConnected) return renderDisconnected("history", "历史");
        if (recent.length === 0) return renderEmpty("history", "无历史");
        return (
          <div data-testid="wtg-view-history" className="p-6 space-y-1.5 max-w-xl">
            <div className="flex items-center gap-2 text-xs text-ink-dim mb-2">
              <History size={13} className="text-ink-mute" /> 最近更新
            </div>
            {recent.map((wi) => (
              <div key={wi.id} className="flex items-center gap-2 rounded border border-line bg-bg-card px-3 py-1.5" data-testid={`wtg-hist-${wi.id}`}>
                <span className="font-mono text-[11px] text-info font-bold">{wi.key}</span>
                <span className="flex-1 truncate text-xs text-ink">{wi.title}</span>
                <span className="text-[9px] font-mono text-ink-mute">{(wi.updated_at ?? "").slice(0, 16)}</span>
              </div>
            ))}
          </div>
        );
      }
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg">
      {/* 顶部视图切换 */}
      <div className="flex items-center gap-4 border-b border-line bg-bg-card px-4 py-2">
        <a
          href="/canvas/canvas-001"
          data-testid="wtg-back-canvas"
          className="flex items-center gap-1 text-[11px] text-ink-mute hover:text-accent shrink-0"
        >
          <Network size={12} /> 画布
        </a>
        <div className="flex items-center gap-1">
          {GRAPH_VIEWS.map((v) => (
            <button
              key={v}
              type="button"
              onClick={() => setActiveView(v)}
              data-testid={`wtg-tab-${v}`}
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
      <main className="flex-1 overflow-auto bg-bg relative" data-testid="wtg-canvas">
        <div
          style={{ transform: `scale(${zoom})`, transformOrigin: "top left" }}
        >
          {renderView()}
        </div>
      </main>
    </div>
  );
}
