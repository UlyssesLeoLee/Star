/*
@cypher schema=1 source_sha256=34836b8339e4510e9ffc0b97ce06b7671985447df58ba5426ce6752028c86c68
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
} from "lucide-react";
import { clsx } from "clsx";
import { useTheme } from "next-themes";
import { useEffect, useState, useCallback } from "react";
import { useNavStore } from "@/lib/nav/navStore";
import { MODULE_MAP, NAV_GROUPS, getCategoryStyles, type ModuleDefinition, type ModuleCategory, type NavGroupId } from "@/lib/nav/registry";
import { moduleHrefMatches } from "@/lib/nav/activeMatch";
import { ProjectRunTree } from "@/components/run/ProjectRunTree";
import { useTranslation, useModuleTranslation } from "@/lib/i18n";

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
  const sidebarFold = useNavStore((s) => s.sidebarFold);
  const toggleSidebarFold = useNavStore((s) => s.toggleSidebarFold);
  const setSidebarFold = useNavStore((s) => s.setSidebarFold);

  useEffect(() => setMounted(true), []);

  const isDark = !mounted || theme === "dark" || (typeof document !== "undefined" && document.documentElement.classList.contains("dark"));
  const isCollapsed = sidebarFold === "collapsed";

  // =====================================================================
  // per 2026-11-14 artifact 参考 UI: 左侧 = 项目列表(LANE 树) + 底部全局组,
  // 用户钉选(pinned/sidebarItemIds)不再在侧栏渲染 — 入口统一走顶栏 App Matrix
  // =====================================================================
  const isModuleActive = useCallback(
    (item: ModuleDefinition) => moduleHrefMatches(item.href, pathname, searchParams),
    [pathname, searchParams]
  );

  // =====================================================================
  // 5 组结构分组解析 (per ui-3pane-arch §1.2 v0.2)
  // =====================================================================
  // 组内模块: 按 NAV_GROUPS id 映射
  // (worktree 保留在 Home 核心区, per ui-3pane-arch §1.2.1 "保留独立路由, 仅导航位置上移")
  const groupModules = (groupId: NavGroupId): ModuleDefinition[] =>
    NAV_GROUPS[groupId]
      .map((id) => MODULE_MAP.get(id))
      .filter((m): m is ModuleDefinition => Boolean(m));

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
          // per ui-3pane-arch §1.2.1: Home 落地页 = Worktree 控制中心首屏
          href="/worktree"
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
              </div>
              <div className="text-[11px] font-mono tracking-wider text-[var(--cel-text-secondary,#94a3b8)] uppercase font-bold">
                Star — Work Management
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
      {/* === Navigation Groups (per ui-3pane-arch §1.2 v0.2: 5 组结构) === */}
      <nav
        className={clsx(
          "flex-1 overflow-y-auto text-sm scrollbar-none",
          isCollapsed ? "px-2 py-3 space-y-2" : "px-3 py-4 space-y-5"
        )}
        aria-label={t.sidebar.projectNavigation}
      >
        {/* Project → LANE → Worktree → Branch 层级树 (per 2026-11-14 用户发令:
            恢复 LANE 层级设计; 无宿主会话时降级为可达的演示目录, 不显示死块) */}
        <ProjectRunTree collapsed={isCollapsed} />
        {/* 全局组 (per 2026-11-14 artifact 参考 UI: 左栏 = 项目列表 + 底部全局组) */}
        <div className="mt-auto">
          <NavGroupSection
            testId="sidebar-group-global"
            label={t.sidebar.navGlobal}
            modules={groupModules("home-personal")}
            collapsed={isCollapsed}
            isActive={isModuleActive}
          />
        </div>
      </nav>


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
