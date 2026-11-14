/*
@cypher schema=1 source_sha256=7619afc27f385095e5b34ced2515ac2070cb56a30f8d6c4f80ae0d6b98245691
MERGE (self:File {path:"frontend/src/app/workflow/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/workflow/page.tsx::WorkflowPage",kind:"component"})
MERGE (canvasPage:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasDetailPage",kind:"component"})
MERGE (self)-[:REDIRECTS_TO]->(canvasPage)
@endcypher
*/
"use client";

// =====================================================================
// /workflow — per 2026-11-14 用户发令: 工作流内容并入 canvas 低代码流程标签
// 本路由保留为兼容入口, redirect 到合并后的画布页
// =====================================================================

import { useEffect } from "react";
import { useRouter } from "next/navigation";

export default function WorkflowPage() {
  const router = useRouter();
  useEffect(() => {
    router.replace("/canvas/canvas-001");
  }, [router]);
  return null;
}
