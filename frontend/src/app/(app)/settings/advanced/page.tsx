/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/page.tsx",type:"file",language:"tsx"}),
  (page:Function {name:"AdvancedSettingsIndexPage",type:"function",signature:"AdvancedSettingsIndexPage()",visibility:"public",complexity:"simple"}),
  (file)-[:CONTAINS]->(page);
*/

import { redirect } from "next/navigation";

export default function AdvancedSettingsIndexPage() {
  redirect("/settings/advanced/hooks");
}
