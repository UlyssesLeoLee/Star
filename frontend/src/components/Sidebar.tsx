/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/Sidebar.tsx"})
MERGE (guardedLink:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (projectTree:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (worktreeIndex:Symbol {id:"frontend/src/components/worktree-shared/ProjectWorktreeIndexLink.tsx::ProjectWorktreeIndexLink",kind:"component"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (translation:Symbol {id:"frontend/src/lib/i18n/useTranslation.ts::useTranslation",kind:"hook"})
MERGE (projectNavigationLabel:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectNavigation",kind:"property"})
MERGE (self)-[:IMPORTS]->(guardedLink)
MERGE (self)-[:IMPORTS]->(projectTree)
MERGE (self)-[:IMPORTS]->(worktreeIndex)
MERGE (self)-[:IMPORTS]->(navStore)
MERGE (self)-[:IMPORTS]->(translation)
MERGE (self)-[:READS]->(projectNavigationLabel)
@endcypher
*/
"use client";

import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { usePathname, useSearchParams } from "next/navigation";
import {
  X,
  PanelLeftClose,
  PanelLeftOpen,
  ChevronDown,
  Bell,
  MessageSquare,
  Inbox,
} from "lucide-react";
import { clsx } from "clsx";
import { useTheme } from "next-themes";
import { useEffect, useState, useCallback } from "react";
import { useNavStore, type SidebarTabId, type TaskStatusFilter } from "@/lib/nav/navStore";
import { MODULE_MAP, NAV_GROUPS, getCategoryStyles, type ModuleDefinition, type NavGroupId } from "@/lib/nav/registry";
import { moduleHrefMatches } from "@/lib/nav/activeMatch";
import { ProjectRunTree } from "@/components/run/ProjectRunTree";
import { useTranslation, useModuleTranslation } from "@/lib/i18n";

// =====================================================================
// Sidebar — Project-first navigation with Directory / AI Updates tabs.
// Project → Lane → Worktree → Branch is the primary hierarchy.
// =====================================================================

const SIDEBAR_WIDTH = {
  expanded: "w-64",   // 256px
  collapsed: "w-16",  // 64px
} as const;

export function Sidebar() {
  const pathname = usePathname() ?? "/";
  const searchParams = useSearchParams();
  const { theme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const { t, tx } = useTranslation();

  // navStore: 折叠状态 / scope / 数据
  const sidebarFold = useNavStore((s) => s.sidebarFold);
  const toggleSidebarFold = useNavStore((s) => s.toggleSidebarFold);
  const setSidebarFold = useNavStore((s) => s.setSidebarFold);
  const selectedProjectId = useNavStore((s) => s.selectedProjectId);
  const sidebarScope = useNavStore((s) => s.sidebarScope);
  const setSidebarScope = useNavStore((s) => s.setSidebarScope);

  useEffect(() => setMounted(true), []);

  const isDark = !mounted || theme === "dark" || (typeof document !== "undefined" && document.documentElement.classList.contains("dark"));
  const isCollapsed = sidebarFold === "collapsed";

  // =====================================================================
  // Tabs state (Directory / AI Updates) - persisted in navStore
  // =====================================================================
  const sidebarTab = useNavStore((s) => s.sidebarTab);
  const setSidebarTab = useNavStore((s) => s.setSidebarTab);
  const taskStatusFilter = useNavStore((s) => s.taskStatusFilter);
  const setTaskStatusFilter = useNavStore((s) => s.setTaskStatusFilter);

  // Sync activeTab from navStore on mount (could be extended to persist)
  useEffect(() => {
    // initial sync - navStore already has persisted values
  }, []);

  // =====================================================================
  // Module active check
  // =====================================================================
  const isModuleActive = useCallback(
    (item: ModuleDefinition) => moduleHrefMatches(item.href, pathname, searchParams),
    [pathname, searchParams]
  );

  // Keyboard shortcut Ctrl+B
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        toggleSidebarFold();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [toggleSidebarFold]);

  // Publish fold state to <html data-fold>
  useEffect(() => {
    const root = document.documentElement;
    root.dataset.fold = sidebarFold;
    return () => {
      delete root.dataset.fold;
    };
  }, [sidebarFold]);

  // =====================================================================
  // Render Directory tab content (Project tree)
  // =====================================================================
  const renderDirectoryTab = () => (
    <ProjectRunTree collapsed={isCollapsed} />
  );

  // =====================================================================
  // Render AI Updates tab content
  // =====================================================================
  const renderAIUpdatesTab = () => {
    const aiTitle = t.sidebar.aiUpdates.title;
    const empty = t.sidebar.aiUpdates.empty;
    const disconnected = t.sidebar.aiUpdates.disconnected;
    const unauthorized = t.sidebar.aiUpdates.unauthorized;

    // TODO: Replace with real API when available
    // For now show "not connected" state per requirements
    return (
      <div className="flex-1 overflow-y-auto p-3 space-y-3" data-testid="sidebar-ai-updates">
        <div className="text-center text-ink-mute py-8">
          <div className="rounded-full bg-warn/10 border border-warn/30 p-3 w-12 h-12 mx-auto mb-2 flex items-center justify-center">
            <MessageSquare size={20} className="text-warn" />
          </div>
          <p className="text-sm font-medium">{aiTitle}</p>
          <p className="text-[11px] mt-1">{disconnected}</p>
          <p className="text-[11px] text-ink-dim mt-1">{unauthorized}</p>
        </div>
        <div className="border-t border-line/60 pt-3">
          <p className="text-[10px] text-ink-mute text-center">{empty}</p>
        </div>
      </div>
    );
  };

  // =====================================================================
  // Task Status Filter Dropdown
  // =====================================================================
  const renderTaskStatusFilter = () => {
    const label = t.sidebar.taskStatus.label;
    const statuses = [
      { value: "all", label: t.sidebar.taskStatus.all },
      { value: "todo", label: t.sidebar.taskStatus.todo },
      { value: "in_progress", label: t.sidebar.taskStatus.inProgress },
      { value: "blocked", label: t.sidebar.taskStatus.blocked },
      { value: "review", label: t.sidebar.taskStatus.review },
      { value: "done", label: t.sidebar.taskStatus.done },
    ] as const;

    return (
      <div className="relative" data-testid="sidebar-task-status-filter">
        <select
          value={taskStatusFilter}
          onChange={(e) => setTaskStatusFilter(e.target.value as TaskStatusFilter)}
          className="w-full appearance-none rounded border border-line bg-bg px-2 py-1.5 pr-8 text-xs text-ink focus:outline focus:outline-1 focus:outline-accent cursor-pointer"
          aria-label={label}
        >
          {statuses.map((s) => (
            <option key={s.value} value={s.value}>
              {s.label}
            </option>
          ))}
        </select>
        <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 size-3.5 text-ink-mute pointer-events-none" aria-hidden="true" />
        <span className="sr-only">{label}</span>
      </div>
    );
  };

  // =====================================================================
  // Quick Entries (Inbox, Notifications, Chat)
  // =====================================================================
  const renderQuickEntries = () => {
    const entries = [
      { id: "inbox", icon: Inbox, label: t.sidebar.quickEntries.inbox, href: "/inbox" },
      { id: "notifications", icon: Bell, label: t.sidebar.quickEntries.notifications, href: "/notification" },
      { id: "chat", icon: MessageSquare, label: t.sidebar.quickEntries.chat, href: "/inbox?type=chat" },
    ] as const;

    return (
      <div className="space-y-1 pt-2 border-t border-line/60" data-testid="sidebar-quick-entries">
        {entries.map((entry) => (
          <Link
            key={entry.id}
            href={entry.href}
            data-testid={`sidebar-quick-${entry.id}`}
            className={clsx(
              "flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition-colors",
              isCollapsed ? "justify-center" : "gap-2.5",
              "text-ink-dim hover:bg-bg-soft/60 hover:text-ink"
            )}
            title={isCollapsed ? entry.label : undefined}
            aria-label={isCollapsed ? entry.label : undefined}
          >
            <span className="size-8 rounded-lg grid place-items-center border border-transparent text-ink-mute shrink-0">
              <entry.icon size={14} strokeWidth={2} />
            </span>
            {!isCollapsed && <span className="flex-1 truncate font-medium">{entry.label}</span>}
          </Link>
        ))}
      </div>
    );
  };

  return (
    <aside
      data-testid="app-sidebar"
      data-fold={sidebarFold}
      data-nav-model="project-run"
      className={clsx(
        "hidden md:flex shrink-0 border-r border-line bg-bg-card/95 backdrop-blur-xl flex-col h-screen sticky top-0 select-none z-40 transition-all duration-200 ease-out text-ink",
        SIDEBAR_WIDTH[sidebarFold]
      )}
    >
      {/* === Brand Block + Fold Toggle === */}
      <div
        data-testid="sidebar-brand-block"
        className={clsx(
          "border-b border-line shrink-0 flex items-center bg-bg-card",
          isCollapsed ? "justify-center px-2 py-4 flex-col gap-3" : "justify-between px-4 py-4"
        )}
      >
        <Link
          href="/worktree"
          className={clsx("flex items-center group", isCollapsed ? "" : "gap-3")}
          data-testid="sidebar-brand"
          aria-label={t.ariaLabels.starHome}
        >
          <div
            aria-hidden="true"
            className="size-9 overflow-hidden border-2 border-ink shrink-0 transition-transform duration-200 group-hover:scale-105 bg-black"
          >
            <img src="/sidebar-icon.png" width={36} height={36} alt="" className="block w-full h-full object-cover" />
          </div>
          {!isCollapsed && (
            <div className="min-w-0">
              <div className="flex items-center gap-1.5">
                <span className={clsx(
                  "text-base font-black tracking-wider uppercase italic transition-colors",
                  isDark ? "text-white group-hover:text-accent" : "text-ink group-hover:text-accent"
                )}>
                  STAR
                </span>
              </div>
              <div className="text-[11px] font-mono tracking-wider text-ink-mute uppercase font-bold">
                Star — Work Management
              </div>
            </div>
          )}
        </Link>

        {/* Fold toggle button */}
        {isCollapsed ? (
          <button
            type="button"
            onClick={() => setSidebarFold("expanded")}
            title={t.sidebar.fold.expand}
            aria-label={t.sidebar.fold.expand}
            data-testid="sidebar-fold-toggle"
            className="p-1.5 rounded-md text-ink-mute hover:text-accent hover:bg-accent/10 transition-colors"
          >
            <PanelLeftOpen size={14} />
          </button>
        ) : (
          <div className="flex items-center gap-1.5" title={t.ariaLabels.tacticalLinkActive}>
            <button
              type="button"
              onClick={toggleSidebarFold}
              title={t.sidebar.fold.collapse}
              aria-label={t.sidebar.fold.collapse}
              data-testid="sidebar-fold-toggle"
              className="p-1.5 rounded-md text-ink-mute hover:text-accent hover:bg-accent/10 transition-colors"
            >
              <PanelLeftClose size={14} />
            </button>
            <span className="size-2 rounded-full bg-ok animate-pulse shadow-[0_0_10px_rgba(16,185,129,0.9)]" />
          </div>
        )}
      </div>

      {/* === Fixed Top Tabs: Directory | AI Updates === */}
      {!isCollapsed && (
        <div className="border-b border-line/60 bg-bg-card/50 px-2 py-1.5" data-testid="sidebar-top-tabs" role="tablist" aria-label={t.sidebar.projectNavigation}>
          <button
            role="tab"
            aria-selected={sidebarTab === "directory"}
            aria-controls="sidebar-directory-panel"
            id="sidebar-tab-directory"
            onClick={() => setSidebarTab("directory")}
            className={clsx(
              "flex-1 flex items-center justify-center gap-1.5 rounded-lg px-2 py-1.5 text-xs font-medium transition-all",
              sidebarTab === "directory"
                ? "bg-accent/15 text-accent border border-accent/30"
                : "text-ink-dim hover:bg-bg-soft hover:text-ink"
            )}
            data-testid="sidebar-tab-directory"
          >
            <span>{t.sidebar.tabs.directory}</span>
          </button>
          <button
            role="tab"
            aria-selected={sidebarTab === "aiUpdates"}
            aria-controls="sidebar-ai-updates-panel"
            id="sidebar-tab-ai-updates"
            onClick={() => setSidebarTab("aiUpdates")}
            className={clsx(
              "flex-1 flex items-center justify-center gap-1.5 rounded-lg px-2 py-1.5 text-xs font-medium transition-all",
              sidebarTab === "aiUpdates"
                ? "bg-accent/15 text-accent border border-accent/30"
                : "text-ink-dim hover:bg-bg-soft hover:text-ink"
            )}
            data-testid="sidebar-tab-ai-updates"
          >
            <span>{t.sidebar.tabs.aiUpdates}</span>
          </button>
        </div>
      )}

      {/* === Middle Area (scrollable) === */}
      <nav
        className={clsx(
          "flex-1 overflow-y-auto text-sm scrollbar-none",
          isCollapsed ? "px-2 py-3 space-y-2" : "px-3 py-4 space-y-5"
        )}
        aria-label={t.sidebar.projectNavigation}
        role="tabpanel"
        aria-labelledby={sidebarTab === "directory" ? "sidebar-tab-directory" : "sidebar-tab-ai-updates"}
      >
        <div
          id="sidebar-directory-panel"
          data-testid="sidebar-directory-panel"
          className={sidebarTab === "directory" ? "block" : "hidden"}
        >
          {renderDirectoryTab()}
        </div>
        <div
          id="sidebar-ai-updates-panel"
          data-testid="sidebar-ai-updates-panel"
          className={sidebarTab === "aiUpdates" ? "block" : "hidden"}
        >
          {renderAIUpdatesTab()}
        </div>

        {/* Global section at bottom (collapsed into quick entries in new design) */}
        {!isCollapsed && sidebarTab === "directory" && (
          <div className="mt-auto border-t border-line/60 pt-3" data-testid="sidebar-group-global">
            <div className="px-2.5 py-1 text-[10px] font-mono uppercase tracking-wider text-ink-mute">
              {t.sidebar.navGlobal}
            </div>
            <ul className="mt-1.5 space-y-1">
              {NAV_GROUPS["home-personal"].map((id) => {
                const item = MODULE_MAP.get(id);
                if (!item) return null;
                const active = isModuleActive(item);
                const Icon = item.icon;
                return (
                  <li key={item.id} className="relative">
                    <Link
                      href={item.href}
                      data-testid={`sidebar-item-${item.id}`}
                      data-active={active ? "true" : "false"}
                      aria-current={active ? "page" : undefined}
                      className={clsx(
                        "flex items-center rounded-lg transition-colors",
                        "gap-2.5 px-2.5 py-2",
                        active
                          ? "bg-accent/10 text-accent font-semibold"
                          : "text-ink-dim hover:bg-bg-soft/60 hover:text-ink"
                      )}
                    >
                      {active && <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 rounded bg-accent" aria-hidden="true" />}
                      <span
                        className={clsx(
                          "rounded-md grid place-items-center shrink-0 border",
                          "size-7",
                          active ? "bg-accent/15 border-accent/50 text-accent" : "border-transparent text-ink-mute"
                        )}
                      >
                        <Icon size={13} />
                      </span>
                      <span className="flex-1 truncate tracking-tight">{useModuleTranslation(item).label}</span>
                      {item.count !== undefined && item.count > 0 && (
                        <span className="min-w-[18px] h-4.5 rounded-full bg-err text-white text-[10px] font-mono grid place-items-center px-1 font-bold">
                          {item.count}
                        </span>
                      )}
                    </Link>
                  </li>
                );
              })}
            </ul>
          </div>
        )}
      </nav>

      {/* === Fixed Bottom: Task Status Filter + Quick Entries === */}
      {!isCollapsed && (
        <div className="border-t border-line shrink-0 bg-bg-card/50 p-3 space-y-3" data-testid="sidebar-bottom-fixed" role="region" aria-label={t.sidebar.taskStatus.label}>
          {sidebarTab === "directory" && (
            <>
              <div data-testid="sidebar-task-status-wrapper">
                <label htmlFor="sidebar-task-status-select" className="block text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1">
                  {t.sidebar.taskStatus.label}
                </label>
                {renderTaskStatusFilter()}
              </div>
              {renderQuickEntries()}
            </>
          )}
          {sidebarTab === "aiUpdates" && (
            <div data-testid="sidebar-ai-updates-bottom" className="text-center text-[11px] text-ink-mute">
              {t.sidebar.aiUpdates.empty}
            </div>
          )}
        </div>
      )}

      {/* Collapsed mode: show quick entries at bottom */}
      {isCollapsed && (
        <div className="mt-auto border-t border-line/60 pt-2 pb-3 px-2 space-y-1.5" data-testid="sidebar-collapsed-quick-entries">
          {[
            { id: "inbox", icon: Inbox, label: t.sidebar.quickEntries.inbox, href: "/inbox" },
            { id: "notifications", icon: Bell, label: t.sidebar.quickEntries.notifications, href: "/notification" },
            { id: "chat", icon: MessageSquare, label: t.sidebar.quickEntries.chat, href: "/inbox?type=chat" },
          ].map((entry) => (
            <Link
              key={entry.id}
              href={entry.href}
              data-testid={`sidebar-collapsed-quick-${entry.id}`}
              title={entry.label}
              aria-label={entry.label}
              className="flex items-center justify-center p-1.5 rounded-lg text-ink-dim hover:bg-bg-soft/60 hover:text-ink transition-colors"
            >
              <span className="size-8 rounded-lg grid place-items-center border border-transparent text-ink-mute">
                <entry.icon size={16} strokeWidth={2} />
              </span>
            </Link>
          ))}
        </div>
      )}
    </aside>
  );
}

// =====================================================================
// NavGroupSection / NavModuleList — Sidebar 5 组结构渲染
// (per ui-3pane-arch §1.2 v0.2, 2026-11-13 用户发令)
// =====================================================================

interface NavModuleListProps {
  modules: ModuleDefinition[];
  collapsed: boolean;
  isActive: (m: ModuleDefinition) => boolean;
  showCoreBadge?: boolean;
}

function NavModuleList({ modules, collapsed, isActive, showCoreBadge }: NavModuleListProps) {
  return (
    <ul className={clsx(collapsed ? "space-y-1.5" : "mt-1.5 space-y-1")}>
      {modules.map((item) => (
        <NavModuleRow
          key={item.id}
          item={item}
          active={isActive(item)}
          collapsed={collapsed}
          showCoreBadge={showCoreBadge}
        />
      ))}
    </ul>
  );
}

function NavModuleRow({
  item,
  active,
  collapsed,
  showCoreBadge,
}: {
  item: ModuleDefinition;
  active: boolean;
  collapsed: boolean;
  showCoreBadge?: boolean;
}) {
  const mod = useModuleTranslation(item);
  const Icon = item.icon;
  const cs = getCategoryStyles(item.category);
  return (
    <li>
      <Link
        href={item.href}
        data-testid={`sidebar-item-${item.id}`}
        data-active={active ? "true" : "false"}
        aria-current={active ? "page" : undefined}
        className={clsx(
          "relative flex items-center rounded-xl text-sm font-medium transition-all duration-200",
          collapsed ? "justify-center p-1.5" : "gap-2.5 px-2.5 py-1.5",
          active
            ? "bg-bg-soft/80 text-ink border border-line shadow-soft font-semibold"
            : "text-ink-dim hover:bg-bg-soft/60 hover:text-ink border border-transparent"
        )}
      >
        <div
          className={clsx(
            "rounded-lg grid place-items-center shrink-0 border transition-all duration-200",
            collapsed ? "size-9" : "size-8",
            cs.bg,
            cs.border,
            cs.text,
            active && clsx(cs.bgActive, cs.borderActive, cs.glow)
          )}
          aria-hidden="true"
        >
          <Icon size={collapsed ? 16 : 15} strokeWidth={2.25} />
        </div>
        {!collapsed && (
          <>
            <span className="flex-1 truncate tracking-tight">{mod.label}</span>
            {/* core 徽标 (per §1.2 核心区 visual treatment) */}
            {showCoreBadge && (
              <span
                data-testid={`sidebar-core-badge-${item.id}`}
                className="text-[8px] font-mono uppercase px-1 py-0.5 rounded border border-accent/50 bg-accent/10 text-accent"
              >
                core
              </span>
            )}
            {item.count !== undefined && item.count > 0 && (
              <span className="text-[10px] font-mono text-ink-mute px-1.5 py-0.5 rounded border border-line/50 bg-bg/40">
                {item.count}
              </span>
            )}
            {active && <span className={clsx("size-1.5 rounded-full animate-pulse", cs.dot)} aria-hidden="true" />}
          </>
        )}
        {collapsed && active && (
          <span className={clsx("absolute top-0.5 right-0.5 size-1.5 rounded-full", cs.dot)} aria-hidden="true" />
        )}
      </Link>
    </li>
  );
}

interface NavGroupSectionProps {
  testId: string;
  label: string;
  modules: ModuleDefinition[];
  collapsed: boolean;
  isActive: (m: ModuleDefinition) => boolean;
  showCoreBadge?: boolean;
  /** 二级子组: 小号 label, 无顶部边框 */
  sub?: boolean;
}

function NavGroupSection({ testId, label, modules, collapsed, isActive, showCoreBadge, sub }: NavGroupSectionProps) {
  return (
    <section data-testid={testId} className={clsx(!sub && "border-t border-line/60 pt-3")}>
      {!collapsed && (
        <div
          className={clsx(
            "px-2.5 py-1 font-mono uppercase tracking-wider text-ink-mute",
            sub ? "text-[9px]" : "text-[10px] font-bold"
          )}
        >
          {label}
        </div>
      )}
      <NavModuleList modules={modules} collapsed={collapsed} isActive={isActive} showCoreBadge={showCoreBadge} />
    </section>
  );
}

// =====================================================================
// EmptyProjectState — Project scope 不可用时的占位 (per 拍板 #1 派生)
// =====================================================================
function EmptyProjectState({ collapsed }: { collapsed: boolean }) {
  const { t } = useTranslation();
  return (
    <div
      data-testid="sidebar-subnav-empty"
      className={clsx(
        "text-[10px] font-mono text-ink-mute/70",
        collapsed ? "px-1 text-center" : "px-3 py-4 text-center"
      )}
    >
      {collapsed ? "—" : t.sidebar.scope.projectDisabledHint}
    </div>
  );
}

// =====================================================================
// SidebarRow — 单行 nav 渲染 (per 2026-08-31 i18n 补缺口)
// =====================================================================
// 抽出来是为了在子组件内合法调用 useModuleTranslation (hook 不能在
// 父组件 .map() 回调里直接调, 否则会破坏 hooks 规则).
//
// data-testid 用 registry 静态 id 生成 (per 2026-09-05 19:13 JST 拍板: label 改 Sprint 后 testid 仍稳定)
// (sidebar-item-inbox / sidebar-item-issues / sidebar-item-projects / ...) 不受语言切换影响.
//
// 折叠态: 只渲染 icon tile + hover tooltip, 隐藏 label/code/count/active dot/remove btn
// =====================================================================
interface SidebarRowProps {
  module: ModuleDefinition;
  active: boolean;
  onRemove: (id: string) => void;
  removeTitle: string;
  dataTestIdBase: string;
  removeDataTestIdBase: string;
  collapsed: boolean;
}

function SidebarRow({
  module: item,
  active,
  onRemove,
  removeTitle,
  dataTestIdBase,
  removeDataTestIdBase,
  collapsed,
}: SidebarRowProps) {
  const mod = useModuleTranslation(item);
  const { tx } = useTranslation();
  const Icon = item.icon;
  // 域分色 icon tile (5 域色, per 2026-09-02 15:42 JST 拍板)
  const cs = getCategoryStyles(item.category);
  // 用 registry 静态 id 生成 testid (per 2026-09-05 19:13 JST: label 改 Sprint 后 testid 仍稳定, 不会随 i18n 漂移)
  const testIdSlug = item.id;
  const tileSize = collapsed ? "size-9" : "size-8";

  // 折叠态: 紧凑 tile, hover tooltip 显示 label
  if (collapsed) {
    return (
      <li className="relative group">
        <Link
          href={item.href}
          data-testid={`${dataTestIdBase}-${testIdSlug}`}
          data-active={active ? "true" : "false"}
          aria-current={active ? "page" : undefined}
          title={mod.label}
          aria-label={mod.label}
          className={clsx(
            "relative flex items-center justify-center p-1.5 rounded-xl transition-all duration-200",
            active
              ? "bg-bg-soft/80 text-ink border border-line shadow-soft"
              : "text-ink-dim hover:bg-bg-soft/60 hover:text-ink border border-transparent"
          )}
        >
          <div
            data-testid={`${dataTestIdBase}-icon-tile-${item.id}`}
            className={clsx(
              "rounded-lg grid place-items-center shrink-0 border transition-all duration-200 group-hover:scale-105",
              tileSize,
              cs.bg,
              cs.border,
              cs.text,
              active && clsx(cs.bgActive, cs.borderActive, cs.glow)
            )}
            aria-hidden="true"
          >
            <Icon size={16} strokeWidth={2.25} />
          </div>
          {active && (
            <span
              data-testid={`${dataTestIdBase}-active-dot-${item.id}`}
              className={clsx("absolute top-0.5 right-0.5 size-1.5 rounded-full", cs.dot)}
              aria-hidden="true"
            />
          )}
        </Link>
      </li>
    );
  }

  return (
    <li className="relative group">
      <Link
        href={item.href}
        data-testid={`${dataTestIdBase}-${testIdSlug}`}
        className={clsx(
          "relative flex items-center gap-2.5 px-2.5 py-1.5 rounded-xl text-sm font-medium transition-all duration-200",
          active
            ? "bg-bg-soft/80 text-ink border border-line shadow-soft font-semibold"
            : "text-ink-dim hover:bg-bg-soft/60 hover:text-ink border border-transparent"
        )}
      >
        {/* 域分色 icon tile — 8x8 圆角色块底 + line icon */}
        <div
          data-testid={`${dataTestIdBase}-icon-tile-${item.id}`}
          className={clsx(
            "rounded-lg grid place-items-center shrink-0 border transition-all duration-200 group-hover:scale-105",
            tileSize,
            cs.bg,
            cs.border,
            cs.text,
            active && clsx(cs.bgActive, cs.borderActive, cs.glow)
          )}
          aria-hidden="true"
        >
          <Icon
            size={15}
            strokeWidth={2.25}
            className="transition-transform duration-200"
          />
        </div>

        <span className="flex-1 truncate tracking-tight">{mod.label}</span>

        <span className="text-[10px] font-mono text-ink-mute px-1.5 py-0.5 rounded border border-line/50 bg-bg/40 opacity-70 group-hover:opacity-100 transition-opacity font-bold">
          {item.code}
        </span>

        {item.count !== undefined && item.count > 0 && (
          <span className="min-w-[18px] h-4.5 rounded-full bg-err text-white text-[10px] font-mono grid place-items-center px-1 shadow-[0_0_8px_rgba(255,51,102,0.7)] font-bold">
            {item.count}
          </span>
        )}

        {active && (
          <span
            data-testid={`${dataTestIdBase}-active-dot-${item.id}`}
            className={clsx("size-1.5 rounded-full animate-pulse", cs.dot)}
            aria-hidden="true"
          />
        )}
      </Link>

      <button
        type="button"
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          onRemove(item.id);
        }}
        title={tx(removeTitle, { label: mod.label })}
        data-testid={`${removeDataTestIdBase}-${item.id}`}
        className="absolute right-1.5 top-1/2 -translate-y-1/2 p-1 rounded-md hover:bg-err/20 hover:text-err text-ink-mute opacity-0 group-hover:opacity-100 transition-all duration-150"
      >
        <X size={11} />
      </button>
    </li>
  );
}