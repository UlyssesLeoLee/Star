// =====================================================================
// panels.test.tsx — 4 panel render smoke tests (U4 极简任务)
// =====================================================================
// 5 个测试:
//   1. /agents renders without error
//   2. /analytics renders without error
//   3. /inbox renders without error
//   4. /settings renders without error
//   5. /settings tab switching (extra)
//
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   - 真实数据全部 mock, 4 panel 标记 P2/P3 缺口
//   - 集成测试 (跨 store / API) 不在本测试范围
// =====================================================================

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, cleanup, fireEvent } from "@testing-library/react";
import type { ReactNode } from "react";

// next/navigation mock — AppRouterContext 避免 import 真实 next
// per DRIFT-α-007 修复验证: settings/page.tsx 现直接从 searchParams 派生 tab
// (不再有本地 useState 镜像), 需要 mock 能读/写同一份 params 才能测 URL 深链 +
// setTab 产生的 href — 用 vi.hoisted 避免 mock factory 引用 TDZ 变量
const { mockPush, getMockSearchParams, setMockSearchParams } = vi.hoisted(() => {
  let params = new URLSearchParams();
  return {
    mockPush: vi.fn((href: string) => {
      const qIndex = href.indexOf("?");
      params = new URLSearchParams(qIndex >= 0 ? href.slice(qIndex + 1) : "");
    }),
    getMockSearchParams: () => params,
    setMockSearchParams: (next: URLSearchParams) => { params = next; },
  };
});
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush, replace: vi.fn(), back: vi.fn() }),
  usePathname: () => "/test",
  useSearchParams: () => getMockSearchParams(),
}));

import AgentsPage from "../agents/page";
import AnalyticsPage from "../analytics/page";
import InboxPage from "../inbox/page";
import SettingsPage from "../settings/page";
import { I18nProvider } from "@/lib/i18n";

// per 2026-08-31 i18n 补缺口: PageHeader 内 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

describe("U4 minimal panels — render smoke", () => {
  beforeEach(() => {
    // 每个测试前清理
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
    setMockSearchParams(new URLSearchParams());
    mockPush.mockClear();
  });
  afterEach(() => {
    cleanup();
  });

  it("renders /agents without error", () => {
    renderWithI18n(<AgentsPage />);
    expect(screen.getByTestId("agents-page")).toBeInTheDocument();
    // v0.6: zh-CN 下 page title "Agent 总览"
    expect(screen.getByText("Agent 总览")).toBeInTheDocument();
    // 5 mock 行
    expect(screen.getByTestId("agent-row-ag-001")).toBeInTheDocument();
    expect(screen.getByTestId("agent-row-ag-005")).toBeInTheDocument();
    // P3 占位
    expect(screen.getByTestId("live-activity-placeholder")).toBeInTheDocument();
  });

  it("renders /analytics without error", () => {
    renderWithI18n(<AnalyticsPage />);
    expect(screen.getByTestId("analytics-page")).toBeInTheDocument();
    // v0.6: zh-CN 下 page title "效能分析"
    expect(screen.getByText("效能分析")).toBeInTheDocument();
    // KPI 4 个 (label 全部渲染)
    expect(screen.getByText("Cost (24h)")).toBeInTheDocument();
    expect(screen.getByText("Tokens (24h)")).toBeInTheDocument();
    // Default tab is burndown
    expect(screen.getByTestId("tab-burndown")).toBeInTheDocument();
    // Switch to cost tab -> SVG 折线图存在
    const costTab = screen.getByRole("tab", { name: /Cost/i });
    fireEvent.click(costTab);
    expect(screen.getByTestId("cost-trend-chart")).toBeInTheDocument();
  });

  it("renders /inbox without error", () => {
    renderWithI18n(<InboxPage />);
    expect(screen.getByTestId("inbox-page")).toBeInTheDocument();
    // v0.6 (per 2026-09-05 拍板 C 全 i18n 接管): zh-CN 下 page title "收件箱"
    expect(screen.getByText("收件箱")).toBeInTheDocument();
    // 10 mock 通知
    const list = screen.getByTestId("inbox-list");
    expect(list.querySelectorAll("li[data-testid^='inbox-item-']").length).toBe(10);
  });

  it("renders /settings without error (default tab = profile)", () => {
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-page")).toBeInTheDocument();
    // v0.6: zh-CN 下 page title "系统设置"
    expect(screen.getByText("系统设置")).toBeInTheDocument();
    // default profile tab
    expect(screen.getByTestId("settings-panel-profile")).toBeInTheDocument();
    // 5 个 tab 通过 Tabs 渲染 — role="tablist"
    expect(screen.getByRole("tablist")).toBeInTheDocument();
  });

  it("switches /settings tab to api keys", () => {
    const { rerender } = renderWithI18n(<SettingsPage />);
    // v0.6: zh-CN 下 tab "API 凭据"
    const apiTab = screen.getByRole("tab", { name: /API 凭据/i });
    fireEvent.click(apiTab);
    // per DRIFT-α-007: tab 纯从 URL 派生 (无本地 state), 点击先触发 router.push
    // 写入正确的 ?tab=apikeys ...
    expect(mockPush).toHaveBeenCalledWith("/test?tab=apikeys");
    // ...真实 App Router 会在 push 后重渲染同一棵树, mock 的 push 已把新
    // params 写回 getMockSearchParams() 读的那份状态 — 这里手动 rerender
    // 模拟那次重渲染, 断言面板真的切到了 apikeys (而不只是 push 参数对了)
    rerender(<I18nProvider initialLanguage="zh-CN"><SettingsPage /></I18nProvider>);
    expect(screen.getByTestId("settings-panel-apikeys")).toBeInTheDocument();
  });

  it("deep-links /settings?tab=billing straight to the Billing panel", () => {
    // per DRIFT-α-007 修复核心验证: URL ?tab= 优先于默认 profile
    setMockSearchParams(new URLSearchParams("tab=billing"));
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-billing")).toBeInTheDocument();
  });

  it("falls back to profile when /settings?tab= is not one of the 5 implemented tabs", () => {
    // per DRIFT-α-007 已知缺口 #3: permissions/members/workspace/integrations
    // 4 个 legacy redirect 目标 tab 本身未实装, 未命中时应落 profile 而非崩溃
    setMockSearchParams(new URLSearchParams("tab=permissions"));
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-profile")).toBeInTheDocument();
  });
});
