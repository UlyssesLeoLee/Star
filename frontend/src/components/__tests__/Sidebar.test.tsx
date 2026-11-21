/*
@cypher schema=1 source_sha256=c00764d97600ea1e481df18a4fbf502d44a24da1fa529b8956243e8a8f3c732e
MERGE (self:File {path:"frontend/src/components/__tests__/Sidebar.test.tsx"})
MERGE (sidebar:Symbol {id:"frontend/src/components/Sidebar.tsx::Sidebar",kind:"component"})
MERGE (matrix:Symbol {id:"frontend/src/components/nav/AppMatrixDrawer.tsx::AppMatrixDrawer",kind:"component"})
MERGE (projectViews:Symbol {id:"frontend/src/components/__tests__/Sidebar.test.tsx::keeps the directory mounted and the filter shared between views",kind:"test"})
MERGE (hover:Symbol {id:"frontend/src/components/__tests__/Sidebar.test.tsx::opens the shortcuts on hover and keeps them outside the Project panels",kind:"test"})
MERGE (pin:Symbol {id:"frontend/src/components/__tests__/Sidebar.test.tsx::pins a module from the shared picker and removes it from the shortcut list",kind:"test"})
MERGE (self)-[:IMPORTS]->(sidebar)
MERGE (self)-[:IMPORTS]->(matrix)
MERGE (self)-[:DEFINES]->(projectViews)
MERGE (self)-[:DEFINES]->(hover)
MERGE (self)-[:DEFINES]->(pin)
MERGE (projectViews)-[:TESTS]->(sidebar)
MERGE (hover)-[:TESTS]->(sidebar)
MERGE (pin)-[:TESTS]->(sidebar)
MERGE (pin)-[:TESTS]->(matrix)
@endcypher
*/
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { Sidebar } from "../Sidebar";
import { AppMatrixDrawer } from "../nav/AppMatrixDrawer";
import { useNavStore } from "@/lib/nav/navStore";
import { I18nProvider } from "@/lib/i18n";

vi.mock("next/navigation", () => ({
  usePathname: () => "/inbox",
  useSearchParams: () => new URLSearchParams(""),
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

function renderSidebar() {
  return render(
    <I18nProvider initialLanguage="zh-CN">
      <Sidebar />
      <AppMatrixDrawer />
    </I18nProvider>,
  );
}

describe("Sidebar Project navigation", () => {
  beforeEach(() => {
    window.localStorage.clear();
    useNavStore.setState({
      sidebarItemIds: [],
      sidebarFold: "expanded",
      sidebarTab: "directory",
      taskStatusFilter: "all",
      isMatrixOpen: false,
    });
  });

  afterEach(cleanup);

  it("keeps the directory mounted and the filter shared between views", () => {
    renderSidebar();

    const tree = screen.getByTestId("project-run-directory");
    const directory = screen.getByTestId("sidebar-directory-panel");
    const directoryTab = screen.getByTestId("sidebar-tab-directory");
    const updates = screen.getByTestId("sidebar-ai-updates-panel");
    const blocked = screen.getByTestId("sidebar-status-blocked");
    expect(directory).toBeVisible();
    expect(screen.getByTestId("sidebar-brand")).toHaveAttribute("href", "/worktree");

    fireEvent.click(blocked);
    fireEvent.keyDown(directoryTab, { key: "ArrowRight" });
    expect(updates).toBeVisible();
    expect(directory).not.toBeVisible();
    expect(screen.getByTestId("sidebar-tab-ai-updates")).toHaveFocus();
    expect(blocked).toBeVisible();
    expect(blocked).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByTestId("sidebar-status-review"));
    fireEvent.click(directoryTab);
    expect(screen.getByTestId("project-run-directory")).toBe(tree);
    expect(directory).toBeVisible();
    expect(screen.getByTestId("sidebar-status-review")).toHaveAttribute("aria-pressed", "true");
  });

  it("opens the shortcuts on hover and keeps them outside the Project panels", () => {
    useNavStore.setState({ sidebarItemIds: ["scm"] });
    renderSidebar();

    const trigger = screen.getByTestId("sidebar-shortcuts-trigger");
    const panel = screen.getByTestId("sidebar-shortcuts-panel");
    expect(panel).not.toBeVisible();
    fireEvent.pointerOver(trigger, { pointerType: "mouse" });
    expect(panel).toBeVisible();
    expect(within(panel).getByTestId("sidebar-item-scm")).toBeVisible();
    expect(within(panel).getByTestId("sidebar-item-inbox")).toHaveAttribute("aria-current", "page");
    expect(within(panel).getByTestId("sidebar-item-agents")).not.toHaveAttribute("aria-current");

    fireEvent.pointerOut(trigger, { relatedTarget: panel, pointerType: "mouse" });
    expect(panel).toBeVisible();
    expect(within(screen.getByTestId("sidebar-directory-panel")).queryByTestId("sidebar-item-scm")).toBeNull();
    expect(within(screen.getByTestId("sidebar-bottom-fixed")).queryByTestId("sidebar-item-inbox")).toBeNull();
    fireEvent.pointerOut(panel, { relatedTarget: document.body, pointerType: "mouse" });
    expect(panel).not.toBeVisible();
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  it("pins a module from the shared picker and removes it from the shortcut list", () => {
    renderSidebar();

    const trigger = screen.getByTestId("sidebar-shortcuts-trigger");
    fireEvent.click(trigger);
    fireEvent.click(screen.getByTestId("sidebar-add-shortcut"));
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByRole("textbox"), { target: { value: "SCM" } });
    fireEvent.click(within(dialog).getByTestId("pin-sidebar-scm"));
    expect(useNavStore.getState().sidebarItemIds).toContain("scm");
    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();

    fireEvent.click(trigger);
    expect(screen.getByTestId("sidebar-item-scm")).toBeVisible();
    fireEvent.click(screen.getByTestId("sidebar-remove-scm"));
    expect(screen.queryByTestId("sidebar-item-scm")).toBeNull();
    expect(useNavStore.getState().sidebarItemIds).not.toContain("scm");
  });

  it("keeps shortcuts available when the Project sidebar is folded", () => {
    renderSidebar();

    const sidebar = screen.getByTestId("app-sidebar");
    fireEvent.click(screen.getByTestId("sidebar-fold-toggle"));
    expect(sidebar).toHaveAttribute("data-fold", "collapsed");
    expect(screen.queryByTestId("sidebar-top-tabs")).toBeNull();

    const trigger = screen.getByTestId("sidebar-shortcuts-trigger");
    fireEvent.click(trigger);
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    const link = screen.getByTestId("sidebar-item-agents");
    link.focus();
    fireEvent.keyDown(link, { key: "Escape" });
    expect(trigger).toHaveFocus();
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });
});
