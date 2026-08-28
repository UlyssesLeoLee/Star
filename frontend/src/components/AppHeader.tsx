// =====================================================================
// AppHeader — 顶栏 (per docs/frontend/design/ui-redesign-multica-style.md §3)
// =====================================================================
// - 左: logo (clip-path `*` 几何) + workspace switcher dropdown
// - 中: 5 视图 tab (Inbox / Issues / Projects / Agents / Analytics) + Settings 齿轮
// - 右: ⌘K 搜索触发 / 🔔 通知 badge / 🟢 Realtime status / 👤 user avatar
// - 高度 64px, border-bottom 1px line, dark theme
// - active tab: 底部 2px accent border + text-accent
// =====================================================================
"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { clsx } from "clsx";
import { useState } from "react";
import { ChevronDown, Bell, Settings, Search } from "lucide-react";
import { useCommandBarStore } from "@/lib/commandBarStore";

// 5 视图 tab — Settings 单独作为齿轮放在 tab 右侧 (per §3 + 任务说明)
const TABS: ReadonlyArray<{ href: string; label: string }> = [
  { href: "/inbox",     label: "Inbox" },
  { href: "/issues",    label: "Issues" },
  { href: "/projects",  label: "Projects" },
  { href: "/agents",    label: "Agents" },
  { href: "/analytics", label: "Analytics" },
];

export function AppHeader() {
  const pathname = usePathname() ?? "/";
  const openCommandBar = useCommandBarStore((s) => s.open);
  const [notifCount] = useState(3); // mock — Phase I+ 接 SSE

  return (
    <header
      data-testid="app-header"
      className="h-16 sticky top-0 z-30 border-b border-line bg-bg/95 backdrop-blur"
    >
      <div className="h-full px-4 flex items-center gap-4">
        {/* === Left: logo + workspace switcher === */}
        <div className="flex items-center gap-2 shrink-0">
          <Link
            href="/"
            className="flex items-center gap-2 group"
            data-testid="app-header-logo"
            aria-label="Star home"
          >
            <div
              aria-hidden="true"
              className="size-7 rounded-md bg-accent/15 border border-accent/40 grid place-items-center text-accent font-bold"
            >
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                {/* `*` 几何 clip-path 风格 logo (multica inspired) */}
                <path d="M8 1l1.8 4.7L15 6.5l-3.8 3.4.9 5L8 12.5 3.9 14.9l.9-5L1 6.5l5.2-.8z" />
              </svg>
            </div>
            <div className="hidden md:block">
              <div className="text-sm font-semibold text-ink group-hover:text-accent transition-colors">Star</div>
              <div className="text-[10px] uppercase tracking-wider text-ink-mute">Vibe Coding WM</div>
            </div>
          </Link>
          <button
            type="button"
            data-testid="workspace-switcher"
            className="flex items-center gap-1 px-2 py-1 text-sm text-ink-dim hover:text-ink rounded-md hover:bg-bg-soft/40 transition-colors"
            aria-label="Switch workspace"
          >
            <span className="truncate max-w-[140px]">ACME Studio</span>
            <ChevronDown size={12} className="text-ink-mute" />
          </button>
        </div>

        {/* === Middle: 5 tab + Settings 齿轮 === */}
        <nav
          className="flex items-center gap-0.5"
          data-testid="primary-tabs"
          aria-label="Primary navigation"
        >
          {TABS.map((tab) => {
            const active = pathname === tab.href || pathname?.startsWith(tab.href + "/");
            return (
              <Link
                key={tab.href}
                href={tab.href}
                data-testid={`tab-${tab.label.toLowerCase()}`}
                data-active={active ? "true" : "false"}
                aria-current={active ? "page" : undefined}
                className={clsx(
                  "relative px-3 h-16 inline-flex items-center text-sm border-b-2 transition-colors",
                  active
                    ? "text-accent border-accent"
                    : "text-ink-dim border-transparent hover:text-ink"
                )}
              >
                {tab.label}
              </Link>
            );
          })}
          <Link
            href="/settings"
            data-testid="settings-gear"
            aria-label="Settings"
            className="ml-1 p-1.5 text-ink-dim hover:text-ink rounded-md hover:bg-bg-soft/40 transition-colors"
          >
            <Settings size={14} />
          </Link>
        </nav>

        {/* === Right: ⌘K, bell, status, avatar === */}
        <div className="ml-auto flex items-center gap-2">
          <button
            type="button"
            onClick={openCommandBar}
            data-testid="command-bar-trigger"
            aria-label="Open command bar (⌘K)"
            className="flex items-center gap-2 px-3 h-8 rounded-md border border-line bg-bg-soft text-ink-dim hover:text-ink hover:border-accent transition-colors text-sm"
          >
            <Search size={13} />
            <span className="hidden sm:inline">Search...</span>
            <kbd className="hidden sm:inline-flex text-[10px] font-mono px-1.5 py-0.5 rounded border border-line text-ink-mute">
              ⌘K
            </kbd>
          </button>

          <button
            type="button"
            data-testid="notifications-bell"
            aria-label={`Notifications (${notifCount} unread)`}
            className="relative p-1.5 text-ink-dim hover:text-ink rounded-md hover:bg-bg-soft/40 transition-colors"
          >
            <Bell size={14} />
            {notifCount > 0 && (
              <span
                data-testid="notifications-badge"
                className="absolute -top-0.5 -right-0.5 min-w-[16px] h-4 rounded-full bg-err text-white text-[10px] grid place-items-center px-1 font-mono"
              >
                {notifCount}
              </span>
            )}
          </button>

          <div
            data-testid="realtime-status"
            className="hidden sm:flex items-center gap-1.5 px-2 h-8 rounded-md border border-line"
            aria-label="Realtime status: online"
          >
            <span className="size-2 rounded-full bg-ok animate-pulse" aria-hidden="true" />
            <span className="text-[11px] text-ink-dim font-mono">online</span>
          </div>

          <button
            type="button"
            data-testid="user-avatar"
            aria-label="User menu: Ulysses (tenant_admin)"
            className="flex items-center gap-2 pl-2 pr-1 h-8 rounded-md border-l border-line hover:bg-bg-soft/40 transition-colors"
          >
            <div className="size-6 rounded-full bg-accent/15 border border-accent/40 grid place-items-center text-accent text-xs font-bold">
              U
            </div>
            <div className="text-left hidden md:block leading-tight">
              <div className="text-xs">Ulysses</div>
              <div className="text-[10px] text-ink-mute">tenant_admin</div>
            </div>
          </button>
        </div>
      </div>
    </header>
  );
}
