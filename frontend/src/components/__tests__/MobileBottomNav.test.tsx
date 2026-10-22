/*
@cypher schema=1 source_sha256=f30c7a6b8009f91b5d02491b5e119f4dac7126d9cfa13354319499c490e99121
MERGE (self:File {path:"frontend/src/components/__tests__/MobileBottomNav.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/__tests__/MobileBottomNav.test.tsx::Mobile Project navigation",kind:"suite"})
MERGE (openDirectory:Symbol {id:"frontend/src/components/__tests__/MobileBottomNav.test.tsx::opens the Project hierarchy and renders its authorized tree",kind:"test"})
MERGE (legacyLinks:Symbol {id:"frontend/src/components/__tests__/MobileBottomNav.test.tsx::does not expose the former module shortcuts",kind:"test"})
MERGE (escape:Symbol {id:"frontend/src/components/__tests__/MobileBottomNav.test.tsx::Escape closes the directory and restores focus",kind:"test"})
MERGE (routeChange:Symbol {id:"frontend/src/components/__tests__/MobileBottomNav.test.tsx::route changes close the directory",kind:"test"})
MERGE (localization:Symbol {id:"frontend/src/components/__tests__/MobileBottomNav.test.tsx::localizes navigation labels for all supported languages",kind:"test"})
MERGE (component:Symbol {id:"frontend/src/components/MobileBottomNav.tsx::MobileBottomNav",kind:"component"})
MERGE (provider:Symbol {id:"frontend/src/lib/i18n/I18nProvider.tsx::I18nProvider",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(openDirectory)
MERGE (suite)-[:DEFINES]->(legacyLinks)
MERGE (suite)-[:DEFINES]->(escape)
MERGE (suite)-[:DEFINES]->(routeChange)
MERGE (suite)-[:DEFINES]->(localization)
MERGE (self)-[:IMPORTS]->(component)
MERGE (self)-[:IMPORTS]->(provider)
MERGE (openDirectory)-[:TESTS]->(component)
MERGE (legacyLinks)-[:TESTS]->(component)
MERGE (escape)-[:TESTS]->(component)
MERGE (routeChange)-[:TESTS]->(component)
MERGE (localization)-[:TESTS]->(component)
MERGE (localization)-[:TESTS]->(provider)
@endcypher
*/

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/lib/i18n/I18nProvider";
import type { Language } from "@/lib/i18n/config";
import { MobileBottomNav } from "../MobileBottomNav";

const mocks = vi.hoisted(() => ({ pathname: "/worktree" }));

vi.mock("next/navigation", () => ({ usePathname: () => mocks.pathname }));
vi.mock("@/components/run/ProjectRunTree", () => ({
  ProjectRunTree: ({ collapsed }: { collapsed: boolean }) => (
    <div data-testid="authorized-project-run-tree" data-collapsed={String(collapsed)}>
      Project → Engineering Run → Worktree → Branch
    </div>
  ),
}));

function renderNavigation(language: Language = "zh-CN") {
  return render(
    <I18nProvider initialLanguage={language}>
      <MobileBottomNav />
    </I18nProvider>,
  );
}

describe("Mobile Project navigation", () => {
  beforeEach(() => {
    mocks.pathname = "/worktree";
    window.localStorage.clear();
  });

  afterEach(() => {
    window.localStorage.clear();
  });

  it.each([
    ["zh-CN", "Project 导航", "打开 Project → Engineering Run → Worktree → Branch 导航目录", "Project 目录", "收起"],
    ["en", "Project navigation", "Open the Project → Engineering Run → Worktree → Branch directory", "Project directory", "Close"],
    ["ja", "Project ナビゲーション", "Project → Engineering Run → Worktree → Branch ディレクトリを開く", "Project ディレクトリ", "閉じる"],
  ] as const)("localizes navigation labels for %s", (language, navigation, openLabel, directoryTitle, closeLabel) => {
    renderNavigation(language);
    fireEvent.click(screen.getByTestId("mobile-nav-project"));

    expect(screen.getByRole("navigation", { name: navigation })).toBeInTheDocument();
    expect(screen.getByTestId("mobile-nav-project")).toHaveAttribute("aria-label", openLabel);
    expect(screen.getByRole("heading", { name: directoryTitle })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: closeLabel })).toBeInTheDocument();
  });

  it("opens the Project hierarchy and renders its authorized tree", () => {
    renderNavigation();

    const trigger = screen.getByTestId("mobile-nav-project");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(trigger);

    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByTestId("mobile-project-directory")).toBeVisible();
    expect(screen.getByTestId("authorized-project-run-tree")).toHaveAttribute("data-collapsed", "false");
  });

  it("does not expose the former module shortcuts", () => {
    renderNavigation();

    expect(screen.queryByText("首页")).not.toBeInTheDocument();
    expect(screen.queryByText("Agent")).not.toBeInTheDocument();
    expect(screen.queryByText("Feedback")).not.toBeInTheDocument();
    expect(screen.queryByText("更多")).not.toBeInTheDocument();
  });

  it("Escape closes the directory and restores focus", () => {
    renderNavigation();
    const trigger = screen.getByTestId("mobile-nav-project");
    fireEvent.click(trigger);

    fireEvent.keyDown(screen.getByTestId("mobile-project-directory"), { key: "Escape" });

    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(document.activeElement).toBe(trigger);
  });

  it("route changes close the directory", async () => {
    const view = renderNavigation();
    fireEvent.click(screen.getByTestId("mobile-nav-project"));
    expect(screen.getByTestId("mobile-project-directory")).toBeVisible();

    mocks.pathname = "/projects/project-1/runs/run-1/worktrees/worktree-1";
    view.rerender(
      <I18nProvider initialLanguage="zh-CN">
        <MobileBottomNav />
      </I18nProvider>,
    );

    await waitFor(() => expect(screen.getByTestId("mobile-project-directory")).not.toBeVisible());
  });
});
