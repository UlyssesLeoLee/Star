/*
@cypher schema=1 source_sha256=56e2e8250522dc6bbd69bcf4fd66c84b3f39b6c1948fcf95cb360528ad8434f3
MERGE (self:File {path:"frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/page.tsx::RunTaskCardsPage",kind:"component"})
MERGE (shell:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::RunWorkspaceShell",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (page)-[:CALLS]->(shell)
@endcypher
*/
/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/page.tsx",type:"file",language:"tsx"}),
 (page:Function {name:"RunTaskCardsPage",type:"function"}),(shell:Function {name:"RunWorkspaceShell",type:"function"}),
 (f)-[:CONTAINS]->(page),(page)-[:CALLS]->(shell);
*/

import { RunWorkspaceShell, type RunWorkspaceRoute } from "@/components/run/RunWorkspaceShell";

export default async function RunTaskCardsPage({ params }: { params: Promise<RunWorkspaceRoute> }) {
  return <RunWorkspaceShell route={await params} />;
}
