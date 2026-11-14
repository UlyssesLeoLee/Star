/*
@cypher schema=1 source_sha256=d767969038b90231a92f1955df44facb75fcb20768253a58176f86d0ab309cd3
MERGE (self:File {path:"frontend/src/app/api/notifications/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/notifications/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextResponse } from "next/server";
import { MOCK_NOTIFS } from "@/mocks/data";

export async function GET() {
  return NextResponse.json(MOCK_NOTIFS);
}
