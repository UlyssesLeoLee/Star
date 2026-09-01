// Star Mobile Bottom Navigation (per 2026-09-01 PHASE-MOBILE-PWA)
// 仅在 <768px 视口显示,与 Sidebar 互斥。
// 5 个核心入口:Dashboard / Worktree / Agent / Feedback / More
"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { clsx } from "clsx";
import {
  Home,
  GitBranch,
  Bot,
  MessageSquare,
  MoreHorizontal,
} from "lucide-react";
import { useState } from "react";

interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: typeof Home;
  match: (pathname: string) => boolean;
}

const ITEMS: NavItem[] = [
  {
    id: "home",
    label: "首页",
    href: "/",
    icon: Home,
    match: (p) => p === "/" || p === "",
  },
  {
    id: "worktree",
    label: "Worktree",
    href: "/worktree",
    icon: GitBranch,
    match: (p) => p?.startsWith("/worktree") ?? false,
  },
  {
    id: "agent",
    label: "Agent",
    href: "/agent",
    icon: Bot,
    match: (p) => p?.startsWith("/agent") ?? false,
  },
  {
    id: "feedback",
    label: "Feedback",
    href: "/feedback",
    icon: MessageSquare,
    match: (p) => p?.startsWith("/feedback") ?? false,
  },
  {
    id: "more",
    label: "更多",
    href: "#more",
    icon: MoreHorizontal,
    match: () => false,
  },
];

export function MobileBottomNav() {
  const pathname = usePathname() ?? "/";
  const [moreOpen, setMoreOpen] = useState(false);

  // "更多" 入口跳出一张速查表(9 个常去页面)
  const moreItems: { label: string; href: string }[] = [
    { label: "项目 Projects", href: "/projects" },
    { label: "工作项 WorkItem", href: "/work-item" },
    { label: "看板 Board", href: "/board" },
    { label: "通知 Notification", href: "/notification" },
    { label: "审计 Audit", href: "/audit" },
    { label: "搜索 ⌘K", href: "/search" },
    { label: "权限 Permission", href: "/permission" },
    { label: "本地 Runtime", href: "/local-runtime" },
    { label: "设置 Settings", href: "/settings" },
  ];

  return (
    <>
      <nav
        data-testid="mobile-bottom-nav"
        aria-label="Mobile primary navigation"
        className="md:hidden fixed bottom-0 inset-x-0 z-40 border-t border-line bg-bg/95 backdrop-blur-xl"
        style={{ paddingBottom: "env(safe-area-inset-bottom)" }}
      >
        <ul className="grid grid-cols-5 h-14">
          {ITEMS.map((item) => {
            const active = item.match(pathname);
            const Icon = item.icon;
            const isMore = item.id === "more";
            return (
              <li key={item.id}>
                {isMore ? (
                  <button
                    type="button"
                    onClick={() => setMoreOpen(true)}
                    data-testid={`mobile-nav-${item.id}`}
                    className="w-full h-full flex flex-col items-center justify-center gap-0.5 text-[10px] font-medium text-ink-dim hover:text-ink transition-colors"
                    aria-label="More navigation"
                  >
                    <Icon size={18} />
                    <span>{item.label}</span>
                  </button>
                ) : (
                  <Link
                    href={item.href}
                    data-testid={`mobile-nav-${item.id}`}
                    aria-current={active ? "page" : undefined}
                    className={clsx(
                      "w-full h-full flex flex-col items-center justify-center gap-0.5 text-[10px] font-medium transition-colors",
                      active ? "text-accent" : "text-ink-dim hover:text-ink",
                    )}
                  >
                    <Icon
                      size={18}
                      className={active ? "drop-shadow-[0_0_6px_rgba(0,240,255,0.6)]" : ""}
                    />
                    <span>{item.label}</span>
                  </Link>
                )}
              </li>
            );
          })}
        </ul>
      </nav>

      {/* "更多" 抽屉 */}
      {moreOpen && (
        <div
          data-testid="mobile-more-drawer"
          className="md:hidden fixed inset-0 z-50 flex items-end"
          onClick={() => setMoreOpen(false)}
        >
          <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" />
          <div
            className="relative w-full max-h-[80vh] overflow-y-auto rounded-t-2xl border-t border-line bg-bg-soft p-4 pb-8"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="w-12 h-1 bg-line rounded-full mx-auto mb-4" />
            <h2 className="text-xs font-mono uppercase tracking-widest text-ink-mute px-2 mb-3">
              所有模块
            </h2>
            <ul className="grid grid-cols-3 gap-2">
              {moreItems.map((it) => (
                <li key={it.href}>
                  <Link
                    href={it.href}
                    onClick={() => setMoreOpen(false)}
                    data-testid={`mobile-more-${it.href.replace(/\W+/g, "-")}`}
                    className="block px-3 py-3 rounded-xl border border-line bg-bg/50 text-xs text-ink-dim hover:text-ink hover:border-accent/40 transition-colors text-center"
                  >
                    {it.label}
                  </Link>
                </li>
              ))}
            </ul>
            <button
              type="button"
              onClick={() => setMoreOpen(false)}
              className="w-full mt-4 py-3 rounded-xl border border-line text-xs text-ink-dim hover:text-ink"
            >
              关闭
            </button>
          </div>
        </div>
      )}
    </>
  );
}
