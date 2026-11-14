/*
@cypher schema=1 source_sha256=aab2087b46b2283eed9acf7681dd529fe6af0f57c19886c0920b80956e2fbac5
MERGE (self:File {path:"frontend/src/app/(app)/kanban/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/kanban/page.tsx::KanbanPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /kanban — 任务 IPA 看板 (per 2026-11-14 artifact: 顶栏第 1 个 Branch 功能 tab)
// 4 态泳道即时拖拽任务看板 (kanban board)
// =====================================================================
"use client";

import { KanbanSquare, Plus, Search, Filter } from "lucide-react";
import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { useTranslation } from "@/lib/i18n";

const COLUMNS = [
  { id: "backlog", title: "积压", color: "bg-slate-500/20" },
  { id: "ready", title: "就绪", color: "bg-cyan-500/20" },
  { id: "in_progress", title: "进行中", color: "bg-blue-500/20" },
  { id: "done", title: "完成", color: "bg-emerald-500/20" },
];

const MOCK_CARDS = [
  { id: "t-001", title: "登录页重构", column: "backlog", assignee: "alex", labels: ["feat", "p1"] },
  { id: "t-002", title: "API 契约测试", column: "ready", assignee: "sam", labels: ["test", "p2"] },
  { id: "t-003", title: "看板拖拽优化", column: "in_progress", assignee: "you", labels: ["perf", "p1"] },
  { id: "t-004", title: "部署流水线", column: "done", assignee: "ci-bot", labels: ["ops", "p3"] },
];

export default function KanbanPage() {
  const { t } = useTranslation();
  return (
    <div className="mx-auto max-w-7xl p-6">
      <header className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-ink">{t.kanban.title}</h1>
          <p className="mt-1 text-sm text-ink-dim">{t.kanban.subtitle}</p>
        </div>
        <div className="flex items-center gap-2">
          <button type="button" className="p-2 rounded-lg border border-line hover:bg-bg-soft" aria-label={t.kanban.filter}>
            <Filter size={16} />
          </button>
          <Link href="/tasks/new" data-testid="kanban-new-task" className="flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-white">
            <Plus size={13} /> {t.kanban.newTask}
          </Link>
        </div>
      </header>

      <div className="grid gap-4 grid-cols-4 overflow-x-auto pb-4">
        {COLUMNS.map((col) => (
          <section key={col.id} data-testid={`kanban-column-${col.id}`} className="flex flex-col min-w-[280px] max-w-[320px] rounded-xl border border-line bg-bg-card">
            <header className="flex items-center justify-between px-3 py-2.5 border-b border-line">
              <h2 className="text-sm font-semibold text-ink">{col.title}</h2>
              <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded-full ${col.color} text-white`}>
                {MOCK_CARDS.filter((c) => c.column === col.id).length}
              </span>
            </header>
            <ul className="flex-1 overflow-y-auto p-2 space-y-2" role="list">
              {MOCK_CARDS.filter((c) => c.column === col.id).map((card) => (
                <li key={card.id} data-testid={`kanban-card-${card.id}`} className="rounded-lg border border-line bg-white p-3 shadow-sm hover:shadow-md transition-shadow">
                  <h3 className="text-sm font-semibold text-ink">{card.title}</h3>
                  <div className="mt-2 flex items-center gap-1.5 flex-wrap">
                    {card.labels.map((l) => (
                      <span key={l} className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-line bg-bg-soft">{l}</span>
                    ))}
                  </div>
                  <div className="mt-2 flex items-center gap-2 text-[10px] text-ink-mute">
                    <span className="font-mono">{card.assignee}</span>
                  </div>
                </li>
              ))}
              {MOCK_CARDS.filter((c) => c.column === col.id).length === 0 && (
                <li className="text-center py-6 text-ink-dim text-sm">{t.kanban.emptyColumn}</li>
              )}
            </ul>
            <footer className="px-2 pb-3">
              <button type="button" className="w-full text-sm text-ink-dim hover:text-accent flex items-center justify-center gap-1.5 py-1.5 rounded-lg hover:bg-bg-soft transition-colors">
                <Plus size={13} /> {t.kanban.addCard}
              </button>
            </footer>
          </section>
        ))}
      </div>
    </div>
  );
}