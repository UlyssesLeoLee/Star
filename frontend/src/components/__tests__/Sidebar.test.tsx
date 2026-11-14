/*
@cypher schema=1 source_sha256=1446030f59650acfa7a15d4701fb667fe583d80838fb46724fcb1c7ebe4372e5
MERGE (self:File {path:"frontend/src/components/__tests__/Sidebar.test.tsx"})
MERGE (sidebar:Symbol {id:"frontend/src/components/Sidebar.tsx::Sidebar",kind:"component"})
MERGE (suite:Symbol {id:"frontend/src/components/__tests__/Sidebar.test.tsx::Sidebar Project-first navigation",kind:"suite"})
MERGE (primaryTest:Symbol {id:"frontend/src/components/__tests__/Sidebar.test.tsx::keeps Project hierarchy primary and secondary routes reachable",kind:"test"})
MERGE (foldTest:Symbol {id:"frontend/src/components/__tests__/Sidebar.test.tsx::keeps fold control available and removes static success indicator",kind:"test"})
MERGE (projectTabsTest:Symbol {id:"frontend/src/components/__tests__/Sidebar.test.tsx::shows Project workspace views without switching navigation scope",kind:"test"})
MERGE (self)-[:IMPORTS]->(sidebar)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(primaryTest)
MERGE (suite)-[:DEFINES]->(foldTest)
MERGE (suite)-[:DEFINES]->(projectTabsTest)
MERGE (primaryTest)-[:TESTS]->(sidebar)
MERGE (foldTest)-[:TESTS]->(sidebar)
MERGE (projectTabsTest)-[:TESTS]->(sidebar)
@endcypher
*/
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import type { ReactNode } from "react";

const mockUsePathname = vi.fn(() => "/inbox");
const mockUseSearchParams = vi.fn(() => new URLSearchParams(""));
vi.mock("next/navigation", () => ({
  usePathname: () => mockUsePathname(),
  useSearchParams: () => mockUseSearchParams(),
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

import { Sidebar } from "../Sidebar";
import { useNavStore } from "@/lib/nav/navStore";
import { I18nProvider } from "@/lib/i18n";

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

function resetNavStore() {
  useNavStore.setState({
    sidebarItemIds: [],
    pinnedViewIds: [],
    headerTabIds: ["kanban", "canvas", "nema", "sandbox", "prototype"],
    sidebarFold: "expanded",
    sidebarScope: "main",
    selectedProjectId: "",
    isMatrixOpen: false,
  });
}

describe("Sidebar Project-first navigation", () => {
  beforeEach(() => {
    cleanup();
    mockUsePathname.mockReturnValue("/inbox");
    mockUseSearchParams.mockReturnValue(new URLSearchParams(""));
    resetNavStore();
    window.localStorage.clear();
  });

  afterEach(() => {
    resetNavStore();
  });

  it("renders reference UI sidebar: LANE 树 + 底部全局组 (收件箱/Agent中心/分析), 无其他组", () => {
    renderWithI18n(<Sidebar />);

    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar).toHaveAttribute("data-nav-model", "project-run");
    expect(sidebar).toHaveAttribute("data-fold", "expanded");
    expect(screen.getByRole("navigation", { name: "Project 导航" })).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-brand")).toHaveAttribute("href", "/worktree");
    // per 2026-11-14 用户发令: 恢复 LANE 层级树 (无宿主时降级为可达演示目录)
    expect(screen.getByTestId("project-run-directory")).toBeInTheDocument();
    expect(screen.getByTestId("project-run-directory-fallback")).toBeInTheDocument();

    // per 2026-11-14 artifact 参考 UI: 侧栏只保留 项目列表 + 全局组;
    // Branch 功能 tabs 在顶栏, 项目/视图/管理/其他入口全部收敛 (App Matrix + 用户菜单)
    expect(screen.queryByTestId("sidebar-group-home-core")).toBeNull();
    expect(screen.queryByTestId("sidebar-group-project")).toBeNull();
    expect(screen.queryByTestId("sidebar-group-view")).toBeNull();
    expect(screen.queryByTestId("sidebar-group-admin")).toBeNull();
    expect(screen.queryByTestId("sidebar-other-entries")).toBeNull();

    // 全局组: 收件箱/Agent 中心/分析 (per artifact 左栏底部 全局)
    const globalGroup = screen.getByTestId("sidebar-group-global");
    expect(within(globalGroup).getByTestId("sidebar-item-inbox")).toBeInTheDocument();
    expect(within(globalGroup).getByTestId("sidebar-item-agents")).toBeInTheDocument();
    expect(within(globalGroup).getByTestId("sidebar-item-analytics")).toBeInTheDocument();
  });

  it("keeps the fold control available and removes the static success indicator", () => {
    renderWithI18n(<Sidebar />);

    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar.className).toMatch(/w-64/);
    fireEvent.click(screen.getByTestId("sidebar-fold-toggle"));
    expect(sidebar).toHaveAttribute("data-fold", "collapsed");
    expect(sidebar.className).toMatch(/w-16/);
    expect(screen.getByTestId("sidebar-group-global")).toBeInTheDocument();
    expect(screen.queryByText("ALL GREEN")).toBeNull();
  });

  it("global group stays constant across page scopes (per reference UI)", () => {
    mockUsePathname.mockReturnValue("/projects");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=kanban"));
    renderWithI18n(<Sidebar />);

    // 全局组不受当前页 scope 影响, 恒为 3 项
    const globalGroup = screen.getByTestId("sidebar-group-global");
    expect(within(globalGroup).getByTestId("sidebar-item-inbox")).toBeInTheDocument();
    expect(within(globalGroup).getByTestId("sidebar-item-agents")).toBeInTheDocument();
    expect(within(globalGroup).getByTestId("sidebar-item-analytics")).toBeInTheDocument();
  });

  // per 2026-11-14 artifact 参考 UI: 侧栏不再渲染 pinned/视图组 — 全站只有一个全局组
  it("does not render pinned or view modules anywhere in the sidebar", () => {
    mockUsePathname.mockReturnValue("/projects");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=timeline"));
    renderWithI18n(<Sidebar />);

    expect(screen.queryByTestId("sidebar-item-timeline")).toBeNull();
    expect(screen.queryByTestId("sidebar-item-kanban")).toBeNull();
    expect(screen.queryByTestId("sidebar-item-projects")).toBeNull();
    // 全局组恒在
    expect(within(screen.getByTestId("sidebar-group-global")).getByTestId("sidebar-item-inbox")).toBeInTheDocument();
  });

  // per 2026-11-12 用户发令: 带 query 的模块按 query 精确判定 active
  it("global group items activate by exact pathname match", () => {
    mockUsePathname.mockReturnValue("/inbox");
    mockUseSearchParams.mockReturnValue(new URLSearchParams(""));
    renderWithI18n(<Sidebar />);

    const globalGroup = screen.getByTestId("sidebar-group-global");
    // NavModuleRow 以 data-active 属性表示 active (非 active-dot)
    expect(within(globalGroup).getByTestId("sidebar-item-inbox").getAttribute("data-active")).toBe("true");
    expect(within(globalGroup).getByTestId("sidebar-item-agents").getAttribute("data-active")).toBe("false");
  });
});
