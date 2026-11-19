/*
@cypher schema=1 source_sha256=5ea829e71278c6d13fd2eb7bf775b280923b81495716c1372692bbf747c5950c
MERGE (self:File {path:"frontend/src/app/api/run-directory/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/run-directory/route.ts::GET",kind:"route"})
MERGE (seed:File {path:"frontend/src/lib/seed.ts"})
MERGE (self)-[:DEFINES]->(r)
MERGE (r)-[:READS]->(seed)
@endcypher
*/
// WBS-NAV-TREE (2026-11-19): Project 树形导航降级目录 — 与 MSW mock 同数据源
//
// 背景: 线上 production build 不启动 MSW browser worker (instrumentation.ts
// 门控 NEXT_PUBLIC_API_MOCKING), 网页版数据走 Next route handler
// (先例: /api/cli-profiles)。本路由为 FallbackProjectRunTree 的
// useFallbackRunDirectory 提供 Project → Lane → Worktree 层级,
// 数据源复用 seed.ts 既有 fixture, 不新造数据。
// real-mode 开启时由 hook 侧短路, 本路由只服务演示降级通道。

import { NextResponse } from "next/server";
import { lanes, projects, worktrees } from "@/lib/seed";

export async function GET() {
  return NextResponse.json({ projects, lanes, worktrees });
}
