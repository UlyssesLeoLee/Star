"use client";

// =====================================================================
// BoardTabs — Board 核心面板的 5 个一级 tab (per 2026-08-29 16:50 JST Ulysses 拍板)
//
// 5 个 tab 对应 5 个产品核心域 (per docs/frontend/design/dynamic-interaction-design.md):
//   - Kanban    /board      (默认, Jira 风格工作流)
//   - Timeline  /planning   (日历 / 路线图)
//   - Backlog   /work-item  (work item 列表 / 详情)
//   - Agents    /agent      (AI agent 控制)
//   - Worktrees /worktree   (git worktree 树)
//
// 纯 URL state (Next.js Link 跨页面),不引 store,符合 scope-ui-only。
// active 判断: 当前 pathname 命中 tab.href 或 tab.href 子路径。
// =====================================================================

import Link from "next/link";
import { usePathname } from "next/navigation";
import { clsx } from "clsx";
import { Trello, Calendar, FileText, Bot, GitBranch } from "lucide-react";

type TabItem = { href: string; label: string; icon: React.ElementType; sub?: string[] };

const TABS: TabItem[] = [
  { href: "/board",     label: "Kanban",    icon: Trello,   sub: ["/board"] },
  { href: "/planning",  label: "Timeline",  icon: Calendar, sub: ["/planning", "/calendar"] },
  { href: "/work-item", label: "Backlog",   icon: FileText, sub: ["/work-item", "/issues"] },
  { href: "/agent",     label: "Agents",    icon: Bot,      sub: ["/agent", "/agents", "/agent-windows"] },
  { href: "/worktree",  label: "Worktrees", icon: GitBranch,sub: ["/worktree"] },
];

export function BoardTabs() {
  const pathname = usePathname();
  return (
    <nav
      data-testid="board-tabs"
      className="flex items-center gap-1 border-b border-line mb-4 overflow-x-auto"
      role="tablist"
    >
      {TABS.map((tab) => {
        const Icon = tab.icon;
        const active = !!pathname && tab.sub.some((p) => p === pathname || pathname.startsWith(p + "/"));
        return (
          <Link
            key={tab.href}
            href={tab.href}
            role="tab"
            aria-selected={active}
            data-testid={`board-tab-${tab.label.toLowerCase()}`}
            className={clsx(
              "relative flex items-center gap-2 px-4 py-2.5 text-sm transition-all duration-150 whitespace-nowrap",
              "border-b-2 -mb-px",
              active
                ? "text-accent border-accent font-medium"
                : "text-ink-dim border-transparent hover:text-ink hover:border-line",
            )}
          >
            <Icon size={14} className={active ? "text-accent" : "text-ink-mute"} />
            <span>{tab.label}</span>
          </Link>
        );
      })}
    </nav>
  );
}
