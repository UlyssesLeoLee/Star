/*
@cypher schema=1 source_sha256=4ee2ec071507a43d1d117a531c9ff190b322309d226f2c8c020ba162988a6a0e
MERGE (self:File {path:"frontend/src/components/run/RunBiPanel.test.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/run/RunBiPanel.tsx::RunBiPanel",kind:"function"})
MERGE (self)-[:TESTS]->(panel)
@endcypher
*/
/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"RunBiPanel.test.tsx",type:"file",language:"tsx"}),
 (fixture:Function {name:"page",type:"function"}),
 (panel:Function {name:"RunBiPanel",type:"function"}),
 (f)-[:CONTAINS]->(fixture),(f)-[:CALLS]->(panel);
*/
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RunBiPanel } from "./RunBiPanel";
import { RunDirectoryError, type EngineeringRun, type RunBiTaskEventPage, type RunDirectoryApiClient } from "@/lib/run/runDirectoryApi";

const projectId = "10000000-0000-4000-8000-000000000001";
const runId = "10000000-0000-4000-8000-000000000002";
const run = { project_id: projectId, engineering_run_id: runId } as EngineeringRun;
const otherRunId = "10000000-0000-4000-8000-000000000003";
function page(run = runId): RunBiTaskEventPage {
  return { project_id: projectId, engineering_run_id: run, items: [], has_more: false, next_cursor: null, coverage: { completeness: "unknown", freshness: "unknown", basis: "no_authoritative_checkpoint" } };
}

describe("RunBiPanel", () => {
  it("describes unavailable analysis without exposing provider internals", async () => {
    const list = vi.fn().mockRejectedValue(new RunDirectoryError(503, "unavailable", "BI_DATABASE_URL missing"));
    render(<RunBiPanel client={{ listRunTaskEvents: list } as unknown as RunDirectoryApiClient} run={run} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Run 分析暂不可用，请稍后重试。");
    expect(screen.queryByText(/BI_DATABASE_URL/)).toBeNull();
    expect(screen.queryByText("尚未观察到任务事件；这不表示全部事件已送达。")).toBeNull();
  });

  it("keeps empty results explicitly incomplete without inventing metrics", async () => {
    const list = vi.fn().mockResolvedValue(page());
    render(<RunBiPanel client={{ listRunTaskEvents: list } as unknown as RunDirectoryApiClient} run={run} />);
    await screen.findByText("尚未观察到任务事件；这不表示全部事件已送达。");
    expect(screen.getByText(/完整性：未知；新鲜度：未知/)).toBeTruthy();
    expect(list).toHaveBeenCalledWith(run, undefined, expect.any(AbortSignal));
  });

  it("aborts the previous scope and rejects its late result", async () => {
    let finish: ((value: RunBiTaskEventPage) => void) | undefined;
    const list = vi.fn().mockImplementationOnce(() => new Promise<RunBiTaskEventPage>((resolve) => { finish = resolve; })).mockResolvedValue(page(otherRunId));
    const client = { listRunTaskEvents: list } as unknown as RunDirectoryApiClient;
    const view = render(<RunBiPanel client={client} run={run} />);
    const signal = list.mock.calls[0][2] as AbortSignal;
    view.rerender(<RunBiPanel client={client} run={{ ...run, engineering_run_id: otherRunId }} />);
    expect(signal.aborted).toBe(true);
    await screen.findByText("尚未观察到任务事件；这不表示全部事件已送达。");
    await act(async () => { finish?.({ ...page(), has_more: true, next_cursor: { before_occurred_at: "2026-10-17T00:00:00Z", before_event_id: runId } }); });
    expect((screen.getByRole("button", { name: "下一页" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("replaces pages using the server cursor and aborts on unmount", async () => {
    const cursor = { before_occurred_at: "2026-10-17T00:00:00Z", before_event_id: runId };
    const list = vi.fn().mockResolvedValueOnce({ ...page(), has_more: true, next_cursor: cursor }).mockResolvedValue(page());
    const view = render(<RunBiPanel client={{ listRunTaskEvents: list } as unknown as RunDirectoryApiClient} run={run} />);
    await screen.findByRole("button", { name: "下一页" });
    fireEvent.click(screen.getByRole("button", { name: "下一页" }));
    await waitFor(() => expect(list).toHaveBeenCalledTimes(2));
    expect(list.mock.calls[1][1]).toEqual(cursor);
    const signal = list.mock.calls[1][2] as AbortSignal;
    view.unmount();
    expect(signal.aborted).toBe(true);
  });
});
