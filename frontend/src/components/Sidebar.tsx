/*
@cypher schema=1 source_sha256=5332506987e0fbf7e0397bf81002ebbf46842108e08d08af900183a38709cbaa
MERGE (self:File {path:"frontend/src/components/Sidebar.tsx"})
MERGE (sidebar:Symbol {id:"frontend/src/components/Sidebar.tsx::Sidebar",kind:"component"})
MERGE (guardedLink:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (projectNav:Symbol {id:"frontend/src/components/nav/ProjectNavigation.tsx::ProjectNavigation",kind:"component"})
MERGE (shortcuts:Symbol {id:"frontend/src/components/nav/SidebarShortcuts.tsx::SidebarShortcuts",kind:"component"})
MERGE (projectTree:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (translation:Symbol {id:"frontend/src/lib/i18n/useTranslation.ts::useTranslation",kind:"hook"})
MERGE (self)-[:DEFINES]->(sidebar)
MERGE (self)-[:IMPORTS]->(guardedLink)
MERGE (self)-[:IMPORTS]->(projectNav)
MERGE (self)-[:IMPORTS]->(shortcuts)
MERGE (self)-[:IMPORTS]->(projectTree)
MERGE (self)-[:IMPORTS]->(navStore)
MERGE (self)-[:IMPORTS]->(translation)
MERGE (sidebar)-[:CALLS]->(projectNav)
MERGE (sidebar)-[:CALLS]->(shortcuts)
MERGE (sidebar)-[:CALLS]->(projectTree)
MERGE (sidebar)-[:CALLS]->(navStore)
@endcypher
*/
"use client";

import { clsx } from "clsx";
import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useEffect } from "react";
import { ProjectNavigation } from "@/components/nav/ProjectNavigation";
import { ShortcutButtons } from "@/components/nav/ShortcutButtons";
import { SidebarShortcuts } from "@/components/nav/SidebarShortcuts";
import { GuardedLink } from "@/components/navigation/GuardedLink";
import { ProjectRunTree } from "@/components/run/ProjectRunTree";
import { useTranslation } from "@/lib/i18n";
import { useNavStore } from "@/lib/nav/navStore";

export function Sidebar() {
  const { t } = useTranslation();
  const sidebarFold = useNavStore((s) => s.sidebarFold);
  const toggleSidebarFold = useNavStore((s) => s.toggleSidebarFold);
  const collapsed = sidebarFold === "collapsed";
  const foldLabel = collapsed ? t.sidebar.fold.expand : t.sidebar.fold.collapse;
  const FoldIcon = collapsed ? PanelLeftOpen : PanelLeftClose;

  useEffect(() => {
    function handleFoldShortcut(event: KeyboardEvent) {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "b") {
        event.preventDefault();
        toggleSidebarFold();
      }
    }
    window.addEventListener("keydown", handleFoldShortcut);
    return () => window.removeEventListener("keydown", handleFoldShortcut);
  }, [toggleSidebarFold]);

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.fold = sidebarFold;
    return () => { delete root.dataset.fold; };
  }, [sidebarFold]);

  return (
    <aside
      data-testid="app-sidebar"
      data-fold={sidebarFold}
      data-nav-model="project-lane"
      className={clsx(
        "sticky top-0 z-40 hidden h-dvh shrink-0 flex-col border-r-2 border-[var(--cel-ink,var(--color-border))] bg-[var(--cel-surface-card,var(--color-surface-2))] text-[var(--cel-text-primary,var(--color-text))] transition-[width] md:flex",
        collapsed ? "w-16" : "w-64",
      )}
    >
      <div
        data-testid="sidebar-brand-block"
        className={clsx(
          "flex shrink-0 items-center border-b-2 border-[var(--cel-ink,var(--color-border))]",
          collapsed ? "flex-col gap-2 px-2 py-3" : "h-[76px] justify-between px-3.5 bg-screentone-dense",
        )}
      >
        <GuardedLink
          href="/worktree"
          aria-label={t.ariaLabels.starHome}
          data-testid="sidebar-brand"
          className="group flex min-w-0 items-center gap-2.5 focus-visible:outline focus-visible:outline-accent [perspective:700px]"
        >
          <img
            src="/sidebar-icon.png"
            width={36}
            height={36}
            alt=""
            className="cel-btn-3d size-9 shrink-0 border-2 border-[var(--cel-ink,var(--color-border))] object-cover shadow-[0_3px_0_0_var(--cel-ink)] transition-transform duration-150 group-hover:-translate-y-0.5"
          />
          {!collapsed && (
            <span className="flex min-w-0 flex-col">
              <span className="truncate text-sm font-black tracking-wider leading-tight">STAR</span>
              <span className="font-mono text-[9px] font-bold tracking-widest text-[var(--cel-gold,var(--color-accent))]">CORE // NAV</span>
            </span>
          )}
        </GuardedLink>
        <button
          type="button"
          onClick={toggleSidebarFold}
          title={foldLabel}
          aria-label={foldLabel}
          data-testid="sidebar-fold-toggle"
          className="cel-btn-3d shrink-0 border border-[var(--cel-ink,var(--color-border))] bg-[var(--cel-surface-sub,var(--color-surface))] p-1.5 text-ink-mute hover:border-[var(--cel-cyan,var(--color-primary))] hover:text-[var(--cel-cyan,var(--color-primary))] focus-visible:outline focus-visible:outline-accent"
        >
          <FoldIcon size={16} aria-hidden="true" />
        </button>
      </div>

      <ProjectNavigation directory={<ProjectRunTree collapsed={collapsed} />} collapsed={collapsed} />
      {/* 自定义快捷入口 (per G-6: shortcutButtons 此前无渲染处) */}
      {!collapsed ? <ShortcutButtons className="px-2 pb-1" /> : null}
      <SidebarShortcuts />
    </aside>
  );
}
