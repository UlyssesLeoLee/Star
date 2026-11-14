/*
@cypher schema=1 source_sha256=8d1cde7f3e9f96dd99e5c07a0384e71b537f99499ae212d59e6b74ddea0442a7
MERGE (self:File {path:"frontend/src/app/api/validation/results/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/validation/results/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_VALIDATION_RESULTS } from "@/mocks/data/validation";

export async function GET() {
  return NextResponse.json(MOCK_VALIDATION_RESULTS);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  return NextResponse.json(body, { status: 201 });
}
