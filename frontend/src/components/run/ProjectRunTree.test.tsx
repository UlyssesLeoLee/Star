/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/run/ProjectRunTree.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::ProjectRunTree behavior and localization",kind:"suite"})
MERGE (selection:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::selecting a Project updates the Project context and clears Worktree focus",kind:"test"})
MERGE (expansion:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::expanding Project loads server Lanes and Lane Worktrees without selecting them",kind:"test"})
MERGE (laneSelect:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::selecting a Lane updates the Lane context and clears Worktree focus",kind:"test"})
MERGE (createLane:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::creating a Lane calls the server Lane API and refreshes the Project",kind:"test"})
MERGE (demoBadge:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::fallback tree marks demo directory data with a badge",kind:"test"})
MERGE (localized:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::localizes hierarchy and fail-closed state",kind:"test"})
MERGE (component:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (api:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient",kind:"type"})
MERGE (provider:Symbol {id:"frontend/src/lib/i18n/I18nProvider.tsx::I18nProvider",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(selection)
MERGE (suite)-[:DEFINES]->(expansion)
MERGE (suite)-[:DEFINES]->(laneSelect)
MERGE (suite)-[:DEFINES]->(createLane)
MERGE (suite)-[:DEFINES]->(demoBadge)
MERGE (suite)-[:DEFINES]->(localized)
MERGE (self)-[:IMPORTS]->(component)
MERGE (self)-[:IMPORTS]->(api)
MERGE (self)-[:IMPORTS]->(provider)
@endcypher
*/

import type { AnchorHTMLAttributes, ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n/I18nProvider";
import type { Language } from "@/lib/i18n/config";
import type { RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { useStore } from "@/lib/store";
import { ProjectRunTree } from "./ProjectRunTree";

const projectId = "11111111-1111-4111-8111-111111111111";
const laneId = "22222222-2222-4222-8222-222222222222";
const worktreeId = "33333333-3333-4333-8333-333333333333";

const mocks = vi.hoisted(() => ({
  message: "No authorized directory connection",
  session: null as unknown,
  nav: {
    selectedProjectId: "",
    selectedLaneId: "",
    selectedWorktreeId: "",
    setSelectedProjectId: vi.fn(),
    setSidebarScope: vi.fn(),
    setLaneContext: vi.fn(),
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
vi.mock("@/lib/run/useFallbackRunDirectory", () => ({
  useFallbackRunDirectory: () => undefined,
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

function laneFixture(overrides: Record<string, unknown> = {}) {
  return {
    lane_id: laneId,
    project_id: projectId,
    name: "core-runtime",
    description: null,
    created_by: "44444444-4444-4444-8444-444444444444",
    lock_version: 1,
    created_at: "2026-11-20T00:00:00Z",
    updated_at: "2026-11-20T00:00:00Z",
    repository_binding: null,
    branch_binding: null,
    ...overrides,
  };
}

function laneWorktreeFixture(overrides: Record<string, unknown> = {}) {
  return {
    worktree_id: worktreeId,
    lane_id: laneId,
    project_id: projectId,
    repository_id: "55555555-5555-4555-8555-555555555555",
    name: "wt-core-checkout",
    branch: "feat/core",
    human_state: "active",
    machine_state: "ready",
    runtime_id: null,
    agent_id: null,
    agent_session_id: null,
    archived: false,
    version: 1,
    binding_id: "66666666-6666-4666-8666-666666666666",
    binding_version: 1,
    engineering_run_id: null,
    ...overrides,
  };
}

function installSession() {
  const client = {
    listProjects: vi.fn().mockResolvedValue({
      rows: [{ project_id: projectId, role: "owner" }], limit: 50, next_cursor: null,
    }),
    listLanes: vi.fn().mockResolvedValue({
      rows: [laneFixture()], limit: 50, next_cursor: null,
    }),
    listLaneWorktrees: vi.fn().mockResolvedValue({
      rows: [laneWorktreeFixture()], limit: 50, next_cursor: null,
    }),
    createLane: vi.fn().mockResolvedValue(laneFixture({ name: "new-lane" })),
  } as unknown as RunDirectoryApiClient;
  const clearFocus = vi.fn();
  mocks.session = { status: "ready", client, message: "", focus: null, clearFocus };
  mocks.nav.selectedProjectId = "";
  mocks.nav.selectedLaneId = "";
  mocks.nav.selectedWorktreeId = "";
  mocks.nav.setSelectedProjectId.mockClear();
  mocks.nav.setSidebarScope.mockClear();
  mocks.nav.setLaneContext.mockClear();
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
    useStore.setState({ projects: [], lanes: [], worktrees: [] });
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

  it("expanding Project loads server Lanes and Lane Worktrees without selecting them", async () => {
    const { client } = installSession();
    renderTree();

    await screen.findByRole("treeitem", { name: `Select Project ${projectId}` });
    fireEvent.click(screen.getByRole("button", { name: `Expand Project ${projectId}` }));
    // Project 的子层 = 服务端 Lane 实体 (listLanes), 不再是前端 EngineeringRun 分组
    const lane = await screen.findByRole("treeitem", { name: "Select Lane core-runtime" });
    expect(lane).toHaveAttribute("href", `/lane/${laneId}?project_id=${projectId}`);
    expect(lane).toHaveAttribute("aria-selected", "false");
    await waitFor(() => expect(client.listLanes).toHaveBeenCalledWith(projectId, undefined, expect.any(AbortSignal)));
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();
    expect(mocks.nav.setLaneContext).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Expand core-runtime" }));
    const worktree = await screen.findByRole("treeitem", { name: /wt-core-checkout/ });
    expect(worktree).toHaveAttribute(
      "href",
      `/lane/${laneId}?project_id=${projectId}&worktree_id=${worktreeId}`,
    );
    await waitFor(() => expect(client.listLaneWorktrees).toHaveBeenCalled());
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();
  });

  it("selecting a Lane updates the Lane context and clears Worktree focus", async () => {
    const { clearFocus } = installSession();
    renderTree();

    await screen.findByRole("treeitem", { name: `Select Project ${projectId}` });
    fireEvent.click(screen.getByRole("button", { name: `Expand Project ${projectId}` }));
    const lane = await screen.findByRole("treeitem", { name: "Select Lane core-runtime" });
    fireEvent.click(lane);

    expect(mocks.nav.setSelectedProjectId).toHaveBeenCalledWith(projectId);
    expect(mocks.nav.setLaneContext).toHaveBeenCalledWith(laneId, "", "");
    expect(clearFocus).toHaveBeenCalledTimes(1);
  });

  it("creating a Lane calls the server Lane API and refreshes the Project", async () => {
    const { client } = installSession();
    renderTree();

    await screen.findByRole("treeitem", { name: `Select Project ${projectId}` });
    fireEvent.click(screen.getByRole("button", { name: `Project ${projectId}: New Lane` }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Lane name"), { target: { value: "new-lane" } });
    fireEvent.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(client.createLane).toHaveBeenCalled());
    const args = (client.createLane as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(args[0]).toBe(projectId);
    expect(args[1]).toBe("new-lane");
  });

  it("fallback tree marks demo directory data with a badge", () => {
    useStore.setState({
      projects: [{ id: projectId, tenant_id: "10000000-0000-4000-8000-000000000001", key: "DEMO", name: "Demo Project", visibility: "private", owner_id: "x", member_count: 1, created_at: "2026-11-01T00:00:00.000Z" }],
      lanes: [{ id: laneId, tenant_id: "10000000-0000-4000-8000-000000000001", project_id: projectId, name: "demo-lane", created_at: "2026-11-01T00:00:00.000Z" }],
      worktrees: [],
    });
    renderTree();
    expect(screen.getByTestId("project-run-directory-fallback")).toBeInTheDocument();
    expect(screen.getByTestId("demo-directory-badge")).toBeInTheDocument();
  });

  it.each([
    ["en", "Project navigation", "Project → LANE → Worktree → Branch directory", "LANE → Local Worktree → Branch"],
    ["zh-CN", "Project 导航", "Project → LANE → Worktree → Branch 目录", "LANE → 本地 Worktree → Branch"],
    ["ja", "Project ナビゲーション", "Project → LANE → Worktree → Branch ディレクトリ", "LANE → ローカル Worktree → Branch"],
  ] as const)(`localizes hierarchy and fail-closed state for %s`, (language, navigation, directoryLabel, hierarchy) => {
    renderTree(language as Language);

    expect(screen.getByText(navigation)).toBeInTheDocument();
    expect(screen.getByRole("region", { name: directoryLabel })).toBeInTheDocument();
    expect(screen.getByText(hierarchy)).toBeInTheDocument();
    // per 2026-11-14 用户发令 "确保导航可达": missing 状态渲染可达降级目录 (演示目录 badge 标注)
    expect(screen.getByTestId("project-run-directory-fallback")).toBeInTheDocument();
    expect(screen.getByRole("tree")).toBeInTheDocument();
  });
});
