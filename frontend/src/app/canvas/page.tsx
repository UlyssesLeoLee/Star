/*
@cypher schema=1 source_sha256=f0f0f8381e97351a757a64c3404137ba8f074f9bd125d27c9ab8e2817545181e
MERGE (self:File {path:"frontend/src/app/canvas/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/page.tsx::CanvasPage",kind:"component"})
MERGE (detail:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasDetailPage",kind:"component"})
MERGE (self)-[:REDIRECTS_TO]->(detail)
@endcypher
*/
// =====================================================================
// /canvas — per 2026-11-14 用户发令: 取消三选一画廊,
// 直接进无限画布 (默认 canvas-001, 标签页切换: 画布/Agent 画布/关系图)
// =====================================================================

import { redirect } from "next/navigation";

export default function CanvasPage() {
  redirect("/canvas/canvas-001");
}
