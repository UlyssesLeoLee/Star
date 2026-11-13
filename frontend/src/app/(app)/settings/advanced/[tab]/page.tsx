/*
@cypher schema=1 source_sha256=1cb7e791c9f8bee0582abd06532d93680eb4748567ad63c5a9f9a6a48cd129be
MERGE (self:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/page.tsx::AdvancedSettingsTabPage",kind:"component"})
MERGE (hooks:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx"})
MERGE (mcp:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx"})
MERGE (panels:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(hooks)
MERGE (self)-[:IMPORTS]->(mcp)
MERGE (self)-[:IMPORTS]->(panels)
@endcypher
*/
/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/page.tsx",type:"file",language:"tsx"}),
  (page:Function {name:"AdvancedSettingsTabPage",type:"function",signature:"AdvancedSettingsTabPage()",visibility:"public",complexity:"moderate"}),
  (placeholder:Function {name:"CapabilityPlaceholder",type:"function",signature:"CapabilityPlaceholder({title,description,registry})",visibility:"private",complexity:"simple"}),
  (params:Variable {name:"params",type:"variable",language:"typescript"}),
  (tab:Variable {name:"tab",type:"variable",language:"typescript"}),
  (page)-[:CALLS]->(placeholder),(page)-[:USES]->(params),(page)-[:USES]->(tab),
  (file)-[:CONTAINS]->(page),(file)-[:CONTAINS]->(placeholder);
*/

"use client";

import { useParams, notFound } from "next/navigation";
import HookPolicyPage from "./HookPolicyPage";
import { McpSettingsPanel } from "./McpSettingsPanel";
import { SkillsSettingsPanel, PluginsSettingsPanel } from "./SkillsPluginsPanels";

export default function AdvancedSettingsTabPage() {
  const params = useParams<{ tab: string }>();
  const tab = params.tab;

  if (tab === "hooks") return <HookPolicyPage />;
  if (tab === "mcp") return <McpSettingsPanel />;
  if (tab === "skills") return <SkillsSettingsPanel />;
  if (tab === "plugins") return <PluginsSettingsPanel />;
  return notFound();
}
