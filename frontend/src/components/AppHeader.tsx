"use client";
// =====================================================================
// AppHeader — 顶栏 (per docs/frontend/design/ui-redesign-multica-style.md §3)
// =====================================================================
// - 左: logo (clip-path `*` 几何) + workspace switcher dropdown
// - 中: 5 视图 tab (Inbox / Issues / Projects / Agents / Analytics) + Settings 齿轮
// - 右: ⌘K 搜索触发 / 🔔 通知 badge / 🟢 Realtime status / 👤 user avatar
// - 高度 64px, border-bottom 1px line, dark theme
// - active tab: 底部 2px accent border + text-accent
// - 反色: 2026-08-29 17:12 JST Ulysses 拍板 "所有字体颜色都应该和它的背景反色"
//   → Star logo + 副标题 + tab 字色随 useTheme 切换, light=深色 / dark=浅色
// =====================================================================
import Link from "next/link";
import { usePathname } from "next/navigation";
import { clsx } from "clsx";
import { useState, useEffect } from "react";
import { useTheme } from "next-themes";
import { ChevronDown, Bell, Settings, Search, Sun, Moon } from "lucide-react";
import { useCommandBarStore } from "@/lib/commandBarStore";
import { UserMenu } from "@/components/UserMenu";

// 5 视图 tab — Settings 单独作为齿轮放在 tab 右侧 (per §3 + 任务说明)
const TABS: ReadonlyArray<{ href: string; label: string; code: string }> = [
  { href: "/inbox",     label: "Inbox",     code: "01" },
  { href: "/issues",    label: "Issues",    code: "02" },
  { href: "/projects",  label: "Projects",  code: "03" },
  { href: "/agents",    label: "Agents",    code: "04" },
  { href: "/analytics", label: "Analytics", code: "05" },
];

export function AppHeader() {
  const pathname = usePathname() ?? "/";
  const openCommandBar = useCommandBarStore((s) => s.open);
  const [notifCount] = useState(3); // mock — Phase I+ 接 SSE
  const [isDark, setIsDark] = useState(true);

  useEffect(() => {
    const saved = localStorage.getItem("star-theme");
    if (saved === "light") {
      setIsDark(false);
      document.documentElement.classList.remove("dark");
      document.documentElement.classList.add("light");
    } else {
      setIsDark(true);
      document.documentElement.classList.remove("light");
      document.documentElement.classList.add("dark");
    }
  }, []);

  const toggleTheme = () => {
    const next = !isDark;
    setIsDark(next);
    if (next) {
      document.documentElement.classList.remove("light");
      document.documentElement.classList.add("dark");
      localStorage.setItem("star-theme", "dark");
    } else {
      document.documentElement.classList.remove("dark");
      document.documentElement.classList.add("light");
      localStorage.setItem("star-theme", "light");
    }
  };

  return (
    <header
      data-testid="app-header"
      className="h-16 sticky top-0 z-30 border-b border-line bg-bg/95 backdrop-blur"
    >
      <div className="h-full px-4 flex items-center gap-4">
        {/* === Left: workspace switcher (Star logo 移到 Sidebar 顶部, 2026-08-29 18:48 JST) === */}
        <div className="flex items-center gap-2 shrink-0">
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

        {/* === Right: Theme Toggle, ⌘K, bell, status, avatar === */}
        <div className="ml-auto flex items-center gap-2">
          {/* 日漫风格日/夜主题切换器 */}
          <button
            type="button"
            onClick={toggleTheme}
            data-testid="theme-toggle"
            aria-label={isDark ? "Switch to Mecha Light Theme" : "Switch to Neo-Tokyo Dark Theme"}
            title={isDark ? "切换至 Mecha 亮色主题" : "切换至 Neo-Tokyo 暗色主题"}
            className="p-1.5 text-ink-dim hover:text-accent rounded-md hover:bg-bg-soft/60 border border-line/60 hover:border-accent/40 transition-all duration-200 active:scale-95"
          >
            {isDark ? (
              <Sun size={15} className="text-warn hover:rotate-45 transition-transform" />
            ) : (
              <Moon size={15} className="text-accent hover:-rotate-12 transition-transform" />
            )}
          </button>

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
                className="absolute -top-0.5 -right-0.5 min-w-[16px] h-4 rounded-full bg-err text-white text-[10px] grid place-items-center px-1 font-mono shadow-[0_0_8px_rgba(255,51,102,0.6)]"
              >
                {notifCount}
              </span>
            )}
          </button>

          <div
            data-testid="realtime-status"
            className="hidden sm:flex items-center gap-1.5 px-2 h-8 rounded-md border border-line bg-bg-soft/30"
            aria-label="Realtime status: online"
          >
            <span className="size-2 rounded-full bg-ok animate-pulse shadow-[0_0_6px_rgba(16,185,129,0.7)]" aria-hidden="true" />
            <span className="text-[10px] text-ink-dim font-mono tracking-wider">SYNCED</span>
          </div>

          <UserMenu />
        </div>
      </div>
    </header>
  );
}
