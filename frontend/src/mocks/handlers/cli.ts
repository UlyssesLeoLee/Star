// frontend/src/mocks/handlers/cli.ts
// MSW handlers for CLI Profile + API Key + Task Window (per 2026-08-29 09:07 JST)
//
// Endpoints:
//   GET    /api/cli-profiles         — 列出 6 内置
//   POST   /api/cli-profiles         — 创建自定义
//   PATCH  /api/cli-profiles/:id     — 启用/禁用 / 重命名
//   DELETE /api/cli-profiles/:id     — 删除自定义
//   GET    /api/api-keys             — 列出
//   POST   /api/api-keys             — 添加 (encrypted / env_var)
//   DELETE /api/api-keys/:id         — 删除
//   GET    /api/task-windows         — 列出
//   POST   /api/task-windows         — 创建
//   POST   /api/task-windows/:wid/upload  — 触发上传
//
// P3-A.7 升级: 每个 handler 头部加 `maybeReal(path, init)` 短路
// real-mode 开启时跳过 MSW 直接转发到真 API (Bearer auth 自动)

import { http, HttpResponse } from "msw";
import { MOCK_CLI_PROFILES, MOCK_API_KEYS, MOCK_TASK_WINDOWS } from "@/mocks/data/cli";
import { isCliProfile, isApiKey } from "@/mocks/schemas/cli";
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

/** PATCH handler factory */
const patch = (path: string, mock: (body: unknown) => Response | Promise<Response>) =>
  http.patch(path, async ({ request }) => {
    const realInit: RequestInit = {
      method: "PATCH",
      body: await request.clone().text(),
      headers: request.headers as HeadersInit,
    };
    const r = await maybeReal(path, realInit);
    if (r) return r;
    const body = await request.json();
    return mock(body);
  });

/** DELETE handler factory */
const del = (path: string, mock: () => Response) =>
  http.delete(path, async ({ request }) => {
    const realInit: RequestInit = {
      method: "DELETE",
      headers: request.headers as HeadersInit,
    };
    const r = await maybeReal(path, realInit);
    return r ?? mock();
  });

export const cliHandlers = [
  // ===== CLI Profiles =====
  get("/api/cli-profiles", () => HttpResponse.json(MOCK_CLI_PROFILES)),

  post("/api/cli-profiles", (body) => {
    if (!isCliProfile(body)) {
      return HttpResponse.json({ error: "Invalid CLI profile" }, { status: 400 });
    }
    return HttpResponse.json(body, { status: 201 });
  }),

  patch("/api/cli-profiles/:id", (body) => {
    // 路径参数 mock 简化: 不在 patch 工厂内取 params
    return HttpResponse.json({ ...(body as object) });
  }),

  del("/api/cli-profiles/:id", () => HttpResponse.json({ deleted: true })),

  // ===== API Keys =====
  get("/api/api-keys", () => HttpResponse.json(MOCK_API_KEYS)),

  post("/api/api-keys", (body) => {
    if (!isApiKey(body)) {
      return HttpResponse.json({ error: "Invalid API key" }, { status: 400 });
    }
    return HttpResponse.json(body, { status: 201 });
  }),

  del("/api/api-keys/:id", () => HttpResponse.json({ deleted: true })),

  // ===== Task Windows =====
  get("/api/task-windows", () => HttpResponse.json(MOCK_TASK_WINDOWS)),

  post("/api/task-windows", (body) => {
    return HttpResponse.json({ id: "w_new", ...(body as object) }, { status: 201 });
  }),

  post("/api/task-windows/:wid/upload", () => {
    return HttpResponse.json({
      taskId: "task_" + Date.now(),
      status: "pending",
      message: "Upload triggered (mock: actually runs in production)",
    });
  }),
];
