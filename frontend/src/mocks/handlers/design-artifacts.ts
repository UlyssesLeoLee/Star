// =====================================================================
// MSW handlers for /api/design-artifacts (per wt-test-t2-dsg 2026-08-31)
// =====================================================================
// 上游依据:
//   - docs/test-design.md §6.3.3   REQ-DSG-001/002
//   - docs/requirements.md §8.3   DesignArtifact 字段
//   - docs/requirements.md §27.4  ReviewRecord 互斥 Target
//
// Endpoints (5):
//   GET    /api/design-artifacts?work_item_id=X     列表 (按 wi 过滤)
//   GET    /api/design-artifacts/:id                单条
//   POST   /api/design-artifacts/:id/review         提交 review 决策
//                                                     body: { decision, reviewer_id, comment? }
//                                                     -> 更新 status + version+1
//                                                        + review_record_id
//   GET    /api/design-artifacts/guard/:work_item_id  Guard 查询
//                                                     -> { all_approved, pending: [...] }
//
// 守门 派生 (per cli.ts P3-A.7 + AGENTS.md §4.1 v1-v14):
//   - maybeReal 短路 (real-mode 开启时直接转发, Bearer auth 自动)
//   - get/post 工厂封装 (P3-A.7 守门实证)
//
// 已知缺口 (per 缺标比错标, 8/26 JST 守门 #1 + #12 引用):
//   1. POST 真实持久化 P3 — Phase F+ 后端就绪时 (现 in-memory mutable)
//   2. ReviewRecord 互斥 Target 字段 TBD — basic-design §27.4 跟进
//      缺标 = 简单 rr-XXX id 字符串; 错标 = 提前 discriminated union
//   3. version +1 在 POST /review 时本地累加, P2 真实持久化时由后端 audit 驱动
// =====================================================================

import { http, HttpResponse } from "msw";
import { MOCK_DESIGN_ARTIFACTS } from "@/mocks/data/design-artifacts";
import { isReviewRequestBody } from "@/mocks/schemas/design-artifact";
import { isRealMode, realFetch } from "@/mocks/real-mode";
import type { DesignArtifact, Uuid } from "@/types/ids";
import { DESIGN_ARTIFACT_STATUSES } from "@/types/ids";

// =====================================================================
// Pure resolver functions (extracted for testability)
// =====================================================================
// 守门 派生: 拆出 pure functions 让 vitest 直接调, 不依赖 MSW node server
// (per handlers.test.ts §2.7 实证: jsdom + MSW 2.x fetch 走真实网络, 拦截失败)
// =====================================================================

/** in-memory mutable store — POST /review 后版本+1 (mock-only) */
let mutableStore: DesignArtifact[] = MOCK_DESIGN_ARTIFACTS.map((a) => ({ ...a }));

/** 列表 + 按 work_item_id 过滤 (caller 用 URLSearchParams 解析) */
export function listDesignArtifacts(workItemId: string | null): DesignArtifact[] {
  if (workItemId) {
    return mutableStore.filter((a) => a.work_item_id === workItemId);
  }
  return mutableStore;
}

/** 按 id 查单条 */
export function getDesignArtifact(id: Uuid): DesignArtifact | null {
  return mutableStore.find((a) => a.id === id) ?? null;
}

/** Guard 查询 — 返回 { all_approved, pending: DesignArtifact[] } */
export function guardDesignArtifacts(workItemId: Uuid): {
  all_approved: boolean;
  pending: DesignArtifact[];
} {
  const items = mutableStore.filter((a) => a.work_item_id === workItemId);
  const pending = items.filter(
    (a) => a.status !== "approved" && a.status !== "superseded",
  );
  return {
    all_approved: items.length > 0 && pending.length === 0,
    pending,
  };
}

/** Status 状态机迁移 (per REQ-DSG-001 5 状态机) */
export function nextStatusFromDecision(
  current: DesignArtifact["status"],
  decision: "approve" | "request_changes",
): DesignArtifact["status"] {
  if (current === "superseded") return "superseded";
  if (decision === "approve") {
    return "approved";
  } else {
    // request_changes: in_review/approved/rejected/draft -> draft
    return "draft";
  }
}

/** POST /review 写回 — 返回 { ok, status, artifact }  */
export function applyReview(
  id: Uuid,
  body: unknown,
): {
  ok: boolean;
  status: number;
  artifact: DesignArtifact | null;
  error?: string;
} {
  if (!isReviewRequestBody(body)) {
    return { ok: false, status: 400, artifact: null, error: "Invalid review request body" };
  }
  const idx = mutableStore.findIndex((a) => a.id === id);
  if (idx === -1) {
    return { ok: false, status: 404, artifact: null, error: "Not found" };
  }
  const current = mutableStore[idx];
  const newStatus = nextStatusFromDecision(current.status, body.decision);
  if (!DESIGN_ARTIFACT_STATUSES.includes(newStatus)) {
    return {
      ok: false,
      status: 500,
      artifact: null,
      error: `Invalid derived status: ${newStatus}`,
    };
  }
  const updated: DesignArtifact = {
    ...current,
    status: newStatus,
    version: current.version + 1,
    review_record_id: `rr-${current.id}-v${current.version + 1}`,
    updated_at: new Date().toISOString(),
  };
  mutableStore[idx] = updated;
  return { ok: true, status: 200, artifact: updated };
}

// =====================================================================
// MSW handlers (thin wrappers around pure resolvers)
// =====================================================================

/** real-mode 短路 */
function maybeReal(path: string, init: RequestInit = {}): Promise<Response> | null {
  if (!isRealMode()) return null;
  return realFetch(path, init);
}

type GetResolverArgs = { request: Request; params: Record<string, string | readonly string[]> };
type PostResolverArgs = GetResolverArgs;

const get = (path: string, mock: (args: GetResolverArgs) => Response) =>
  http.get(path, async (args) => {
    const r = await maybeReal(path);
    return r ?? mock(args as GetResolverArgs);
  });

const post = (
  path: string,
  mock: (body: unknown, args: PostResolverArgs) => Response | Promise<Response>,
) =>
  http.post(path, async (args) => {
    const req = (args as GetResolverArgs).request;
    const realInit: RequestInit = {
      method: "POST",
      body: await req.clone().text(),
      headers: req.headers as HeadersInit,
    };
    const r = await maybeReal(path, realInit);
    if (r) return r;
    const body = await req.json();
    return mock(body, args as PostResolverArgs);
  });

export const designArtifactHandlers = [
  // ===== GET 列表 (按 work_item_id 过滤) =====
  get("/api/design-artifacts", ({ request }) => {
    // 解析 ?work_item_id=... query string
    const url = new URL(request.url);
    const workItemId = url.searchParams.get("work_item_id");
    return HttpResponse.json(listDesignArtifacts(workItemId));
  }),

  // ===== GET /api/design-artifacts/guard/:work_item_id =====
  // 必须在 :id 之前注册 — MSW 路由匹配按声明顺序
  get("/api/design-artifacts/guard/:work_item_id", ({ params }) => {
    const workItemId = String(params["work_item_id"]) as Uuid;
    return HttpResponse.json(guardDesignArtifacts(workItemId));
  }),

  // ===== GET 单条 =====
  get("/api/design-artifacts/:id", ({ params }) => {
    const id = String(params["id"]) as Uuid;
    const found = getDesignArtifact(id);
    if (!found) {
      return HttpResponse.json({ error: "Not found" }, { status: 404 });
    }
    return HttpResponse.json(found);
  }),

  // ===== POST review =====
  post(
    "/api/design-artifacts/:id/review",
    (body, { params }) => {
      const id = String(params["id"]) as Uuid;
      const result = applyReview(id, body);
      if (!result.ok) {
        return HttpResponse.json({ error: result.error }, { status: result.status });
      }
      return HttpResponse.json(result.artifact, { status: 200 });
    },
  ),
];

/** 测试辅助: 重置 mutable store (per test 隔离) */
export function __resetDesignArtifactStore() {
  mutableStore = MOCK_DESIGN_ARTIFACTS.map((a) => ({ ...a }));
}

/** 测试辅助: 暴露当前 mutable store 引用 (per 断言) */
export function __getDesignArtifactStore(): ReadonlyArray<DesignArtifact> {
  return mutableStore;
}
