/*
@cypher schema=1 source_sha256=3fb63e8fd177e52853b6484ba03d0d5b8ac4ee3153c0206eebee15a3c8a98071
MERGE (self:File {path:"frontend/src/app/api/gemarc/route.ts"})
MERGE (handler:Symbol {id:"frontend/src/app/api/gemarc/route.ts::POST",kind:"function"})
MERGE (self)-[:DEFINES]->(handler)
@endcypher
*/
// =====================================================================
// /api/gemarc — GemArc MCP Streamable HTTP 代理
// 后端 = 本地 GemArc (E:\GemArc, uvicorn 127.0.0.1:8760, POST /mcp)
// per INTEGRATION-MAP-GITHUB-REPOS-001 §2.1 (2026-11-14 用户授权集成)
// 前端 /prototype 通过本代理调用 gemarc_brief/validate/simulate/build/verify
// =====================================================================

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

// 双通道: 本地 GemArc (full toolset, 当前 Branch 的接口) / 远端 Vercel (stateless, 示例数据)
// 远端 = https://gem-arc.vercel.app/mcp (用户 2026-11-14 提供)
const GEMARC_LOCAL = process.env.GEMARC_MCP_URL ?? "http://127.0.0.1:8760/mcp";
const GEMARC_REMOTE = process.env.GEMARC_REMOTE_URL ?? "https://gem-arc.vercel.app/mcp";
const GEMARC_REMOTE_TOKEN = process.env.GEMARC_REMOTE_TOKEN ?? "";

/** 只代理 MCP 工具调用 (tools/call), 白名单工具防滥用 */
const ALLOWED_TOOLS = new Set([
  "gemarc_brief",
  "gemarc_contracts",
  "gemarc_assets",
  "gemarc_new_model",
  "gemarc_validate",
  "gemarc_simulate",
  "gemarc_diff",
  "gemarc_render",
  "gemarc_build",
  "gemarc_verify",
  "gemarc_inspect",
  "gemarc_media_doc_guide",
]);

export async function POST(request: Request) {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ error: "invalid JSON body" }, { status: 400 });
  }

  // 白名单校验: 只放行 tools/call + 白名单工具
  const b = body as { method?: string; params?: { name?: string } };
  if (b?.method === "tools/call" && b?.params?.name) {
    if (!ALLOWED_TOOLS.has(b.params.name)) {
      return Response.json({ error: `tool not allowed via proxy: ${b.params.name}` }, { status: 403 });
    }
  } else if (b?.method !== "initialize" && b?.method !== "tools/list") {
    return Response.json({ error: `method not allowed via proxy: ${b?.method}` }, { status: 403 });
  }

  // 数据来源: body.params._meta.source === "remote" → 远端 stateless; 默认本地 full
  const b2 = body as { params?: { _meta?: { source?: string } } };
  const useRemote = b2?.params?._meta?.source === "remote";
  const target = useRemote ? GEMARC_REMOTE : GEMARC_LOCAL;

  const upstream = await fetch(target, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json, text/event-stream",
      ...(useRemote && GEMARC_REMOTE_TOKEN ? { Authorization: `Bearer ${GEMARC_REMOTE_TOKEN}` } : {}),
    },
    body: JSON.stringify(body),
  });

  return new Response(upstream.body, {
    status: upstream.status,
    headers: {
      "Content-Type": upstream.headers.get("Content-Type") ?? "application/json",
      "MCP-Session-Id": upstream.headers.get("MCP-Session-Id") ?? "",
    },
  });
}

export async function GET() {
  const [local, remote] = await Promise.all([
    fetch(GEMARC_LOCAL.replace("/mcp", "/healthz"), { signal: AbortSignal.timeout(4000) })
      .then((r) => (r.ok ? r.json() : null))
      .catch(() => null),
    fetch(GEMARC_REMOTE.replace("/mcp", "/healthz"), { signal: AbortSignal.timeout(8000) })
      .then((r) => (r.ok ? r.json() : null))
      .catch(() => null),
  ]);
  return Response.json({
    local: local ? { ...(local as object), url: GEMARC_LOCAL } : { status: "down" },
    remote: remote ? { ...(remote as object), url: GEMARC_REMOTE } : { status: "down" },
  });
}
