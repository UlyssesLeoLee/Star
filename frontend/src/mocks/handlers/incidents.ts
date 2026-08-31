// =====================================================================
// frontend/src/mocks/handlers/incidents.ts
// MSW handlers for IncidentRecord (per test-design §6.3.4 / REQ-OPS-001/002/003)
// =====================================================================
//
// 触发: T3 (per test-design.md v0.3 §6.3.4 V1 Should-Have Test, TBD)
// 设计书: docs/test-design.md §6.3.4 + docs/requirements.md §29.1/§30.6
//
// 端点清单 (per 任务要求):
//
//   【允许的能力】(2 个):
//     GET  /api/incidents                 — 列出所有 / 按 ?work_item_id= 过滤
//     POST /api/incidents                 — 创建 (校验 source ∈ allowed)
//
//   【3 项非能力端点 — Negative Missing Tests 核心】:
//     GET  /api/incidents/probe-production
//          → 404 + "Capability not implemented (per REQ-OPS-003 §30.6 boundary)"
//     POST /api/incidents/process-alert
//          → 404 + 同上
//     POST /api/incidents/:id/auto-rollback
//          → 404 + 同上
//
//   错误文案 TBD 占位 (per 守门 #3 缺标比错标): 等 basic-design §30.6 拍板
//   后回填具体文案。当前统一用 "REQ-OPS-003 boundary" 占位, 保证测试
//   可识别但不假装定稿。
//
// 风格: 复用 handlers/cli.ts 模式 (maybeReal 短路 + get/post 工厂)
// =====================================================================

import { http, HttpResponse } from "msw";
import { MOCK_INCIDENTS } from "@/mocks/data/incidents";
import { isIncidentRecord, isIncidentSource } from "@/mocks/schemas/incident";
import { isRealMode, realFetch } from "@/mocks/real-mode";

/** real-mode 短路: 开启时返回 realFetch promise, 关闭时返回 null 走 mock */
function maybeReal(path: string, init: RequestInit = {}): Promise<Response> | null {
  if (!isRealMode()) return null;
  return realFetch(path, init);
}

/** GET handler factory: 头部加 real-mode 短路 */
const get = (path: string, mock: () => Response) =>
  http.get(path, async () => {
    const r = await maybeReal(path);
    return r ?? mock();
  });

/** POST handler factory */
const post = (path: string, mock: (body: unknown) => Response | Promise<Response>) =>
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

/**
 * 3 项非能力端点的统一 404 文案占位 (per REQ-OPS-003 §30.6 边界)
 * TBD: 等 basic-design §30.6 拍板后回填具体错误模型 (test-design §6.3.4
 * 备注: 当前无法在 Schema 中分类)。当前用 "REQ-OPS-003 boundary" 占位
 * 即可被测试识别 (测试用 `toMatch(/REQ-OPS-003/)`)。
 */
const NOT_IMPLEMENTED_404 = (capability: string) =>
  HttpResponse.json(
    {
      error: "Capability not implemented (per REQ-OPS-003 §30.6 boundary)",
      capability,
      // 守门 #3 缺标比错标: TBD 显式列在响应里, 让调用方知道这是边界不是 bug
      note: "TBD: error message schema per basic-design §30.6",
    },
    { status: 404 },
  );

export const incidentHandlers = [
  // ===== 允许: GET /api/incidents (按 ?work_item_id= 过滤) =====
  get("/api/incidents", () => {
    // 简化: URL 查询参数 (MSW handler 头部不接 params 时, 取不到 query string)
    // 当前实现统一返回全部; work_item_id 过滤由 lib/incident-guard.ts 调用
    // 端做 client-side 过滤 (P2+ 可改为 server 端 query 处理)
    return HttpResponse.json(MOCK_INCIDENTS);
  }),

  // ===== 允许: POST /api/incidents (创建 + 严格 source 校验) =====
  post("/api/incidents", (body) => {
    // 1. 整体 shape 校验
    if (!isIncidentRecord(body)) {
      return HttpResponse.json(
        {
          error: "Invalid IncidentRecord",
          hint: "source must be human_entry | integration_webhook (per REQ-OPS-003)",
        },
        { status: 400 },
      );
    }
    // 2. source 二次强校验 (防御 isIncidentRecord 漂移)
    if (!isIncidentSource((body as { source: unknown }).source)) {
      return HttpResponse.json(
        {
          error: "Invalid source",
          // 关键: 测试断言里查 "Invalid source" 子串
          message: "Invalid source: only 'human_entry' or 'integration_webhook' allowed (per REQ-OPS-003)",
        },
        { status: 400 },
      );
    }
    // P3 缺口: 真实持久化 — Phase F+ 后端就绪
    return HttpResponse.json(body, { status: 201 });
  }),

  // ===== 非能力 1: 主动探查生产 =====
  // (per REQ-OPS-003: 系统**不得**实现 ① 主动探查生产)
  get("/api/incidents/probe-production", () => {
    return NOT_IMPLEMENTED_404("probe-production");
  }),

  // ===== 非能力 2: 处理告警 =====
  // (per REQ-OPS-003: 系统**不得**实现 ② 处理告警)
  post("/api/incidents/process-alert", () => {
    return NOT_IMPLEMENTED_404("process-alert");
  }),

  // ===== 非能力 3: 自动回滚 =====
  // (per REQ-OPS-003: 系统**不得**实现 ③ 自动回滚/自动修复)
  post("/api/incidents/:id/auto-rollback", () => {
    return NOT_IMPLEMENTED_404("auto-rollback");
  }),
];
