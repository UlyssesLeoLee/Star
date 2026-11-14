/*
@cypher schema=1 source_sha256=ed5982032a17fdf763122026e02396972f8ab519574d396893b80628399c9651
MERGE (self:File {path:"frontend/src/app/api/annotations/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/annotations/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { DEMO_DIFF_LINES, DEMO_AGENT_RUN_ID, DEMO_FILE_PATH } from "@/mocks/data/annotations";

export async function GET() {
  return NextResponse.json({
    agentRunId: DEMO_AGENT_RUN_ID,
    filePath: DEMO_FILE_PATH,
    diffLines: DEMO_DIFF_LINES,
  });
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  return NextResponse.json(body, { status: 201 });
}
