"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { FileText, Tag, User, Hash, Flag } from "lucide-react";
import { useState } from "react";
import { WORKITEM_SM, type WorkItemStatus } from "@/types/ids";
import { StateMachineDiagram } from "@/components/StateMachineDiagram";

export default function WorkItemPage() {
  const { workItems, transitionWorkItem } = useStore();
  const [selected, setSelected] = useState<string | null>(null);
  const [kindFilter, setKindFilter] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<string>("all");

  const filtered = workItems.filter((w) =>
    (kindFilter === "all" || w.kind === kindFilter) &&
    (statusFilter === "all" || w.status === statusFilter),
  );

  const wi = workItems.find((w) => w.id === selected);
  const allowed = wi ? WORKITEM_SM.transitions.filter((t) => t.from === wi.status).map((t) => t.to) : [];

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Work Items"
        subtitle="Story / Task / Bug / Spike / Epic — 6 默认状态 (todo/in_progress/review/blocked/done/wontfix)。每 work-item 必带 tenant_id + project_id + key (PHYSIS-123)。"
        icon={<FileText className="text-accent" size={20} />}
        track="D"
        count={workItems.length}
      />

      <SectionTitle>状态机 (6 状态, INV-PM-01~05)</SectionTitle>
      <div className="mb-5">
        <StateMachineDiagram sm={WORKITEM_SM} highlightState={wi?.status} />
      </div>

      {/* Filters */}
      <div className="card mb-3 flex items-center gap-2 text-xs">
        <span className="text-ink-dim">Filter:</span>
        <select value={kindFilter} onChange={(e) => setKindFilter(e.target.value)} className="bg-bg-soft border border-line rounded px-2 py-1 text-xs">
          <option value="all">all kinds</option>
          <option value="story">story</option>
          <option value="task">task</option>
          <option value="bug">bug</option>
          <option value="spike">spike</option>
          <option value="epic">epic</option>
        </select>
        <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)} className="bg-bg-soft border border-line rounded px-2 py-1 text-xs">
          <option value="all">all statuses</option>
          {["todo","in_progress","review","blocked","done","wontfix"].map((s) => <option key={s} value={s}>{s}</option>)}
        </select>
        <span className="ml-auto text-ink-mute font-mono">{filtered.length} of {workItems.length}</span>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <div className="lg:col-span-2 card">
          <table className="table">
            <thead>
              <tr>
                <th>Key</th>
                <th>Title</th>
                <th>Kind</th>
                <th>Status</th>
                <th>Priority</th>
                <th>SP</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((w) => (
                <tr key={w.id} onClick={() => setSelected(w.id)} className="cursor-pointer">
                  <td className="font-mono text-xs text-info">{w.key}</td>
                  <td className="font-medium">{w.title}</td>
                  <td><StatusPill value={w.kind} size="xs" /></td>
                  <td><StatusPill value={w.status} size="xs" /></td>
                  <td>
                    <span className={`font-mono text-xs ${
                      w.priority === "p0" ? "text-err" :
                      w.priority === "p1" ? "text-warn" :
                      w.priority === "p2" ? "text-info" : "text-ink-dim"
                    }`}>{w.priority.toUpperCase()}</span>
                  </td>
                  <td className="font-mono text-xs">{w.story_points ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {wi ? (
          <div className="card sticky top-16">
            <div className="flex items-center justify-between mb-3">
              <div>
                <div className="text-xs text-ink-mute font-mono">{wi.key}</div>
                <div className="text-base font-semibold">{wi.title}</div>
              </div>
              <StatusPill value={wi.status} />
            </div>
            <dl className="text-xs space-y-1.5 mb-4">
              <Row label={<><Tag size={10} className="inline mr-1" />Kind</>} value={wi.kind} />
              <Row label={<><Flag size={10} className="inline mr-1" />Priority</>} value={wi.priority.toUpperCase()} />
              <Row label={<><User size={10} className="inline mr-1" />Assignee</>} value={wi.assignee_id ?? "unassigned"} />
              <Row label={<><Hash size={10} className="inline mr-1" />Story points</>} value={wi.story_points ?? "—"} />
              <Row label="Sprint" value={wi.sprint_id ?? "—"} />
              <Row label="Workflow" value={wi.workflow_id ?? "—"} />
              <Row label="Created" value={new Date(wi.created_at).toLocaleDateString()} />
            </dl>
            {wi.labels.length > 0 && (
              <div className="mb-3">
                <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Labels</div>
                <div className="flex flex-wrap gap-1">
                  {wi.labels.map((l) => <span key={l} className="pill border-line text-ink-dim text-[10px]">{l}</span>)}
                </div>
              </div>
            )}
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5">Transition</div>
            <div className="flex flex-wrap gap-1.5">
              {allowed.map((to) => (
                <button key={to} onClick={() => transitionWorkItem(wi.id, to as WorkItemStatus)} className="btn-primary">→ {to}</button>
              ))}
              {allowed.length === 0 && <span className="text-xs text-ink-mute italic">终态, 无下游迁移</span>}
            </div>
            <div className="mt-3 pt-3 border-t border-line">
              <Link
                href={`/canvas/canvas-001?highlight=el-wi-001`}
                className="btn text-[10px]"
                title="在 Miro 模式画布中查看(双击 element 可跳回)"
              >
                <span className="font-mono">⊞</span> 打开在 Canvas
              </Link>
            </div>
          </div>
        ) : (
          <div className="card text-center text-ink-mute text-sm">← 选择一个 work-item 查看详情</div>
        )}
      </div>
    </div>
  );
}

function Row({ label, value }: { label: React.ReactNode; value: React.ReactNode }) {
  return (
    <div className="flex justify-between">
      <dt className="text-ink-mute">{label}</dt>
      <dd className="text-ink font-mono text-[11px]">{value}</dd>
    </div>
  );
}
