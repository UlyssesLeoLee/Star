/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx",type:"file",language:"tsx"}),
 (page:Function {name:"RunWorktreePage",type:"function"}),(shell:Function {name:"RunWorkspaceShell",type:"function"}),
 (route:Class {name:"RunWorkspaceRoute",type:"class"}),
 (f)-[:CONTAINS]->(page),(page)-[:CALLS]->(shell),(page)-[:USES]->(route);
*/

import { RunWorkspaceShell, type RunWorkspaceRoute } from "@/components/run/RunWorkspaceShell";

export default async function RunWorktreePage({ params }: { params: Promise<RunWorkspaceRoute> }) {
  return <RunWorkspaceShell route={await params} />;
}
