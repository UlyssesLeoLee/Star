// frontend/src/mocks/client.ts
// MSW browser worker (per docs/frontend/design/mock-msw-handlers.md §4 P2 缺口 #2)
//
// Phase F.2 完整实装:
// - MSW 2.x browser worker (service worker 拦截 fetch in browser)
// - 与 node server (server.ts) 共享 handlers
// - production build 自动跳过 (per NEXT_PUBLIC_API_MOCKING !== "enabled")
// - 真实 API 接入 (Phase F+) 时删 client.ts + instrumentation.ts
//
// 设计 (per docs/frontend/design/mock-msw-handlers.md §2.3):
// - 仅 client 端有效 (typeof window !== "undefined")
// - onUnhandledRequest: "bypass" (per 真实 fetch 走 MSW 不存在路径时, 不报警)
// - 缺标比错标安全 (8/26 JST): 不编造 handler, 复用 M2-A 已有 handlers/
//
// 守门: 0 unsafe (TS 严模式), 仅 1 devDep (msw)

import { setupWorker } from "msw/browser";
import { handlers } from "./handlers";

export const worker = setupWorker(...handlers);
