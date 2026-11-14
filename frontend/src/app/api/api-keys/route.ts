/*
@cypher schema=1 source_sha256=f036d431f8156a1900784de6895ea669d4a1e8bccb69649144d6edaa98c0aaf6
MERGE (self:File {path:"frontend/src/app/api/api-keys/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/api-keys/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_API_KEYS } from "@/mocks/data/cli";
import { isApiKey } from "@/mocks/schemas/cli";

export async function GET() {
  return NextResponse.json(MOCK_API_KEYS);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  if (!isApiKey(body)) return NextResponse.json({ error: "INVALID_API_KEY" }, { status: 400 });
  return NextResponse.json(body, { status: 201 });
}
