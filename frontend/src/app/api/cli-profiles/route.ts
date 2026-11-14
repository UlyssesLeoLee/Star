/*
@cypher schema=1 source_sha256=d14775e845f55feec3fe4a553ac786e295bc53490df368f4bacccfa52b626c9c
MERGE (self:File {path:"frontend/src/app/api/cli-profiles/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/cli-profiles/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_CLI_PROFILES } from "@/mocks/data/cli";
import { isCliProfile } from "@/mocks/schemas/cli";

export async function GET() {
  return NextResponse.json(MOCK_CLI_PROFILES);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  if (!isCliProfile(body)) return NextResponse.json({ error: "INVALID_CLI_PROFILE" }, { status: 400 });
  return NextResponse.json(body, { status: 201 });
}
