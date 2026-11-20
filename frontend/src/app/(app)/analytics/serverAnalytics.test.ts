// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/analytics/serverAnalytics.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.test.ts::serverAnalytics tests",kind:"test"})
MERGE (client:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::BiAnalyticsApiClient",kind:"class"})
MERGE (resolve:Symbol {id:"frontend/src/app/(app)/analytics/serverAnalytics.ts::resolveBiRunId",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(client)
MERGE (suite)-[:TESTS]->(resolve)
@endcypher
*/
// per P1 BI 接通任务: serverAnalytics fetch 封装 (token/错误/snake_case) 组件测试
import { describe, it, expect, vi } from "vitest";
import {
  BiAnalyticsApiClient,
  resolveBiRunId,
  BI_RUN_PARAM_KEYS,
} from "./serverAnalytics";
import { GroupApiError } from "@/lib/group/worktreeGroupApi";

const RUN_ID = "60000000-0000-0000-0000-000000000001";

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

describe("serverAnalytics fetch 封装", () => {
  it("resolveBiRunId 只接受合法 UUID 且优先级按 BI_RUN_PARAM_KEYS", () => {
    expect(resolveBiRunId(new URLSearchParams("?run_id=60000000-0000-0000-0000-000000000001"))).toBe(RUN_ID);
    expect(resolveBiRunId(new URLSearchParams("?engineering_run_id=60000000-0000-0000-0000-000000000002"))).toBe("60000000-0000-0000-0000-000000000002");
    expect(resolveBiRunId(new URLSearchParams(""))).toBeNull();
    expect(resolveBiRunId(new URLSearchParams("?run_id=not-a-uuid"))).toBeNull();
    expect(resolveBiRunId(new URLSearchParams("?run_id=60000000-0000-0000-0000-000000000001&engineering_run_id=bad"))).toBe(RUN_ID);
    expect(BI_RUN_PARAM_KEYS[0]).toBe("engineering_run_id");
  });

  it("listTaskEvents 构建 cursor 分页查询并携带 Bearer token", async () => {
    const fetcher = vi.fn(async (_url: RequestInfo | URL, init?: RequestInit) => {
      expect(String(_url)).toBe(
        `/api/v1/engineering-runs/${RUN_ID}/bi/task-events?limit=50&before_occurred_at=2026-11-01T00%3A00%3A00Z&before_event_id=00000000-0000-0000-0000-000000000042`,
      );
      expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer tok-1");
      expect(init?.cache).toBe("no-store");
      return jsonResponse({
        project_id: "20000000-0000-0000-0000-000000000001",
        engineering_run_id: RUN_ID,
        items: [],
        has_more: false,
        next_cursor: null,
        coverage: { completeness: "unknown", freshness: "unknown", basis: "test" },
      });
    });
    const client = new BiAnalyticsApiClient(async () => "tok-1", fetcher as unknown as typeof fetch);
    const page = await client.listTaskEvents(RUN_ID, {
      limit: 50,
      before_occurred_at: "2026-11-01T00:00:00Z",
      before_event_id: "00000000-0000-0000-0000-000000000042",
    });
    expect(page.has_more).toBe(false);
    expect(page.items).toEqual([]);
    expect(fetcher).toHaveBeenCalledTimes(1);
  });

  it("getTaskSummary 命中 task-summary 路由并返回 snake_case 形状", async () => {
    const fetcher = vi.fn(async () => jsonResponse({
      project_id: "20000000-0000-0000-0000-000000000001",
      engineering_run_id: RUN_ID,
      status_distribution: [{ status: "completed", count: 2 }],
      completed_count: 2,
      total_count: 5,
      work_item_count: 3,
      as_of: "2026-11-20T08:00:00Z",
      source_event_range: {
        earliest_occurred_at: "2026-11-18T08:00:00Z",
        latest_occurred_at: "2026-11-20T07:59:00Z",
        event_count: 5,
      },
      coverage: { completeness: "unknown", freshness: "unknown", basis: "test" },
    }));
    const client = new BiAnalyticsApiClient(async () => "tok-2", fetcher as unknown as typeof fetch);
    const summary = await client.getTaskSummary(RUN_ID);
    expect(summary.total_count).toBe(5);
    expect(summary.completed_count).toBe(2);
    expect(summary.status_distribution[0]).toEqual({ status: "completed", count: 2 });
    expect(String(fetcher.mock.calls[0][0])).toBe(
      `/api/v1/engineering-runs/${RUN_ID}/bi/task-summary`,
    );
  });

  it("无 token 时抛 session_required (不发起网络请求)", async () => {
    const fetcher = vi.fn();
    const client = new BiAnalyticsApiClient(async () => null, fetcher as unknown as typeof fetch);
    await expect(client.getTaskSummary(RUN_ID)).rejects.toMatchObject<GroupApiError>({
      status: 401,
      code: "session_required",
    });
    expect(fetcher).not.toHaveBeenCalled();
  });

  it("非 2xx 响应映射为 GroupApiError (错误码不暴露 secret)", async () => {
    const fetcher = vi.fn(async () =>
      jsonResponse({ error: { code: "insufficient_scope" } }, 403),
    );
    const client = new BiAnalyticsApiClient(async () => "tok-3", fetcher as unknown as typeof fetch);
    await expect(client.listTaskEvents(RUN_ID)).rejects.toMatchObject<GroupApiError>({
      status: 403,
      code: "insufficient_scope",
    });
  });
});
