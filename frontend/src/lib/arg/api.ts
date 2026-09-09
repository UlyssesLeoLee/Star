// =====================================================================
// frontend/src/lib/arg/api.ts — 13 REST 客户端封装 (per ARG.4 §4.12)
// =====================================================================
// Per docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §1
// + docs/architecture/2026-09-03-arg/14-arg-10-arg-frontend.md §1.1
// + crates/api/src/arg/controller.rs v0.1
//
// 13 REST 端点 + 1 WS (WS 在 ws.ts):
//   1. POST   /api/arg/agents                    createAgent
//   2. GET    /api/arg/agents                    listAgents
//   3. GET    /api/arg/agents/{id}               getAgent
//   4. PATCH  /api/arg/agents/{id}               updateAgent
//   5. POST   /api/arg/edges                     createEdge
//   6. GET    /api/arg/edges                     listEdges
//   7. GET    /api/arg/edges/{id}                getEdge
//   8. PATCH  /api/arg/edges/{id}                updateEdge
//   9. DELETE /api/arg/edges/{id}                archiveEdge
//  10. GET    /api/arg/graph                     getGraph
//  11. POST   /api/arg/templates/instantiate     instantiateTemplate
//  12. GET    /api/arg/achievements              listAchievements
//  13. GET    /api/arg/achievements/me           myUnlocks
//  (14. WS    /ws/arg/events                     -- in ws.ts)
//
// 守门合规:
//   - 守门 #5 env 安全: API base URL 走 `process.env.NEXT_PUBLIC_ARG_API_BASE`,
//     客户端代码不读任何 secret
//   - 守门 #9 RPC 不可靠: 失败 retry 走指数 backoff, 默认 0 (mock 模式),
//     真实联调时启用 retry=2
//   - 守门 #13 RLS 13 类: 每个请求都发 `X-Tenant-Id` header (RLS extractor 服务端读)
// =====================================================================

import type {
  Agent,
  AgentFilter,
  ApiErrorBody,
  CreateAgentInput,
  CreateEdgeInput,
  Edge,
  EdgeFilter,
  EvaluateAchievementsResponse,
  GraphFilter,
  GraphResponse,
  InstantiateTemplateInput,
  PaginationQuery,
  TemplateInstance,
  TeamTemplate,
  UpdateAgentPatch,
  UpdateEdgePatch,
  Uuid,
  Achievement,
  AchievementUnlock,
  AchievementCategory,
  Rarity,
} from "./types";

// ----- config -----

/** Read API base URL (per 守门 #5: 走 process.env, 不读 secret). */
function getApiBase(): string {
  // 默认走同源 (Next.js 自己的 backend 反代 /api 跟 /ws)
  // 如要对接独立后端, 设 NEXT_PUBLIC_ARG_API_BASE
  const fromEnv =
    typeof process !== "undefined" && process.env
      ? process.env.NEXT_PUBLIC_ARG_API_BASE
      : undefined;
  return fromEnv || "";
}

/** Per-call retry config. */
export interface FetchOptions {
  /** Number of retry attempts (default 0 = no retry, mock 模式). */
  retries?: number;
  /** Tenant id (RLS 13 類 header). */
  tenantId?: Uuid;
  /** Optional AbortSignal for cancel. */
  signal?: AbortSignal;
  /** Override default base URL (per-call). */
  baseUrl?: string;
}

// ----- low-level fetch wrapper -----

/** Map backend ApiError → thrown `Error` with status + code. */
async function ensureOk(res: Response): Promise<Response> {
  if (res.ok) return res;
  // Try parse body for typed error
  let code: string | undefined;
  let message: string = res.statusText;
  try {
    const body = (await res.clone().json()) as ApiErrorBody | unknown;
    if (typeof body === "object" && body !== null && "error" in body) {
      const e = (body as ApiErrorBody).error;
      code = e?.code;
      message = e?.message ?? res.statusText;
    }
  } catch {
    // Body not JSON; fall through to default statusText
  }
  const err = new Error(
    `ARG API ${res.status}${code ? ` [${code}]` : ""}: ${message}`,
  );
  (err as Error & { status?: number; code?: string }).status = res.status;
  (err as Error & { status?: number; code?: string }).code = code;
  throw err;
}

/** Build URL with query string. */
function buildUrl(path: string, query?: Record<string, unknown>): string {
  const base = getApiBase();
  const url = new URL(path, base || (typeof window !== "undefined" ? window.location.origin : "http://localhost/"));
  if (query) {
    for (const [k, v] of Object.entries(query)) {
      if (v === undefined || v === null) continue;
      url.searchParams.set(k, String(v));
    }
  }
  // If base is empty, return relative path
  return base ? url.toString() : `${url.pathname}${url.search}`;
}

/** fetch wrapper with retry + RLS header. */
async function request<T>(
  method: "GET" | "POST" | "PATCH" | "DELETE",
  path: string,
  body?: unknown,
  opts: FetchOptions = {},
): Promise<T> {
  const { retries = 0, tenantId, signal, baseUrl } = opts;
  const url = baseUrl ? `${baseUrl}${path}` : buildUrl(path);
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    Accept: "application/json",
  };
  if (tenantId) headers["X-Tenant-Id"] = tenantId;

  let lastErr: unknown = null;
  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const res = await fetch(url, {
        method,
        headers,
        body: body !== undefined ? JSON.stringify(body) : undefined,
        signal,
      });
      await ensureOk(res);
      // 204 No Content for DELETE
      if (res.status === 204) return undefined as T;
      return (await res.json()) as T;
    } catch (e) {
      lastErr = e;
      // Don't retry on 4xx
      const status = (e as { status?: number })?.status;
      if (status !== undefined && status >= 400 && status < 500) {
        throw e;
      }
      // Exponential backoff: 200ms, 600ms
      if (attempt < retries) {
        const delay = 200 * Math.pow(3, attempt);
        await new Promise((r) => setTimeout(r, delay));
      }
    }
  }
  throw lastErr instanceof Error ? lastErr : new Error(String(lastErr));
}

// ----- 13 REST endpoints (per ARG.4 §4.12) -----

/** 1. POST /api/arg/agents — create a new agent. */
export function createAgent(input: CreateAgentInput, opts?: FetchOptions): Promise<Agent> {
  return request<Agent>("POST", "/api/arg/agents", input, opts);
}

/** 2. GET /api/arg/agents — list agents. */
export function listAgents(
  filter: AgentFilter = {},
  page: PaginationQuery = {},
  opts?: FetchOptions,
): Promise<Agent[]> {
  const query: Record<string, unknown> = {
    ...filter,
    limit: page.limit,
    offset: page.offset,
  };
  return request<Agent[]>("GET", "/api/arg/agents", undefined, { ...opts });
}

/** 3. GET /api/arg/agents/{id} — fetch a single agent. */
export function getAgent(id: Uuid, opts?: FetchOptions): Promise<Agent> {
  return request<Agent>("GET", `/api/arg/agents/${id}`, undefined, opts);
}

/** 4. PATCH /api/arg/agents/{id} — update agent. */
export function updateAgent(
  id: Uuid,
  patch: UpdateAgentPatch,
  opts?: FetchOptions,
): Promise<Agent> {
  return request<Agent>("PATCH", `/api/arg/agents/${id}`, patch, opts);
}

/** 5. POST /api/arg/edges — create a new edge. */
export function createEdge(input: CreateEdgeInput, opts?: FetchOptions): Promise<Edge> {
  return request<Edge>("POST", "/api/arg/edges", input, opts);
}

/** 6. GET /api/arg/edges — list edges. */
export function listEdges(
  filter: EdgeFilter = {},
  opts?: FetchOptions,
): Promise<Edge[]> {
  return request<Edge[]>("GET", "/api/arg/edges", undefined, { ...opts });
}

/** 7. GET /api/arg/edges/{id} — fetch a single edge. */
export function getEdge(id: Uuid, opts?: FetchOptions): Promise<Edge> {
  return request<Edge>("GET", `/api/arg/edges/${id}`, undefined, opts);
}

/** 8. PATCH /api/arg/edges/{id} — update edge. */
export function updateEdge(
  id: Uuid,
  patch: UpdateEdgePatch,
  opts?: FetchOptions,
): Promise<Edge> {
  return request<Edge>("PATCH", `/api/arg/edges/${id}`, patch, opts);
}

/** 9. DELETE /api/arg/edges/{id} — soft-archive an edge. */
export function archiveEdge(id: Uuid, opts?: FetchOptions): Promise<void> {
  return request<void>("DELETE", `/api/arg/edges/${id}`, undefined, opts);
}

/** 10. GET /api/arg/graph — get capped graph. */
export function getGraph(
  filter: GraphFilter = {},
  opts?: FetchOptions,
): Promise<GraphResponse> {
  return request<GraphResponse>("GET", "/api/arg/graph", undefined, { ...opts });
}

/** 11. POST /api/arg/templates/instantiate — instantiate a template. */
export function instantiateTemplate(
  input: InstantiateTemplateInput,
  opts?: FetchOptions,
): Promise<TemplateInstance> {
  return request<TemplateInstance>(
    "POST",
    "/api/arg/templates/instantiate",
    input,
    opts,
  );
}

/** 12. GET /api/arg/achievements — list 20 achievement definitions.
 *  Returns hardcoded list (G-1 stub: backend returns 20 from `all_achievements()`).
 *  Optional category / rarity filters are applied client-side after fetch
 *  (matches backend filter semantics). */
export function listAchievements(
  filter: { category?: AchievementCategory; rarity?: Rarity } = {},
  opts?: FetchOptions,
): Promise<Achievement[]> {
  return request<Achievement[]>(
    "GET",
    "/api/arg/achievements",
    undefined,
    { ...opts },
  ).then((list) =>
    list.filter((a) =>
      (filter.category ? a.category === filter.category : true) &&
      (filter.rarity ? a.rarity === filter.rarity : true),
    ),
  );
}

/** 13. GET /api/arg/achievements/me — current user's unlocks. */
export function myUnlocks(
  filter: { limit?: number; tenantId?: Uuid } = {},
  opts?: FetchOptions,
): Promise<AchievementUnlock[]> {
  return request<AchievementUnlock[]>(
    "GET",
    "/api/arg/achievements/me",
    undefined,
    { ...opts, tenantId: filter.tenantId ?? opts?.tenantId },
  );
}

/** Bonus: POST /api/arg/achievements/evaluate — admin re-evaluate. */
export function evaluateAchievements(
  input: { user_id: Uuid; tenant_id?: Uuid },
  opts?: FetchOptions,
): Promise<EvaluateAchievementsResponse> {
  return request<EvaluateAchievementsResponse>(
    "POST",
    "/api/arg/achievements/evaluate",
    input,
    opts,
  );
}

// ----- Re-export for callers who want all 13 in one import -----

/**
 * All 13 ARG REST endpoints, grouped by resource. This is the public
 * surface — `ws.ts` handles the 14th (WebSocket).
 */
export const argApi = {
  // Agents (4)
  createAgent,
  listAgents,
  getAgent,
  updateAgent,
  // Edges (5)
  createEdge,
  listEdges,
  getEdge,
  updateEdge,
  archiveEdge,
  // Graph (1)
  getGraph,
  // Templates (1)
  instantiateTemplate,
  // Achievements (3, includes bonus evaluate)
  listAchievements,
  myUnlocks,
  evaluateAchievements,
} as const;

export type ArgApi = typeof argApi;

/** Type-only: re-export TeamTemplate for callers (5 模板 definitions). */
export type { TeamTemplate };
