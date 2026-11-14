/*
@cypher schema=1 source_sha256=0158ffca79f0930d23290df5105c6adeba451039c4d41249c1de288e4c710025
MERGE (self:File {path:"frontend/src/app/api/arg/edges/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/arg/edges/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";

// ARG 边域 (对齐 crates/api/src/arg/controller.rs 13 端点; 真实数据 W2 后端联通后接)
export async function GET() {
  return NextResponse.json([]);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  return NextResponse.json(body, { status: 201 });
}
