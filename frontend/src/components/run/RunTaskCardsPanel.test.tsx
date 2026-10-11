/*
@cypher schema=1 source_sha256=bc640143e0de73d6f3ba2d1dee1a59a7c00bf1b6c37bcd662ea2535fe85cfdb9
MERGE (self:File {path:"frontend/src/components/run/RunTaskCardsPanel.test.tsx"})
MERGE (create:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::creates a Run-owned Task Card against the current authorized focus and refreshes the Run list",kind:"test"})
MERGE (retry:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::retries an uncertain create with the exact same idempotency key and body",kind:"test"})
MERGE (scope:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::does not offer task creation without the current Run write scope",kind:"test"})
MERGE (focus:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::blocks a retry if the focused Worktree changed after an uncertain create",kind:"test"})
MERGE (remount:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::restores an uncertain request after panel remount and retries the exact body and key",kind:"test"})
MERGE (actor:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::does not restore another actor's pending create in the same Run",kind:"test"})
MERGE (binding:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::blocks a recovered retry when a Worktree binding or authorization version changed",kind:"test"})
MERGE (storage:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::fails closed if the pending command cannot be saved before POST",kind:"test"})
MERGE (discard:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::requires confirmation before deleting a pending recovery record",kind:"test"})
MERGE (expired:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.test.tsx::does not retry an expired recovery record",kind:"test"})
MERGE (panel:Symbol {id:"frontend/src/components/run/RunTaskCardsPanel.tsx::RunTaskCardsPanel",kind:"function"})
MERGE (storeTest:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::persists one pending command across store instances without replacing its idempotency key",kind:"test"})
MERGE (boundedTest:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::rejects the 33rd unresolved command instead of growing without a bound",kind:"test"})
MERGE (storeFactory:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::createTaskCardCreateRecoveryStore",kind:"function"})
MERGE (storeScope:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryScopeKey",kind:"function"})
MERGE (storeTarget:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryTarget",kind:"function"})
MERGE (api:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.createRunTaskCard",kind:"method"})
MERGE (self)-[:DEFINES]->(create)
MERGE (self)-[:DEFINES]->(retry)
MERGE (self)-[:DEFINES]->(scope)
MERGE (self)-[:DEFINES]->(focus)
MERGE (create)-[:TESTS]->(panel)
MERGE (create)-[:TESTS]->(api)
MERGE (retry)-[:TESTS]->(panel)
MERGE (scope)-[:TESTS]->(panel)
MERGE (focus)-[:TESTS]->(panel)
MERGE (remount)-[:TESTS]->(panel)
MERGE (remount)-[:TESTS]->(api)
MERGE (actor)-[:TESTS]->(panel)
MERGE (binding)-[:TESTS]->(panel)
MERGE (storage)-[:TESTS]->(panel)
MERGE (discard)-[:TESTS]->(panel)
MERGE (expired)-[:TESTS]->(panel)
MERGE (self)-[:DEFINES]->(storeTest)
MERGE (self)-[:DEFINES]->(boundedTest)
MERGE (storeTest)-[:TESTS]->(storeFactory)
MERGE (storeTest)-[:TESTS]->(storeScope)
MERGE (storeTest)-[:TESTS]->(storeTarget)
MERGE (boundedTest)-[:TESTS]->(storeFactory)
@endcypher
*/
/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/RunTaskCardsPanel.test.tsx",type:"file",language:"tsx"}),
 (runFixture:Function {name:"runFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (contextFixture:Function {name:"contextFixture",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (cardFixture:Function {name:"cardFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (clientFixture:Function {name:"clientFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (closedTest:Function {name:"does not request or display tasks while Run Apps are unavailable",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (liveTest:Function {name:"renders authorized Run tasks while leaving task CLI disabled",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (createTest:Function {name:"creates a Run-owned Task Card against the current authorized focus and refreshes the Run list",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (retryTest:Function {name:"retries an uncertain create with the exact same idempotency key and body",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (f)-[:CONTAINS]->(runFixture),(f)-[:CONTAINS]->(contextFixture),(f)-[:CONTAINS]->(cardFixture),(f)-[:CONTAINS]->(clientFixture),(f)-[:CONTAINS]->(closedTest),(f)-[:CONTAINS]->(liveTest),(f)-[:CONTAINS]->(createTest),(f)-[:CONTAINS]->(retryTest),
 (contextFixture)-[:CALLS]->(runFixture),(clientFixture)-[:CALLS]->(cardFixture),(closedTest)-[:CALLS]->(contextFixture),(closedTest)-[:CALLS]->(clientFixture),(liveTest)-[:CALLS]->(contextFixture),(liveTest)-[:CALLS]->(cardFixture),(liveTest)-[:CALLS]->(clientFixture),(createTest)-[:CALLS]->(contextFixture),(createTest)-[:CALLS]->(clientFixture),(retryTest)-[:CALLS]->(contextFixture),(retryTest)-[:CALLS]->(clientFixture);
*/

import "fake-indexeddb/auto";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RunTaskCardsPanel } from "./RunTaskCardsPanel";
import {
  createTaskCardCreateRecoveryStore,
  taskCardCreateRecoveryScopeKey,
  taskCardCreateRecoveryTarget,
} from "@/lib/run/taskCardCreateRecoveryStore";
import { RUN_TASK_CARD_PAGE_SIZE, type CreateRunTaskCardInput, type EngineeringRun, type RunContextEnvelope, type RunDirectoryApiClient, type RunTaskCardPage } from "@/lib/run/runDirectoryApi";

function runFixture(): EngineeringRun {
  return {
    engineering_run_id: "10000000-0000-4000-8000-000000000001",
    project_id: "10000000-0000-4000-8000-000000000002",
    repository_id: "10000000-0000-4000-8000-000000000003",
    branch_id: "10000000-0000-4000-8000-000000000004",
    title: "Run",
    state: "active",
    owner_user_id: null,
    version: 1,
  };
}

function cardFixture() {
  return {
    work_item_id: "20000000-0000-4000-8000-000000000001",
    item_type: "task",
    title: "Real Run task",
    description: "Persisted service projection",
    priority: "high",
    labels: ["run-owned"],
    lifecycle: { status: "pending", review_state: "not_required", active_worktree_id: null, version: 1, updated_at: "2026-10-02T00:00:00Z" },
    metadata_updated_at: "2026-10-02T00:00:00Z",
  };
}

function contextFixture(): RunContextEnvelope {
  const run = runFixture();
  const branch = {
    branch_id: run.branch_id, project_id: run.project_id, repository_id: run.repository_id,
    name: "feature/run-task", full_ref: "refs/heads/feature/run-task", head_commit_id: null, state: "active", version: 1,
  };
  const focus = {
    worktree_id: "30000000-0000-4000-8000-000000000001", engineering_run_id: run.engineering_run_id,
    project_id: run.project_id, repository_id: run.repository_id, workspace_id: "30000000-0000-4000-8000-000000000002",
    name: "run-focus", branch: "feature/run-task", human_state: "active", machine_state: "ready",
    agent_id: null, agent_session_id: null, runtime_id: null, archived: false, version: 1,
    binding_id: "30000000-0000-4000-8000-000000000003", binding_version: 1,
    project_binding_id: "30000000-0000-4000-8000-000000000004", project_binding_version: 1,
  };
  return {
    run_context: {
      tenant_id: "10000000-0000-4000-8000-000000000005", project_id: run.project_id,
      repository_id: run.repository_id, branch_id: run.branch_id, engineering_run_id: run.engineering_run_id,
      actor_id: "10000000-0000-4000-8000-000000000006", actor_kind: "user",
      authorization: {
        project: { binding_id: "40000000-0000-4000-8000-000000000001", role: "developer", version: 1 },
        branch: { binding_id: "40000000-0000-4000-8000-000000000002", role: "developer", version: 1 },
        engineering_run: { binding_id: "40000000-0000-4000-8000-000000000003", role: "developer", version: 1 },
      },
      granted_scopes: ["work-item:read", "work-item:write"], context_version: "ctx:v1",
      resolved_at: "2026-10-02T00:00:00Z", correlation_id: "10000000-0000-4000-8000-000000000007",
    },
    branch, engineering_run: run, focus, focus_version: "focus:v1",
    capabilities: { run_owned_apps_available: true, execution_admission_available: false },
  };
}

function clientFixture() {
  const page: RunTaskCardPage = { rows: [cardFixture()], limit: RUN_TASK_CARD_PAGE_SIZE, next_after: null };
  const listRunWorkItems = vi.fn(async () => page);
  const createRunTaskCard = vi.fn(async (_context: RunContextEnvelope, _input: CreateRunTaskCardInput, _key: string) => cardFixture());
  return { client: { listRunWorkItems, createRunTaskCard } as unknown as RunDirectoryApiClient, listRunWorkItems, createRunTaskCard };
}

function recoveryStoreFixture() {
  return createTaskCardCreateRecoveryStore(`test-task-card-recovery-${globalThis.crypto.randomUUID()}`);
}

async function clickNewTaskCard() {
  const button = await screen.findByRole("button", { name: "新建 Task Card" });
  await waitFor(() => expect(button).toBeEnabled());
  fireEvent.click(button);
}

describe("RunTaskCardsPanel", () => {
  it("does not request or display tasks while Run Apps are unavailable", () => {
    const fixture = clientFixture();
    render(<RunTaskCardsPanel client={fixture.client} context={contextFixture()} available={false} recoveryStore={recoveryStoreFixture()} />);

    expect(screen.getByText(/服务端尚未开放 Run-owned Apps/)).toBeInTheDocument();
    expect(fixture.listRunWorkItems).not.toHaveBeenCalled();
    expect(fixture.createRunTaskCard).not.toHaveBeenCalled();
    expect(screen.queryByRole("button", { name: "新建 Task Card" })).not.toBeInTheDocument();
  });

  it("does not offer task creation without the current Run write scope", async () => {
    const fixture = clientFixture();
    const context = contextFixture();
    context.run_context.granted_scopes = ["work-item:read"];
    render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStoreFixture()} />);

    const create = await screen.findByRole("button", { name: "新建 Task Card" });
    expect(create).toBeDisabled();
    expect(screen.getByText(/需要当前 Run 中具有 work-item:write 授权/)).toBeInTheDocument();
    expect(fixture.createRunTaskCard).not.toHaveBeenCalled();
  });

  it("renders authorized Run tasks while leaving task CLI disabled", async () => {
    const fixture = clientFixture();
    render(<RunTaskCardsPanel client={fixture.client} context={contextFixture()} available recoveryStore={recoveryStoreFixture()} />);

    expect(await screen.findByText("Real Run task")).toBeInTheDocument();
    expect(fixture.listRunWorkItems).toHaveBeenCalledOnce();
    expect(screen.getByRole("button", { name: /打开 CLI/ })).toBeDisabled();
  });

  it("creates a Run-owned Task Card against the current authorized focus and refreshes the Run list", async () => {
    const fixture = clientFixture();
    const createdCard = { ...cardFixture(), work_item_id: "20000000-0000-4000-8000-000000000002", title: "Created from Run" };
    let created = false;
    fixture.listRunWorkItems.mockImplementation(async () => ({
      rows: created ? [createdCard] : [], limit: RUN_TASK_CARD_PAGE_SIZE, next_after: null,
    }));
    fixture.createRunTaskCard.mockImplementation(async () => { created = true; return createdCard; });
    const context = contextFixture();
    render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStoreFixture()} />);

    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Created from Run" } });
    fireEvent.change(screen.getByLabelText("描述"), { target: { value: "Run-owned" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));

    expect(await screen.findByText("Created from Run")).toBeInTheDocument();
    expect(fixture.createRunTaskCard).toHaveBeenCalledOnce();
    expect(fixture.createRunTaskCard).toHaveBeenCalledWith(context, expect.objectContaining({
      title: "Created from Run", description: "Run-owned", priority: "medium",
      correlation_id: expect.stringMatching(/^[0-9a-f-]{36}$/i),
    }), expect.stringMatching(/^[0-9a-f-]{36}$/i));
    await waitFor(() => expect(fixture.listRunWorkItems).toHaveBeenCalledTimes(2));
  });

  it("retries an uncertain create with the exact same idempotency key and body", async () => {
    const fixture = clientFixture();
    fixture.createRunTaskCard.mockRejectedValueOnce(new Error("temporary network interruption"));
    const context = contextFixture();
    render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStoreFixture()} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Retry safely" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));

    const retry = await screen.findByRole("button", { name: "使用同一请求重试" });
    fireEvent.click(retry);
    await waitFor(() => expect(fixture.createRunTaskCard).toHaveBeenCalledTimes(2));
    expect(fixture.createRunTaskCard.mock.calls[1]).toEqual(fixture.createRunTaskCard.mock.calls[0]);
  });

  it("blocks a retry if the focused Worktree changed after an uncertain create", async () => {
    const fixture = clientFixture();
    fixture.createRunTaskCard.mockRejectedValueOnce(new Error("temporary network interruption"));
    const context = contextFixture();
    const recoveryStore = recoveryStoreFixture();
    const view = render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Retry safely" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));
    await screen.findByRole("button", { name: "使用同一请求重试" });

    const changed = contextFixture();
    changed.focus = { ...changed.focus!, worktree_id: "30000000-0000-4000-8000-000000000009" };
    view.rerender(<RunTaskCardsPanel client={fixture.client} context={changed} available recoveryStore={recoveryStore} />);

    expect(await screen.findByText(/当前 Worktree、Run binding 或授权版本不同/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "使用同一请求重试" })).toBeDisabled();
    expect(fixture.createRunTaskCard).toHaveBeenCalledOnce();
  });

  it("restores an uncertain request after panel remount and retries the exact body and key", async () => {
    const fixture = clientFixture();
    fixture.createRunTaskCard.mockRejectedValueOnce(new Error("temporary network interruption"));
    const context = contextFixture();
    const recoveryStore = recoveryStoreFixture();
    const first = render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Restore after remount" } });
    fireEvent.change(screen.getByLabelText("描述"), { target: { value: "Recover exact request" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));
    await screen.findByRole("button", { name: "使用同一请求重试" });
    const firstCall = fixture.createRunTaskCard.mock.calls[0];

    first.unmount();
    render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);
    expect(await screen.findByText(/发现待确认创建“Restore after remount”/)).toBeInTheDocument();
    fireEvent.click(await screen.findByRole("button", { name: "使用同一请求重试" }));

    await waitFor(() => expect(fixture.createRunTaskCard).toHaveBeenCalledTimes(2));
    expect(fixture.createRunTaskCard.mock.calls[1]).toEqual(firstCall);
    await waitFor(() => expect(screen.queryByText(/发现待确认创建“Restore after remount”/)).not.toBeInTheDocument());
  });

  it("does not restore another actor's pending create in the same Run", async () => {
    const fixture = clientFixture();
    fixture.createRunTaskCard.mockRejectedValueOnce(new Error("temporary network interruption"));
    const context = contextFixture();
    const recoveryStore = recoveryStoreFixture();
    const first = render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Private pending request" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));
    await screen.findByRole("button", { name: "使用同一请求重试" });
    first.unmount();

    const otherActorContext = contextFixture();
    otherActorContext.run_context.actor_id = "10000000-0000-4000-8000-000000000099";
    render(<RunTaskCardsPanel client={fixture.client} context={otherActorContext} available recoveryStore={recoveryStore} />);

    const create = await screen.findByRole("button", { name: "新建 Task Card" });
    await waitFor(() => expect(create).toBeEnabled());
    expect(screen.queryByText(/Private pending request/)).not.toBeInTheDocument();
    expect(fixture.createRunTaskCard).toHaveBeenCalledOnce();
  });

  it("blocks a recovered retry when a Worktree binding or authorization version changed", async () => {
    const fixture = clientFixture();
    fixture.createRunTaskCard.mockRejectedValueOnce(new Error("temporary network interruption"));
    const context = contextFixture();
    const recoveryStore = recoveryStoreFixture();
    const first = render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Binding must match" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));
    await screen.findByRole("button", { name: "使用同一请求重试" });

    const changedContext = contextFixture();
    changedContext.focus!.binding_version += 1;
    changedContext.run_context.authorization.engineering_run.version += 1;
    first.rerender(<RunTaskCardsPanel client={fixture.client} context={changedContext} available recoveryStore={recoveryStore} />);

    expect(await screen.findByText(/当前 Worktree、Run binding 或授权版本不同/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "使用同一请求重试" })).toBeDisabled();
    expect(fixture.createRunTaskCard).toHaveBeenCalledOnce();
  });

  it("fails closed if the pending command cannot be saved before POST", async () => {
    const fixture = clientFixture();
    const recoveryStore = {
      load: vi.fn(async () => null),
      saveIfAbsent: vi.fn(async () => { throw new Error("storage quota exceeded"); }),
      removeIfMatches: vi.fn(async () => undefined),
    };
    render(<RunTaskCardsPanel client={fixture.client} context={contextFixture()} available recoveryStore={recoveryStore} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Do not post without recovery" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));

    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect(recoveryStore.saveIfAbsent).toHaveBeenCalledOnce();
    expect(fixture.createRunTaskCard).not.toHaveBeenCalled();
  });

  it("requires confirmation before deleting a pending recovery record", async () => {
    const fixture = clientFixture();
    fixture.createRunTaskCard.mockRejectedValueOnce(new Error("temporary network interruption"));
    const context = contextFixture();
    const recoveryStore = recoveryStoreFixture();
    render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Reconcile before clearing" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));
    await screen.findByText(/发现待确认创建“Reconcile before clearing”/);
    const confirm = vi.fn(() => true);
    vi.stubGlobal("confirm", confirm);
    fireEvent.click(screen.getByRole("button", { name: "我已核对列表，清除恢复记录" }));

    await waitFor(() => expect(screen.queryByText(/发现待确认创建“Reconcile before clearing”/)).not.toBeInTheDocument());
    const scopeKey = taskCardCreateRecoveryScopeKey(context);
    if (!scopeKey) throw new Error("fixture must contain recovery scope");
    expect(confirm).toHaveBeenCalledOnce();
    expect(await recoveryStore.load(scopeKey)).toBeNull();
    expect(fixture.createRunTaskCard).toHaveBeenCalledOnce();
    vi.unstubAllGlobals();
  });

  it("does not retry an expired recovery record", async () => {
    const fixture = clientFixture();
    fixture.createRunTaskCard.mockRejectedValueOnce(new Error("temporary network interruption"));
    const context = contextFixture();
    const recoveryStore = recoveryStoreFixture();
    const first = render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);
    await clickNewTaskCard();
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "Expired recovery" } });
    fireEvent.click(screen.getByRole("button", { name: "创建任务卡" }));
    await screen.findByRole("button", { name: "使用同一请求重试" });
    first.unmount();

    const scopeKey = taskCardCreateRecoveryScopeKey(context);
    if (!scopeKey || !taskCardCreateRecoveryTarget(context)) throw new Error("fixture must contain recovery scope and target");
    const record = await recoveryStore.load(scopeKey);
    if (!record) throw new Error("pending request must persist before dispatch");
    await recoveryStore.removeIfMatches(scopeKey, record.idempotency_key);
    await recoveryStore.saveIfAbsent({ ...record, created_at_ms: Date.now() - 30 * 24 * 60 * 60 * 1000 });
    render(<RunTaskCardsPanel client={fixture.client} context={context} available recoveryStore={recoveryStore} />);

    expect(await screen.findByText(/已超过安全重试期限/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "使用同一请求重试" })).toBeDisabled();
    expect(fixture.createRunTaskCard).toHaveBeenCalledOnce();
  });
});
