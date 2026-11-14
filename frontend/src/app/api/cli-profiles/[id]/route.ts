/*
@cypher schema=1 source_sha256=7465f8fe77c30d120c2f1aeb136e23aacfb899c0c6ed9a7e2d8e1b131acc7a33
MERGE (self:File {path:"frontend/src/app/api/cli-profiles/[id]/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/cli-profiles/[id]/route.ts::Route",kind:"route"})
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

export async function DELETE(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return NextResponse.json({ id, deleted: true });
}
