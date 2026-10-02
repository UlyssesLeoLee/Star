/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/components/run/RunTaskCardsPanel.test.tsx",type:"file",language:"tsx"}),
 (runFixture:Function {name:"runFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (cardFixture:Function {name:"cardFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (clientFixture:Function {name:"clientFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (closedTest:Function {name:"does not request or display tasks while Run Apps are unavailable",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (liveTest:Function {name:"renders authorized Run tasks while leaving task CLI disabled",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (f)-[:CONTAINS]->(runFixture),(f)-[:CONTAINS]->(cardFixture),(f)-[:CONTAINS]->(clientFixture),(f)-[:CONTAINS]->(closedTest),(f)-[:CONTAINS]->(liveTest),
 (clientFixture)-[:CALLS]->(cardFixture),(closedTest)-[:CALLS]->(runFixture),(closedTest)-[:CALLS]->(clientFixture),(liveTest)-[:CALLS]->(runFixture),(liveTest)-[:CALLS]->(cardFixture),(liveTest)-[:CALLS]->(clientFixture);
*/

import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RunTaskCardsPanel } from "./RunTaskCardsPanel";
import { RUN_TASK_CARD_PAGE_SIZE, type EngineeringRun, type RunDirectoryApiClient, type RunTaskCardPage } from "@/lib/run/runDirectoryApi";

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

function clientFixture() {
  const page: RunTaskCardPage = { rows: [cardFixture()], limit: RUN_TASK_CARD_PAGE_SIZE, next_after: null };
  const listRunWorkItems = vi.fn(async () => page);
  return { client: { listRunWorkItems } as unknown as RunDirectoryApiClient, listRunWorkItems };
}

describe("RunTaskCardsPanel", () => {
  it("does not request or display tasks while Run Apps are unavailable", () => {
    const fixture = clientFixture();
    render(<RunTaskCardsPanel client={fixture.client} run={runFixture()} available={false} />);

    expect(screen.getByText(/服务端尚未开放 Run-owned Apps/)).toBeInTheDocument();
    expect(fixture.listRunWorkItems).not.toHaveBeenCalled();
  });

  it("renders authorized Run tasks while leaving task CLI disabled", async () => {
    const fixture = clientFixture();
    render(<RunTaskCardsPanel client={fixture.client} run={runFixture()} available />);

    expect(await screen.findByText("Real Run task")).toBeInTheDocument();
    expect(fixture.listRunWorkItems).toHaveBeenCalledOnce();
    expect(screen.getByRole("button", { name: /打开 CLI/ })).toBeDisabled();
  });
});
