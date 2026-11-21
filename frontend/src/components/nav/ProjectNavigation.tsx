/*
@cypher schema=1 source_sha256=7322547e9e5278247238e09a0be3631df3ad4748fe469436b4aa122bea250eba
MERGE (self:File {path:"frontend/src/components/nav/ProjectNavigation.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/nav/ProjectNavigation.tsx::ProjectNavigation",kind:"component"})
MERGE (state:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (translation:Symbol {id:"frontend/src/lib/i18n/useTranslation.ts::useTranslation",kind:"hook"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:IMPORTS]->(state)
MERGE (self)-[:IMPORTS]->(translation)
MERGE (panel)-[:CALLS]->(state)
MERGE (panel)-[:CALLS]->(translation)
MERGE (keyboard:Symbol {id:"frontend/src/components/nav/ProjectNavigation.tsx::moveTabFocus",kind:"function"})
MERGE (panel)-[:DEFINES]->(keyboard)
@endcypher
*/
"use client";

import { clsx } from "clsx";
import { useId, type KeyboardEvent, type ReactNode } from "react";
import { useTranslation } from "@/lib/i18n";
import { useNavStore, type SidebarTabId, type TaskStatusFilter } from "@/lib/nav/navStore";

export function ProjectNavigation({
  directory,
  updates,
  collapsed = false,
  testIdPrefix = "sidebar",
}: {
  directory: ReactNode;
  updates?: ReactNode;
  collapsed?: boolean;
  testIdPrefix?: string;
}) {
  const { t } = useTranslation();
  const id = useId();
  const tab = useNavStore((s) => s.sidebarTab);
  const setTab = useNavStore((s) => s.setSidebarTab);
  const filter = useNavStore((s) => s.taskStatusFilter);
  const setFilter = useNavStore((s) => s.setTaskStatusFilter);
  const tabs = [
    { value: "directory", label: t.sidebar.tabs.directory, slug: "directory" },
    { value: "aiUpdates", label: t.sidebar.tabs.aiUpdates, slug: "ai-updates" },
  ] as const;
  const statuses: { value: TaskStatusFilter; label: string }[] = [
    { value: "all", label: t.sidebar.taskStatus.all },
    { value: "todo", label: t.sidebar.taskStatus.todo },
    { value: "in_progress", label: t.sidebar.taskStatus.inProgress },
    { value: "review", label: t.sidebar.taskStatus.review },
    { value: "blocked", label: t.sidebar.taskStatus.blocked },
    { value: "done", label: t.sidebar.taskStatus.done },
  ];
  const visibleTab = collapsed ? "directory" : tab;

  function moveTabFocus(event: KeyboardEvent<HTMLDivElement>) {
    let next: SidebarTabId;
    switch (event.key) {
      case "Home": next = "directory"; break;
      case "End": next = "aiUpdates"; break;
      case "ArrowLeft":
      case "ArrowRight": next = tab === "directory" ? "aiUpdates" : "directory"; break;
      default: return;
    }
    event.preventDefault();
    setTab(next);
    event.currentTarget.querySelector<HTMLButtonElement>(`[data-view="${next}"]`)?.focus();
  }

  return (
    <section className="flex min-h-0 flex-1 flex-col" aria-label={t.sidebar.projectNavigation}>
      {!collapsed && (
        <div
          className="grid shrink-0 grid-cols-2 gap-1 border-b border-line px-2 py-1.5"
          role="tablist"
          aria-label={t.sidebar.projectNavigation}
          data-testid={`${testIdPrefix}-top-tabs`}
          onKeyDown={moveTabFocus}
        >
          {tabs.map(({ value, label, slug }) => (
            <button
              key={value}
              type="button"
              role="tab"
              id={`${id}-${slug}-tab`}
              aria-selected={tab === value}
              aria-controls={`${id}-${slug}-panel`}
              tabIndex={tab === value ? 0 : -1}
              data-view={value}
              data-testid={`${testIdPrefix}-tab-${slug}`}
              onClick={() => setTab(value)}
              className={clsx(
                "rounded-md border px-2 py-1.5 text-xs font-medium focus-visible:outline focus-visible:outline-accent",
                tab === value ? "border-accent/30 bg-accent/15 text-accent" : "border-transparent text-ink-dim hover:bg-bg-soft hover:text-ink",
              )}
            >
              {label}
            </button>
          ))}
        </div>
      )}

      <div
        id={`${id}-directory-panel`}
        role={collapsed ? undefined : "tabpanel"}
        aria-labelledby={collapsed ? undefined : `${id}-directory-tab`}
        data-testid={`${testIdPrefix}-directory-panel`}
        hidden={visibleTab !== "directory"}
        className={clsx("flex min-h-0 flex-1 flex-col overflow-y-auto text-sm", collapsed ? "p-2" : "px-2 py-3")}
      >
        {directory}
      </div>
      {!collapsed && (
        <div
          id={`${id}-ai-updates-panel`}
          role="tabpanel"
          aria-labelledby={`${id}-ai-updates-tab`}
          data-testid={`${testIdPrefix}-ai-updates-panel`}
          hidden={visibleTab !== "aiUpdates"}
          className="min-h-0 flex-1 overflow-y-auto px-3 py-3 text-sm"
        >
          {updates ?? <p className="py-3 text-xs text-ink-mute" role="status">{t.sidebar.aiUpdates.disconnected}</p>}
        </div>
      )}

      {!collapsed && (
        <div className="shrink-0 border-t border-line p-2" data-testid={`${testIdPrefix}-bottom-fixed`}>
          <div className="mb-1.5 px-1 text-[11px] font-medium text-ink-mute">{t.sidebar.taskStatus.label}</div>
          <div className="grid grid-cols-3 gap-1" role="group" aria-label={t.sidebar.taskStatus.label} data-testid={`${testIdPrefix}-task-status-filter`}>
            {statuses.map(({ value, label }) => (
              <button
                key={value}
                type="button"
                aria-pressed={filter === value}
                aria-controls={`${id}-${visibleTab === "directory" ? "directory" : "ai-updates"}-panel`}
                data-testid={`${testIdPrefix}-status-${value}`}
                onClick={() => setFilter(value)}
                className={clsx(
                  "min-w-0 rounded border px-1 py-1.5 text-[11px] font-medium leading-tight focus-visible:outline focus-visible:outline-accent",
                  filter === value ? "border-accent/40 bg-accent/10 text-accent" : "border-transparent text-ink-mute hover:bg-bg-soft hover:text-ink",
                )}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
