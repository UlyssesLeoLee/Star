"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard, FolderTree, Building2, Users, FolderKanban,
  FileText, ShieldCheck, GitBranch, ListTodo, MessageSquare,
  Lock, Bot, MessageCircleWarning, Search, Plug, Workflow,
  Cog, Hammer, GitFork, Calendar, Trello, Server, Network,
  Briefcase, History, Zap,
  ChevronRight, Boxes,
} from "lucide-react";
import { clsx } from "clsx";

type IconType = React.ComponentType<{ size?: number; className?: string }>;
type NavItem = { href: string; label: string; icon: React.ElementType; track: string; core?: boolean };
type NavGroup = { label: string; items: NavItem[] };

const NAV: NavGroup[] = [
  {
    label: "Overview",
    items: [
      { href: "/",            label: "Dashboard",     icon: LayoutDashboard, track: "—",  },
    ],
  },
  {
    label: "Pinned",
    items: [
      { href: "/board",       label: "Board",         icon: Trello,           track: "E", core: true },
    ],
  },
  {
    label: "Foundational (5)",
    items: [
      { href: "/tenant",      label: "Tenant",        icon: Building2,        track: "D" },
      { href: "/project",     label: "Project",       icon: FolderTree,       track: "D" },
      { href: "/identity",    label: "Identity",      icon: Users,            track: "D" },
      { href: "/work-item",   label: "Work Item",     icon: FileText,         track: "D" },
      { href: "/comment",     label: "Comment",       icon: MessageSquare,    track: "D" },
    ],
  },
  {
    label: "Work Management (4)",
    items: [
      { href: "/workflow",     label: "Workflow",      icon: Workflow,         track: "D" },
      { href: "/permission",   label: "Permission",    icon: ShieldCheck,      track: "D" },
      { href: "/development",  label: "Development",   icon: Hammer,           track: "D" },
      { href: "/planning",     label: "Planning",      icon: Calendar,         track: "E" },
    ],
  },
  {
    label: "Worktree / Agent (5)",
    items: [
      { href: "/worktree",     label: "Worktree",      icon: GitBranch,        track: "B" },
      { href: "/agent",        label: "Agent",         icon: Bot,              track: "B" },
      { href: "/feedback",     label: "Feedback",      icon: MessageCircleWarning, track: "B" },
      { href: "/context",      label: "Context",       icon: ListTodo,         track: "B" },
      { href: "/validation",   label: "Validation",    icon: ShieldCheck,      track: "B" },
    ],
  },
  {
    label: "Integration & Search (4)",
    items: [
      { href: "/scm",          label: "SCM",           icon: GitFork,          track: "C" },
      { href: "/integration",  label: "Integration",   icon: Plug,             track: "C" },
      { href: "/notification", label: "Notification",  icon: MessageSquare,    track: "B" },
      { href: "/search",       label: "Search",        icon: Search,           track: "B" },
    ],
  },
  {
    label: "Runtime & Audit (4)",
    items: [
      { href: "/local-runtime", label: "Local Runtime", icon: Server,         track: "E" },
      { href: "/collaboration", label: "Collaboration", icon: Boxes,          track: "E" },
      { href: "/audit",         label: "Audit",         icon: History,        track: "E" },
      { href: "/automation",    label: "Automation",    icon: Zap,            track: "E" },
    ],
  },
  {
    label: "Meta",
    items: [
      { href: "/relation",     label: "Relation",      icon: Network,          track: "E" },
      { href: "/workspace",    label: "Workspace",     icon: Briefcase,        track: "E" },
    ],
  },
];

export function Sidebar() {
  const pathname = usePathname();
  return (
    <aside className="w-60 shrink-0 border-r border-line bg-bg-soft/40 flex flex-col h-screen sticky top-0">
      <nav className="flex-1 overflow-y-auto py-3 text-sm">
        {NAV.map((group) => (
          <div key={group.label} className="mb-2">
            <div className="px-4 py-1.5 text-[10px] uppercase tracking-wider text-ink-mute">
              {group.label}
            </div>
            <ul>
              {group.items.map((item) => {
                const active = item.href === "/" ? pathname === "/" : pathname?.startsWith(item.href);
                const Icon = item.icon;
                return (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className={clsx(
                        "relative flex items-center gap-2.5 px-4 py-1.5 mx-1 rounded-md transition-all duration-150 group",
                        active
                          ? "bg-accent/15 text-accent font-medium shadow-[inset_0_0_12px_rgba(0,240,255,0.08)] border-l-2 border-accent"
                          : item.core
                            ? "text-accent hover:bg-accent/10 border-l-2 border-accent/40"
                            : "text-ink-dim hover:bg-bg-card/70 hover:text-ink border-l-2 border-transparent",
                      )}
                    >
                      <Icon size={15} className={clsx("shrink-0 transition-transform group-hover:scale-110", (active || item.core) ? "text-accent" : "text-ink-dim")} />
                      <span className="flex-1 truncate tracking-tight">{item.label}</span>
                      {item.core && (
                        <span className="text-[8px] font-mono uppercase tracking-wider text-accent/80 px-1 py-0.2 rounded border border-accent/40 bg-accent/5">
                          core
                        </span>
                      )}
                      {item.track !== "—" && !item.core && (
                        <span className="text-[9px] font-mono text-ink-mute px-1 py-0.2 rounded border border-line/50 bg-bg-card/80 group-hover:border-accent/30 transition-colors">
                          TRK_{item.track}
                        </span>
                      )}
                      {active && <ChevronRight size={12} className="text-accent animate-pulse" />}
                    </Link>
                  </li>
                );
              })}
            </ul>
          </div>
        ))}
      </nav>
      <div className="border-t border-line px-4 py-2.5 text-[10px] text-ink-mute font-mono flex items-center justify-between">
        <span>SYS // v0.1.0</span>
        <span className="text-accent/60">ONLINE</span>
      </div>
    </aside>
  );
}
