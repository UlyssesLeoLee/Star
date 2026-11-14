/*
@cypher schema=1 source_sha256=9361b125393b3fc81d34c7a625157afe225404c72e645dccaae91e306b2a42d8
MERGE (self:File {path:"frontend/src/app/(app)/__tests__/panels.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/__tests__/panels.test.tsx::U4 minimal panels",kind:"suite"})
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
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

  it("renders /settings without error (default panel = MCP 服务器, per 2026-11-14 参考 UI)", () => {
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-page")).toBeInTheDocument();
    // 左导航 5 组 (per mockup Settings.dc.html)
    expect(screen.getByTestId("settings-nav")).toBeInTheDocument();
    expect(screen.getByTestId("settings-nav-profile")).toBeInTheDocument();
    expect(screen.getByTestId("settings-nav-mcp")).toBeInTheDocument();
    expect(screen.getByTestId("settings-nav-dev-mode")).toBeInTheDocument();
    // 默认面板 = MCP 服务器 (mockup 主区)
    expect(screen.getByTestId("settings-panel-mcp")).toBeInTheDocument();
    // MCP 服务器卡片 (官方目录)
    expect(screen.getByTestId("settings-mcp-GitHub")).toBeInTheDocument();
  });

  it("deep-links /settings?tab=apikeys to 凭据与 API Key panel", () => {
    // per mockup: 左导航 = <a href="/settings?tab=...">, 面板从 URL 派生
    setMockSearchParams(new URLSearchParams("tab=apikeys"));
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-apikeys")).toBeInTheDocument();
  });

  it("deep-links /settings?tab=billing to the 租户 panel (legacy tab map)", () => {
    // per DRIFT-α-007: URL ?tab= 优先; billing → 租户 (per mockup 组织与权限组)
    setMockSearchParams(new URLSearchParams("tab=billing"));
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-tenant")).toBeInTheDocument();
  });

  it("falls back to MCP panel when /settings?tab= is not implemented", () => {
    // 未实装/未知 tab → 默认 MCP 面板 (不崩溃)
    setMockSearchParams(new URLSearchParams("tab=permissions"));
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-mcp")).toBeInTheDocument();
  });
});
