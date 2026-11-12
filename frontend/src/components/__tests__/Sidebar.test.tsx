/*
@cypher schema=1 source_sha256=8fe84e3376a742ccb0cef39893f419a85b50a9c370576eb3d0834e06da1a4bda
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

  it("keeps Project hierarchy primary and secondary routes reachable", () => {
    renderWithI18n(<Sidebar />);

    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar).toHaveAttribute("data-nav-model", "project-run");
    expect(sidebar).toHaveAttribute("data-fold", "expanded");
    expect(screen.getByRole("navigation", { name: "Project 导航" })).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-brand")).toHaveAttribute("href", "/projects");
    expect(screen.getByTestId("project-run-directory")).toBeInTheDocument();
    expect(screen.queryByTestId("sidebar-scope-toggle")).toBeNull();

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

  it("shows Project workspace views without switching navigation scope", () => {
    mockUsePathname.mockReturnValue("/projects");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=kanban"));
    renderWithI18n(<Sidebar />);

    expect(screen.queryByTestId("sidebar-scope-toggle")).toBeNull();
    expect(screen.getByTestId("sidebar-subnav-item-kanban")).toHaveAttribute("data-active", "true");
    expect(screen.getByTestId("sidebar-subnav-item-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-subnav-item-backlog")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-subnav-item-agents")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-subnav-item-worktrees")).toBeInTheDocument();
  });

  // per 2026-11-12 用户发令 (梳理重复导航 / 整合在一起):
  //   /projects 有 SubNav 时, pinned 的 kanban/timeline (同 pathname)
  //   不再重复渲染 — SubNav 已是 6 tab 的单一来源。
  it("hides pinned modules that duplicate the current page's SubNav", () => {
    mockUsePathname.mockReturnValue("/projects");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=kanban"));
    renderWithI18n(<Sidebar />);

    // SubNav 仍然完整
    expect(screen.getByTestId("sidebar-subnav-item-timeline")).toBeInTheDocument();
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
