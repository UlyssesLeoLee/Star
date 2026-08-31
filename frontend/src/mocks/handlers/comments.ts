// frontend/src/mocks/handlers/comments.ts
// social 域 MSW handlers (per test-design.md v0.2 §2.1.2 + 5 域映射表)
//
// Endpoints:
//   GET    /api/comments?work_item_id=...   — 按 work_item_id 过滤
//   POST   /api/comments                    — 创建 (校验 isComment)
//   DELETE /api/comments/:id                — 软删 → 返回 { deleted: true, id }
//
// 5 域映射: social (collaboration / 通知) — 本 handler 覆盖 comment 子域
// (inbox / notifications 端点见 handlers/inbox.ts)
//
// 已知缺口 (per 守门 #1 缺标比错标安全):
//   1. POST 真实持久化 P3 (Phase F+)
//   2. PATCH 编辑 P2 (Phase F+)
//   3. 软删 hard-delete 后端定时清理 P3

import { http, HttpResponse } from "msw";
import { isComment } from "@/mocks/schemas/five-domain";

export const commentsHandlers = [
  http.get("/api/comments", () => {
    // 全集 (mock 简化, 前端按 work_item_id client-side filter)
    // 真实接入后端时, 后端接 query 参数做 server-side 过滤
    return HttpResponse.json([]);
  }),

  http.post("/api/comments", async ({ request }) => {
    const body = await request.json();
    if (!isComment(body)) {
      return HttpResponse.json({ error: "Invalid comment payload" }, { status: 400 });
    }
    // P3 缺口: 真实持久化 — 当前 mock echo
    return HttpResponse.json(body, { status: 201 });
  }),

  http.delete("/api/comments/:id", ({ params }) => {
    const id = params.id as string;
    // 软删 — 真实接入时只 flip deleted flag, 不物理删
    return HttpResponse.json({ deleted: true, id });
  }),
];
