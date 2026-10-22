/*
@cypher schema=1 source_sha256=3e9bb7d698741beaae4e81f118695e97606b43d0576240b254c3f0e69757090c
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
  Plus,
  X,
  PanelLeftClose,
  PanelLeftOpen,
  FolderKanban,
} from "lucide-react";
import { clsx } from "clsx";
import { useTheme } from "next-themes";
import { useEffect, useMemo, useState } from "react";
import { useNavStore } from "@/lib/nav/navStore";
import { MODULE_MAP, getCategoryStyles, type ModuleDefinition, type ModuleCategory } from "@/lib/nav/registry";
import {
  findSubNavGroup,
  findActiveSubNavItem,
  type SubNavEntry,
} from "@/lib/nav/subNavRegistry";
import { useTranslation, useModuleTranslation } from "@/lib/i18n";
import { ProjectWorktreeIndexLink } from "@/components/worktree-shared/ProjectWorktreeIndexLink";
import { ProjectRunTree } from "@/components/run/ProjectRunTree";
import { useRunDirectory } from "@/lib/run/runDirectorySession";

// =====================================================================
// Sidebar — Project-first navigation.
// Project → Engineering Run → Worktree → Branch is the primary hierarchy.
// Existing pages remain reachable in a secondary, collapsible section.
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
  const sidebarItemIds = useNavStore((s) => s.sidebarItemIds);
  const removeSidebarItem = useNavStore((s) => s.removeSidebarItem);
  const pinnedViewIds = useNavStore((s) => s.pinnedViewIds);
  const removePinnedView = useNavStore((s) => s.removePinnedView);
  const openMatrix = useNavStore((s) => s.openMatrix);
  const sidebarFold = useNavStore((s) => s.sidebarFold);
  const toggleSidebarFold = useNavStore((s) => s.toggleSidebarFold);
  const setSidebarFold = useNavStore((s) => s.setSidebarFold);
  const selectedProjectId = useNavStore((s) => s.selectedProjectId);
  const { focus: runFocus } = useRunDirectory();
  const projectIndexHint = runFocus?.run_context.project_id ?? selectedProjectId;

  useEffect(() => setMounted(true), []);

  const isDark = !mounted || theme === "dark" || (typeof document !== "undefined" && document.documentElement.classList.contains("dark"));
  const isCollapsed = sidebarFold === "collapsed";

  // 解析当前用户钉选到左侧的模块对象
  const activeSidebarModules = sidebarItemIds
    .map((id) => MODULE_MAP.get(id))
    .filter((m): m is ModuleDefinition => Boolean(m));

  const activePinnedModules = pinnedViewIds
    .map((id) => MODULE_MAP.get(id))
    .filter((m): m is ModuleDefinition => Boolean(m));

  // =====================================================================
  // Project scope 数据源 (per 2026-09-03 12:36 JST 拍板 #1 复用 SubNav 数据源)
  // =====================================================================
  // - 从 subNavRegistry 按 pathname 查 group
  // - 若未命中, project scope 不可用 (灰显 + tooltip 提示)
  // - active item 由 query 字符串决定 (e.g. /projects?tab=kanban → kanban)
  const subNavGroup = useMemo(() => findSubNavGroup(pathname), [pathname]);
  const searchString = useMemo(() => {
    const s = searchParams?.toString();
    return s ? `?${s}` : "";
  }, [searchParams]);
  const activeSubNavId = useMemo(
    () => (subNavGroup ? findActiveSubNavItem(subNavGroup, searchString) : null),
    [subNavGroup, searchString]
  );
  // Project scope stays available in the Project workspace and the Project Worktree Index.
  // A validated Run focus supplies the route hint; the Index still reloads its authorized directory.
  const hasSelectedProject = projectIndexHint.trim().length > 0;
  const isProjectWorkspacePath = pathname === "/projects" || pathname.startsWith("/projects/");
  const isProjectWorktreePath = pathname === "/worktree" || pathname.startsWith("/worktree/");
  const isOtherEntryActive =
    activeSidebarModules.some((item) => pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href))) ||
    activePinnedModules.some((item) => pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href)));

  // 键盘快捷键 Ctrl+B 折叠/展开 (per 守门 #11 缺标比错标: shortcut 是 power user
  // 必备延伸, 不强制; 这里落地)
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // Ctrl+B / Cmd+B
      if ((e.ctrlKey || e.metaKey) && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        toggleSidebarFold();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [toggleSidebarFold]);

  // 发布折叠态到 <html data-fold> — CSS 变量 --sidebar-w 的单一驱动源
  // (per INV-LAYOUT-03: 所有 fixed 全宽 dock 读 var(--sidebar-w)，禁止硬编码 px/rem)。
  // 不用 :root[data-fold=...] 是因为 Tailwind v4 的 cascade 顺序无法保证，
  // 挂在 <html> 上语义最直白，devtools 里也一眼能看到当前宽度状态。
  useEffect(() => {
    const root = document.documentElement;
    root.dataset.fold = sidebarFold;
    return () => {
      delete root.dataset.fold;
    };
  }, [sidebarFold]);

  return (
    <aside
      data-testid="app-sidebar"
      data-fold={sidebarFold}
      data-nav-model="project-run"
      // 桌面: 256/64 sticky; <768px: 隐藏 (由 MobileBottomNav + AppMatrixDrawer 抽屉替代, per 2026-09-01 PHASE-MOBILE-PWA)
      // z-40: 侧栏是结构性骨架层, 必须高于所有内容级 dock (L3 z-[35])
      // (per INV-LAYOUT-01/04: 修复方式是提高 Sidebar 层级, 不是降低 dock 层级)
      className={clsx(
        "hidden md:flex shrink-0 border-r-2 border-[var(--cel-ink,var(--color-border))] bg-[var(--cel-surface-card,var(--color-surface-2))]/95 cel-shadow backdrop-blur-xl flex-col h-screen sticky top-0 select-none z-40 transition-all duration-200 ease-out text-[var(--cel-text-primary,var(--color-text))]",
        SIDEBAR_WIDTH[sidebarFold]
      )}
    >
      {/* === Star Anime Cyber Crest (Brand Block) + Fold Toggle === */}
      <div
        data-testid="sidebar-brand-block"
        className={clsx(
          "border-b-2 border-[var(--cel-ink,var(--color-border))] shrink-0 flex items-center bg-[var(--cel-surface-card,var(--color-surface-2))]",
          isCollapsed ? "justify-center px-2 py-4 flex-col gap-3" : "justify-between px-4 py-4"
        )}
      >
        <Link
          href="/projects"
          className={clsx(
            "flex items-center group",
            isCollapsed ? "" : "gap-3"
          )}
          data-testid="sidebar-brand"
          aria-label={t.ariaLabels.starHome}
        >
          <div
            aria-hidden="true"
            className="size-9 overflow-hidden border-2 border-[var(--cel-ink,var(--color-border))] cel-shadow shrink-0 transition-transform duration-200 group-hover:scale-105 bg-black"
          >
            <img
              src="/sidebar-icon.png"
              width={36}
              height={36}
              alt=""
              className="block w-full h-full object-cover"
            />
          </div>
          {!isCollapsed && (
            <div className="min-w-0">
              <div className="flex items-center gap-1.5">
                <span className={clsx(
                  "text-base font-black tracking-wider uppercase italic transition-colors",
                  isDark ? "text-white group-hover:text-[var(--cel-cyan,#00f0ff)]" : "text-[var(--cel-text-primary,#0d1117)] group-hover:text-[var(--cel-crimson,#e8295a)]"
                )}>
                  STAR
                </span>
                <span className="font-mono text-[10px] font-black px-1.5 py-0.5 bg-[var(--cel-crimson,var(--color-secondary,#ff184c))] text-[var(--color-on-primary,#ffffff)] border border-[var(--cel-ink,var(--color-border))] italic">
                  CEL-NPR
                </span>
              </div>
              <div className="text-[11px] font-mono tracking-wider text-[var(--cel-text-secondary,#94a3b8)] uppercase font-bold">
                時の限界を打ち砕け！！
              </div>
            </div>
          )}
        </Link>

        {/* Fold toggle button — 始终可见, 折叠态下显示在 icon 下方 */}
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

      {/* === Scope Toggle (Main / Project) — 仅展开态可见 === */}
      {/* per 2026-09-03 12:36 JST 拍板 #2: 顶部 brand block 下方 */}
      {/* === Navigation Groups === */}
      <nav
        className={clsx(
          "flex-1 overflow-y-auto text-sm scrollbar-none",
          isCollapsed ? "px-2 py-3 space-y-2" : "px-3 py-4 space-y-5"
        )}
        aria-label={t.sidebar.projectNavigation}
      >
        <ProjectRunTree collapsed={isCollapsed} />
        {isProjectWorkspacePath && subNavGroup && (
          <SubNavGroupList
            group={subNavGroup}
            activeId={activeSubNavId}
            collapsed={isCollapsed}
          />
        )}
        {(isProjectWorkspacePath || hasSelectedProject || isProjectWorktreePath) && (
          <ProjectWorktreeIndexLink
            projectId={projectIndexHint}
            active={pathname === "/worktree"}
            collapsed={isCollapsed}
          />
        )}
        {!isProjectWorkspacePath && !hasSelectedProject && !isProjectWorktreePath && (
          <EmptyProjectState collapsed={isCollapsed} />
        )}

        <details
          data-testid="sidebar-other-entries"
          className="border-t border-line/60 pt-3"
          open={isOtherEntryActive}
        >
          <summary className="flex cursor-pointer list-none items-center justify-between gap-2 rounded px-2 py-2 text-xs font-semibold text-ink-mute hover:bg-bg-soft/60 hover:text-ink focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent">
            <span className="flex min-w-0 items-center gap-2">
              <FolderKanban size={15} aria-hidden="true" />
              <span className={clsx(isCollapsed && "sr-only")}>{t.sidebar.otherPages}</span>
            </span>
            <span className={clsx("text-[10px] font-mono", isCollapsed && "sr-only")}>
              {activeSidebarModules.length + activePinnedModules.length}
            </span>
          </summary>
          <div className="mt-3 space-y-5">
            <div>
              {!isCollapsed && (
                <div className="px-2.5 py-1 flex items-center justify-between text-[10px] font-mono uppercase tracking-wider text-ink-mute">
                  <span>{t.sidebar.groupWorkspaces}</span>
                  <span className="text-[9px] font-mono opacity-70 border border-line px-1.5 py-0.2 rounded">
                    {activeSidebarModules.length} {t.sidebar.pinned}
                  </span>
                </div>
              )}
              <ul className={clsx(isCollapsed ? "space-y-1.5" : "mt-1.5 space-y-1")}>
                {activeSidebarModules.map((item) => (
                  <SidebarRow
                    key={item.id}
                    module={item}
                    active={pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href))}
                    onRemove={removeSidebarItem}
                    removeTitle={tx(t.sidebar.removeFromSidebar, { label: "" })}
                    dataTestIdBase="sidebar-item"
                    removeDataTestIdBase="remove-sidebar"
                    collapsed={isCollapsed}
                  />
                ))}
              </ul>
            </div>

            {activePinnedModules.length > 0 && (
              <div>
                {!isCollapsed && (
                  <div className="px-2.5 py-1 flex items-center justify-between text-[10px] font-mono uppercase tracking-wider text-ink-mute">
                    <span>{t.sidebar.groupTactical}</span>
                    <span className="text-[9px] font-mono opacity-70 border border-line px-1.5 py-0.2 rounded">{t.sidebar.pinned}</span>
                  </div>
                )}
                <ul className={clsx(isCollapsed ? "space-y-1.5" : "mt-1.5 space-y-1")}>
                  {activePinnedModules.map((item) => (
                    <SidebarRow
                      key={item.id}
                      module={item}
                      active={pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href))}
                      onRemove={removePinnedView}
                      removeTitle={tx(t.sidebar.removeFromPinned, { label: "" })}
                      dataTestIdBase="sidebar-item"
                      removeDataTestIdBase="remove-pinned"
                      collapsed={isCollapsed}
                    />
                  ))}
                </ul>
              </div>
            )}

            {!isCollapsed && (
              <div className="pt-2">
                <button
                  type="button"
                  onClick={openMatrix}
                  data-testid="sidebar-add-custom"
                  className="w-full flex items-center justify-center gap-2 px-3 py-2.5 border-2 border-dashed border-[var(--cel-ink)] hover:border-accent/60 bg-bg-soft/40 hover:bg-accent/10 text-xs font-mono text-ink-mute hover:text-accent transition-all duration-200 group hover:shadow-[2px_2px_0px_0px_var(--cel-ink),0_0_12px_rgba(0,240,255,0.15)]"
                >
                  <Plus size={13} className="text-accent group-hover:rotate-90 transition-transform duration-300" />
                  <span className="font-semibold">{t.sidebar.customAdd}</span>
                </button>
              </div>
            )}
          </div>
        </details>
      </nav>


    </aside>
  );
}

interface SubNavGroupListProps {
  group: ReturnType<typeof findSubNavGroup>;
  activeId: string | null;
  collapsed: boolean;
}

function SubNavGroupList({ group, activeId, collapsed }: SubNavGroupListProps) {
  if (!group) return null;
  return (
    <div data-testid="sidebar-subnav-group" data-prefix={group.pathnamePrefix}>
      {!collapsed && (
        <div
          data-testid="sidebar-subnav-top-label"
          className="px-2.5 py-1 text-xs font-mono uppercase tracking-wider text-ink-mute font-bold"
        >
          {group.topLabel}
        </div>
      )}
      <ul className={clsx(collapsed ? "space-y-1.5" : "mt-1.5 space-y-1")}>
        {group.items.map((item) => (
          <SubNavGroupRow
            key={item.id}
            item={item}
            href={`${group.pathnamePrefix}?${item.query}`}
            active={activeId === item.id}
            collapsed={collapsed}
          />
        ))}
      </ul>
    </div>
  );
}

interface SubNavGroupRowProps {
  item: SubNavEntry;
  href: string;
  active: boolean;
  collapsed: boolean;
}

function SubNavGroupRow({ item, href, active, collapsed }: SubNavGroupRowProps) {
  const cs = getCategoryStyles(item.category);
  const Icon = item.icon;
  return (
    <li>
      <Link
        href={href}
        data-testid={`sidebar-subnav-item-${item.id}`}
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
            <span className="flex-1 truncate tracking-tight">{item.label}</span>
            <span className="text-[10px] font-mono text-ink-mute px-1.5 py-0.5 rounded border border-line/50 bg-bg/40 opacity-70 group-hover:opacity-100 transition-opacity font-bold">
              {item.code}
            </span>
            {active && (
              <span
                className={clsx("size-1.5 rounded-full animate-pulse", cs.dot)}
                aria-hidden="true"
              />
            )}
          </>
        )}
        {collapsed && active && (
          <span
            data-testid={`sidebar-subnav-active-dot-${item.id}`}
            className={clsx("absolute top-0.5 right-0.5 size-1.5 rounded-full", cs.dot)}
            aria-hidden="true"
          />
        )}
      </Link>
    </li>
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
