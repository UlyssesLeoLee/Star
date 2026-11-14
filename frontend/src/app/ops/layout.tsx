/*
@cypher schema=1 source_sha256=ecd22b4625d86d346137756f4475673fcfb0d3a5b16e3af6891424aab83ecd5b
MERGE (self:File {path:"frontend/src/app/ops/layout.tsx"})
MERGE (layout:Symbol {id:"frontend/src/app/ops/layout.tsx::OpsLayout",kind:"component"})
MERGE (self)-[:DEFINES]->(layout)
@endcypher
*/
// =====================================================================
// ops layout — AppShell 挂载 (修: 顶层路由不走 (app) layout, 顶栏消失)
// per 2026-11-14 用户反馈
// =====================================================================
"use client";

import { ReactNode } from "react";
import { AppShell } from "@/components/AppShell";

export default function OpsLayout({ children }: { children: ReactNode }) {
  return <AppShell fullBleed>{children}</AppShell>;
}
