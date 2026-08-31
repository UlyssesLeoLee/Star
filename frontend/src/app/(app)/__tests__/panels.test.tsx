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
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), back: vi.fn() }),
  usePathname: () => "/test",
  useSearchParams: () => new URLSearchParams(),
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
  });
  afterEach(() => {
    cleanup();
  });

  it("renders /agents without error", () => {
    renderWithI18n(<AgentsPage />);
    expect(screen.getByTestId("agents-page")).toBeInTheDocument();
    expect(screen.getByText("Agents")).toBeInTheDocument();
    // 5 mock 行
    expect(screen.getByTestId("agent-row-ag-001")).toBeInTheDocument();
    expect(screen.getByTestId("agent-row-ag-005")).toBeInTheDocument();
    // P3 占位
    expect(screen.getByTestId("live-activity-placeholder")).toBeInTheDocument();
  });

  it("renders /analytics without error", () => {
    renderWithI18n(<AnalyticsPage />);
    expect(screen.getByTestId("analytics-page")).toBeInTheDocument();
    expect(screen.getByText("Analytics")).toBeInTheDocument();
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
    expect(screen.getByText("Inbox")).toBeInTheDocument();
    // 10 mock 通知
    const list = screen.getByTestId("inbox-list");
    expect(list.querySelectorAll("li[data-testid^='inbox-item-']").length).toBe(10);
  });

  it("renders /settings without error (default tab = profile)", () => {
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-page")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
    // default profile tab
    expect(screen.getByTestId("settings-panel-profile")).toBeInTheDocument();
    // 5 个 tab 通过 Tabs 渲染 — role="tablist"
    expect(screen.getByRole("tablist")).toBeInTheDocument();
  });

  it("switches /settings tab to api keys", () => {
    renderWithI18n(<SettingsPage />);
    const apiTab = screen.getByRole("tab", { name: /API Keys/i });
    fireEvent.click(apiTab);
    expect(screen.getByTestId("settings-panel-apikeys")).toBeInTheDocument();
  });
});
