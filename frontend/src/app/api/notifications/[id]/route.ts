/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/api/notifications/[id]/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/notifications/[id]/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// =====================================================================
// DEPRECATED (2026-11-17): 已读/未读切换改走真实后端
//   PATCH /api/v1/notifications/{id} {read:true|false} (realFetch)。
//   本 mock 路由 (原 PATCH 只回 {id, read:true}) 返回 410 Gone,
//   残留调用方显式失败, 杜绝 mock 写操作冒充真实持久化。
// =====================================================================

import { NextRequest, NextResponse } from "next/server";

const DEPRECATED_BODY = {
  error: {
    code: "deprecated",
    message:
      "frontend /api/notifications/{id} 已废弃 (2026-11-17): 已读状态改由真实后端 PATCH /api/v1/notifications/{id} 维护。请勿依赖本路由的 mock 写操作。",
  },
};

export async function GET() {
  return NextResponse.json(DEPRECATED_BODY, { status: 410 });
}

export async function PATCH(_req: NextRequest) {
  return NextResponse.json(DEPRECATED_BODY, { status: 410 });
}
