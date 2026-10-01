// CYPHER STRUCTURAL MANIFEST
// CREATE
//   (file:File {name:"ProjectWorktreeIndexLink.tsx",type:"file",language:"typescript"}),
//   (module:Module {name:"ProjectWorktreeIndexLink",type:"module",language:"typescript"}),
//   (props:Class {name:"ProjectWorktreeIndexLinkProps",type:"class",language:"typescript",visibility:"private"}),
//   (component:Function {name:"ProjectWorktreeIndexLink",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
//   (projectId:Variable {name:"projectId",type:"variable",language:"typescript"}),
//   (active:Variable {name:"active",type:"variable",language:"typescript"}),
//   (collapsed:Variable {name:"collapsed",type:"variable",language:"typescript"}),
//   (href:Variable {name:"href",type:"variable",language:"typescript"}),
//   (encodeURIComponent:Function {name:"encodeURIComponent",type:"function",language:"typescript"}),
//   (clsx:Function {name:"clsx",type:"function",language:"typescript"}),
//   (Link:Function {name:"Link",type:"function",language:"typescript"}),
//   (FolderKanban:Function {name:"FolderKanban",type:"function",language:"typescript"}),
//   (file)-[:CONTAINS]->(module),(module)-[:CONTAINS]->(props),(module)-[:CONTAINS]->(component),(component)-[:USES]->(props),
//   (component)-[:USES]->(projectId),(component)-[:USES]->(active),(component)-[:USES]->(collapsed),(component)-[:USES]->(href),
//   (component)-[:CALLS]->(encodeURIComponent),(component)-[:CALLS]->(clsx),(component)-[:CALLS]->(Link),(component)-[:CALLS]->(FolderKanban);

import Link from "next/link";
import { FolderKanban } from "lucide-react";
import { clsx } from "clsx";

interface ProjectWorktreeIndexLinkProps {
  projectId: string;
  active: boolean;
  collapsed: boolean;
}

export function ProjectWorktreeIndexLink({
  projectId,
  active,
  collapsed,
}: ProjectWorktreeIndexLinkProps) {
  const href = projectId
    ? `/worktree?project_id=${encodeURIComponent(projectId)}`
    : "/worktree";

  return (
    <div className="mt-4 border-t border-line/60 pt-3">
      {!collapsed && (
        <div className="px-2.5 py-1 text-[10px] font-mono uppercase tracking-wider text-ink-mute">
          Project
        </div>
      )}
      <Link
        href={href}
        data-testid="sidebar-project-worktree-index"
        data-active={active ? "true" : "false"}
        aria-current={active ? "page" : undefined}
        aria-label="Project Worktree Index"
        title="Project Worktree Index"
        className={clsx(
          "relative flex items-center rounded-xl text-sm font-medium transition-all duration-200",
          collapsed ? "justify-center p-1.5" : "gap-2.5 px-2.5 py-2",
          active
            ? "bg-bg-soft/80 text-ink border border-line shadow-soft font-semibold"
            : "text-ink-dim hover:bg-bg-soft/60 hover:text-ink border border-transparent",
        )}
      >
        <span
          className={clsx(
            "grid place-items-center shrink-0 rounded-lg border border-[var(--cel-ink)] bg-[var(--cel-cyan,#00f0ff)]/10 text-[var(--cel-cyan,#00f0ff)]",
            collapsed ? "size-9" : "size-8",
          )}
          aria-hidden="true"
        >
          <FolderKanban size={collapsed ? 16 : 15} strokeWidth={2.25} />
        </span>
        {!collapsed && <span className="truncate tracking-tight">Project Worktree Index</span>}
      </Link>
    </div>
  );
}
