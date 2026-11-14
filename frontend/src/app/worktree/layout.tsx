/*
@cypher schema=1 source_sha256=ad10b480722874067551d063fbe3913a89af8bdf7bd1ce73a1e96862c2c6b3b2
MERGE (self:File {path:"frontend/src/app/worktree/layout.tsx"})
MERGE (layout:Symbol {id:"frontend/src/app/worktree/layout.tsx::WorktreeLayout",kind:"component"})
MERGE (self)-[:DEFINES]->(layout)
@endcypher
*/
// =====================================================================
// worktree layout — AppShell 挂载 (修: 顶层路由不走 (app) layout, 顶栏消失)
// per 2026-11-14 用户反馈
// =====================================================================
"use client";

import { ReactNode } from "react";
import { AppShell } from "@/components/AppShell";

export default function WorktreeLayout({ children }: { children: ReactNode }) {
  return <AppShell>{children}</AppShell>;
}
