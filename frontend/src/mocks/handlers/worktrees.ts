// frontend/src/mocks/handlers/worktrees.ts
// match 域 MSW handlers (per test-design.md v0.2 §2.1.2 + 5 域映射表)
//
// Endpoints:
//   GET  /api/worktrees?project_id=...&status=...   — 按 project / status 过滤
//   GET  /api/worktrees/:id                         — 单条
//   POST /api/worktrees/:id/transition              — 模拟状态转换 (mock echo + status: "ok")
//
// 5 域映射: match (workflow / 状态机 / saga) — 本 handler 覆盖 worktree 状态机
// 状态机源: frontend/src/types/ids.ts WorktreeStatus (17 states per §7.1)
//
// 风格: 跟 handlers/cli.ts 对齐 (含 maybeReal real-mode 短路 + get/post 工厂)
//
// 已知缺口 (per 守门 #1 缺标比错标安全):
//   1. POST /transition 真实状态机执行 (saga 持久化) P3 (Phase F+)
//   2. PR / 评审联动 (per docs/frontend/design/) P2 (Phase F+)
//   3. 跨 worktree 的并发冲突检测 P3 (Phase F+)

import { http, HttpResponse } from "msw";
import { MOCK_WORKTREES } from "@/mocks/data/five-domain";
import { isRealMode, realFetch } from "@/mocks/real-mode";
import type { WorktreeStatus } from "@/types/ids";

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

export const worktreesHandlers = [
  get("/api/worktrees", () => {
    // 过滤 (按 query string) 在 fetch consumer 端做; 这里返回全集
    // 这样前端 useEffect+fetch 拿到全表再 client-side filter
    return HttpResponse.json(MOCK_WORKTREES);
  }),

  http.get("/api/worktrees/:id", ({ params }) => {
    const id = params.id as string;
    const found = MOCK_WORKTREES.find((w) => w.id === id);
    if (!found) {
      return HttpResponse.json({ error: `Worktree ${id} not found` }, { status: 404 });
    }
    return HttpResponse.json(found);
  }),

  post("/api/worktrees/:id/transition", (body) => {
    const b = body as { to?: unknown };
    if (typeof b.to !== "string" || b.to.length === 0) {
      return HttpResponse.json({ error: "Invalid transition: missing 'to' field" }, { status: 400 });
    }
    // mock 简化: echo + status: "ok" (不真做状态机执行)
    return HttpResponse.json({
      id: "(echo)",
      to: b.to as WorktreeStatus,
      status: "ok",
      at: new Date().toISOString(),
    });
  }),
];
