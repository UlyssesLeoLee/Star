/*
@cypher schema=1 source_sha256=368928452b1c5d44442d961ea4ae3dbf02fb604a58915d4c607f528ff403ea46
MERGE (self:File {path:"frontend/src/app/api/design-artifacts/[id]/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/design-artifacts/[id]/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";

export async function PATCH(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const body = await req.json();
  return NextResponse.json({ id, ...body });
}
