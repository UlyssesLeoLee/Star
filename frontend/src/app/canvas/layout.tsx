/*
@cypher schema=1 source_sha256=deedead3cc6314cd7edc49ae7042ce3b4d28ef4fd115724471b81c6036962be2
MERGE (self:File {path:"frontend/src/app/canvas/layout.tsx"})
MERGE (layout:Symbol {id:"frontend/src/app/canvas/layout.tsx::CanvasLayout",kind:"component"})
MERGE (self)-[:DEFINES]->(layout)
@endcypher
*/
// =====================================================================
// canvas layout — AppShell 挂载 (修: 顶层路由不走 (app) layout, 顶栏消失)
// 无限画布类页面 → fullBleed (per 2026-11-14 用户反馈 "Canvas 打开后顶端导航条消失")
// =====================================================================
"use client";

import { ReactNode } from "react";
import { AppShell } from "@/components/AppShell";

export default function CanvasLayout({ children }: { children: ReactNode }) {
  return <AppShell fullBleed>{children}</AppShell>;
}
