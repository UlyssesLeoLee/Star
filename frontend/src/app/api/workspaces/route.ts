/*
@cypher schema=1 source_sha256=4bcdb62aafc9b348828900f077ec9ce1b84118c55be21445cb335c33ca461f93
MERGE (self:File {path:"frontend/src/app/api/workspaces/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/workspaces/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_WORKSPACES } from "@/mocks/data/five-domain";

export async function GET() {
  return NextResponse.json(MOCK_WORKSPACES);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  return NextResponse.json(body, { status: 201 });
}
