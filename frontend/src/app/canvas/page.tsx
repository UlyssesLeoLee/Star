/*
@cypher schema=1 source_sha256=8ba55e88cc4738c7bdd68e6ca309ad8314200bc369bf636ab5c7f2ea6c13e3a7
MERGE (self:File {path:"frontend/src/app/canvas/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/page.tsx::CanvasGalleryPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /canvas — 无限画布中心 (per 2026-11-14 用户发令 "多种类无限画布整合在一起")
// 统一入口: 3 类画布画廊
//   1. Canvas 画布 (帧/端点/连接线)  → /canvas/canvas-001
//   2. Worktree 图 (树/依赖/风险 5 视图) → /canvas/worktree-graph
//   3. Agent 拓扑 (无限画布 agent 关联)  → /agent-view
// =====================================================================
"use client";

import { Frame, Network, Bot, ArrowRight } from "lucide-react";
import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { useTranslation } from "@/lib/i18n";

const CANVAS_ENTRIES = [
  {
    id: "canvas-docs",
    href: "/canvas/canvas-001",
    icon: Frame,
    title: "Canvas 画布",
    desc: "帧 / 输入输出端点 / 连接线 · seed canvas-001 (关联 prj-physis)",
    badge: "prj-physis",
  },
  {
    id: "canvas-worktree-graph",
    href: "/canvas/worktree-graph",
    icon: Network,
    title: "Worktree 图",
    desc: "AI Worktree Graph · 树 / 依赖 / 风险 / Agent / 历史 5 视图 + 6 级缩放",
    badge: "Project → LANE → Worktree",
  },
  {
    id: "canvas-agent-topology",
    href: "/agent-view",
    icon: Bot,
    title: "Agent 拓扑",
    desc: "无限画布 · 当前工作 agent 关联 worktree / work-items 拓扑图",
    badge: "worktree-scoped",
  },
];

export default function CanvasGalleryPage() {
  const { t } = useTranslation();
  return (
    <div className="mx-auto max-w-5xl p-6">
      <header className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight text-ink">{t.canvasGallery.title}</h1>
        <p className="mt-1 text-sm text-ink-dim">{t.canvasGallery.subtitle}</p>
      </header>
      <div className="grid gap-4 md:grid-cols-3">
        {CANVAS_ENTRIES.map((entry) => {
          const Icon = entry.icon;
          return (
            <Link
              key={entry.id}
              href={entry.href}
              data-testid={`canvas-gallery-${entry.id}`}
              className="group rounded-xl border border-line bg-bg-card p-5 hover:border-accent/60 hover:shadow-soft transition-all"
            >
              <div className="flex items-start justify-between">
                <div className="size-10 rounded-lg grid place-items-center border border-cyan-500/40 bg-cyan-500/15 text-cyan-400">
                  <Icon size={18} />
                </div>
                <span className="text-[10px] font-mono px-1.5 py-0.5 rounded border border-line/60 text-ink-mute">
                  {entry.badge}
                </span>
              </div>
              <h2 className="mt-3 font-semibold text-ink text-sm">{entry.title}</h2>
              <p className="mt-1 text-xs text-ink-dim leading-relaxed">{entry.desc}</p>
              <div className="mt-3 flex items-center gap-1 text-xs text-accent opacity-0 group-hover:opacity-100 transition-opacity">
                {t.canvasGallery.open} <ArrowRight size={12} />
              </div>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
