/*
@cypher schema=1 source_sha256=dcf4ea3a0b52f4903a3dee82bcc677d1902af493aa364df7a828d9859c8cddcf
MERGE (self:File {path:"frontend/src/app/api/notifications/[id]/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/notifications/[id]/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";

export async function PATCH(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return NextResponse.json({ id, read: true });
}
