// vitest setup — 注册 @testing-library/jest-dom matchers
// + 全局 mock next/navigation (避免 KanbanCard / MonthView / WeekView 在 jsdom 抛 "app router not mounted")
// + MSW node server 启动 (per mock-msw-handlers.md §2.3)
//
// 触发场景:
//   - KanbanCard.tsx:18 / MonthView.tsx:4 / WeekView.tsx:4 都 import { useRouter } from "next/navigation"
//   - jsdom 没有 App Router context, 调 useRouter() 抛 invariant
//   - 全局 mock 后, 任何用 next/navigation 的组件在测试里都拿到 stub push/replace, 不报错
//   - MSW node server 启动后, 测试内 fetch("/api/...") 被 handlers 拦截返回 mock 数据
//
// 历史: 2026-08-28 W3 (W2) 写入时未配置 mock; W5 接手 store/persist 时也未补; U3 接手 projects 页面时补上
//       2026-08-28 M2-A 补 MSW setupServer (per mock-msw-handlers.md §2.3 + §3.1)
import "@testing-library/jest-dom/vitest";
import { afterAll, afterEach, beforeAll, vi } from "vitest";

import { server } from "./src/mocks/server";

beforeAll(() => server.listen({ onUnhandledRequest: "warn" }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    refresh: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    prefetch: vi.fn(),
  }),
  usePathname: () => "/test",
  useSearchParams: () => new URLSearchParams(),
  useParams: () => ({}),
}));
