/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/tasks/api/tmo/spawn_subtask/route.ts"})
@endcypher
*/
/**
 * /tasks/api/tmo/spawn_subtask proxy → FastAPI 8080
 * (per docs/architecture/2026-09-09-subtask-binding/02-basic-design.md §5 第 9 端点
 *  + 守门 #24 v2: 浏览器 → Next.js API route → console_server.py → subprocess)
 *
 * manifest 路径 per docs/briefs/wt-sb-01-subtask-binding-impl.md §3.1:
 *   frontend/src/app/tasks/api/tmo/spawn_subtask/route.ts (对应 URL /tasks/api/tmo/spawn_subtask)
 *
 * 守门 #12 缺标比错标: 任何失败路径必返回 status="failed" + failure_reason.
 */

import { NextRequest, NextResponse } from "next/server";

const FASTAPI_BASE = process.env.FASTAPI_BASE || "http://127.0.0.1:8080";

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const res = await fetch(`${FASTAPI_BASE}/api/tmo/spawn_subtask`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    });
    if (!res.ok) {
      return NextResponse.json(
        { status: "failed", failure_reason: `FastAPI ${res.status}` },
        { status: res.status }
      );
    }
    return NextResponse.json(await res.json());
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : String(e);
    return NextResponse.json(
      { status: "failed", failure_reason: `Failed to proxy to FastAPI: ${message}` },
      { status: 502 }
    );
  }
}
