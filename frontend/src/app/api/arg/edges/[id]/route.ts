/*
@cypher schema=1 source_sha256=05891ed3f67875d72a392f56484aceb11ad975dc6f86af69e42022d363f9bf95
MERGE (self:File {path:"frontend/src/app/api/arg/edges/[id]/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/arg/edges/[id]/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";

export async function GET(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return NextResponse.json({ id, error: "NOT_FOUND" }, { status: 404 });
}

export async function PATCH(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const body = await req.json();
  return NextResponse.json({ id, ...body });
}

export async function DELETE(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return NextResponse.json({ id, archived: true });
}
