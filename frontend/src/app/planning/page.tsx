"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Calendar, TrendingDown, Flag, Target } from "lucide-react";

export default function PlanningPage() {
  const sprints = useStore((s) => s.sprints);
  const milestones = useStore((s) => s.milestones);
  const burndown = useStore((s) => s.burndownSeries);

  const maxRemaining = Math.max(...burndown.map((b) => b.remaining_points), 1);

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Planning"
        subtitle="Sprint / Milestone / Burndown 三件套。每个 sprint 有 capacity / committed / completed 三维度。"
        icon={<Calendar className="text-accent" size={20} />}
        track="E"
        count={`${sprints.length} sprints / ${milestones.length} milestones`}
      />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-3 mb-5">
        <div className="card">
          <SectionTitle><TrendingDown size={11} className="inline mr-1" /> Burndown</SectionTitle>
          <svg viewBox="0 0 500 200" className="w-full h-48">
            <defs>
              <linearGradient id="burn-fill" x1="0" x2="0" y1="0" y2="1">
                <stop offset="0%" stopColor="#2f81f7" stopOpacity="0.3" />
                <stop offset="100%" stopColor="#2f81f7" stopOpacity="0" />
              </linearGradient>
            </defs>
            {/* Axes */}
            <line x1="30" y1="170" x2="490" y2="170" stroke="#21262d" />
            <line x1="30" y1="20"  x2="30"  y2="170" stroke="#21262d" />
            {/* Ideal line */}
            <line x1="30" y1="30" x2="490" y2="155" stroke="#6e7681" strokeDasharray="3,3" />
            {/* Actual area */}
            <path
              d={`M 30 ${170 - (burndown[0].remaining_points / maxRemaining) * 140} ` +
                 burndown.map((b, i) => `L ${30 + (i * 460 / (burndown.length - 1))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ") +
                 ` L 490 170 L 30 170 Z`}
              fill="url(#burn-fill)"
            />
            {/* Actual line */}
            <path
              d={`M 30 ${170 - (burndown[0].remaining_points / maxRemaining) * 140} ` +
                 burndown.map((b, i) => `L ${30 + (i * 460 / (burndown.length - 1))} ${170 - (b.remaining_points / maxRemaining) * 140}`).join(" ")}
              fill="none"
              stroke="#2f81f7"
              strokeWidth="2"
            />
            {/* Y label */}
            <text x="5" y="30" fontSize="9" fill="#6e7681">{maxRemaining}</text>
            <text x="5" y="170" fontSize="9" fill="#6e7681">0</text>
            <text x="240" y="190" fontSize="9" fill="#6e7681" textAnchor="middle">days</text>
          </svg>
          <div className="flex items-center gap-3 text-[10px] text-ink-mute">
            <span className="flex items-center gap-1"><span className="w-3 h-0.5 bg-info" /> actual</span>
            <span className="flex items-center gap-1"><span className="w-3 h-0.5 bg-ink-mute" style={{borderTop:"1px dashed #6e7681"}} /> ideal</span>
          </div>
        </div>

        <div className="card">
          <SectionTitle><Flag size={11} className="inline mr-1" /> Milestones</SectionTitle>
          <div className="space-y-2">
            {milestones.map((m) => (
              <div key={m.id} className="p-2 rounded border border-line bg-bg-soft/40">
                <div className="flex items-center justify-between mb-1">
                  <div className="text-sm font-medium">{m.name}</div>
                  <span className="text-[10px] text-ink-mute font-mono">due {new Date(m.due_date).toLocaleDateString()}</span>
                </div>
                <div className="h-1.5 rounded bg-bg-soft overflow-hidden">
                  <div className="h-full bg-accent" style={{ width: `${m.progress * 100}%` }} />
                </div>
                <div className="mt-1 flex items-center justify-between text-[10px] text-ink-mute font-mono">
                  <span>{m.work_item_ids.length} work-items</span>
                  <span>{(m.progress * 100).toFixed(0)}% complete</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <SectionTitle><Target size={11} className="inline mr-1" /> Sprints</SectionTitle>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {sprints.map((sp) => {
          const pct = sp.committed_points > 0 ? sp.completed_points / sp.committed_points : 0;
          return (
            <div key={sp.id} className="card">
              <div className="flex items-center justify-between mb-2">
                <div>
                  <div className="text-sm font-semibold">{sp.name}</div>
                  <div className="text-xs text-ink-dim mt-0.5">{sp.goal}</div>
                </div>
                <StatusPill value={sp.status} />
              </div>
              <div className="grid grid-cols-3 gap-2 mb-2">
                <Stat label="Capacity" value={sp.capacity_points} />
                <Stat label="Committed" value={sp.committed_points} tone="info" />
                <Stat label="Completed" value={sp.completed_points} tone="ok" />
              </div>
              <div className="h-2 rounded bg-bg-soft overflow-hidden">
                <div className="h-full bg-gradient-to-r from-info to-ok" style={{ width: `${pct * 100}%` }} />
              </div>
              <div className="mt-1 text-[10px] text-ink-mute font-mono">
                {new Date(sp.start_date).toLocaleDateString()} → {new Date(sp.end_date).toLocaleDateString()}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
