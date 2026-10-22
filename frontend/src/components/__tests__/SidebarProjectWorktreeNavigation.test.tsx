/*
@cypher schema=1 source_sha256=4a0f2d091e56074639ad1f15691e3b3c336b2f6a29ec028d9b5c77b051f35246
MERGE (self:File {path:"frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx"})
MERGE (sidebar:File {path:"frontend/src/components/Sidebar.tsx"})
MERGE (tree:File {path:"frontend/src/components/run/ProjectRunTree.tsx"})
MERGE (i18n:File {path:"frontend/src/lib/i18n/index.ts"})
MERGE (session:File {path:"frontend/src/lib/run/runDirectorySession.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx::Project Worktree Index sidebar navigation",kind:"suite"})
MERGE (navigationTest:Symbol {id:"frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx::keeps the Project hierarchy primary and exposes the authorized Project Worktree Index",kind:"test"})
MERGE (statusTest:Symbol {id:"frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx::Run Worktree rows distinguish lifecycle and binding facts without exposing identifiers",kind:"test"})
MERGE (fetcher:Symbol {id:"frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx::directoryStatusFetcher",kind:"function"})
MERGE (self)-[:IMPORTS]->(sidebar)
MERGE (self)-[:IMPORTS]->(tree)
MERGE (self)-[:IMPORTS]->(i18n)
MERGE (self)-[:IMPORTS]->(session)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(navigationTest)
MERGE (suite)-[:DEFINES]->(statusTest)
MERGE (statusTest)-[:USES]->(fetcher)
MERGE (statusTest)-[:TESTS]->(tree)
MERGE (navigationTest)-[:TESTS]->(sidebar)
@endcypher
*/

// CYPHER STRUCTURAL MANIFEST
// CREATE
//   (file:File {name:"SidebarProjectWorktreeNavigation.test.tsx",type:"file",language:"typescript"}),
//   (module:Module {name:"SidebarProjectWorktreeNavigation",type:"module",language:"typescript"}),
//   (suite:Function {name:"Project Worktree Index sidebar navigation suite",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
//   (test:Function {name:"keeps the Project hierarchy primary and exposes the authorized Project Worktree Index",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
//   (statusTest:Function {name:"Run Worktree rows distinguish lifecycle and binding facts without exposing identifiers",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
//   (projectId:Variable {name:"projectId",type:"variable",language:"typescript"}),
//   (mockUsePathname:Function {name:"mockUsePathname",type:"function",language:"typescript"}),
//   (mockUseSearchParams:Function {name:"mockUseSearchParams",type:"function",language:"typescript"}),
//   (navigationMockFactory:Function {name:"navigationMockFactory",type:"function",language:"typescript"}),
//   (usePathnameMock:Function {name:"usePathnameMock",type:"function",language:"typescript"}),
//   (useSearchParamsMock:Function {name:"useSearchParamsMock",type:"function",language:"typescript"}),
//   (describe:Function {name:"describe",type:"function",language:"typescript"}),
//   (it:Function {name:"it",type:"function",language:"typescript"}),
//   (viMock:Function {name:"vi.mock",type:"function",language:"typescript"}),
//   (viFn:Function {name:"vi.fn",type:"function",language:"typescript"}),
//   (render:Function {name:"render",type:"function",language:"typescript"}),
//   (fireEvent:Function {name:"fireEvent.click",type:"function",language:"typescript"}),
//   (getByTestId:Function {name:"screen.getByTestId",type:"function",language:"typescript"}),
//   (expect:Function {name:"expect",type:"function",language:"typescript"}),
//   (toHaveAttribute:Function {name:"toHaveAttribute",type:"function",language:"typescript"}),
//   (urlSearchParams:Function {name:"URLSearchParams",type:"function",language:"typescript"}),
//   (encodeURIComponent:Function {name:"encodeURIComponent",type:"function",language:"typescript"}),
//   (Sidebar:Function {name:"Sidebar",type:"function",language:"typescript"}),
//   (ProjectRunTree:Function {name:"ProjectRunTree",type:"function",language:"typescript"}),
//   (RunDirectoryProvider:Function {name:"RunDirectoryProvider",type:"function",language:"typescript"}),
//   (I18nProvider:Function {name:"I18nProvider",type:"function",language:"typescript"}),
//   (fetcher:Function {name:"directoryStatusFetcher",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
//   (newUrl:Function {name:"URL",type:"function",language:"typescript"}),
//   (newResponse:Function {name:"Response",type:"function",language:"typescript"}),
//   (findByRole:Function {name:"screen.findByRole",type:"function",language:"typescript"}),
//   (useNavStore:Function {name:"useNavStore.setState",type:"function",language:"typescript"}),
//   (file)-[:CONTAINS]->(module),(module)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(test),
//   (module)-[:CONTAINS]->(projectId),(module)-[:CONTAINS]->(mockUsePathname),(module)-[:CONTAINS]->(mockUseSearchParams),
//   (module)-[:CONTAINS]->(navigationMockFactory),(navigationMockFactory)-[:CALLS]->(mockUsePathname),(navigationMockFactory)-[:CALLS]->(mockUseSearchParams),
//   (navigationMockFactory)-[:CALLS]->(usePathnameMock),(navigationMockFactory)-[:CALLS]->(useSearchParamsMock),(navigationMockFactory)-[:CALLS]->(viFn),
//   (usePathnameMock)-[:CALLS]->(mockUsePathname),(useSearchParamsMock)-[:CALLS]->(mockUseSearchParams),
//   (mockUsePathname)-[:CALLS]->(viFn),(mockUseSearchParams)-[:CALLS]->(viFn),(mockUseSearchParams)-[:CALLS]->(urlSearchParams),
//   (module)-[:CALLS]->(viMock),(viMock)-[:CALLS]->(navigationMockFactory),
//   (suite)-[:CALLS]->(describe),(suite)-[:CALLS]->(it),(test)-[:CALLS]->(useNavStore),(test)-[:CALLS]->(render),
//   (test)-[:CALLS]->(I18nProvider),(test)-[:CALLS]->(Sidebar),(test)-[:CALLS]->(fireEvent),(test)-[:CALLS]->(getByTestId),
//   (test)-[:CALLS]->(expect),(test)-[:CALLS]->(toHaveAttribute),(test)-[:CALLS]->(mockUsePathname),(test)-[:CALLS]->(mockUseSearchParams),
//   (test)-[:CALLS]->(urlSearchParams),(test)-[:CALLS]->(encodeURIComponent),(test)-[:USES]->(projectId),
//   (statusTest)-[:CALLS]->(render),(statusTest)-[:CALLS]->(I18nProvider),(statusTest)-[:CALLS]->(RunDirectoryProvider),
//   (statusTest)-[:CALLS]->(ProjectRunTree),(statusTest)-[:CALLS]->(fireEvent),(statusTest)-[:CALLS]->(expect),
//   (statusTest)-[:CALLS]->(findByRole),(statusTest)-[:USES]->(fetcher),(fetcher)-[:CALLS]->(newUrl),
//   (fetcher)-[:CALLS]->(newResponse);

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { Sidebar } from "../Sidebar";
import { ProjectRunTree } from "../run/ProjectRunTree";
import { useNavStore } from "@/lib/nav/navStore";
import { I18nProvider } from "@/lib/i18n";
import { RunDirectoryProvider } from "@/lib/run/runDirectorySession";

const projectId = "prj-authorized-123";
const mockUsePathname = vi.fn(() => "/worktree");
const mockUseSearchParams = vi.fn(() => new URLSearchParams(""));

beforeEach(() => {
  window.localStorage.clear();
});

afterEach(() => {
  window.localStorage.clear();
});

vi.mock("next/navigation", () => ({
  usePathname: () => mockUsePathname(),
  useSearchParams: () => mockUseSearchParams(),
}));

describe("Project Worktree Index sidebar navigation", () => {
  it("keeps the Project hierarchy primary and exposes the authorized Project Worktree Index", () => {
    useNavStore.setState({
      sidebarItemIds: ["inbox", "issues", "projects", "agents"],
      pinnedViewIds: ["kanban", "timeline"],
      headerTabIds: ["inbox", "issues", "projects", "agents", "analytics"],
      sidebarFold: "expanded",
      sidebarScope: "main",
      selectedProjectId: projectId,
      isMatrixOpen: false,
    });

    render(
      <I18nProvider initialLanguage="zh-CN">
        <Sidebar />
      </I18nProvider>,
    );

    expect(screen.getByTestId("app-sidebar")).toHaveAttribute("data-nav-model", "project-run");
    expect(screen.getByTestId("project-run-directory")).toBeInTheDocument();
    const link = screen.getByTestId("sidebar-project-worktree-index");
    expect(link).toHaveAttribute(
      "href",
      `/worktree?project_id=${encodeURIComponent(projectId)}`,
    );
    expect(link).toHaveAttribute("data-active", "true");
  });

  it("Run Worktree rows distinguish lifecycle and binding facts without exposing identifiers", async () => {
    const runProjectId = "11111111-1111-4111-8111-111111111111";
    const repositoryId = "22222222-2222-4222-8222-222222222222";
    const runId = "33333333-3333-4333-8333-333333333333";
    const assignedWorktreeId = "44444444-4444-4444-8444-444444444444";
    const unassignedWorktreeId = "55555555-5555-4555-8555-555555555555";
    const unknownWorktreeId = "f1111111-f111-4111-8111-111111111111";
    const agentId = "66666666-6666-4666-8666-666666666666";
    const sessionId = "77777777-7777-4777-8777-777777777777";
    const runtimeId = "88888888-8888-4888-8888-888888888888";
    const projectBindingId = "99999999-9999-4999-8999-999999999999";
    const runBindingId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const futureMachineState = `future-${"x".repeat(50)}`;
    const worktrees = [
      {
        worktree_id: assignedWorktreeId, engineering_run_id: runId, project_id: runProjectId, repository_id: repositoryId,
        workspace_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", name: "assigned-checkout", branch: "agent/assigned",
        human_state: "waiting", machine_state: "agent_executing", agent_id: agentId, agent_session_id: sessionId,
        runtime_id: runtimeId, archived: true, version: 4, binding_id: runBindingId, binding_version: 2,
        project_binding_id: projectBindingId, project_binding_version: 3,
      },
      {
        worktree_id: unassignedWorktreeId, engineering_run_id: runId, project_id: runProjectId, repository_id: repositoryId,
        workspace_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", name: "unassigned-checkout", branch: "agent/unassigned",
        human_state: "ready", machine_state: "git_clean", agent_id: null, agent_session_id: null,
        runtime_id: null, archived: false, version: 5, binding_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd", binding_version: 6,
        project_binding_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee", project_binding_version: 7,
      },
      {
        worktree_id: unknownWorktreeId, engineering_run_id: runId, project_id: runProjectId, repository_id: repositoryId,
        workspace_id: "f2222222-f222-4222-8222-222222222222", name: "future-state-checkout", branch: "agent/future-state",
        human_state: "ready", machine_state: futureMachineState, agent_id: null, agent_session_id: null,
        runtime_id: null, archived: false, version: 8, binding_id: "f3333333-f333-4333-8333-333333333333", binding_version: 9,
        project_binding_id: "f4444444-f444-4444-8444-444444444444", project_binding_version: 10,
      },
    ];
    const directoryStatusFetcher: typeof fetch = async (input) => {
      const url = new URL(typeof input === "string" ? input : input instanceof URL ? input.toString() : input.url);
      let body: Record<string, unknown>;
      if (url.pathname === "/api/v1/projects") {
        body = { projects: [{ project_id: runProjectId, role: "owner" }], limit: 10, next_cursor: null };
      } else if (url.pathname === `/api/v1/projects/${runProjectId}/engineering-runs`) {
        body = {
          project_id: runProjectId,
          engineering_runs: [{ engineering_run_id: runId, project_id: runProjectId, repository_id: repositoryId,
            branch_id: null, title: "Run Alpha", state: "active", owner_user_id: null, version: 1 }],
          limit: 10, next_cursor: null,
        };
      } else if (url.pathname === `/api/v1/engineering-runs/${runId}/worktrees`) {
        body = { project_id: runProjectId, branch_id: null, engineering_run_id: runId, worktrees, limit: 10, next_cursor: null };
      } else {
        return new Response("{}", { status: 404, headers: { "content-type": "application/json" } });
      }
      return new Response(JSON.stringify(body), { status: 200, headers: { "content-type": "application/json" } });
    };

    render(
      <I18nProvider initialLanguage="en">
        <RunDirectoryProvider session={{
          generation: "directory-status-test", getAccessToken: async () => "test-token",
          baseUrl: "https://star.test", trustedOrigins: ["https://star.test"], fetcher: directoryStatusFetcher,
        }}>
          <ProjectRunTree collapsed={false} />
        </RunDirectoryProvider>
      </I18nProvider>,
    );

    fireEvent.click(await screen.findByRole("button", { name: `Expand Project ${runProjectId}` }));
    fireEvent.click(await screen.findByRole("button", { name: "Expand Run Alpha" }));
    const assignedRow = await screen.findByRole("treeitem", { name: /^assigned-checkout/ });
    const assignedDescription = assignedRow.getAttribute("aria-label") ?? "";
    expect(assignedDescription).toContain("Human: Waiting");
    expect(assignedDescription).toContain("Machine: Agent executing");
    expect(assignedDescription).toContain("Session bound");
    expect(assignedDescription).toContain("Runtime bound");
    expect(assignedDescription).toContain("Archived");
    expect(assignedDescription).toContain("Run binding v2");
    expect(assignedDescription).toContain("Project binding v3");
    expect(assignedDescription).toContain("Lock status unknown");
    expect(assignedDescription).toContain("Dirty change details unknown");
    expect(assignedDescription).toContain("Observation time unknown");
    expect(assignedDescription).toContain("Branch source and freshness unknown");
    expect(assignedDescription).not.toContain(agentId);
    expect(assignedDescription).not.toContain(sessionId);
    expect(assignedDescription).not.toContain(runtimeId);
    expect(assignedDescription).not.toContain(assignedWorktreeId);

    const unassignedRow = await screen.findByRole("treeitem", { name: /^unassigned-checkout/ });
    const unassignedDescription = unassignedRow.getAttribute("aria-label") ?? "";
    expect(unassignedDescription).toContain("Human: Ready");
    expect(unassignedDescription).toContain("Machine: Git clean");
    expect(unassignedDescription).toContain("No session bound");
    expect(unassignedDescription).toContain("No runtime bound");
    expect(unassignedDescription).toContain("Not archived");
    expect(unassignedDescription).not.toContain(unassignedWorktreeId);

    const unknownRow = await screen.findByRole("treeitem", { name: /^future-state-checkout/ });
    const unknownDescription = unknownRow.getAttribute("aria-label") ?? "";
    expect(unknownDescription).toContain(`Machine: Unknown state: ${futureMachineState.slice(0, 32)}`);
    expect(unknownDescription).not.toContain(futureMachineState);
    expect(unknownDescription).not.toContain(unknownWorktreeId);
  });
});
