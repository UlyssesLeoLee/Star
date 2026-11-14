/*
@cypher schema=1 source_sha256=4f66965f6edcd65af274d812ab2243e5fe29470d29d993e12fb3c443831dae9a
MERGE (self:File {path:"frontend/src/app/api/agents/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/agents/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextResponse } from "next/server";
import { MOCK_AGENTS } from "@/mocks/data/agents";

export async function GET() {
  return NextResponse.json(MOCK_AGENTS);
}
