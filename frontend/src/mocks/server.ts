// frontend/src/mocks/server.ts
// MSW node server (per mock-msw-handlers.md §2.3)
//
// 用途: vitest setup 阶段启动, fetch 调用被 handlers 拦截返回 mock 数据
// 不 mount React, 直接测 handler 行为 (per §2.7)
//
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. MSW browser worker (page 在 production build 时仍走 MSW) P2 (Phase E.3+)
//   2. handler 错误日志未持久化 P3

import { setupServer } from "msw/node";
import { handlers } from "./handlers";

export const server = setupServer(...handlers);
