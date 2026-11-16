/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/api/agents/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/agents/route.ts::Route",kind:"route"})
MERGE (backend:ExternalService {name:"star-api-rest"})
MERGE (self)-[:DEFINES]->(r)
MERGE (self)-[:PROXIES]->(backend)
@endcypher
*/

// =====================================================================
// /api/agents — 诚实代理真实 Agent 运行时 (per 2026-11-17 清 mock 纪律)
//
// 原实现直接返回 MOCK_AGENTS 冒充真实 API 数据; 现改为转发
// star-api-rest 的 GET /api/v1/agents, 后端不可达时返回 502 诚实报错。
// (Agent 中心页面本体走同源 /api/v1/agents rewrite, 不经过本路由;
//  本路由保留给未来消费方, 不再提供 mock 兜底。)
// =====================================================================

import { NextResponse } from "next/server";

const BACKEND_ORIGIN = process.env.STAR_API_BACKEND_ORIGIN ?? "http://127.0.0.1:8081";

export async function GET() {
  try {
    const res = await fetch(`${BACKEND_ORIGIN}/api/v1/agents`, { cache: "no-store" });
    const text = await res.text();
    return new NextResponse(text, {
      status: res.status,
      headers: {
        "content-type": res.headers.get("content-type") ?? "application/json",
      },
    });
  } catch {
    return NextResponse.json(
      { error: "agent_runtime_unreachable", detail: "真实 Agent 运行时不可达" },
      { status: 502 },
    );
  }
}
