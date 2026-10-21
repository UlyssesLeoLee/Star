/*
@cypher schema=1 source_sha256=70d7d0812dd68afabcbeef01e5f84512295f9e6b39fcd9ed595f6cb0fc72f825
MERGE (self:File {path:"frontend/src/components/run/RunWorkspaceShell.test.tsx"})
MERGE (shell:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.tsx::RunWorkspaceShell",kind:"function"})
MERGE (suite:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::RunWorkspaceShell focus gating",kind:"suite"})
MERGE (unfocused:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::unfocused Run exposes directory choices without mounting Run Apps",kind:"test"})
MERGE (focused:Symbol {id:"frontend/src/components/run/RunWorkspaceShell.test.tsx::verified Worktree focus reveals permitted Run Apps",kind:"test"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(unfocused)
MERGE (suite)-[:DEFINES]->(focused)
MERGE (unfocused)-[:TESTS]->(shell)
MERGE (focused)-[:TESTS]->(shell)
@endcypher
*/

import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import type { AnchorHTMLAttributes } from "react";
import { RunWorkspaceShell } from "./RunWorkspaceShell";
import type { RunContextEnvelope, RunDirectoryApiClient, RunWorktree } from "@/lib/run/runDirectoryApi";

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
