/**
 * /api/scripts proxy → FastAPI 8080
 * (per docs/automation-design.md v0.2 §12.4 Next.js API route)
 *
 * 避免浏览器 CORS: Next.js 服务端 proxy 转发到 FastAPI 8080
 */

import { NextRequest, NextResponse } from "next/server";

const FASTAPI_BASE = process.env.FASTAPI_BASE || "http://127.0.0.1:8080";

export async function GET(_request: NextRequest) {
  try {
    const res = await fetch(`${FASTAPI_BASE}/api/scripts`, { cache: "no-store" });
    if (!res.ok) {
      return NextResponse.json(
        { error: `FastAPI ${res.status}` },
        { status: res.status }
      );
    }
    const data = await res.json();
    return NextResponse.json(data);
  } catch (e: any) {
    return NextResponse.json(
      { error: `Failed to proxy to FastAPI: ${e.message}` },
      { status: 502 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const res = await fetch(`${FASTAPI_BASE}/api/scripts/${body.script_id}/toggle?status=${body.status}`, {
      method: "POST",
    });
    if (!res.ok) {
      return NextResponse.json(
        { error: `FastAPI ${res.status}` },
        { status: res.status }
      );
    }
    return NextResponse.json(await res.json());
  } catch (e: any) {
    return NextResponse.json(
      { error: `Failed to proxy to FastAPI: ${e.message}` },
      { status: 502 }
    );
  }
}
