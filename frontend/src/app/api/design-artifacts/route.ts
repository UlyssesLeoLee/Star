/*
@cypher schema=1 source_sha256=5c5e85ed79aab3829311de98b23a059c0d69873bb4fd8e9c5c5a9e49dd200697
MERGE (self:File {path:"frontend/src/app/api/design-artifacts/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/design-artifacts/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_DESIGN_ARTIFACTS } from "@/mocks/data/design-artifacts";

export async function GET() {
  return NextResponse.json(MOCK_DESIGN_ARTIFACTS);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  return NextResponse.json(body, { status: 201 });
}
