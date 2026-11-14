/*
@cypher schema=1 source_sha256=37fda15d3d39d5416ea6287dcd6eeea2cd8cae138450a290d3696becdd6dc336
MERGE (self:File {path:"frontend/src/app/(app)/prototype/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/prototype/page.tsx::PrototypePage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /prototype — 原型 (per 2026-11-14 页面结构: 2d · 原型)
// 设计原型与实验区画廊: wireframes / 画布原型 / 交互原型统一入口
// =====================================================================
"use client";

import { Frame, FileCode2, Palette } from "lucide-react";
import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { useTranslation } from "@/lib/i18n";

const PROTOTYPE_ENTRIES = [
  {
    id: "proto-wireframes",
    href: "/canvas/canvas-001",
    icon: Frame,
    title: "Wireframe 画布",
    desc: "UI wireframes (per docs/frontend/design/ui-wireframes.pdf) 帧结构原型",
  },
  {
    id: "proto-canvas-game",
    href: "/canvas/canvas-001",
    icon: Palette,
    title: "Canvas 游戏原型",
    desc: "游戏化画布 P1 (弹幕 5 模式 + 大招 + Roguelike, WBS 8.5.3)",
  },
  {
    id: "proto-ipa-kanban",
    href: "/projects?tab=kanban",
    icon: FileCode2,
    title: "IPA 看板原型",
    desc: "任务 IPA 看板 · 4 态泳道 + 拖拽 (per 2026-11-14 页面结构 1b)",
  },
];

export default function PrototypePage() {
  const { t } = useTranslation();
  return (
    <div className="mx-auto max-w-5xl p-6">
      <header className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight text-ink">{t.prototype.title}</h1>
        <p className="mt-1 text-sm text-ink-dim">{t.prototype.subtitle}</p>
      </header>
      <div className="grid gap-4 md:grid-cols-3">
        {PROTOTYPE_ENTRIES.map((entry) => {
          const Icon = entry.icon;
          return (
            <Link
              key={entry.id}
              href={entry.href}
              data-testid={`prototype-entry-${entry.id}`}
              className="group rounded-xl border border-line bg-bg-card p-5 hover:border-accent/60 transition-all"
            >
              <div className="size-10 rounded-lg grid place-items-center border border-violet-500/40 bg-violet-500/15 text-violet-400">
                <Icon size={18} />
              </div>
              <h2 className="mt-3 text-sm font-semibold text-ink">{entry.title}</h2>
              <p className="mt-1 text-xs text-ink-dim leading-relaxed">{entry.desc}</p>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
