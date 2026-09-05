"use client";

import Link from "next/link";
import { usePathname, useSearchParams } from "next/navigation";
import {
  Radio,
  Plus,
  X,
  PanelLeftClose,
  PanelLeftOpen,
  Globe,
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

// =====================================================================
// Sidebar — 折叠 + scope toggle 双模态侧栏
// (per 2026-09-03 12:36 JST 拍板: 4 项推荐全部命中)
// =====================================================================
// 折叠态 (w-16=64px): 只显示 icon + code, 隐藏 label/count/group label/footer
// 展开态 (w-64=256px): 完整内容, 上方 brand block 下方加 Main/Project scope toggle
// 切换条件:
//   - fold toggle button  (顶部 brand block 右上角)
//   - scope toggle pills  (brand block 下方, 仅 expanded 可见)
//   - 持久化到 localStorage (zustand persist key=star-nav-store:v2)
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
  const sidebarScope = useNavStore((s) => s.sidebarScope);
  const setSidebarScope = useNavStore((s) => s.setSidebarScope);
  const toggleSidebarFold = useNavStore((s) => s.toggleSidebarFold);
  const setSidebarFold = useNavStore((s) => s.setSidebarFold);

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
  // project scope 可用性: 仅在 /projects 路径下, 其他路径下灰显
  // (per 拍板 #2: 项目专属导航条仅在选中项目时才有意义, 路径外不可达)
  const isProjectScopeAvailable = pathname === "/projects" || pathname.startsWith("/projects/") || pathname.startsWith("/projects?");

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

  // 自动 fallback: pathname 离开 /projects 时, scope 强制回 main
  // 防止用户折叠后忘了切回, 看到的是空 project scope
  useEffect(() => {
    if (sidebarScope === "project" && !isProjectScopeAvailable) {
      setSidebarScope("main");
    }
  }, [sidebarScope, isProjectScopeAvailable, setSidebarScope]);

  return (
    <aside
      data-testid="app-sidebar"
      data-fold={sidebarFold}
      data-scope={sidebarScope}
      // 桌面: 256/64 sticky; <768px: 隐藏 (由 MobileBottomNav + AppMatrixDrawer 抽屉替代, per 2026-09-01 PHASE-MOBILE-PWA)
      className={clsx(
        "hidden md:flex shrink-0 border-r-2 border-black bg-bg-soft/95 cel-shadow backdrop-blur-xl flex-col h-screen sticky top-0 select-none z-20 transition-all duration-200 ease-out",
        SIDEBAR_WIDTH[sidebarFold]
      )}
    >
      {/* === Star Anime Cyber Crest (Brand Block) + Fold Toggle === */}
      <div
        data-testid="sidebar-brand-block"
        className={clsx(
          "border-b border-line shrink-0 flex items-center",
          isCollapsed ? "justify-center px-2 py-4 flex-col gap-3" : "justify-between px-4 py-4"
        )}
      >
        <Link
          href="/inbox"
          className={clsx(
            "flex items-center group",
            isCollapsed ? "" : "gap-3"
          )}
          data-testid="sidebar-brand"
          aria-label={t.ariaLabels.starHome}
        >
          <div
            aria-hidden="true"
            className="size-9 rounded-xl overflow-hidden border border-accent/40 shadow-[0_0_16px_rgba(0,240,255,0.3)] shrink-0 transition-transform duration-300 group-hover:scale-105 group-hover:rotate-6 bg-black"
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
                  "text-sm font-black tracking-tight transition-colors",
                  isDark ? "text-slate-100 group-hover:text-cyan-300" : "text-slate-900 group-hover:text-sky-600"
                )}>
                  STAR
                </span>
                <span className="font-mono text-[9px] font-bold px-1.5 py-0.2 rounded border border-accent/40 bg-accent/10 text-accent">
                  v0.2
                </span>
              </div>
              <div className="text-[9px] font-mono tracking-widest text-ink-mute uppercase font-medium">
                {t.sidebar.brandTagline}
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
      {!isCollapsed && (
        <ScopeToggle
          active={sidebarScope}
          projectAvailable={isProjectScopeAvailable}
          onChange={setSidebarScope}
        />
      )}

      {/* === Navigation Groups === */}
      <nav
        className={clsx(
          "flex-1 overflow-y-auto text-sm scrollbar-none",
          isCollapsed ? "px-2 py-3 space-y-2" : "px-3 py-4 space-y-5"
        )}
        aria-label={sidebarScope === "project" ? "Project Navigation" : "Main Navigation"}
      >
        {sidebarScope === "project" ? (
          subNavGroup ? (
            <SubNavGroupList
              group={subNavGroup}
              activeId={activeSubNavId}
              collapsed={isCollapsed}
            />
          ) : (
            <EmptyProjectState collapsed={isCollapsed} />
          )
        ) : (
          <>
            {/* Core Workspaces (用户自定义定制列表) */}
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
                    active={pathname === item.href || (item.href !== "/" && pathname?.startsWith(item.href))}
                    onRemove={removeSidebarItem}
                    removeTitle={tx(t.sidebar.removeFromSidebar, { label: "" })}
                    dataTestIdBase="sidebar-item"
                    removeDataTestIdBase="remove-sidebar"
                    collapsed={isCollapsed}
                  />
                ))}
              </ul>
            </div>

            {/* Tactical Views (如看板、甘特排期) */}
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
                      active={pathname === item.href || (item.href !== "/" && pathname?.startsWith(item.href))}
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

            {/* + 自定义添加导航项按钮 — 折叠态隐藏, 用户从 AppMatrix 抽屉添加 */}
            {!isCollapsed && (
              <div className="pt-2">
                <button
                  type="button"
                  onClick={openMatrix}
                  data-testid="sidebar-add-custom"
                  className="w-full flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-dashed border-line hover:border-accent/60 bg-bg-soft/40 hover:bg-accent/10 text-xs font-mono text-ink-mute hover:text-accent transition-all duration-200 group shadow-sm hover:shadow-[0_0_12px_rgba(0,240,255,0.15)]"
                >
                  <Plus size={13} className="text-accent group-hover:rotate-90 transition-transform duration-300" />
                  <span className="font-semibold">{t.sidebar.customAdd}</span>
                </button>
              </div>
            )}
          </>
        )}
      </nav>

      {/* === Bottom Tactical HUD Footer (仅展开态可见) === */}
      {!isCollapsed && (
        <div className="border-t border-line px-4 py-3 shrink-0 bg-bg-soft/50 space-y-1.5">
          <div className="flex items-center justify-between text-[10px] font-mono text-ink-mute">
            <span className="flex items-center gap-1.5">
              <Radio size={12} className="text-accent animate-spin" />
              <span>{t.sidebar.footerStatus}</span>
            </span>
            <span className="text-ok font-bold drop-shadow-[0_0_6px_rgba(16,185,129,0.7)]">
              {t.sidebar.footerStatusAllGreen}
            </span>
          </div>
          <div className="text-[9px] font-mono text-ink-mute/70 truncate">
            {t.sidebar.footerNode}
          </div>
        </div>
      )}
    </aside>
  );
}

// =====================================================================
// ScopeToggle — Main / Project 两段切换 (per 2026-09-03 12:36 JST 拍板 #2)
// =====================================================================
// iOS Settings.app 风格: 两个 pill 段, 选中状态 accent 染色, project 不可用
// 时灰显 + tooltip 提示 (per 拍板 #4 + 拍板 #1 "复用 SubNav 数据源" 派生).
// =====================================================================
interface ScopeToggleProps {
  active: "main" | "project";
  projectAvailable: boolean;
  onChange: (scope: "main" | "project") => void;
}

function ScopeToggle({ active, projectAvailable, onChange }: ScopeToggleProps) {
  const { t } = useTranslation();
  return (
    <div
      data-testid="sidebar-scope-toggle"
      data-scope={active}
      role="tablist"
      aria-label={t.ariaLabels.sidebarScope}
      className="px-3 pt-3 pb-2 border-b border-line/60 shrink-0"
    >
      <div className="flex items-center gap-1 p-0.5 rounded-lg bg-bg-soft/40 border border-line">
        {/* Main pill */}
        <button
          type="button"
          role="tab"
          aria-selected={active === "main"}
          data-testid="sidebar-scope-main"
          title={t.sidebar.scope.mainHint}
          onClick={() => onChange("main")}
          className={clsx(
            "flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-md text-[10px] font-mono uppercase tracking-wider transition-all duration-150",
            active === "main"
              ? "bg-accent/15 text-accent border border-accent/40 shadow-[0_0_8px_rgba(0,240,255,0.25)]"
              : "text-ink-mute hover:text-ink border border-transparent"
          )}
        >
          <Globe size={11} />
          <span className="font-semibold">{t.sidebar.scope.main}</span>
        </button>
        {/* Project pill */}
        <button
          type="button"
          role="tab"
          aria-selected={active === "project"}
          data-testid="sidebar-scope-project"
          title={projectAvailable ? t.sidebar.scope.projectHint : t.sidebar.scope.projectDisabledHint}
          aria-disabled={!projectAvailable}
          onClick={() => projectAvailable && onChange("project")}
          disabled={!projectAvailable}
          className={clsx(
            "flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-md text-[10px] font-mono uppercase tracking-wider transition-all duration-150",
            active === "project" && projectAvailable
              ? "bg-accent/15 text-accent border border-accent/40 shadow-[0_0_8px_rgba(0,240,255,0.25)]"
              : projectAvailable
                ? "text-ink-mute hover:text-ink border border-transparent"
                : "text-ink-mute/40 cursor-not-allowed border border-transparent"
          )}
        >
          <FolderKanban size={11} />
          <span className="font-semibold">{t.sidebar.scope.project}</span>
        </button>
      </div>
    </div>
  );
}

// =====================================================================
// SubNavGroupList — Project scope 渲染 (per 2026-09-03 12:36 JST 拍板 #1)
// =====================================================================
// 复用 subNavRegistry 数据源, 不重复造轮子. 渲染风格跟 SubNav.tsx
// 类似, 但用 Sidebar 自身的 domain-color + icon tile 系统.
// =====================================================================
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
          className="px-2.5 py-1 text-[10px] font-mono uppercase tracking-wider text-ink-mute"
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
          "relative flex items-center rounded-xl text-xs font-medium transition-all duration-200",
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
            <span className="text-[9px] font-mono text-ink-mute px-1.5 py-0.2 rounded border border-line/50 bg-bg/40 opacity-70 group-hover:opacity-100 transition-opacity">
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
// data-testid 仍用 registry 原 label (英文) 生成, 保证既有测试稳定
// (sidebar-item-inbox / sidebar-item-issues / ...) 不受语言切换影响.
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
  // Jira 风格: 域分色 icon tile (5 域色, per 2026-09-02 15:42 JST 拍板)
  const cs = getCategoryStyles(item.category);
  // 用 registry 静态 label 生成 testid, 避免翻译切换导致 testid 漂移
  const testIdSlug = item.label.toLowerCase().replace(/\s+/g, "-");
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
          "relative flex items-center gap-2.5 px-2.5 py-1.5 rounded-xl text-xs font-medium transition-all duration-200",
          active
            ? "bg-bg-soft/80 text-ink border border-line shadow-soft font-semibold"
            : "text-ink-dim hover:bg-bg-soft/60 hover:text-ink border border-transparent"
        )}
      >
        {/* Jira 风格 icon tile — 8x8 圆角色块底 + 域分色 + line icon */}
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

        <span className="text-[9px] font-mono text-ink-mute px-1.5 py-0.2 rounded border border-line/50 bg-bg/40 opacity-70 group-hover:opacity-100 transition-opacity">
          {item.code}
        </span>

        {item.count !== undefined && item.count > 0 && (
          <span className="min-w-[16px] h-4 rounded-full bg-err text-white text-[9px] font-mono grid place-items-center px-1 shadow-[0_0_8px_rgba(255,51,102,0.7)] font-bold">
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
