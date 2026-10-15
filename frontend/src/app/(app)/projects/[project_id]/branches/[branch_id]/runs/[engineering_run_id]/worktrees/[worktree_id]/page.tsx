/*
@cypher schema=1 source_sha256=eb32dd1c504d86a128cd883dc3d7de288cd5c955d5321796c8a65ecd89f913b9
MERGE (self:File {path:"frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx::LegacyRunWorktreePage",kind:"function"})
MERGE (href:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::canonicalRunWorktreeHref",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (page)-[:CALLS]->(href)
@endcypher
*/

/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx",type:"file",language:"tsx"}),
 (page:Function {name:"LegacyRunWorktreePage",type:"function"}),(href:Function {name:"canonicalRunWorktreeHref",type:"function"}),
 (f)-[:CONTAINS]->(page),(page)-[:CALLS]->(href);
*/

import { notFound, redirect } from "next/navigation";
import { canonicalRunWorktreeHref } from "@/lib/run/runDirectoryApi";

/** Compatibility entry for previously issued URLs that placed Branch before Run. */
export default async function LegacyRunWorktreePage({ params }: {
  params: Promise<{ project_id: string; branch_id: string; engineering_run_id: string; worktree_id: string }>;
}) {
  const { project_id, engineering_run_id, worktree_id } = await params;
  let href: string;
  try {
    href = canonicalRunWorktreeHref(project_id, engineering_run_id, worktree_id);
  } catch {
    notFound();
  }
  redirect(href);
}
