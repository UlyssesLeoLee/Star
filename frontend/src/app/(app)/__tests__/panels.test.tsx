/*
@cypher schema=1 source_sha256=9ad6180ccf025d1151ce1c7abd6e4445df476bceb345e85af9dfcf1f82ce339e
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

// settings 页深链改为 window.location.search 解析 (不用 useSearchParams, per 2026-11-14)
let mockLocationSearch = "";
function setMockLocationSearch(qs: string) {
  mockLocationSearch = qs;
  const location = window.location;
  delete (window as unknown as Record<string, unknown>).location;
  (window as unknown as Record<string, unknown>).location = {
    ...location,
    search: qs,
    href: location.href.split("?")[0] + qs,
    assign: vi.fn(),
    replace: vi.fn(),
  };
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

  it("renders /analytics without error (per 2026-11-14 参考 UI: 3 子 tab + KPI + IPA 阶段进度)", () => {
    renderWithI18n(<AnalyticsPage />);
    // 4 KPI 卡片 (真实 store 数据驱动)
    expect(screen.getByTestId("analytics-kpi-0")).toBeInTheDocument();
    expect(screen.getByTestId("analytics-kpi-3")).toBeInTheDocument();
    expect(screen.getByText("完成任务")).toBeInTheDocument();
    expect(screen.getByText("IPA 技能覆盖")).toBeInTheDocument();
    // IPA 阶段进度条 (P2 → P6.2)
    expect(screen.getByTestId("analytics-ipa-phases")).toBeInTheDocument();
    expect(screen.getByText("需求定义")).toBeInTheDocument();
    expect(screen.getByText("集成测试")).toBeInTheDocument();
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
    // per mockup: 左导航 = <a href="/settings?tab=...">, 面板从 URL 派生 (window.location.search)
    setMockLocationSearch("?tab=apikeys");
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-apikeys")).toBeInTheDocument();
  });

  it("deep-links /settings?tab=billing to the 租户 panel (legacy tab map)", () => {
    // per DRIFT-α-007: URL ?tab= 优先; billing → 租户 (per mockup 组织与权限组)
    setMockLocationSearch("?tab=billing");
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-tenant")).toBeInTheDocument();
  });

  it("falls back to MCP panel when /settings?tab= is not implemented", () => {
    // 未实装/未知 tab → 默认 MCP 面板 (不崩溃)
    setMockLocationSearch("?tab=permissions");
    renderWithI18n(<SettingsPage />);
    expect(screen.getByTestId("settings-panel-mcp")).toBeInTheDocument();
  });
});
