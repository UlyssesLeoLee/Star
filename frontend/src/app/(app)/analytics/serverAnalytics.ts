// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/analytics/serverAnalytics.ts"})
MERGE (client:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::BiAnalyticsApiClient",kind:"class"})
MERGE (request:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::BiAnalyticsApiClient.request",kind:"method"})
MERGE (listEvents:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::BiAnalyticsApiClient.listTaskEvents",kind:"method"})
MERGE (summary:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::BiAnalyticsApiClient.getTaskSummary",kind:"method"})
MERGE (scope:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::useBiAnalyticsScope",kind:"hook"})
MERGE (resolve:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::resolveBiRunId",kind:"function"})
MERGE (token:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::defaultBiAccessTokenProvider",kind:"function"})
MERGE (readError:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::readBiApiError",kind:"function"})
MERGE (groupError:Type {id:"frontend/src/lib/group/worktreeGroupApi.ts::GroupApiError",kind:"class"})
MERGE (session:Symbol {id:"frontend/src/lib/auth.ts::getSession",kind:"function"})
MERGE (eventsEndpoint:Endpoint {id:"GET /api/v1/engineering-runs/{engineering_run_id}/bi/task-events",kind:"http_route"})
MERGE (summaryEndpoint:Endpoint {id:"GET /api/v1/engineering-runs/{engineering_run_id}/bi/task-summary",kind:"http_route"})
MERGE (self)-[:DEFINES]->(client)
MERGE (self)-[:DEFINES]->(request)
MERGE (self)-[:DEFINES]->(listEvents)
MERGE (self)-[:DEFINES]->(summary)
MERGE (self)-[:DEFINES]->(scope)
MERGE (self)-[:DEFINES]->(resolve)
MERGE (self)-[:DEFINES]->(token)
MERGE (self)-[:DEFINES]->(readError)
MERGE (self)-[:IMPORTS]->(groupError)
MERGE (self)-[:IMPORTS]->(session)
MERGE (request)-[:CALLS]->(token)
MERGE (request)-[:CALLS]->(readError)
MERGE (listEvents)-[:CALLS]->(request)
MERGE (summary)-[:CALLS]->(request)
MERGE (listEvents)-[:TARGETS]->(eventsEndpoint)
MERGE (summary)-[:TARGETS]->(summaryEndpoint)
MERGE (token)-[:CALLS]->(session)
@endcypher
*/
// =====================================================================
// serverAnalytics — Run 级 BI 服务端数据封装 (per P1 BI 接通任务)
//
// 数据来源: 服务端 /api/v1/engineering-runs/{run_id}/bi/task-events 与
//   .../bi/task-summary (均带 authorize 权限校验, 复用同一 task-events 查询真源)。
// 未接入授权会话 / 无 engineering run 上下文时, 本模块明确返回未接通状态,
// 不使用任何本地估算冒充服务端 BI。
// =====================================================================
"use client";

import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "next/navigation";
import { GroupApiError } from "@/lib/group/worktreeGroupApi";
import { getSession } from "@/lib/auth";

/** 与 WorktreeGroupApiClient 同形的 token 提供者 (每次请求取最新会话 token)。 */
export type BiAccessTokenProvider = () => Promise<string | null>;

/** allowlisted task-events 事实 (snake_case, 与服务端 page_response 对齐)。 */
export interface BiTaskEventItem {
  event_id: string;
  work_item_id: string;
  aggregate_type: string;
  aggregate_version: number;
  event_type: string;
  schema_version: number;
  occurred_at: string;
  lifecycle_status: string | null;
  review_state: string | null;
  from_status: string | null;
  to_status: string | null;
  owner_changed: boolean;
}

export interface BiTaskEventCoverage {
  completeness: string;
  freshness: string;
  basis: string;
}

export interface BiTaskEventPage {
  project_id: string;
  engineering_run_id: string;
  items: BiTaskEventItem[];
  has_more: boolean;
  next_cursor: { before_occurred_at: string; before_event_id: string } | null;
  coverage: BiTaskEventCoverage;
}

export interface BiTaskSummaryStatusBucket {
  status: string;
  count: number;
}

export interface BiTaskSummarySourceRange {
  earliest_occurred_at: string | null;
  latest_occurred_at: string | null;
  event_count: number;
}

export interface BiTaskSummary {
  project_id: string;
  engineering_run_id: string;
  status_distribution: BiTaskSummaryStatusBucket[];
  completed_count: number;
  total_count: number;
  work_item_count: number;
  as_of: string;
  source_event_range: BiTaskSummarySourceRange;
  coverage: BiTaskEventCoverage;
}

export interface BiTaskEventsQuery {
  limit?: number;
  before_occurred_at?: string;
  before_event_id?: string;
}

/** URL 参数里承载 engineering run 上下文的候选 key (同 Run 深链语义)。 */
export const BI_RUN_PARAM_KEYS = ["engineering_run_id", "run_id"] as const;

const RUN_ID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** 从查询参数解析合法的 engineering run id; 无参数返回 null。 */
export function resolveBiRunId(params: Pick<URLSearchParams, "get">): string | null {
  for (const key of BI_RUN_PARAM_KEYS) {
    const raw = params.get(key);
    if (raw && RUN_ID_RE.test(raw.trim())) return raw.trim().toLowerCase();
  }
  return null;
}

/** 默认 token 来源: 浏览器里已登录的 Star 会话 (lib/auth 同源约定)。 */
export function defaultBiAccessTokenProvider(): Promise<string | null> {
  return Promise.resolve(getSession()?.token ?? null);
}

/** 读取服务端 BI 错误, 对齐 GroupApiError (不暴露 token/secret)。 */
async function readBiApiError(response: Response): Promise<GroupApiError> {
  const payload = (await response.json().catch(() => null)) as
    | { error?: { code?: unknown; message?: unknown } }
    | null;
  const code = typeof payload?.error?.code === "string" ? payload.error.code : "bi_api_error";
  const message = typeof payload?.error?.message === "string"
    ? payload.error.message
    : "BI analytics request failed.";
  return new GroupApiError(response.status, code, message);
}

/** Run 级 BI 只读客户端: 只打 task-events 与 task-summary 两条授权路由。 */
export class BiAnalyticsApiClient {
  private readonly baseUrl: string;

  constructor(
    private readonly getAccessToken: BiAccessTokenProvider = defaultBiAccessTokenProvider,
    private readonly fetcher: typeof fetch = fetch,
    baseUrl = "",
  ) {
    this.baseUrl = baseUrl.trim().replace(/\/+$/, "");
  }

  private async request<T>(path: string, signal?: AbortSignal): Promise<T> {
    const token = (await this.getAccessToken())?.trim();
    if (!token) {
      throw new GroupApiError(401, "session_required", "BI analytics requires an authenticated user session.");
    }
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      headers: {
        Accept: "application/json",
        Authorization: `Bearer ${token}`,
      },
      credentials: "omit",
      cache: "no-store",
      signal,
    });
    if (!response.ok) {
      throw await readBiApiError(response);
    }
    return (await response.json()) as T;
  }

  /** 拉取一页 task-events 源记录 (cursor 分页, limit 上限 50)。 */
  listTaskEvents(
    runId: string,
    query: BiTaskEventsQuery = {},
    signal?: AbortSignal,
  ): Promise<BiTaskEventPage> {
    const params = new URLSearchParams();
    if (query.limit !== undefined) params.set("limit", String(query.limit));
    if (query.before_occurred_at) params.set("before_occurred_at", query.before_occurred_at);
    if (query.before_event_id) params.set("before_event_id", query.before_event_id);
    const serialized = params.toString();
    const suffix = serialized ? `?${serialized}` : "";
    return this.request(
      `/api/v1/engineering-runs/${encodeURIComponent(runId)}/bi/task-events${suffix}`,
      signal,
    );
  }

  /** 拉取 Run 级任务汇总 (状态分布 + 完成量 + 源记录范围, 与 task-events 同源)。 */
  getTaskSummary(runId: string, signal?: AbortSignal): Promise<BiTaskSummary> {
    return this.request(
      `/api/v1/engineering-runs/${encodeURIComponent(runId)}/bi/task-summary`,
      signal,
    );
  }
}

/** 服务端 BI 会话范围: 无会话 / 无 run / 就绪 三态, 未接通时明确显示状态。 */
export type BiAnalyticsScope =
  | { kind: "no-session"; runId: null }
  | { kind: "invalid-run"; runId: null }
  | { kind: "no-run"; runId: null }
  | { kind: "ready"; runId: string; client: BiAnalyticsApiClient };

/** 从当前 URL (engineering_run_id/run_id) + 登录会话推导服务端 BI 可达性。 */
export function useBiAnalyticsScope(): BiAnalyticsScope {
  const searchParams = useSearchParams();
  const client = useMemo(() => new BiAnalyticsApiClient(), []);
  // 会话检测延迟到挂载后: 首渲染 (含 SSR) 一律 no-session, 避免 hydration 不一致;
  // 挂载后按真实登录态重新推导。token 本身只在发请求时读取, 不进 state。
  const [sessionAvailable, setSessionAvailable] = useState(false);
  useEffect(() => {
    setSessionAvailable(typeof window !== "undefined" && Boolean(getSession()));
  }, []);
  if (!sessionAvailable) return { kind: "no-session", runId: null };
  const rawRunParam = BI_RUN_PARAM_KEYS
    .map((key) => searchParams.get(key))
    .find((value) => value !== null);
  if (rawRunParam !== undefined && !RUN_ID_RE.test(rawRunParam.trim())) {
    return { kind: "invalid-run", runId: null };
  }
  const runId = resolveBiRunId(searchParams);
  if (!runId) return { kind: "no-run", runId: null };
  return { kind: "ready", runId, client };
}
