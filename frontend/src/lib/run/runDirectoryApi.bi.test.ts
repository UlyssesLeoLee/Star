/*
@cypher schema=1 source_sha256=8508250cd5eee44b4a1c899ed5a92c244e5ce195753729e9f72e5375ab268084
MERGE (self:File {path:"frontend/src/lib/run/runDirectoryApi.bi.test.ts"})
MERGE (query:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.listRunTaskEvents",kind:"method"})
MERGE (self)-[:TESTS]->(query)
@endcypher
*/
import { describe, expect, it, vi } from "vitest";
import { RunDirectoryApiClient, type EngineeringRun } from "./runDirectoryApi";

const RUN = "10000000-0000-4000-8000-000000000001";
const PROJECT = "10000000-0000-4000-8000-000000000002";
const EVENT = "10000000-0000-4000-8000-000000000003";
const run: EngineeringRun = { engineering_run_id: RUN, project_id: PROJECT, repository_id: null, branch_id: null, title: "Run", state: "active", owner_user_id: null, version: 1 };
function item() { return { event_id: EVENT, work_item_id: RUN, aggregate_type: "task", aggregate_version: 1, event_type: "task.created", schema_version: 1, occurred_at: "2026-10-17T00:00:00.000123Z", lifecycle_status: "pending", review_state: null, from_status: null, to_status: null, owner_changed: false }; }
function page() { return { project_id: PROJECT, engineering_run_id: RUN, items: [item()], has_more: false, next_cursor: null, coverage: { completeness: "unknown", freshness: "unknown", basis: "observed facts" } }; }
function setup(body: unknown, status = 200) {
  const fetcher = vi.fn<typeof fetch>().mockResolvedValue(new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json" } }));
  return { fetcher, client: new RunDirectoryApiClient({ getAccessToken: async () => "token", fetcher }) };
}
describe("Run BI event client", () => {
  it("reads a bounded hostless Run page through the authenticated request", async () => {
    const { client, fetcher } = setup(page());
    expect((await client.listRunTaskEvents(run)).items[0].event_id).toBe(EVENT);
    expect(String(fetcher.mock.calls[0][0])).toContain(`/engineering-runs/${RUN}/bi/task-events?limit=50`);
    expect(fetcher.mock.calls[0][1]?.headers).toMatchObject({ Authorization: "Bearer token" });
  });
  it("rejects another Project or Run", async () => {
    for (const patch of [{ project_id: EVENT }, { engineering_run_id: EVENT }]) {
      await expect(setup({ ...page(), ...patch }).client.listRunTaskEvents(run)).rejects.toMatchObject({ code: "scope_mismatch" });
    }
  });
  it("rejects excessive rows, duplicate facts, false coverage and unexpected payload", async () => {
    const cases = [
      { ...page(), items: Array(51).fill(item()) },
      { ...page(), items: [item(), item()] },
      { ...page(), coverage: { ...page().coverage, completeness: "complete" } },
      { ...page(), items: [{ ...item(), payload: { secret: true } }] },
    ];
    for (const body of cases) await expect(setup(body).client.listRunTaskEvents(run)).rejects.toMatchObject({ code: "invalid_projection" });
  });
  it("validates cursor identity and exact microsecond ordering", async () => {
    const cursor = { before_occurred_at: "2026-10-17T00:00:00.000124Z", before_event_id: EVENT };
    const good = setup(page());
    await expect(good.client.listRunTaskEvents(run, cursor)).resolves.toMatchObject({ has_more: false });
    expect(String(good.fetcher.mock.calls[0][0])).toContain("before_event_id=");
    await expect(setup(page()).client.listRunTaskEvents(run, { ...cursor, before_occurred_at: "2026-10-17T00:00:00.000122Z" })).rejects.toMatchObject({ code: "invalid_projection" });
    const bad = setup(page());
    await expect(bad.client.listRunTaskEvents(run, { ...cursor, before_event_id: "invalid" })).rejects.toMatchObject({ code: "invalid_identity" });
    expect(bad.fetcher).not.toHaveBeenCalled();
  });
  it("rejects a continuation cursor inconsistent with the last returned fact", async () => {
    const body = { ...page(), has_more: true, next_cursor: { before_occurred_at: item().occurred_at, before_event_id: RUN } };
    await expect(setup(body).client.listRunTaskEvents(run)).rejects.toMatchObject({ code: "invalid_projection" });
  });
  it("preserves provider 503 without inventing an empty page", async () => {
    await expect(setup({}, 503).client.listRunTaskEvents(run)).rejects.toMatchObject({ status: 503 });
  });
  it("aborts before fetch when the caller is already cancelled", async () => {
    const { client, fetcher } = setup(page());
    const controller = new AbortController(); controller.abort();
    await expect(client.listRunTaskEvents(run, undefined, controller.signal)).rejects.toMatchObject({ name: "AbortError" });
    expect(fetcher).not.toHaveBeenCalled();
  });
});
