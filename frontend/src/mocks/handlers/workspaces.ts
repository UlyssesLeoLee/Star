// frontend/src/mocks/handlers/workspaces.ts
// player 域 MSW handlers (per test-design.md v0.2 §2.1.2 + 5 域映射表)
//
// Endpoints:
//   GET  /api/workspaces      — 列出所有
//   GET  /api/workspaces/:id  — 单条
//   POST /api/workspaces      — 创建 (校验 isWorkspace)
//
// 5 域映射: player (user/identity/workspace) — 本 handler 覆盖 workspace 子域
// 已知缺口 (per 守门 #1 缺标比错标安全):
//   1. POST 真实持久化 P3 (Phase F+)
//   2. PATCH / DELETE P3 (Phase F+)
//   3. real-mode 短路 P3 (P3-A.7 §3 缺口 #1)

import { http, HttpResponse } from "msw";
import { MOCK_WORKSPACES } from "@/mocks/data/five-domain";
import { isWorkspace } from "@/mocks/schemas/five-domain";

export const workspacesHandlers = [
  http.get("/api/workspaces", () => {
    return HttpResponse.json(MOCK_WORKSPACES);
  }),

  http.get("/api/workspaces/:id", ({ params }) => {
    const id = params.id as string;
    const found = MOCK_WORKSPACES.find((w) => w.id === id);
    if (!found) {
      return HttpResponse.json({ error: `Workspace ${id} not found` }, { status: 404 });
    }
    return HttpResponse.json(found);
  }),

  http.post("/api/workspaces", async ({ request }) => {
    const body = await request.json();
    if (!isWorkspace(body)) {
      return HttpResponse.json({ error: "Invalid workspace payload" }, { status: 400 });
    }
    // P3 缺口: 真实持久化 — 当前 mock echo
    return HttpResponse.json(body, { status: 201 });
  }),
];
