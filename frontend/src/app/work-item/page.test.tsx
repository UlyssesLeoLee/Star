// =====================================================================
// work-item/page.test.tsx — U2 改造: redirect 到 /issues
// =====================================================================
// 1 个测试 (per U2 spec):
//   1. /work-item 渲染时触发 redirect("/sprint")
//
// 已知缺口 (per 缺标比错标):
//   - next/navigation 的 redirect() 在 server-side 抛 NEXT_REDIRECT, 在 vitest jsdom 环境
//     通过 mock + throw 模拟行为。这里只验证 redirect 被调用。
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// ---- mock next/navigation ----
const mockRedirect = vi.fn((url: string) => {
  // next/navigation 的 redirect 真实行为: throw NEXT_REDIRECT
  // 这里 throw 模拟, 让 redirect 之后的代码不执行
  const err = new Error(`NEXT_REDIRECT: ${url}`);
  (err as any).digest = `NEXT_REDIRECT;replace;/issues;307;`;
  throw err;
});

vi.mock("next/navigation", () => ({
  redirect: (url: string) => mockRedirect(url),
}));

import WorkItemListPage from "./page";

describe("work-item redirect (U2)", () => {
  beforeEach(() => {
    mockRedirect.mockClear();
  });
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("redirects to /issues when rendered", () => {
    // redirect 抛 NEXT_REDIRECT, 用 try/catch 接住
    expect(() => WorkItemListPage()).toThrow(/NEXT_REDIRECT/);
    expect(mockRedirect).toHaveBeenCalledTimes(1);
    expect(mockRedirect).toHaveBeenCalledWith("/sprint");
  });
});
