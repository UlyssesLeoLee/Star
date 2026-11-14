/*
@cypher schema=1 source_sha256=02e91eeb4c9f49b452419944a01a8ceba604ed6703927f51a3da42ddb84f1136
MERGE (self:File {path:"frontend/src/app/canvas/[id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasDetailPage",kind:"component"})
MERGE (workflowPage:Symbol {id:"frontend/src/app/workflow/page.tsx::WorkflowPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:REUSES]->(workflowPage)
@endcypher
*/
"use client";

// =====================================================================
// /canvas/[id] — 画布详情 (per 2026-11-14 用户发令: 页面合并)
//   3 标签页切换 (不再分白板; CanvasView 白板由 worktree group 页承载):
//     1. 关系图  — Worktree/任务/文件/上下文 4 层
//     2. IPA (V字模型) — 需求→设计→编码→测试→验收
//     3. 低代码流程 — workflow 工作流状态机 + Agent 可视化编排
//        (原 /workflow 内容并入: workflows 列表 + 状态流 + transitions 表)
// =====================================================================

import { useState } from "react";
import { useStore } from "@/lib/store";
import { StatusPill } from "@/components/StatusPill";
import { ArrowRight, GitBranch, Workflow, Network } from "lucide-react";

const CANVAS_SUB_TABS = ["关系图", "IPA (V字模型)", "低代码流程"] as const;

export default function CanvasDetailPage() {
  const [activeTab, setActiveTab] = useState<typeof CANVAS_SUB_TABS[number]>("低代码流程");
  const workflows = useStore((s) => s.workflows);
  const worktrees = useStore((s) => s.worktrees);

  const renderCanvasContent = () => {
    switch (activeTab) {
      case "关系图":
        return (
          <div className="h-full overflow-y-auto p-4" data-testid="canvas-tab-relation">
            <div className="bg-bg-card rounded-xl border border-line p-4 mb-3">
              <h2 className="text-sm font-bold text-ink mb-3 flex items-center gap-2">
                <Network size={14} className="text-accent" /> 关系图 · Worktree / 任务 / 文件 / 上下文
              </h2>
              <p className="text-xs text-ink-dim mb-3">
                4 层关系: Worktree 层 → 任务层 → 文件层 → 上下文层。点击节点跳对应详情。
              </p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                {worktrees.slice(0, 6).map((wt) => (
                  <div key={wt.id} className="rounded-lg border border-line bg-bg p-2.5 flex items-center gap-2">
                    <GitBranch size={12} className="text-accent shrink-0" />
                    <div className="min-w-0">
                      <div className="text-xs font-semibold text-ink truncate">{wt.name}</div>
                      <div className="text-[9px] font-mono text-ink-mute">{wt.id} · {wt.status}</div>
                    </div>
                  </div>
                ))}
              </div>
              <div className="mt-3 flex flex-wrap gap-1.5">
                <a href="/canvas/worktree-graph" className="text-[11px] px-2 py-1 rounded border border-line text-accent hover:bg-accent/10">Worktree 图 →</a>
                <a href="/agent-relationships" className="text-[11px] px-2 py-1 rounded border border-line text-accent hover:bg-accent/10">Agent 关系 →</a>
                <a href="/context" className="text-[11px] px-2 py-1 rounded border border-line text-accent hover:bg-accent/10">上下文 →</a>
                <a href="/collaboration" className="text-[11px] px-2 py-1 rounded border border-line text-accent hover:bg-accent/10">协同 →</a>
              </div>
            </div>
          </div>
        );
      case "IPA (V字模型)":
        return (
          <div className="h-full overflow-y-auto p-4" data-testid="canvas-tab-ipa">
            <div className="bg-bg-card rounded-xl border border-line p-4">
              <h2 className="text-sm font-bold text-ink mb-3">IPA V字模型 · 需求 → 验收</h2>
              <p className="text-xs text-ink-dim mb-4">需求定义 → 系统设计 → 模块设计 → 编码 → 单测 → 集成测 → 系统测 → 验收</p>
              <div className="flex items-center gap-1.5 flex-wrap">
                {["需求定义", "系统设计", "模块设计", "编码", "单测", "集成测", "系统测", "验收"].map((s, i) => (
                  <div key={s} className="flex items-center gap-1.5">
                    <span className={i < 4
                      ? "px-2.5 py-1.5 rounded-md border border-accent/40 bg-accent/10 text-accent text-xs font-semibold"
                      : "px-2.5 py-1.5 rounded-md border border-ok/40 bg-ok/10 text-ok text-xs font-semibold"}>
                      {i + 1}. {s}
                    </span>
                    {i < 7 && <ArrowRight size={11} className="text-ink-mute" />}
                  </div>
                ))}
              </div>
            </div>
          </div>
        );
      case "低代码流程":
        // ===== 原 /workflow 工作流内容 (per 2026-11-14 用户发令: 并入低代码流程标签) =====
        return (
          <div className="h-full overflow-y-auto p-4" data-testid="canvas-tab-lowcode">
            <div className="bg-bg-card rounded-xl border border-line p-4 mb-3">
              <h2 className="text-sm font-bold text-ink mb-1 flex items-center gap-2">
                <Workflow size={14} className="text-accent" /> 低代码流程 · 工作流状态机
              </h2>
              <p className="text-[11px] text-ink-dim mb-4">
                每个 project 可绑定一个 workflow。state.category 决定 work-item status 字段;
                transition.trigger + guard 控制迁移。Agent 与 Workflow 均为节点，可视化编排。
              </p>

              {workflows.length === 0 ? (
                <p className="text-xs text-ink-mute p-4 text-center">暂无工作流定义</p>
              ) : (
                <div className="grid grid-cols-1 gap-4">
                  {workflows.map((wf) => (
                    <div key={wf.id} className="rounded-xl border border-line bg-bg p-3.5" data-testid={`lowcode-wf-${wf.id}`}>
                      <div className="flex items-center justify-between mb-3">
                        <div>
                          <div className="text-sm font-semibold text-ink">{wf.name}</div>
                          <div className="text-xs text-ink-mute font-mono">{wf.id} · project {wf.project_id}</div>
                        </div>
                        <div className="flex items-center gap-2">
                          {wf.is_default && <span className="pill border-accent/40 text-accent bg-accent/10 text-[10px]">default</span>}
                          <span className="text-xs text-ink-dim font-mono">{wf.states.length} states · {wf.transitions.length} transitions</span>
                        </div>
                      </div>

                      {/* State flow */}
                      <div className="flex items-center gap-1 overflow-x-auto pb-2">
                        {wf.states.map((s, i) => (
                          <div key={s.id} className="flex items-center gap-1 shrink-0">
                            <div className={`px-3 py-2 rounded-md border text-xs font-mono ${
                              s.kind === "initial" ? "border-accent/40 bg-accent/10 text-accent" :
                              s.kind === "final"   ? "border-ok/40 bg-ok/10 text-ok" :
                              "border-line bg-bg-soft text-ink"
                            }`}>
                              <div className="font-semibold">{s.name}</div>
                              <div className="text-[9px] text-ink-mute">{s.category}</div>
                            </div>
                            {i < wf.states.length - 1 && <ArrowRight size={12} className="text-ink-mute" />}
                          </div>
                        ))}
                      </div>

                      {/* Transitions */}
                      <div className="mt-3 pt-3 border-t border-line">
                        <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-2">Transitions</div>
                        <table className="w-full text-xs">
                          <thead><tr className="text-left text-ink-mute"><th className="pb-1.5">From</th><th className="pb-1.5">To</th><th className="pb-1.5">Trigger</th><th className="pb-1.5">Guard</th></tr></thead>
                          <tbody>
                            {wf.transitions.map((t, i) => {
                              const from = wf.states.find((s) => s.id === t.from_state_id);
                              const to = wf.states.find((s) => s.id === t.to_state_id);
                              return (
                                <tr key={i} className="border-t border-line/60">
                                  <td className="py-1.5"><StatusPill value={from?.category ?? "?"} size="xs" /></td>
                                  <td className="py-1.5"><StatusPill value={to?.category ?? "?"} size="xs" /></td>
                                  <td className="py-1.5 font-mono">{t.trigger}</td>
                                  <td className="py-1.5 font-mono text-ink-dim">{t.guard ?? "—"}</td>
                                </tr>
                              );
                            })}
                          </tbody>
                        </table>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg" data-testid="canvas-detail-page">
      {/* 标签页切换 (per 2026-11-14: 合并后直接呈现, 不再分白板选项) */}
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

      {/* 内容区 */}
      <main className="flex-1 overflow-hidden">
        {renderCanvasContent()}
      </main>
    </div>
  );
}
