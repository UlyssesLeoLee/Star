/*
@cypher schema=1 source_sha256=55b72f76dab3894b76b98133f85bea0a58574669cde3bddfc78d8064dd191244
MERGE (self:File {path:"frontend/src/components/run/ProjectRunTree.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::ProjectRunTree behavior and localization",kind:"suite"})
MERGE (selection:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::selecting a Project updates the Project context and clears Worktree focus",kind:"test"})
MERGE (expansion:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::expanding Project and Run loads descendants without selecting them",kind:"test"})
MERGE (keyboard:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::ArrowRight expands Project and Run without selecting either",kind:"test"})
MERGE (localized:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::localizes hierarchy and fail-closed state",kind:"test"})
MERGE (component:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (api:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient",kind:"type"})
MERGE (provider:Symbol {id:"frontend/src/lib/i18n/I18nProvider.tsx::I18nProvider",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(selection)
MERGE (suite)-[:DEFINES]->(expansion)
MERGE (suite)-[:DEFINES]->(keyboard)
MERGE (suite)-[:DEFINES]->(localized)
MERGE (self)-[:IMPORTS]->(component)
MERGE (self)-[:IMPORTS]->(api)
MERGE (self)-[:IMPORTS]->(provider)
MERGE (selection)-[:TESTS]->(component)
MERGE (selection)-[:USES_TYPE]->(api)
MERGE (expansion)-[:TESTS]->(component)
MERGE (expansion)-[:USES_TYPE]->(api)
MERGE (keyboard)-[:TESTS]->(component)
MERGE (keyboard)-[:USES_TYPE]->(api)
MERGE (localized)-[:TESTS]->(component)
MERGE (localized)-[:TESTS]->(provider)
@endcypher
*/

import type { AnchorHTMLAttributes, ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n/I18nProvider";
import type { Language } from "@/lib/i18n/config";
import type { RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { ProjectRunTree } from "./ProjectRunTree";

const projectId = "11111111-1111-4111-8111-111111111111";
const runId = "22222222-2222-4222-8222-222222222222";

const mocks = vi.hoisted(() => ({
  message: "No authorized directory connection",
  session: null as unknown,
  nav: {
    selectedProjectId: "",
    setSelectedProjectId: vi.fn(),
    setSidebarScope: vi.fn(),
  },
}));

vi.mock("@/lib/run/runDirectorySession", () => ({
  useRunDirectory: () => mocks.session ?? {
    status: "unavailable",
    message: mocks.message,
    focus: null,
    clearFocus: vi.fn(),
  },
}));
vi.mock("@/lib/nav/navStore", () => ({
  useNavStore: (selector: (state: typeof mocks.nav) => unknown) => selector(mocks.nav),
}));
vi.mock("next/navigation", () => ({ usePathname: () => "/" }));
vi.mock("@/components/navigation/GuardedLink", () => ({
  GuardedLink: ({ href, children, onNavigate, ...props }: AnchorHTMLAttributes<HTMLAnchorElement> & {
    href: string;
    children: ReactNode;
    onNavigate?: () => void;
  }) => <a href={href} {...props} onClick={(event) => {
    event.preventDefault();
    onNavigate?.();
  }}>{children}</a>,
}));

function installSession() {
  const client = {
    listProjects: vi.fn().mockResolvedValue({
      rows: [{ project_id: projectId, role: "owner" }], limit: 50, next_cursor: null,
    }),
    listProjectRuns: vi.fn().mockResolvedValue({
      rows: [{ project_id: projectId, engineering_run_id: runId, title: "Focused Run", state: "active" }],
      limit: 50,
      next_cursor: null,
    }),
    listWorktrees: vi.fn().mockResolvedValue({ rows: [], limit: 50, next_cursor: null }),
  } as unknown as RunDirectoryApiClient;
  const clearFocus = vi.fn();
  mocks.session = { status: "ready", client, message: "", focus: null, clearFocus };
  mocks.nav.selectedProjectId = "";
  mocks.nav.setSelectedProjectId.mockClear();
  mocks.nav.setSidebarScope.mockClear();
  return { client, clearFocus };
}

function renderTree(language: Language = "en") {
  return render(
    <I18nProvider initialLanguage={language}>
      <ProjectRunTree collapsed={false} />
    </I18nProvider>,
  );
}

describe("ProjectRunTree behavior and localization", () => {
  beforeEach(() => {
    window.localStorage.clear();
    mocks.session = null;
    mocks.message = "No authorized directory connection";
  });

  afterEach(() => {
    window.localStorage.clear();
  });

  it("selecting a Project updates the Project context and clears Worktree focus", async () => {
    const { clearFocus } = installSession();
    renderTree();

    const project = await screen.findByRole("treeitem", { name: `Select Project ${projectId}` });
    expect(project).toHaveAttribute("href", `/worktree?project_id=${projectId}`);
    expect(project).toHaveAttribute("aria-selected", "false");
    fireEvent.click(project);

    expect(mocks.nav.setSelectedProjectId).toHaveBeenCalledWith(projectId);
    expect(mocks.nav.setSidebarScope).toHaveBeenCalledWith("project");
    expect(clearFocus).toHaveBeenCalledTimes(1);
  });

  it("expanding Project and Run loads descendants without selecting them", async () => {
    const { client } = installSession();
    renderTree();

    await screen.findByRole("treeitem", { name: `Select Project ${projectId}` });
    fireEvent.click(screen.getByRole("button", { name: `Expand Project ${projectId}` }));
    const run = await screen.findByRole("treeitem", { name: "Open Engineering Run Focused Run" });
    expect(run).toHaveAttribute("href", `/projects/${projectId}/runs/${runId}`);
    expect(run).toHaveAttribute("aria-selected", "false");
    expect(screen.queryByRole("link", { name: /Open Task Cards/ })).toBeNull();
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Expand Focused Run" }));
    await waitFor(() => expect(client.listWorktrees).toHaveBeenCalled());
    expect(client.listProjectRuns).toHaveBeenCalledWith(projectId, undefined, expect.any(AbortSignal));
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();
  });

  it("ArrowRight expands Project and Run without selecting either", async () => {
    const { client } = installSession();
    renderTree();

    const project = await screen.findByRole("treeitem", { name: `Select Project ${projectId}` });
    fireEvent.keyDown(project, { key: "ArrowRight" });
    const run = await screen.findByRole("treeitem", { name: "Open Engineering Run Focused Run" });
    fireEvent.keyDown(run, { key: "ArrowRight" });

    await waitFor(() => expect(client.listWorktrees).toHaveBeenCalled());
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();
    expect(mocks.nav.setSidebarScope).not.toHaveBeenCalled();
  });

  it.each([
    ["en", "Project navigation", "Project → Engineering Run → Worktree → Branch directory", "Engineering Run → Local Worktree → Branch"],
    ["zh-CN", "Project 导航", "Project → Engineering Run → Worktree → Branch 目录", "Engineering Run → 本地 Worktree → Branch"],
    ["ja", "Project ナビゲーション", "Project → Engineering Run → Worktree → Branch ディレクトリ", "Engineering Run → ローカル Worktree → Branch"],
  ] as const)("localizes hierarchy and fail-closed state for %s", (language, navigation, directoryLabel, hierarchy) => {
    renderTree(language as Language);

    expect(screen.getByText(navigation)).toBeInTheDocument();
    expect(screen.getByRole("region", { name: directoryLabel })).toBeInTheDocument();
    expect(screen.getByText(hierarchy)).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("No authorized directory connection");
  });
});
