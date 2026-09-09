"use client";

// =====================================================================
// AgentViewTab — /agent-view Relationship 视角集成 (per brief v0.50 §2.1 A)
// =====================================================================
// Per docs/design/BD-AGENT-RELATIONSHIP-001.md v0.1 §1.1.4
// + docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §1.1
//
// /agent-view 页面加 1 tab "Relationship" 切到 ARG 视角:
//   - 通过 URL 跳转 (per existing agent-view /agent-view?view=... pattern)
//   - 跟既有 viewMode (canvas / settings) 共存; 加 viewMode="relationship"
// =====================================================================

import { useEffect, useState, Suspense } from "react";
import { useRouter, useSearchParams, usePathname } from "next/navigation";
import { Network } from "lucide-react";
import { useARGStore } from "@/lib/arg/store";
import { RelationshipView } from "./view/RelationshipView";
import { AchievementWall } from "./achievements/AchievementWall";

export type RelationshipSubView = "view" | "achievements";

function AgentRelationshipTabContent() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const subParam = searchParams.get("argView");
  const subView: RelationshipSubView =
    subParam === "achievements" ? "achievements" : "view";
  const [hydrated, setHydrated] = useState(false);
  useEffect(() => setHydrated(true), []);

  const setSubView = (next: RelationshipSubView) => {
    const params = new URLSearchParams(searchParams.toString());
    if (next === "view") params.delete("argView");
    else params.set("argView", next);
    router.replace(`${pathname}?${params.toString()}`, { scroll: false });
  };

  const agentCount = useARGStore((s) => s.agents.size);
  const edgeCount = useARGStore((s) => s.edges.size);

  return (
    <div className="space-y-3" data-testid="agent-relationships-tab-panel">
      <div className="flex items-center gap-2 text-xs font-mono">
        <button
          type="button"
          onClick={() => setSubView("view")}
          data-testid="arg-subview-view"
          className={`pill border px-2 py-0.5 ${
            subView === "view"
              ? "border-accent text-accent bg-accent/10"
              : "border-line text-ink-dim"
          }`}
        >
          <Network size={11} className="inline mr-1" />
          Relationship Graph
        </button>
        <button
          type="button"
          onClick={() => setSubView("achievements")}
          data-testid="arg-subview-achievements"
          className={`pill border px-2 py-0.5 ${
            subView === "achievements"
              ? "border-accent text-accent bg-accent/10"
              : "border-line text-ink-dim"
          }`}
        >
          Achievements
        </button>
        <div className="ml-auto text-ink-dim" data-testid="arg-tab-counts">
          {hydrated ? `${agentCount} agents · ${edgeCount} edges` : "—"}
        </div>
      </div>
      {subView === "view" ? <RelationshipView /> : <AchievementWall />}
    </div>
  );
}

export function AgentViewTab() {
  return (
    <Suspense
      fallback={
        <div className="p-4 text-xs font-mono text-ink-dim">
          Loading ARG tab…
        </div>
      }
    >
      <AgentRelationshipTabContent />
    </Suspense>
  );
}
