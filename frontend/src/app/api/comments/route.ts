/*
@cypher schema=1 source_sha256=f622dbb88ffa8373d2f601a70240ee257b24ca37cbe34c44b38e10bd78974627
MERGE (self:File {path:"frontend/src/app/api/comments/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/comments/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_COMMENTS } from "@/mocks/data/five-domain";

export async function GET() {
  return NextResponse.json(MOCK_COMMENTS);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  return NextResponse.json(body, { status: 201 });
}
