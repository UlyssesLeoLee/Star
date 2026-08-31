// frontend/src/mocks/handlers/validation.ts
// MSW handlers for /api/validation/* (per REQ-TST-001/002)
//
// 设计依据 (per 缺标比错标安全, 8/26 JST 守门):
//   - docs/test-design.md §6.2.1 (V1 Should-Have Test, 4 Level 维度)
//   - docs/requirements.md §27.6 + REQ-TST-001/002
//   - docs/frontend/design/mock-msw-handlers.md §2.6 (handler factory 风格)
//   - handlers/cli.ts P3-A.7 升级: 每个 handler 头部加 `maybeReal(path, init)` 短路
//
// Endpoints:
//   GET  /api/validation/results?work_item_id=...&level=...
//                                  — 列表 (按 work_item_id 过滤 + 按 level 过滤)
//   GET  /api/validation/coverage/:work_item_id
//                                  — AcceptanceCoverageReport (per REQ-TST-002
//                                    uncovered_by_level 显式指出缺口)
//   POST /api/validation/results   — 提交 ValidationResult (校验 isValidationResult,
//                                    201 或 400)
//
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. POST 真实持久化 P3 — Phase F+ 后端就绪时
//   2. coverage endpoint 仅按 work_item_id 查, 不支持 level 二次过滤 (per test-design §6.2.1 留 P2)
//   3. MOCK_VALIDATION_RESULTS 与 MOCK_ACCEPTANCE_COVERAGE 数据联动派生 P2 (mock 用于展示形态)

import { http, HttpResponse } from "msw";
import {
  MOCK_VALIDATION_RESULTS,
  MOCK_ACCEPTANCE_COVERAGE,
} from "@/mocks/data/validation";
import {
  isValidationResultRecord,
  isTestLevel,
} from "@/mocks/schemas/validation";
import { isRealMode, realFetch } from "@/mocks/real-mode";
// Note: TestLevel is imported via schemas/validation for re-use; 路径稳定以便
// 后续 data/validation.ts 真实接入时直接复用类型.

/** real-mode 短路: 开启时返回 realFetch promise, 关闭时返回 null 走 mock */
function maybeReal(path: string, init: RequestInit = {}): Promise<Response> | null {
  if (!isRealMode()) return null;
  return realFetch(path, init);
}

/** POST handler factory: real-mode 短路 + 读 body */
const post = (
  path: string,
  mock: (body: unknown) => Response | Promise<Response>,
) =>
  http.post(path, async ({ request }) => {
    const realInit: RequestInit = {
      method: "POST",
      body: await request.clone().text(),
      headers: request.headers as HeadersInit,
    };
    const r = await maybeReal(path, realInit);
    if (r) return r;
    const body = await request.json();
    return mock(body);
  });

export const validationHandlers = [
  // GET /api/validation/results?work_item_id=...&level=...
  http.get("/api/validation/results", async ({ request }) => {
    const realR = await maybeReal(request.url);
    if (realR) return realR;
    const url = new URL(request.url);
    const wid = url.searchParams.get("work_item_id");
    const lvl = url.searchParams.get("level");
    let list: typeof MOCK_VALIDATION_RESULTS[number][] = [
      ...MOCK_VALIDATION_RESULTS,
    ];
    if (wid) list = list.filter((v) => v.work_item_id === wid);
    if (lvl) {
      if (!isTestLevel(lvl)) {
        return HttpResponse.json(
          { error: `Invalid level: ${lvl}` },
          { status: 400 },
        );
      }
      list = list.filter((v) => v.level === lvl);
    }
    return HttpResponse.json(list);
  }),

  // GET /api/validation/coverage/:work_item_id
  http.get("/api/validation/coverage/:work_item_id", async ({ params }) => {
    const realR = await maybeReal(`/api/validation/coverage/${params.work_item_id as string}`);
    if (realR) return realR;
    const wid = params.work_item_id as string;
    const found = MOCK_ACCEPTANCE_COVERAGE.find(
      (r) => r.work_item_id === wid,
    );
    if (!found) {
      return HttpResponse.json(
        { error: `No coverage report for work_item_id=${wid}` },
        { status: 404 },
      );
    }
    return HttpResponse.json(found);
  }),

  // POST /api/validation/results
  post("/api/validation/results", (body) => {
    if (!isValidationResultRecord(body)) {
      return HttpResponse.json(
        { error: "Invalid ValidationResult payload" },
        { status: 400 },
      );
    }
    return HttpResponse.json(body, { status: 201 });
  }),
];
