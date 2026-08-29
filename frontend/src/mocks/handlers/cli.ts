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

import { http, HttpResponse } from "msw";
import { MOCK_CLI_PROFILES, MOCK_API_KEYS, MOCK_TASK_WINDOWS } from "@/mocks/data/cli";
import { isCliProfile, isApiKey } from "@/mocks/schemas/cli";

export const cliHandlers = [
  // ===== CLI Profiles =====
  http.get("/api/cli-profiles", () => HttpResponse.json(MOCK_CLI_PROFILES)),

  http.post("/api/cli-profiles", async ({ request }) => {
    const body = await request.json();
    if (!isCliProfile(body)) {
      return HttpResponse.json({ error: "Invalid CLI profile" }, { status: 400 });
    }
    return HttpResponse.json(body, { status: 201 });
  }),

  http.patch("/api/cli-profiles/:id", async ({ request, params }) => {
    const body = await request.json();
    const id = params.id as string;
    const idx = MOCK_CLI_PROFILES.findIndex((p) => p.id === id);
    if (idx === -1) return HttpResponse.json({ error: "not found" }, { status: 404 });
    const updated = { ...MOCK_CLI_PROFILES[idx], ...(body as Partial<typeof MOCK_CLI_PROFILES[0]>) };
    return HttpResponse.json(updated);
  }),

  http.delete("/api/cli-profiles/:id", ({ params }) => {
    const id = params.id as string;
    // 内置 profile (claude/codex/openclaw/hermes/gemini/aider) 不允许删
    if (["claude", "codex", "openclaw", "hermes", "gemini", "aider"].includes(id)) {
      return HttpResponse.json({ error: "Cannot delete built-in profile" }, { status: 403 });
    }
    return HttpResponse.json({ deleted: id });
  }),

  // ===== API Keys =====
  http.get("/api/api-keys", () => HttpResponse.json(MOCK_API_KEYS)),

  http.post("/api/api-keys", async ({ request }) => {
    const body = await request.json();
    if (!isApiKey(body)) {
      return HttpResponse.json({ error: "Invalid API key" }, { status: 400 });
    }
    return HttpResponse.json(body, { status: 201 });
  }),

  http.delete("/api/api-keys/:id", ({ params }) => {
    return HttpResponse.json({ deleted: params.id });
  }),

  // ===== Task Windows =====
  http.get("/api/task-windows", () => HttpResponse.json(MOCK_TASK_WINDOWS)),

  http.post("/api/task-windows", async ({ request }) => {
    const body = await request.json();
    return HttpResponse.json({ id: "w_new", ...(body as object) }, { status: 201 });
  }),

  http.post("/api/task-windows/:wid/upload", ({ params }) => {
    return HttpResponse.json({
      taskId: "task_" + Date.now(),
      windowId: params.wid,
      status: "pending",
      message: "Upload triggered (mock: actually runs in production)",
    });
  }),
];
