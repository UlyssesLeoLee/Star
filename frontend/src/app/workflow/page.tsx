/*
@cypher schema=1 source_sha256=8c1227b79cdfb42011283af7ae334e9c1874f0cac28c18d645e233742e425b11
MERGE (self:File {path:"frontend/src/app/workflow/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/workflow/page.tsx::WorkflowPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
"use client";

// =====================================================================
// /workflow — 低代码流程独立页 (per 2026-11-14 用户发令:
//   无限画布类全部归纳进 canvas; 不能整合的 (低代码流程/状态机) 改单独页面)
//   workflows 状态机: 状态流 + transitions 表 (触发/守卫)
// =====================================================================

import { useStore } from "@/lib/store";
import { StatusPill } from "@/components/StatusPill";
import { ArrowRight, Workflow } from "lucide-react";

export default function WorkflowPage() {
  const workflows = useStore((s) => s.workflows);
  return (
    <div className="h-full overflow-y-auto p-4 sm:p-6" data-testid="workflow-page">
      <div className="max-w-5xl mx-auto">
        <header className="mb-5">
          <h1 className="text-lg font-bold text-ink flex items-center gap-2">
            <Workflow size={18} className="text-accent" /> 低代码流程
          </h1>
          <p className="mt-1 text-xs text-ink-dim">
            每个 project 可绑定一个 workflow。state.category 决定 work-item status 字段;
            transition.trigger + guard 控制迁移。Agent 与 Workflow 均为节点，可视化编排。
          </p>
        </header>

        {workflows.length === 0 ? (
          <p className="text-xs text-ink-mute p-4 text-center">暂无工作流定义</p>
        ) : (
          <div className="grid grid-cols-1 gap-4">
            {workflows.map((wf) => (
              <div key={wf.id} className="rounded-xl border border-line bg-bg-card p-4" data-testid={`lowcode-wf-${wf.id}`}>
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
}
