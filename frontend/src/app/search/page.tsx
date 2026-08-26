"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { Search, Bookmark, FileText, GitBranch, MessageCircle, Cpu } from "lucide-react";
import { useState } from "react";

const ICON = {
  work_item: FileText,
  worktree: GitBranch,
  feedback: MessageCircle,
  agent_session: Cpu,
};

export default function SearchPage() {
  const hits = useStore((s) => s.searchHits);
  const saved = useStore((s) => s.savedSearches);
  const [q, setQ] = useState("");

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Search"
        subtitle="Projection + tenant 隔离 (INV-SR-01/02)。index 不作为业务事实源(§12 REQ-SEARCH-001)。"
        icon={<Search className="text-accent" size={20} />}
        track="B"
        count={hits.length}
      />

      <div className="card mb-5">
        <div className="relative">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-ink-mute" />
          <input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Search across work-items, worktrees, feedback, agent sessions..."
            className="w-full rounded-md border border-line bg-bg-soft pl-9 pr-3 py-2 text-sm placeholder:text-ink-mute focus:outline-none focus:border-accent"
          />
        </div>
        <div className="mt-2 text-[10px] text-ink-mute">
          Try: <code className="font-mono text-ink-dim">status:in_progress</code> · <code className="font-mono text-ink-dim">agent:awaiting_human</code> · <code className="font-mono text-ink-dim">priority:p0</code>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <div className="lg:col-span-2">
          <SectionTitle>Results</SectionTitle>
          <div className="space-y-2">
            {hits.map((h) => {
              const Icon = ICON[h.kind as keyof typeof ICON] ?? FileText;
              return (
                <div key={`${h.kind}-${h.id}`} className="card hover:border-accent/60 cursor-pointer transition-colors">
                  <div className="flex items-start gap-3">
                    <Icon size={14} className="text-accent mt-0.5" />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-0.5">
                        <span className="text-[10px] uppercase tracking-wider text-ink-mute">{h.kind}</span>
                        <span className="font-mono text-[10px] text-ink-mute">{h.id}</span>
                        <span className="ml-auto font-mono text-[10px] text-info">score {h.score.toFixed(2)}</span>
                      </div>
                      <div className="text-sm font-medium">{h.title}</div>
                      <p className="text-xs text-ink-dim mt-0.5 line-clamp-2">{h.snippet}</p>
                      <div className="mt-1.5 flex items-center gap-2">
                        <a
                          href={`/canvas/canvas-001?highlight=${
                            h.id === "wi-001" ? "el-wi-001" :
                            h.id === "wi-002" ? "el-wi-002" :
                            h.id === "wt-001" ? "el-wt-001" :
                            h.id === "wt-002" ? "el-wt-002" :
                            h.id === "wt-003" ? "el-wt-003" :
                            h.id === "fb-001" ? "el-fb-001" :
                            h.id === "fb-002" ? "el-fb-002" :
                            "el-wi-001"
                          }`}
                          className="text-[10px] text-accent hover:underline font-mono"
                        >
                          ⊞ 在 Canvas 中查看
                        </a>
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        <div>
          <SectionTitle><Bookmark size={11} className="inline mr-1" /> Saved searches</SectionTitle>
          <div className="space-y-2">
            {saved.map((s) => (
              <div key={s.id} className="card cursor-pointer hover:border-accent/60 transition-colors">
                <div className="text-sm font-medium">{s.name}</div>
                <div className="text-xs text-ink-dim font-mono">{s.query}</div>
                <div className="text-[10px] text-ink-mute font-mono mt-1">by {s.created_by}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
