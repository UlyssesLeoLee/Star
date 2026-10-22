/*
@cypher schema=1 source_sha256=f3cce489c3616909776521e40a0014b42b05fa9e9957b70f79d0125ccafe48ce
MERGE (self:File {path:"frontend/src/components/run/ProjectRunTree.test.tsx"})
MERGE (tree:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (suite:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::ProjectRunTree selection and expansion",kind:"suite"})
MERGE (projectSelect:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::selecting a Project navigates to its Worktree index and sets Project scope",kind:"test"})
MERGE (runExpand:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::Project and Run expansion remain separate from selection",kind:"test"})
MERGE (keyboardExpand:Symbol {id:"frontend/src/components/run/ProjectRunTree.test.tsx::ArrowRight expands Project and Run without selecting either",kind:"test"})
MERGE (userEvents:ExternalService {name:"@testing-library/user-event"})
MERGE (fireEvent:ExternalService {name:"@testing-library/react fireEvent"})
MERGE (session:Symbol {id:"frontend/src/lib/run/runDirectorySession.tsx::useRunDirectory",kind:"hook"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(projectSelect)
MERGE (suite)-[:DEFINES]->(runExpand)
MERGE (suite)-[:DEFINES]->(keyboardExpand)
MERGE (projectSelect)-[:TESTS]->(tree)
MERGE (runExpand)-[:TESTS]->(tree)
MERGE (keyboardExpand)-[:TESTS]->(tree)
MERGE (projectSelect)-[:CALLS]->(userEvents)
MERGE (runExpand)-[:CALLS]->(userEvents)
MERGE (keyboardExpand)-[:CALLS]->(fireEvent)
MERGE (projectSelect)-[:USES_TYPE]->(session)
MERGE (projectSelect)-[:USES_TYPE]->(navStore)
MERGE (self)-[:IMPORTS]->(userEvents)
@endcypher
*/

import { beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { AnchorHTMLAttributes, ReactNode } from "react";
import type { RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";
import { ProjectRunTree } from "./ProjectRunTree";

const projectId = "11111111-1111-4111-8111-111111111111";
const runId = "22222222-2222-4222-8222-222222222222";

const mocks = vi.hoisted(() => ({
  pathname: "/",
  session: null as unknown,
  labels: {
    humanStates: {}, machineStates: {}, unknownState: "未知", humanState: "人工状态", machineState: "机器状态",
    agentAssigned: "Agent 已分配", agentUnassigned: "Agent 未分配", sessionBound: "Session 已绑定", sessionUnbound: "Session 未绑定",
    archived: "已归档", notArchived: "未归档", runtimeBound: "Runtime 已绑定", runtimeUnbound: "Runtime 未绑定",
    runBindingVersion: "Run binding {version}", projectBindingVersion: "Project binding {version}", lockUnknown: "锁未知",
    dirtyDetailsUnknown: "变更未知", observedAtUnknown: "观测时间未知", branchFreshnessUnknown: "Branch 新鲜度未知",
    localWorktree: "本地 Worktree", branchRecord: "本地 Branch", branchUnavailable: "Branch 未关联",
    newRun: "新建 Run",
  },
  nav: {
    selectedProjectId: "",
    setSelectedProjectId: vi.fn(),
    setSidebarScope: vi.fn(),
  },
}));

vi.mock("@/lib/run/runDirectorySession", () => ({ useRunDirectory: () => mocks.session }));
vi.mock("@/lib/nav/navStore", () => ({ useNavStore: (selector: (state: typeof mocks.nav) => unknown) => selector(mocks.nav) }));
vi.mock("next/navigation", () => ({ usePathname: () => mocks.pathname }));
vi.mock("@/lib/i18n", () => ({ useTranslation: () => ({ t: { runDirectory: mocks.labels } }) }));
vi.mock("@/components/navigation/GuardedLink", () => ({
  GuardedLink: ({ href, children, onNavigate, ...props }: AnchorHTMLAttributes<HTMLAnchorElement> & { href: string; children: ReactNode; onNavigate?: () => void }) =>
    <a href={href} {...props} onClick={(event) => { event.preventDefault(); onNavigate?.(); props.onClick?.(event); }}>{children}</a>,
}));

function installSession() {
  const client = {
    listProjects: vi.fn().mockResolvedValue({ rows: [{ project_id: projectId, role: "owner" }], limit: 50, next_cursor: null }),
    listProjectRuns: vi.fn().mockResolvedValue({ rows: [{ project_id: projectId, engineering_run_id: runId, title: "Focused Run", state: "active" }], limit: 50, next_cursor: null }),
    listWorktrees: vi.fn().mockResolvedValue({ rows: [], limit: 50, next_cursor: null }),
  } as unknown as RunDirectoryApiClient;
  const clearFocus = vi.fn();
  mocks.session = { status: "ready", client, message: "", focus: null, clearFocus };
  mocks.nav.selectedProjectId = "";
  mocks.nav.setSelectedProjectId.mockClear();
  mocks.nav.setSidebarScope.mockClear();
  mocks.pathname = "/";
  return { client, clearFocus };
}

describe("ProjectRunTree selection and expansion", () => {
  beforeEach(() => {
    cleanup();
  });

  it("selecting a Project navigates to its Worktree index and sets Project scope", async () => {
    const { clearFocus } = installSession();
    render(<ProjectRunTree collapsed={false} />);

    const project = await screen.findByRole("treeitem", { name: `选择 Project ${projectId}` });
    expect(project).toHaveAttribute("href", `/worktree?project_id=${projectId}`);
    expect(project).toHaveAttribute("aria-selected", "false");
    fireEvent.click(project);

    expect(mocks.nav.setSelectedProjectId).toHaveBeenCalledWith(projectId);
    expect(mocks.nav.setSidebarScope).toHaveBeenCalledWith("project");
    expect(clearFocus).toHaveBeenCalledTimes(1);
  });

  it("Project and Run expansion remain separate from selection", async () => {
    const { client } = installSession();
    const user = userEvent.setup();
    render(<ProjectRunTree collapsed={false} />);

    await user.click(await screen.findByRole("button", { name: `展开 Project ${projectId}` }));
    const run = await screen.findByRole("treeitem", { name: "打开 Engineering Run Focused Run" });
    expect(run).toHaveAttribute("href", `/projects/${projectId}/runs/${runId}`);
    expect(run).toHaveAttribute("aria-selected", "false");
    expect(screen.queryByRole("link", { name: /打开 Task Cards/ })).toBeNull();
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "展开 Focused Run" }));
    await screen.findByText("暂无授权条目；点击刷新");
    expect(client.listProjectRuns).toHaveBeenCalledWith(projectId, undefined, expect.any(AbortSignal));
    expect(client.listWorktrees).toHaveBeenCalled();
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();
  });

  it("ArrowRight expands Project and Run without selecting either", async () => {
    const { client } = installSession();
    render(<ProjectRunTree collapsed={false} />);

    const project = await screen.findByRole("treeitem", { name: `选择 Project ${projectId}` });
    fireEvent.keyDown(project, { key: "ArrowRight" });
    const run = await screen.findByRole("treeitem", { name: "打开 Engineering Run Focused Run" });
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();

    fireEvent.keyDown(run, { key: "ArrowRight" });
    await screen.findByText("暂无授权条目；点击刷新");
    expect(client.listWorktrees).toHaveBeenCalled();
    expect(mocks.nav.setSelectedProjectId).not.toHaveBeenCalled();
    expect(mocks.nav.setSidebarScope).not.toHaveBeenCalled();
  });
});
