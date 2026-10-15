/*
@cypher schema=1 source_sha256=6938ef2a16ae63c61264baca73e457f3a13889b7dccf09acf4232b8dc66160af
MERGE (self:File {path:"frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx::RunWorktreePage",kind:"function"})
MERGE (shell:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::RunWorkspaceShell",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
MERGE (page)-[:CALLS]->(shell)
@endcypher
*/

/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx",type:"file",language:"tsx"}),
 (page:Function {name:"RunWorktreePage",type:"function"}),(shell:Function {name:"RunWorkspaceShell",type:"function"}),
 (route:Class {name:"RunWorkspaceRoute",type:"class"}),
 (f)-[:CONTAINS]->(page),(page)-[:CALLS]->(shell),(page)-[:USES]->(route);
*/

import { RunWorkspaceShell, type RunWorkspaceRoute } from "@/components/run/RunWorkspaceShell";

export default async function RunWorktreePage({ params }: { params: Promise<RunWorkspaceRoute> }) {
  return <RunWorkspaceShell route={await params} />;
}
