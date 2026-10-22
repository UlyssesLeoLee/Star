/*
@cypher schema=1 source_sha256=45e9a378cee3baf311ba0a45055a15c072b75a90b1a7479c1f7d8addfcc980ae
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
});
