/*
@cypher schema=1 source_sha256=2dd63df2db2ee00ed165abacfdda258222f4eeaa87cbb83a91f34c37ab0a1861
MERGE (self:File {path:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts"})
MERGE (roundTrip:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::persists one pending command across store instances without replacing its idempotency key",kind:"test"})
MERGE (bounded:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::rejects the 33rd unresolved command instead of growing without a bound",kind:"test"})
MERGE (missingFocusVersion:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::rejects a focus without a context version",kind:"test"})
MERGE (scope:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryScopeKey",kind:"function"})
MERGE (target:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryTarget",kind:"function"})
MERGE (factory:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::createTaskCardCreateRecoveryStore",kind:"function"})
MERGE (maxRecords:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TASK_CARD_CREATE_RECOVERY_MAX_RECORDS",kind:"constant"})
MERGE (roundTrip)-[:TESTS]->(scope)
MERGE (roundTrip)-[:TESTS]->(target)
MERGE (roundTrip)-[:TESTS]->(factory)
MERGE (bounded)-[:TESTS]->(factory)
MERGE (bounded)-[:TESTS]->(maxRecords)
MERGE (missingFocusVersion)-[:TESTS]->(target)
@endcypher
*/

import "fake-indexeddb/auto";
import { describe, expect, it } from "vitest";
import {
  createTaskCardCreateRecoveryStore,
  TASK_CARD_CREATE_RECOVERY_MAX_RECORDS,
  taskCardCreateRecoveryScopeKey,
  taskCardCreateRecoveryTarget,
  type PendingTaskCardCreate,
} from "./taskCardCreateRecoveryStore";
import type { RunContextEnvelope } from "./runDirectoryApi";

function contextFixture(): RunContextEnvelope {
  const project = "10000000-0000-4000-8000-000000000001";
  const repository = "10000000-0000-4000-8000-000000000002";
  const branch = "10000000-0000-4000-8000-000000000003";
  const run = "10000000-0000-4000-8000-000000000004";
  return {
    run_context: {
      tenant_id: "10000000-0000-4000-8000-000000000005",
      project_id: project,
      repository_id: repository,
      branch_id: branch,
      engineering_run_id: run,
      actor_id: "10000000-0000-4000-8000-000000000006",
      actor_kind: "user",
      authorization: {
        project: { binding_id: "40000000-0000-4000-8000-000000000001", role: "developer", version: 1 },
        branch: { binding_id: "40000000-0000-4000-8000-000000000002", role: "developer", version: 1 },
        engineering_run: { binding_id: "40000000-0000-4000-8000-000000000003", role: "developer", version: 1 },
      },
      granted_scopes: ["work-item:read", "work-item:write"],
      context_version: "ctx:v1",
      resolved_at: "2026-10-09T00:00:00Z",
      correlation_id: "10000000-0000-4000-8000-000000000007",
    },
    branch: {
      branch_id: branch, project_id: project, repository_id: repository, name: "feature/recovery",
      full_ref: "refs/heads/feature/recovery", head_commit_id: null, state: "active", version: 1,
    },
    engineering_run: {
      engineering_run_id: run, project_id: project, repository_id: repository, branch_id: branch,
      title: "Run", state: "active", owner_user_id: null, version: 1,
    },
    focus: {
      worktree_id: "30000000-0000-4000-8000-000000000001", engineering_run_id: run,
      project_id: project, repository_id: repository, workspace_id: "30000000-0000-4000-8000-000000000002",
      name: "focus", branch: "feature/recovery", human_state: "active", machine_state: "ready",
      agent_id: null, agent_session_id: null, runtime_id: null, archived: false, version: 1,
      binding_id: "30000000-0000-4000-8000-000000000003", binding_version: 1,
      project_binding_id: "30000000-0000-4000-8000-000000000004", project_binding_version: 1,
    },
    focus_version: "focus:v1",
    capabilities: { run_owned_apps_available: true, execution_admission_available: false },
  };
}

describe("Task Card create recovery store", () => {
  it("rejects a focus without a context version", () => {
    const context = contextFixture();
    context.focus_version = null;
    expect(taskCardCreateRecoveryTarget(context)).toBeNull();
  });

  it("persists one pending command across store instances without replacing its idempotency key", async () => {
    const context = contextFixture();
    const scopeKey = taskCardCreateRecoveryScopeKey(context);
    const target = taskCardCreateRecoveryTarget(context);
    if (!scopeKey || !target) throw new Error("fixture must contain a Run and Worktree focus");
    const record: PendingTaskCardCreate = {
      schema_version: 1,
      scope_key: scopeKey,
      created_at_ms: Date.now(),
      input: {
        title: "Keep the command after reload",
        description: "Exact request body",
        priority: "high",
        correlation_id: "10000000-0000-4000-8000-000000000008",
      },
      idempotency_key: "10000000-0000-4000-8000-000000000009",
      target,
    };
    const databaseName = `task-card-recovery-test-${globalThis.crypto.randomUUID()}`;
    const firstStore = createTaskCardCreateRecoveryStore(databaseName);
    const secondStore = createTaskCardCreateRecoveryStore(databaseName);

    expect(await firstStore.saveIfAbsent(record)).toEqual(record);
    expect(await secondStore.load(scopeKey)).toEqual(record);
    expect(await secondStore.saveIfAbsent({ ...record, idempotency_key: "10000000-0000-4000-8000-000000000010" })).toEqual(record);
  });

  it("rejects the 33rd unresolved command instead of growing without a bound", async () => {
    const databaseName = `task-card-recovery-cap-${globalThis.crypto.randomUUID()}`;
    const store = createTaskCardCreateRecoveryStore(databaseName);
    const records: PendingTaskCardCreate[] = [];
    for (let index = 1; index <= TASK_CARD_CREATE_RECOVERY_MAX_RECORDS + 1; index += 1) {
      const context = contextFixture();
      const runId = `10000000-0000-4000-8000-${index.toString().padStart(12, "0")}`;
      context.engineering_run.engineering_run_id = runId;
      context.run_context.engineering_run_id = runId;
      context.focus!.engineering_run_id = runId;
      const scopeKey = taskCardCreateRecoveryScopeKey(context);
      const target = taskCardCreateRecoveryTarget(context);
      if (!scopeKey || !target) throw new Error("fixture must contain a Run and Worktree focus");
      records.push({
        schema_version: 1,
        scope_key: scopeKey,
        created_at_ms: Date.now(),
        input: {
          title: `Pending ${index}`,
          description: "",
          priority: "medium",
          correlation_id: globalThis.crypto.randomUUID(),
        },
        idempotency_key: globalThis.crypto.randomUUID(),
        target,
      });
    }

    for (const record of records.slice(0, TASK_CARD_CREATE_RECOVERY_MAX_RECORDS)) {
      expect(await store.saveIfAbsent(record)).toEqual(record);
    }
    await expect(store.saveIfAbsent(records[TASK_CARD_CREATE_RECOVERY_MAX_RECORDS])).rejects.toThrow("本地待恢复记录已达上限");
  });
});
