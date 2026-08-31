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
import { MODULE_MAP, type ModuleDefinition } from "@/lib/nav/registry";
import { AppMatrixDrawer } from "@/components/nav/AppMatrixDrawer";
import { useTranslation } from "@/lib/i18n";

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
        className="h-16 sticky top-0 z-30 border-b border-line bg-bg/90 backdrop-blur-xl transition-all select-none"
      >
        <div className="h-full px-6 flex items-center gap-4">
          {/* === Left: Workspace Switcher === */}
          <div className="flex items-center gap-2 shrink-0">
            <button
              type="button"
              data-testid="workspace-switcher"
              className="flex items-center gap-2 px-3 py-1.5 text-xs font-mono text-ink-dim hover:text-ink rounded-lg hover:bg-bg-soft/70 border border-transparent hover:border-line transition-all duration-200"
              aria-label={t.appHeader.workspaceSwitcher}
            >
              <span className="size-2 rounded-sm bg-accent rotate-45 shadow-[0_0_8px_rgba(0,240,255,0.9)]" />
              <span className="truncate max-w-[140px] font-bold text-ink tracking-tight">ACME Studio</span>
              <span className="text-[9px] text-accent font-mono font-bold px-1.5 py-0.2 rounded bg-accent/10 border border-accent/30">CORE</span>
              <ChevronDown size={12} className="text-ink-mute ml-0.5" />
            </button>
          </div>

          {/* === Middle: Primary Navigation Tabs (用户自由增删) === */}
          <nav
            className="flex items-center gap-1 overflow-x-auto scrollbar-none"
            data-testid="primary-tabs"
            aria-label="Primary navigation"
          >
            {activeHeaderTabs.map((tab) => {
              const active = pathname === tab.href || pathname?.startsWith(tab.href + "/");
              return (
                <div key={tab.id} className="relative group flex items-center">
                  <Link
                    href={tab.href}
                    data-testid={`tab-${tab.label.toLowerCase()}`}
                    data-active={active ? "true" : "false"}
                    aria-current={active ? "page" : undefined}
                    className={clsx(
                      "relative px-3.5 h-16 inline-flex items-center gap-1.5 text-xs font-medium border-b-2 transition-all duration-200",
                      active
                        ? "text-accent border-accent font-semibold shadow-[inset_0_-2px_14px_rgba(0,240,255,0.18)]"
                        : "text-ink-dim border-transparent hover:text-ink hover:border-line/60"
                    )}
                  >
                    <span className={clsx(
                      "text-[9px] font-mono transition-colors",
                      active ? "text-accent font-bold" : "text-ink-mute group-hover:text-ink-dim"
                    )}>
                      {tab.code}
                    </span>
                    <span>{tab.label}</span>
                  </Link>

                  {/* 顶栏 Tab 删除按钮 */}
                  <button
                    type="button"
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      removeHeaderTab(tab.id);
                    }}
                    title={tx(t.appHeader.removeFromHeader, { label: tab.label })}
                    data-testid={`remove-header-tab-${tab.id}`}
                    className="p-0.5 ml-[-6px] mr-1 rounded hover:bg-err/20 hover:text-err text-ink-mute opacity-0 group-hover:opacity-100 transition-all duration-150 z-10"
                  >
                    <X size={10} />
                  </button>
                </div>
              );
            })}

            {/* + 添加顶栏标签 */}
            <button
              type="button"
              onClick={openMatrix}
              data-testid="header-add-tab"
              title={t.appHeader.addMoreTabs}
              className="p-1.5 text-ink-mute hover:text-accent rounded-lg hover:bg-bg-soft transition-colors"
            >
              <Plus size={13} />
            </button>

            <Link
              href="/settings"
              data-testid="settings-gear"
              aria-label="Settings"
              className={clsx(
                "ml-1 p-2 text-ink-dim hover:text-ink rounded-lg hover:bg-bg-soft/70 transition-colors",
                pathname.startsWith("/settings") && "text-accent bg-accent/10 border border-accent/20"
              )}
            >
              <Settings size={15} />
            </Link>
          </nav>

          {/* === Right: App Matrix, Theme Toggle, ⌘K, bell, status, avatar === */}
          <div className="ml-auto flex items-center gap-3">
            {/* === 右上角应用菜单 / App Matrix 抽屉按钮 === */}
            <button
              type="button"
              onClick={openMatrix}
              data-testid="app-matrix-trigger"
              aria-label="Open App Matrix (All Modules)"
              className="flex items-center gap-1.5 px-3 h-8 rounded-lg border border-line bg-bg-soft/70 text-ink-dim hover:text-accent hover:border-accent transition-all duration-200 text-xs font-mono group shadow-sm hover:shadow-[0_0_14px_rgba(0,240,255,0.22)]"
            >
              <LayoutGrid size={13} className="text-accent group-hover:scale-110 transition-transform duration-200" />
              <span className="hidden lg:inline font-bold">{t.appHeader.allApps}</span>
              <span className="text-[9px] px-1.5 py-0.2 rounded bg-accent/10 text-accent font-bold border border-accent/30">{t.appHeader.appsCount}</span>
            </button>

            <ThemeSwitcher />

            <button
              type="button"
              onClick={openCommandBar}
              data-testid="command-bar-trigger"
              aria-label="Open command bar (⌘K)"
              className="flex items-center gap-2 px-3 h-8 rounded-lg border border-line bg-bg-soft/70 text-ink-dim hover:text-ink hover:border-accent transition-all duration-200 text-xs shadow-sm hover:shadow-[0_0_12px_rgba(0,240,255,0.18)]"
            >
              <Search size={13} className="text-accent" />
              <span className="hidden sm:inline font-medium">{t.appHeader.tacticalJump}</span>
              <kbd className="hidden sm:inline-flex text-[10px] font-mono px-1.5 py-0.5 rounded border border-line bg-bg text-ink-mute font-semibold">
                ⌘K
              </kbd>
            </button>

            <button
              type="button"
              data-testid="notifications-bell"
              aria-label={tx(t.appHeader.notifications, { count: notifCount })}
              className="relative p-2 text-ink-dim hover:text-ink rounded-lg hover:bg-bg-soft transition-colors"
            >
              <Bell size={15} />
              {notifCount > 0 && (
                <span
                  data-testid="notifications-badge"
                  className="absolute -top-0.5 -right-0.5 min-w-[16px] h-4 rounded-full bg-err text-white text-[10px] grid place-items-center px-1 font-mono shadow-[0_0_10px_rgba(255,51,102,0.8)] font-bold"
                >
                  {notifCount}
                </span>
              )}
            </button>

            <div
              data-testid="realtime-status"
              className="hidden sm:flex items-center gap-1.5 px-3 h-8 rounded-lg border border-line bg-bg-soft/50"
              aria-label={t.appHeader.realtimeOnline}
            >
              <span className="size-2 rounded-full bg-ok animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.9)]" aria-hidden="true" />
              <span className="text-[10px] text-ink-dim font-mono tracking-wider font-bold">{t.appHeader.synced}</span>
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
