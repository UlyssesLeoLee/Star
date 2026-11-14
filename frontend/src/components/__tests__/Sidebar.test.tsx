/*
@cypher schema=1 source_sha256=d9c7a19b1312e39a5e5e0db5952bc7abab4eeb448007d4e50c26421e5eff839e
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
    sidebarItemIds: ["inbox", "issues", "projects", "agents", "advanced-settings"],
    pinnedViewIds: ["kanban", "timeline"],
    headerTabIds: ["inbox", "issues", "projects", "agents", "analytics"],
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

  it("keeps Home 核心区 primary per ui-3pane-arch §1.2 (5 项常驻) and secondary routes reachable", () => {
    renderWithI18n(<Sidebar />);

    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar).toHaveAttribute("data-nav-model", "project-run");
    expect(sidebar).toHaveAttribute("data-fold", "expanded");
    expect(screen.getByRole("navigation", { name: "Project 导航" })).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-brand")).toHaveAttribute("href", "/worktree");
    // 设计书 §1.2: 侧栏顶部不再渲染 Project→Run→Worktree 状态树
    // (该状态树移入 Worktree 控制中心页; 2026-11-14 实测对照设计书)
    expect(screen.queryByTestId("project-run-directory")).toBeNull();
    expect(screen.queryByTestId("sidebar-scope-toggle")).toBeNull();

    // Home 核心区 5 项 (Worktree 控制中心 / Agent / Feedback / Validation / Review)
    const homeCore = screen.getByTestId("sidebar-group-home-core");
    expect(within(homeCore).getByTestId("sidebar-item-worktree")).toBeInTheDocument();
    expect(within(homeCore).getByTestId("sidebar-item-agents")).toBeInTheDocument();
    expect(within(homeCore).getByTestId("sidebar-item-feedback")).toBeInTheDocument();
    expect(within(homeCore).getByTestId("sidebar-item-validation")).toBeInTheDocument();
    expect(within(homeCore).getByTestId("sidebar-item-review")).toBeInTheDocument();

    const otherEntries = screen.getByTestId("sidebar-other-entries");
    expect(otherEntries.tagName).toBe("DETAILS");
    expect(within(otherEntries).getByTestId("sidebar-item-inbox")).toBeInTheDocument();
    expect(within(otherEntries).getByTestId("sidebar-item-advanced-settings")).toBeInTheDocument();
  });

  it("keeps the fold control available and removes the static success indicator", () => {
    renderWithI18n(<Sidebar />);

    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar.className).toMatch(/w-64/);
    fireEvent.click(screen.getByTestId("sidebar-fold-toggle"));
    expect(sidebar).toHaveAttribute("data-fold", "collapsed");
    expect(sidebar.className).toMatch(/w-16/);
    expect(screen.getByTestId("sidebar-other-entries")).toBeInTheDocument();
    expect(screen.queryByText("ALL GREEN")).toBeNull();
  });

  it("shows Project workspace views in the 视图组 without switching navigation scope", () => {
    mockUsePathname.mockReturnValue("/projects");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=kanban"));
    renderWithI18n(<Sidebar />);

    expect(screen.queryByTestId("sidebar-scope-toggle")).toBeNull();
    // 视图组 (per ui-3pane-arch §1.2): 看板/时间线/待办池 + 重构/搜索
    const viewGroup = screen.getByTestId("sidebar-group-view");
    expect(within(viewGroup).getByTestId("sidebar-item-kanban")).toBeInTheDocument();
    expect(within(viewGroup).getByTestId("sidebar-item-timeline")).toBeInTheDocument();
    expect(within(viewGroup).getByTestId("sidebar-item-backlog")).toBeInTheDocument();
    expect(within(viewGroup).getByTestId("sidebar-item-refactor")).toBeInTheDocument();
    expect(within(viewGroup).getByTestId("sidebar-item-search")).toBeInTheDocument();
  });

  // per 2026-11-12 用户发令 (梳理重复导航 / 整合在一起):
  //   /projects 有 SubNav 时, pinned 的 kanban/timeline (同 pathname)
  //   不再重复渲染 — 视图组已是单一来源。
  it("hides pinned modules that duplicate the current page's views", () => {
    mockUsePathname.mockReturnValue("/projects");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=kanban"));
    renderWithI18n(<Sidebar />);

    // 视图组仍然完整
    const viewGroup = screen.getByTestId("sidebar-group-view");
    expect(within(viewGroup).getByTestId("sidebar-item-timeline")).toBeInTheDocument();
    // pinned kanban/timeline/projects 不在 "other entries" 里重复出现
    const otherEntries = screen.getByTestId("sidebar-other-entries");
    expect(within(otherEntries).queryByTestId("sidebar-item-kanban")).toBeNull();
    expect(within(otherEntries).queryByTestId("sidebar-item-timeline")).toBeNull();
    expect(within(otherEntries).queryByTestId("sidebar-item-projects")).toBeNull();
    // 非本页模块保留 (inbox)
    expect(within(otherEntries).getByTestId("sidebar-item-inbox")).toBeInTheDocument();
  });

  // per 2026-11-12 用户发令: 带 query 的模块按 query 精确判定 active
  it("query-carrying modules activate only when the tab matches", () => {
    useNavStore.setState({
      sidebarItemIds: [],
      pinnedViewIds: ["kanban", "webhooks"],
    });
    mockUsePathname.mockReturnValue("/settings/developer");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=webhooks"));
    renderWithI18n(<Sidebar />);

    const otherEntries = screen.getByTestId("sidebar-other-entries");
    // 展开态 active 以 active-dot 表示 (SidebarRow 无 data-active 属性)
    expect(within(otherEntries).getByTestId("sidebar-item-active-dot-webhooks")).toBeInTheDocument();
    // kanban 指向 /projects, pathname 不同 → inactive (无 active-dot)
    expect(within(otherEntries).queryByTestId("sidebar-item-active-dot-kanban")).toBeNull();
  });
});
