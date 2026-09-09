"use client";

// =====================================================================
// /agent-relationships — 3 tab container (per brief v0.50 §2.1 A)
// =====================================================================
// Per docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §1.1
//
// Tabs:
//   1. View      — RelationshipView (图谱浏览)
//   2. Editor    — RelationshipEditor (拖拽建关系 + EdgeTypeSelector)
//   3. Achievements — AchievementWall (20 成就 + 4 稀有度筛选)
// Template Gallery 也作为 View tab 的子区域显示 (per brief §2.1 A 列表).
// =====================================================================

import { useState, useMemo, useCallback, Suspense } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import { Network, Edit3, Trophy, Layers } from "lucide-react";
import { PageHeader } from "@/components/PageHeader";
import { RelationshipView } from "./view/RelationshipView";
import { RelationshipEditor } from "./editor/RelationshipEditor";
import { AchievementWall } from "./achievements/AchievementWall";
import { TemplateGallery } from "./templates/TemplateGallery";

type Tab = "view" | "editor" | "achievements" | "templates";

const TAB_LABEL: Record<Tab, string> = {
  view: "View",
  editor: "Editor",
  achievements: "Achievements",
  templates: "Templates",
};

function AgentRelationshipsContent() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const tabParam = searchParams.get("tab");
  const tab: Tab = useMemo(() => {
    if (tabParam === "editor") return "editor";
    if (tabParam === "achievements") return "achievements";
    if (tabParam === "templates") return "templates";
    return "view";
  }, [tabParam]);

  const setTab = useCallback(
    (next: Tab) => {
      const params = new URLSearchParams(searchParams.toString());
      if (next === "view") params.delete("tab");
      else params.set("tab", next);
      router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    },
    [router, pathname, searchParams],
  );

  return (
    <div className="space-y-3">
      <PageHeader
        title="Agent Relationships"
        subtitle="ARG 视角 · 关系图谱 + 编辑 + 成就墙 + 5 模板库"
        icon={<Network className="text-accent" size={20} />}
        track="C"
      />
      <div
        className="flex items-center gap-2 border-b-2 border-black bg-[var(--cel-surface-card,#0f1422)] px-4 py-2 cel-shadow"
        data-testid="agent-relationships-tabs"
      >
        {(["view", "editor", "achievements", "templates"] as Tab[]).map((t) => (
          <button
            key={t}
            type="button"
            onClick={() => setTab(t)}
            data-testid={`arg-tab-${t}`}
            className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${
              tab === t
                ? "bg-[var(--cel-cyan,#00f0ff)] text-black"
                : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"
            }`}
          >
            {t === "view" && <Network size={13} />}
            {t === "editor" && <Edit3 size={13} />}
            {t === "achievements" && <Trophy size={13} />}
            {t === "templates" && <Layers size={13} />}
            {TAB_LABEL[t]}
          </button>
        ))}
      </div>
      <div data-testid={`arg-tab-panel-${tab}`}>
        {tab === "view" && <RelationshipView />}
        {tab === "editor" && <RelationshipEditor />}
        {tab === "achievements" && <AchievementWall />}
        {tab === "templates" && <TemplateGallery />}
      </div>
    </div>
  );
}

export default function AgentRelationshipsPage() {
  return (
    <Suspense
      fallback={
        <div className="p-8 text-center text-sm font-mono text-ink-dim">
          Loading Agent Relationships…
        </div>
      }
    >
      <AgentRelationshipsContent />
    </Suspense>
  );
}
