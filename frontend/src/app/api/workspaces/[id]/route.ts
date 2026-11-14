/*
@cypher schema=1 source_sha256=5367421a8540f2860e100d72a2ba39a22a09fcfddc7104e1cadd29fa60e452b9
MERGE (self:File {path:"frontend/src/app/api/workspaces/[id]/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/workspaces/[id]/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_WORKSPACES } from "@/mocks/data/five-domain";

export async function GET(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const found = MOCK_WORKSPACES.find((w) => w.id === id);
  return found ? NextResponse.json(found) : NextResponse.json({ error: "NOT_FOUND" }, { status: 404 });
}
