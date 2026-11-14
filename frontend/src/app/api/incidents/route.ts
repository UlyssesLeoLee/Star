/*
@cypher schema=1 source_sha256=23c088f83019fc855c52fa33904ee627e23abdc50632c878c4e0deacbbcf43d8
MERGE (self:File {path:"frontend/src/app/api/incidents/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/incidents/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextResponse } from "next/server";
import { MOCK_INCIDENTS } from "@/mocks/data/incidents";

export async function GET() {
  return NextResponse.json(MOCK_INCIDENTS);
}
