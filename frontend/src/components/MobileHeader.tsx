// Star Mobile Header (per 2026-09-01 PHASE-MOBILE-PWA)
// 仅 <768px 视口显示:汉堡按钮 + 标题 + 通知 icon
// 与 AppHeader(桌面顶栏)互斥,通过 CSS md:hidden 控制。
"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Bell, Menu, Search } from "lucide-react";
import { useState } from "react";
import { useNavStore } from "@/lib/nav/navStore";

const ROUTE_TITLES: { match: RegExp; title: string }[] = [
  { match: /^\/$/, title: "Star" },
  { match: /^\/worktree/, title: "Worktree" },
  { match: /^\/agent/, title: "Agent" },
  { match: /^\/feedback/, title: "Feedback" },
  { match: /^\/notification/, title: "通知" },
  { match: /^\/work-item/, title: "工作项" },
  { match: /^\/projects?/, title: "项目" },
  { match: /^\/board/, title: "看板" },
  { match: /^\/audit/, title: "审计" },
  { match: /^\/search/, title: "搜索" },
  { match: /^\/settings/, title: "设置" },
];

export function MobileHeader() {
  const pathname = usePathname() ?? "/";
  const openCommandBar = useNavStore((s) => s.openMatrix);
  const [bellCount] = useState(3);

  const title = ROUTE_TITLES.find((r) => r.match.test(pathname))?.title ?? "Star";

  return (
    <header
      data-testid="mobile-header"
      className="md:hidden sticky top-0 z-30 h-14 border-b border-line bg-bg/95 backdrop-blur-xl flex items-center px-3 gap-3"
      style={{ paddingTop: "env(safe-area-inset-top)" }}
    >
      <Link
        href="/"
        data-testid="mobile-header-home"
        aria-label="Star home"
        className="flex items-center gap-2 min-w-0"
      >
        <span className="size-7 rounded-lg overflow-hidden border border-accent/40 shrink-0">
          <img
            src="/sidebar-icon.png"
            alt=""
            width={28}
            height={28}
            className="w-full h-full object-cover"
          />
        </span>
        <span className="text-sm font-black tracking-tight text-ink truncate">
          {title}
        </span>
      </Link>

      <div className="ml-auto flex items-center gap-1">
        <button
          type="button"
          onClick={openCommandBar}
          data-testid="mobile-header-search"
          aria-label="Search (Cmd+K)"
          className="p-2 text-ink-dim hover:text-ink rounded-lg hover:bg-bg-soft"
        >
          <Search size={17} />
        </button>
        <Link
          href="/notification"
          data-testid="mobile-header-bell"
          aria-label="Notifications"
          className="relative p-2 text-ink-dim hover:text-ink rounded-lg hover:bg-bg-soft"
        >
          <Bell size={17} />
          {bellCount > 0 && (
            <span className="absolute top-0.5 right-0.5 min-w-[14px] h-3.5 rounded-full bg-err text-white text-[8px] grid place-items-center px-1 font-mono font-bold">
              {bellCount}
            </span>
          )}
        </Link>
        <button
          type="button"
          onClick={openCommandBar}
          data-testid="mobile-header-menu"
          aria-label="Open App Matrix"
          className="p-2 text-ink-dim hover:text-ink rounded-lg hover:bg-bg-soft"
        >
          <Menu size={17} />
        </button>
      </div>
    </header>
  );
}
