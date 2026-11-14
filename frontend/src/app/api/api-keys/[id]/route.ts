/*
@cypher schema=1 source_sha256=cbfe107803b9987276d94aad31df6242b86ce18b839762669e2e03963a6df11f
MERGE (self:File {path:"frontend/src/app/api/api-keys/[id]/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/api-keys/[id]/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";

export async function DELETE(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return NextResponse.json({ id, deleted: true });
}
