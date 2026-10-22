/*
@cypher schema=1 source_sha256=9aaad51b069ec0310de07bf38df6c40e089078bf66f2c17dd2c9294b238db012
MERGE (self:File {path:"frontend/src/components/run/RunWorkspaceShell.test.tsx"})
MERGE (shell:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::RunWorkspaceShell",kind:"function"})
MERGE (suite:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::RunWorkspaceShell focus gating",kind:"suite"})
MERGE (unfocused:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::unfocused Run exposes directory choices without mounting Run Apps",kind:"test"})
MERGE (focused:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::verified Worktree focus reveals permitted Run Apps",kind:"test"})
MERGE (picker:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::bounded picker loads more rows and searches across all loaded rows",kind:"test"})
MERGE (duplicates:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::rejects duplicate Worktree IDs within a returned page and keeps prior rows",kind:"test"})
MERGE (oversizedLater:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::rejects an oversized later page without exceeding the picker row budget",kind:"test"})
MERGE (oversizedFirst:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::rejects an oversized first page before rendering unbounded choices",kind:"test"})
MERGE (installer:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::installSession",kind:"function"})
MERGE (makeWorktrees:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::makeWorktrees",kind:"function"})
MERGE (userEvent:ExternalService {name:"@testing-library/user-event"})
MERGE (pageSize:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RUN_DIRECTORY_PAGE_SIZE",kind:"constant"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(unfocused)
MERGE (suite)-[:DEFINES]->(focused)
MERGE (suite)-[:DEFINES]->(picker)
MERGE (suite)-[:DEFINES]->(duplicates)
MERGE (suite)-[:DEFINES]->(oversizedLater)
MERGE (suite)-[:DEFINES]->(oversizedFirst)
MERGE (unfocused)-[:TESTS]->(shell)
MERGE (focused)-[:TESTS]->(shell)
MERGE (picker)-[:TESTS]->(shell)
MERGE (picker)-[:CALLS]->(installer)
MERGE (duplicates)-[:TESTS]->(shell)
MERGE (oversizedLater)-[:TESTS]->(shell)
MERGE (oversizedFirst)-[:TESTS]->(shell)
MERGE (oversizedLater)-[:CALLS]->(makeWorktrees)
MERGE (oversizedFirst)-[:CALLS]->(makeWorktrees)
MERGE (self)-[:IMPORTS]->(userEvent)
MERGE (self)-[:IMPORTS]->(pageSize)
@endcypher
*/

import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { AnchorHTMLAttributes } from "react";
import { RunWorkspaceShell } from "./RunWorkspaceShell";
import { RUN_DIRECTORY_PAGE_SIZE, type RunContextEnvelope, type RunDirectoryApiClient, type RunWorktree } from "@/lib/run/runDirectoryApi";

const mocks = vi.hoisted(() => ({
  session: null as unknown,
  taskCardsMounted: vi.fn(),
  biMounted: vi.fn(),
}));

vi.mock("@/lib/run/runDirectorySession", () => ({ useRunDirectory: () => mocks.session }));
vi.mock("@/components/run/RunTaskCardsPanel", () => ({
  RunTaskCardsPanel: () => { mocks.taskCardsMounted(); return <div data-testid="task-cards-panel" />; },
}));
vi.mock("@/components/run/RunBiPanel", () => ({
  RunBiPanel: () => { mocks.biMounted(); return <div data-testid="run-bi-panel" />; },
}));
vi.mock("next/link", () => ({
  default: ({ href, children, prefetch: _prefetch, ...props }: AnchorHTMLAttributes<HTMLAnchorElement> & { href: string; prefetch?: boolean }) =>
    <a href={href} {...props}>{children}</a>,
}));

const projectId = "11111111-1111-4111-8111-111111111111";
const runId = "22222222-2222-4222-8222-222222222222";
const worktreeId = "33333333-3333-4333-8333-333333333333";

function makeWorktree(): RunWorktree {
  return {
    worktree_id: worktreeId,
    engineering_run_id: runId,
    project_id: projectId,
    repository_id: "44444444-4444-4444-8444-444444444444",
    workspace_id: "55555555-5555-4555-8555-555555555555",
    name: "feature-checkout",
    branch: "feature/ui-focus",
    human_state: "active",
    machine_state: "ready",
    agent_id: null,
    agent_session_id: null,
    runtime_id: null,
    archived: false,
    version: 1,
    binding_id: "66666666-6666-4666-8666-666666666666",
    binding_version: 1,
    project_binding_id: "77777777-7777-4777-8777-777777777777",
    project_binding_version: 1,
  };
}

function makeWorktrees(count: number): RunWorktree[] {
  return Array.from({ length: count }, (_, index) => ({
    ...makeWorktree(),
    worktree_id: `aaaaaaaa-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
    name: `bounded-checkout-${index + 1}`,
  }));
}

function makeEnvelope(focus: RunWorktree | null): RunContextEnvelope {
  return {
    run_context: {
      tenant_id: "88888888-8888-4888-8888-888888888888",
      project_id: projectId,
      repository_id: null,
      branch_id: null,
      engineering_run_id: runId,
      actor_id: "99999999-9999-4999-8999-999999999999",
      actor_kind: "user",
      authorization: {
        project: { binding_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", role: "owner", version: 1 },
        branch: null,
        engineering_run: { binding_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", role: "owner", version: 1 },
      },
      granted_scopes: ["run:read"],
      context_version: "ctx-1",
      resolved_at: "2026-10-22T00:00:00Z",
      correlation_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    },
    branch: null,
    engineering_run: {
      engineering_run_id: runId,
      project_id: projectId,
      repository_id: null,
      branch_id: null,
      title: "Run Focus Acceptance",
      state: "active",
      owner_user_id: "99999999-9999-4999-8999-999999999999",
      version: 1,
    },
    focus,
    focus_version: focus ? "focus-1" : null,
    capabilities: { task_cards_available: true, run_owned_apps_available: true, execution_admission_available: false },
  };
}

function installSession(envelope: RunContextEnvelope) {
  const client = {
    getRunContext: vi.fn().mockResolvedValue(envelope),
    listWorktrees: vi.fn().mockResolvedValue({ rows: [makeWorktree()], limit: 50, next_cursor: null }),
    listRunWorkItems: vi.fn(),
    listRunTaskEvents: vi.fn(),
  } as unknown as RunDirectoryApiClient;
  mocks.taskCardsMounted.mockClear();
  mocks.biMounted.mockClear();
  mocks.session = {
    client,
    status: "ready",
    message: "",
    setFocus: vi.fn(),
    clearFocus: vi.fn(),
  };
  return client;
}

describe("RunWorkspaceShell focus gating", () => {
  it("unfocused Run exposes directory choices without mounting Run Apps", async () => {
    const client = installSession(makeEnvelope(null));

    render(<RunWorkspaceShell route={{ project_id: projectId, engineering_run_id: runId }} />);

    expect(await screen.findByRole("heading", { name: "选择 Worktree 继续" })).toBeTruthy();
    expect(await screen.findByRole("link", { name: /feature-checkout/ })).toHaveAttribute(
      "href",
      `/projects/${projectId}/runs/${runId}/worktrees/${worktreeId}`,
    );
    expect(screen.queryByRole("tablist", { name: "Engineering Run Apps" })).toBeNull();
    expect(screen.queryByTestId("task-cards-panel")).toBeNull();
    expect(screen.queryByTestId("run-bi-panel")).toBeNull();
    expect(screen.getByRole("button", { name: "GLOBAL" })).toHaveAttribute("aria-pressed", "true");
    await waitFor(() => expect(client.listWorktrees).toHaveBeenCalledTimes(1));
    expect(client.listRunWorkItems).not.toHaveBeenCalled();
    expect(client.listRunTaskEvents).not.toHaveBeenCalled();
    expect(mocks.taskCardsMounted).not.toHaveBeenCalled();
    expect(mocks.biMounted).not.toHaveBeenCalled();
  });

  it("bounded picker loads more rows and searches across all loaded rows", async () => {
    const secondWorktree = {
      ...makeWorktree(),
      worktree_id: "aaaaaaaa-0000-4000-8000-000000000001",
      name: "telemetry-checkout",
      branch: "feature/telemetry",
    };
    const client = installSession(makeEnvelope(null));
    vi.mocked(client.listWorktrees)
      .mockResolvedValueOnce({ rows: [makeWorktree()], limit: 50, next_cursor: "cursor-1" })
      .mockResolvedValueOnce({ rows: [secondWorktree], limit: 50, next_cursor: null });
    const user = userEvent.setup();

    render(<RunWorkspaceShell route={{ project_id: projectId, engineering_run_id: runId }} />);

    const search = await screen.findByRole("searchbox", { name: "搜索已加载的 Worktree" });
    await user.type(search, "telemetry");
    expect(await screen.findByText(/没有匹配当前已加载条目的 Worktree/)).toBeTruthy();
    expect(within(screen.getByTestId("run-worktree-selection")).getAllByRole("status")[0]?.textContent).toContain("搜索范围是已加载条目");

    await user.click(screen.getByRole("button", { name: "加载更多 Worktree" }));

    expect(await screen.findByRole("link", { name: /telemetry-checkout/ })).toHaveAttribute(
      "href",
      "/projects/" + projectId + "/runs/" + runId + "/worktrees/" + secondWorktree.worktree_id,
    );
    expect(within(screen.getByTestId("run-worktree-selection")).getAllByRole("status")[0]?.textContent).toContain("显示 1 / 2 项");
    expect(client.listWorktrees).toHaveBeenCalledTimes(2);
    expect(client.listWorktrees.mock.calls[1]?.[1]).toBe("cursor-1");
  });

  it("rejects duplicate Worktree IDs within a returned page and keeps prior rows", async () => {
    const repeatedRow = { ...makeWorktree(), worktree_id: "aaaaaaaa-0000-4000-8000-000000000001", name: "duplicate-checkout" };
    const client = installSession(makeEnvelope(null));
    vi.mocked(client.listWorktrees)
      .mockResolvedValueOnce({ rows: [makeWorktree()], limit: 50, next_cursor: "cursor-1" })
      .mockResolvedValueOnce({ rows: [repeatedRow, repeatedRow], limit: 50, next_cursor: null });
    const user = userEvent.setup();

    render(<RunWorkspaceShell route={{ project_id: projectId, engineering_run_id: runId }} />);
    await user.click(await screen.findByRole("button", { name: "加载更多 Worktree" }));

    expect(await screen.findByText("Worktree 分页包含重复条目，请刷新目录后重试。")).toBeTruthy();
    const rows = screen.getByRole("list", { name: "已加载的授权 Worktree" });
    expect(within(rows).getAllByRole("link")).toHaveLength(1);
    expect(within(rows).getByRole("link", { name: /feature-checkout/ })).toBeTruthy();
  });

  it("rejects an oversized later page without exceeding the picker row budget", async () => {
    const client = installSession(makeEnvelope(null));
    vi.mocked(client.listWorktrees)
      .mockResolvedValueOnce({ rows: [makeWorktree()], limit: 50, next_cursor: "cursor-1" })
      .mockResolvedValueOnce({ rows: makeWorktrees(RUN_DIRECTORY_PAGE_SIZE + 1), limit: 50, next_cursor: null });
    const user = userEvent.setup();

    render(<RunWorkspaceShell route={{ project_id: projectId, engineering_run_id: runId }} />);
    await user.click(await screen.findByRole("button", { name: "加载更多 Worktree" }));

    expect(await screen.findByText("Worktree 分页超过单页或选择器读取上限，请刷新目录后重试。")).toBeTruthy();
    const rows = screen.getByRole("list", { name: "已加载的授权 Worktree" });
    expect(within(rows).getAllByRole("link")).toHaveLength(1);
  });

  it("rejects an oversized first page before rendering unbounded choices", async () => {
    const client = installSession(makeEnvelope(null));
    vi.mocked(client.listWorktrees).mockResolvedValueOnce({ rows: makeWorktrees(RUN_DIRECTORY_PAGE_SIZE + 1), limit: 50, next_cursor: null });

    render(<RunWorkspaceShell route={{ project_id: projectId, engineering_run_id: runId }} />);

    expect(await screen.findByText("Worktree 分页超过单页或选择器读取上限，请刷新目录后重试。")).toBeTruthy();
    expect(screen.queryByRole("searchbox", { name: "搜索已加载的 Worktree" })).toBeNull();
    expect(screen.queryByRole("list", { name: "已加载的授权 Worktree" })).toBeNull();
  });

  it("verified Worktree focus reveals permitted Run Apps", async () => {
    const client = installSession(makeEnvelope(makeWorktree()));

    render(<RunWorkspaceShell route={{ project_id: projectId, engineering_run_id: runId, worktree_id: worktreeId }} />);

    expect(await screen.findByRole("tablist", { name: "Engineering Run Apps" })).toBeTruthy();
    expect(await screen.findByTestId("task-cards-panel")).toBeTruthy();
    expect(screen.queryByTestId("run-worktree-selection")).toBeNull();
    expect(screen.getByRole("button", { name: "WORKTREE" })).toHaveAttribute("aria-pressed", "true");
    expect(mocks.taskCardsMounted).toHaveBeenCalledTimes(1);
    expect(client.listWorktrees).not.toHaveBeenCalled();
  });
});
