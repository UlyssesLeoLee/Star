"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { clsx } from "clsx";
import { useState } from "react";
import {
  ChevronDown,
  Bell,
  Settings,
  Search,
  LayoutGrid,
  Plus,
  X,
  Sparkles,
} from "lucide-react";
import { useCommandBarStore } from "@/lib/commandBarStore";
import { UserMenu } from "@/components/UserMenu";
import { ThemeSwitcher } from "@/components/theme/ThemeSwitcher";
import { useNavStore } from "@/lib/nav/navStore";
import { MODULE_MAP, getCategoryStyles, type ModuleDefinition } from "@/lib/nav/registry";
import { AppMatrixDrawer } from "@/components/nav/AppMatrixDrawer";
import { useTranslation, useModuleTranslation } from "@/lib/i18n";

// GitHub Worktree 标识符 (per ULYS-176 §3 用户反馈: 左上角 icon 下方显示当前启动的 worktree 编号)
// 数据源优先级: process.env.NEXT_PUBLIC_WORKTREE_ID (Multica spawn dev server 时注入)
//             → process.env.NEXT_PUBLIC_MULTICA_ISSUE_ID (备选)
//             → '' (用户多 worktree 并开时仍可读)
const WORKTREE_ID =
  process.env.NEXT_PUBLIC_WORKTREE_ID ||
  process.env.NEXT_PUBLIC_MULTICA_ISSUE_ID ||
  "";
const WORKTREE_BRANCH =
  process.env.NEXT_PUBLIC_WORKTREE_BRANCH ||
  process.env.NEXT_PUBLIC_GIT_BRANCH ||
  "";

export function AppHeader() {
  const pathname = usePathname() ?? "/";
  const openCommandBar = useCommandBarStore((s) => s.open);
  const { t, tx } = useTranslation();
  const [notifCount] = useState(3); // mock — Phase I+ 接 SSE

  const headerTabIds = useNavStore((s) => s.headerTabIds);
  const removeHeaderTab = useNavStore((s) => s.removeHeaderTab);
  const openMatrix = useNavStore((s) => s.openMatrix);

  // 解析当前顶部钉选的标签对象
  const activeHeaderTabs = headerTabIds
    .map((id) => MODULE_MAP.get(id))
    .filter((m): m is ModuleDefinition => Boolean(m));

  return (
    <>
      <header
        data-testid="app-header"
        className="h-[76px] sticky top-0 z-30 border-b-2 border-black bg-[var(--cel-surface-card,#0f1422)]/95 backdrop-blur-xl cel-shadow transition-all select-none text-[var(--cel-text-primary,#ffffff)]"
      >
        <div className="h-full px-6 flex items-center gap-4">
          {/* === Left: Workspace Switcher + Worktree ID sub-label ===
              per ULYS-176 §3: 在 ACME Studio CORE 按钮下方添加一行 monospace badge,
              显示当前启动的 GitHub worktree 编号 (Multica issue identifier), 方便一眼识别
              是哪个分支的 dev server (避免多 worktree 并开时混淆)。
              数据源: process.env.NEXT_PUBLIC_WORKTREE_ID / NEXT_PUBLIC_WORKTREE_BRANCH
              (Multica 平台在 spawn dev server 时注入; 缺省时该 sub-label 整行 hidden, 不破坏原布局) */}
          <div className="flex flex-col items-start gap-1 shrink-0">
            <button
              type="button"
              data-testid="workspace-switcher"
              className="flex items-center gap-2 px-3 h-9 text-xs font-mono text-[var(--cel-text-primary,#ffffff)] hover:text-white border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] cel-shadow transition-all duration-150"
              aria-label={t.appHeader.workspaceSwitcher}
            >
              <span className="size-2 bg-[var(--cel-cyan,#00f0ff)] rotate-45 border border-black" />
              <span className="truncate max-w-[140px] font-black tracking-tight">ACME Studio</span>
              <span className="text-[10px] text-black font-mono font-black px-1.5 py-0.5 bg-[var(--cel-gold,#ffc400)] border border-black">CORE</span>
              <ChevronDown size={12} className="text-[var(--cel-text-secondary)] ml-0.5" />
            </button>
            {WORKTREE_ID && (
              <div
                data-testid="worktree-id-badge"
                className="flex items-center gap-1.5 pl-1 pr-2 h-5 text-[10px] font-mono font-bold border border-black bg-[var(--cel-surface-stage,#090d16)] text-[var(--cel-cyan,#00f0ff)] whitespace-nowrap"
                title={`GitHub worktree: ${WORKTREE_ID}${WORKTREE_BRANCH ? ` @ ${WORKTREE_BRANCH}` : ""}`}
              >
                <span className="size-1.5 bg-[var(--cel-cyan,#00f0ff)] animate-pulse" aria-hidden="true" />
                <span className="text-[var(--cel-text-secondary,#94a3b8)] font-black tracking-wider">WT</span>
                <span className="text-[var(--cel-text-primary,#ffffff)] font-black">{WORKTREE_ID}</span>
                {WORKTREE_BRANCH && (
                  <>
                    <span className="text-[var(--cel-text-secondary,#94a3b8)]">·</span>
                    <span className="text-[var(--cel-gold,#ffc400)] font-black tracking-tight">{WORKTREE_BRANCH}</span>
                  </>
                )}
              </div>
            )}
          </div>

          {/* === Middle: Primary Navigation Tabs (用户自由增删) === */}
          <nav
            className="flex items-center gap-1 overflow-x-auto scrollbar-none"
            data-testid="primary-tabs"
            aria-label={t.ariaLabels.primaryNav}
          >
            {activeHeaderTabs.map((tab) => (
              <HeaderTab
                key={tab.id}
                module={tab}
                active={pathname === tab.href || pathname?.startsWith(tab.href + "/")}
                onRemove={() => removeHeaderTab(tab.id)}
              />
            ))}

            {/* + 添加顶栏标签 */}
            <button
              type="button"
              onClick={openMatrix}
              data-testid="header-add-tab"
              title={t.appHeader.addMoreTabs}
              className="h-9 w-9 grid place-items-center text-[var(--cel-text-secondary)] hover:text-[var(--cel-cyan,#00f0ff)] border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] cel-shadow transition-colors shrink-0"
            >
              <Plus size={14} />
            </button>

            <Link
              href="/settings"
              data-testid="settings-gear"
              aria-label={t.ariaLabels.settings}
              className={clsx(
                "ml-1 h-9 w-9 grid place-items-center text-[var(--cel-text-secondary)] hover:text-white border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] cel-shadow transition-colors shrink-0",
                pathname.startsWith("/settings") && "text-[var(--cel-cyan,#00f0ff)] border-[var(--cel-cyan,#00f0ff)]"
              )}
            >
              <Settings size={15} />
            </Link>
          </nav>

          {/* === Right: App Matrix, Theme Toggle, ⌘K, bell, dot, avatar ===
                        per 2026-09-27 用户反馈: 顶栏按钮过挤, 删 TacticalCore3D HUD + 文字说明,
                        只保留图标/dot/borderless 状态指示. 5 个核心控件, 1520px 顶栏不再 overflow. */}
                    <div className="ml-auto flex items-center gap-2">
                      {/* === 右上角应用菜单 / App Matrix 抽屉按钮 (icon-only) === */}
                      <button
                        type="button"
                        onClick={openMatrix}
                        data-testid="app-matrix-trigger"
                        aria-label={t.ariaLabels.openAppMatrix}
                        className="size-9 grid place-items-center border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-cyan,#00f0ff)] hover:bg-[var(--cel-surface-card,#0f1422)] transition-all cel-shadow shrink-0 group"
                      >
                        <LayoutGrid size={14} className="group-hover:scale-110 transition-transform duration-200" />
                      </button>

                      <ThemeSwitcher />

                      <button
                        type="button"
                        onClick={openCommandBar}
                        data-testid="command-bar-trigger"
                        aria-label={t.ariaLabels.openCommandBar}
                        className="flex items-center gap-1.5 h-9 px-2 border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-primary,#ffffff)] hover:bg-[var(--cel-surface-card,#0f1422)] transition-all text-xs cel-shadow shrink-0 whitespace-nowrap"
                      >
                        <Search size={13} className="text-[var(--cel-cyan,#00f0ff)] shrink-0" />
                        <kbd className="inline-flex items-center justify-center text-[10px] font-mono px-1.5 border border-black bg-[var(--cel-surface-stage,#090d16)] text-[var(--cel-gold,#ffc400)] font-black">⌘K</kbd>
                      </button>

                      <button
                        type="button"
                        data-testid="notifications-bell"
                        aria-label={tx(t.appHeader.notifications, { count: notifCount })}
                        className="relative size-9 grid place-items-center text-[var(--cel-text-primary,#ffffff)] hover:text-white border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] cel-shadow transition-colors shrink-0"
                      >
                        <Bell size={15} />
                        {notifCount > 0 && (
                          <span
                            data-testid="notifications-badge"
                            className="absolute -top-1.5 -right-1.5 min-w-[18px] h-4.5 border border-black bg-[var(--cel-crimson,#ff184c)] text-black text-[10px] grid place-items-center px-1 font-mono font-black"
                          >
                            {notifCount}
                          </span>
                        )}
                      </button>

                      {/* Realtime status — dot only, no text label */}
                      <div
                        data-testid="realtime-status"
                        className="size-9 grid place-items-center border-2 border-black bg-[var(--cel-surface-stage,#090d16)] cel-shadow shrink-0"
                        aria-label={t.appHeader.realtimeOnline}
                        title={t.appHeader.realtimeOnline}
                      >
                        <span className="size-2.5 bg-ok rounded-full border border-black" aria-hidden="true" />
                      </div>

                      <UserMenu />
                    </div>
        </div>
      </header>

      {/* App Matrix Modal / Drawer */}
      <AppMatrixDrawer />
    </>
  );
}

// =====================================================================
// HeaderTab — 顶栏单个 tab (per 2026-08-31 i18n 补缺口)
// =====================================================================
// 子组件, 在内合法调用 useModuleTranslation 拿翻译后 label
// data-testid 仍用 registry 原 label 生成, 保持既有测试稳定
// =====================================================================
interface HeaderTabProps {
  module: ModuleDefinition;
  active: boolean;
  onRemove: () => void;
}

function HeaderTab({ module: tab, active, onRemove }: HeaderTabProps) {
  const mod = useModuleTranslation(tab);
  const { t, tx } = useTranslation();
  // Jira 风格: 顶栏 active 用域色 + icon tile (per 2026-09-02 18:16 + 18:23 JST 推)
  // 之前 active 全用 accent 青色, 跟 module 业务域脱钩, 且只用 01/02 短码
  // 现在: Inbox=cyan, Issues/Projects/Analytics=blue, Agents=emerald, Settings=amber
  //      配 lucide icon (来自 ModuleDefinition.icon), 跟 Sidebar icon tile 风格统一
  const cs = getCategoryStyles(tab.category);
  const Icon = tab.icon;
  // 用 registry 静态 id 生成 testid (per 2026-09-05 19:13 JST: label 改 Sprint 后 testid 仍稳定, 不会随 i18n 漂移)
  const testIdSlug = tab.id;
  return (
    <div className="relative group flex items-center shrink-0">
      <Link
        href={tab.href}
        data-testid={`tab-${testIdSlug}`}
        data-active={active ? "true" : "false"}
        aria-current={active ? "page" : undefined}
        className={clsx(
          "relative px-3.5 h-16 inline-flex items-center gap-2 text-sm font-semibold border-b-2 transition-all duration-200 shrink-0 whitespace-nowrap",
          active
            ? // 域色 active: text 域色 + border 域色 + glow
              clsx(cs.text, cs.borderActive, "font-bold", cs.glow)
            : "text-ink-dim border-transparent hover:text-ink hover:border-line/60"
        )}
      >
        {/* Jira 风格 icon tile — 7x7 圆角色块 + 域分色 + line icon (per 18:23 JST 推) */}
        <div
          data-testid={`tab-icon-tile-${tab.id}`}
          aria-hidden="true"
          className={clsx(
            "size-7 rounded-lg grid place-items-center shrink-0 border transition-all duration-200",
            active
              ? // active: 域色 bg + 域色 border + 域色 text + glow
                clsx(cs.bg, cs.border, cs.text, cs.glow)
              : // inactive: 透明 + 灰 text, hover 域色微染
                "border-transparent text-ink-mute group-hover:text-ink-dim"
          )}
        >
          <Icon size={15} strokeWidth={2.25} />
        </div>
        <span>{mod.label}</span>
      </Link>

      {/* 顶栏 Tab 删除按钮 */}
      <button
        type="button"
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          onRemove();
        }}
        title={tx(t.appHeader.removeFromHeader, { label: mod.label })}
        data-testid={`remove-header-tab-${tab.id}`}
        className="p-0.5 ml-[-6px] mr-1 rounded hover:bg-err/20 hover:text-err text-ink-mute opacity-0 group-hover:opacity-100 transition-all duration-150 z-10"
      >
        <X size={12} />
      </button>
    </div>
  );
}