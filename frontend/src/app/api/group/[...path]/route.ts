/*
@cypher schema=1 source_sha256=81187c97296ba9cabf85c93b3953aaa4193e1440606ee6db8306a7863642492b
MERGE (self:File {path:"frontend/src/app/api/group/[...path]/route.ts"});
CREATE
  (ctx:Class {name:"RouteContext",type:"class",language:"typescript"}),
  (methods:Variable {name:"ALLOWED_METHODS",type:"variable",language:"typescript"}),
  (headers:Variable {name:"FORWARDED_HEADERS",type:"variable",language:"typescript"}),
  (hopHeaders:Variable {name:"HOP_BY_HOP_HEADERS",type:"variable",language:"typescript"}),
  (bodyLimit:Variable {name:"MAX_REQUEST_BODY_BYTES",type:"variable",language:"typescript"}),
  (allowPath:Function {name:"isAllowedGroupApiPath",type:"function",signature:"isAllowedGroupApiPath(method: string, path: string[]): boolean",visibility:"private",complexity:"simple"}),
  (readBody:Function {name:"readBoundedBody",type:"function",signature:"readBoundedBody(request: Request): Promise<{body: ArrayBuffer | null; tooLarge: boolean}>",visibility:"private",complexity:"moderate"}),
  (proxy:Function {name:"proxyGroupRequest",type:"function",signature:"proxyGroupRequest(request: NextRequest, context: RouteContext): Promise<Response>",visibility:"private",complexity:"complex"}),
  (get:Function {name:"GET",type:"function",signature:"GET(request: NextRequest, context: RouteContext): Promise<Response>",visibility:"public",complexity:"simple"}),
  (post:Function {name:"POST",type:"function",signature:"POST(request: NextRequest, context: RouteContext): Promise<Response>",visibility:"public",complexity:"simple"}),
  (put:Function {name:"PUT",type:"function",signature:"PUT(request: NextRequest, context: RouteContext): Promise<Response>",visibility:"public",complexity:"simple"}),
  (patch:Function {name:"PATCH",type:"function",signature:"PATCH(request: NextRequest, context: RouteContext): Promise<Response>",visibility:"public",complexity:"simple"}),
  (del:Function {name:"DELETE",type:"function",signature:"DELETE(request: NextRequest, context: RouteContext): Promise<Response>",visibility:"public",complexity:"simple"}),
  (self)-[:CONTAINS]->(methods),
  (self)-[:CONTAINS]->(headers),
  (self)-[:CONTAINS]->(ctx),
  (self)-[:CONTAINS]->(hopHeaders),
  (self)-[:CONTAINS]->(bodyLimit),
  (self)-[:CONTAINS]->(allowPath),
  (self)-[:CONTAINS]->(readBody),
  (self)-[:CONTAINS]->(proxy),
  (self)-[:CONTAINS]->(get),
  (self)-[:CONTAINS]->(post),
  (self)-[:CONTAINS]->(put),
  (self)-[:CONTAINS]->(patch),
  (self)-[:CONTAINS]->(del),
  (proxy)-[:USES]->(methods),
  (proxy)-[:USES]->(headers),
  (proxy)-[:USES]->(hopHeaders),
  (proxy)-[:CALLS]->(allowPath),
  (proxy)-[:CALLS]->(readBody),
  (readBody)-[:USES]->(bodyLimit);
@endcypher
*/

import { cookies } from "next/headers";
import type { NextRequest } from "next/server";

type RouteContext = { params: Promise<{ path: string[] }> };

const ALLOWED_METHODS = new Set(["GET", "POST", "PUT", "PATCH", "DELETE"]);
const FORWARDED_HEADERS = [
  "accept",
  "content-type",
  "idempotency-key",
  "if-match",
  "x-correlation-id",
];
const HOP_BY_HOP_HEADERS = [
  "connection",
  "content-encoding",
  "content-length",
  "keep-alive",
  "proxy-authenticate",
  "proxy-authorization",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
  "set-cookie",
];
const MAX_REQUEST_BODY_BYTES = 2_000_000;

function isAllowedGroupApiPath(method: string, path: string[]): boolean {
  const isWorktreeResource = path[0] === "worktrees" && path.length >= 2;
  const isProjectWorktreeIndex =
    method === "GET" &&
    path[0] === "projects" &&
    path.length === 3 &&
    path[2] === "worktrees";
  return isWorktreeResource || isProjectWorktreeIndex;
}

async function readBoundedBody(
  request: Request,
): Promise<{ body: ArrayBuffer | null; tooLarge: boolean }> {
  const reader = request.body?.getReader();
  if (!reader) return { body: null, tooLarge: false };

  const chunks: Uint8Array<ArrayBufferLike>[] = [];
  let totalBytes = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    totalBytes += value.byteLength;
    if (totalBytes > MAX_REQUEST_BODY_BYTES) {
      await reader.cancel();
      return { body: null, tooLarge: true };
    }
    chunks.push(value);
  }

  const buffer = new Uint8Array(totalBytes);
  let offset = 0;
  for (const chunk of chunks) {
    buffer.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return { body: buffer.buffer, tooLarge: false };
}

async function proxyGroupRequest(request: NextRequest, context: RouteContext): Promise<Response> {
  const method = request.method.toUpperCase();
  if (!ALLOWED_METHODS.has(method)) return Response.json({ error: { code: "method_not_allowed" } }, { status: 405 });

  if (method !== "GET") {
    const origin = request.headers.get("origin");
    if (!origin || origin !== request.nextUrl.origin) {
      return Response.json({ error: { code: "cross_origin_request_denied" } }, { status: 403 });
    }
  }

  const cookieName = process.env.GROUP_ACCESS_TOKEN_COOKIE || "star_access_token";
  if (!/^[A-Za-z0-9_-]{1,64}$/.test(cookieName)) {
    return Response.json({ error: { code: "group_auth_configuration_error" } }, { status: 500 });
  }
  const accessToken = (await cookies()).get(cookieName)?.value;
  if (!accessToken || accessToken.length > 8192) {
    return Response.json({ error: { code: "authentication_required" } }, { status: 401 });
  }

  const apiBase = process.env.GROUP_API_BASE_URL;
  if (!apiBase) return Response.json({ error: { code: "group_api_unavailable" } }, { status: 503 });

  let upstream: URL;
  try {
    upstream = new URL(apiBase);
  } catch {
    return Response.json({ error: { code: "group_api_configuration_error" } }, { status: 500 });
  }
  const isLoopback = ["localhost", "127.0.0.1", "::1"].includes(upstream.hostname);
  const insecureDevelopmentTarget =
    process.env.NODE_ENV !== "production" &&
    upstream.protocol === "http:" &&
    isLoopback;
  if (
    (!insecureDevelopmentTarget && upstream.protocol !== "https:") ||
    upstream.username ||
    upstream.password ||
    upstream.search ||
    upstream.hash
  ) {
    return Response.json({ error: { code: "group_api_configuration_error" } }, { status: 500 });
  }

  const { path } = await context.params;
  const resourcePath = path.slice(2);
  if (
    path[0] !== "api" ||
    path[1] !== "v1" ||
    !isAllowedGroupApiPath(method, resourcePath) ||
    path.some(
      (segment) =>
        !segment ||
        segment === "." ||
        segment === ".." ||
        segment.includes("/") ||
        segment.includes("\\"),
    )
  ) {
    return Response.json({ error: { code: "invalid_group_api_path" } }, { status: 400 });
  }

  const basePath = upstream.pathname.replace(/\/$/, "");
  upstream.pathname = `${basePath}/api/v1/${resourcePath.map(encodeURIComponent).join("/")}`;
  upstream.search = request.nextUrl.search;

  const headers = new Headers();
  for (const name of FORWARDED_HEADERS) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  headers.set("authorization", `Bearer ${accessToken}`);

  const contentLength = Number(request.headers.get("content-length") ?? "0");
  if (Number.isFinite(contentLength) && contentLength > MAX_REQUEST_BODY_BYTES) {
    return Response.json({ error: { code: "request_body_too_large" } }, { status: 413 });
  }
  let body: ArrayBuffer | undefined;
  if (method !== "GET") {
    let bodyResult: { body: ArrayBuffer | null; tooLarge: boolean };
    try {
      bodyResult = await readBoundedBody(request);
    } catch {
      return Response.json({ error: { code: "request_body_unreadable" } }, { status: 400 });
    }
    if (bodyResult.tooLarge) {
      return Response.json({ error: { code: "request_body_too_large" } }, { status: 413 });
    }
    body = bodyResult.body ?? undefined;
  }
  try {
    const response = await fetch(upstream, {
      method,
      headers,
      body,
      cache: "no-store",
      redirect: "manual",
      signal: request.signal,
    });
    if (response.status >= 300 && response.status < 400) {
      return Response.json({ error: { code: "group_api_redirect_rejected" } }, { status: 502 });
    }
    const responseHeaders = new Headers(response.headers);
    for (const name of HOP_BY_HOP_HEADERS) responseHeaders.delete(name);
    responseHeaders.set("cache-control", "no-store");
    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers: responseHeaders,
    });
  } catch {
    return Response.json({ error: { code: "group_api_unavailable" } }, { status: 502 });
  }
}

export async function GET(request: NextRequest, context: RouteContext): Promise<Response> {
  return proxyGroupRequest(request, context);
}

export async function POST(request: NextRequest, context: RouteContext): Promise<Response> {
  return proxyGroupRequest(request, context);
}

export async function PUT(request: NextRequest, context: RouteContext): Promise<Response> {
  return proxyGroupRequest(request, context);
}

export async function PATCH(request: NextRequest, context: RouteContext): Promise<Response> {
  return proxyGroupRequest(request, context);
}

export async function DELETE(request: NextRequest, context: RouteContext): Promise<Response> {
  return proxyGroupRequest(request, context);
}
