/*
@cypher schema=1 source_sha256=7a00e510348b6f59a54036db8d3d46191f589a605893b2b4d6c6115a019cc55a
MERGE (self:File {path:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts"})
MERGE (roundTrip:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::persists one pending command across store instances without replacing its idempotency key",kind:"test"})
MERGE (bounded:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::rejects the 33rd unresolved command instead of growing without a bound",kind:"test"})
MERGE (missingFocusVersion:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::rejects a focus without a context version",kind:"test"})
MERGE (hostless:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::builds a hostless Run target with no Worktree focus or Branch authorization",kind:"test"})
MERGE (legacy:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.test.ts::loads schema v1 recovery unchanged and matches only its original legacy target",kind:"test"})
MERGE (scope:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryScopeKey",kind:"function"})
MERGE (target:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryTarget",kind:"function"})
MERGE (matchesContext:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryTargetMatchesContext",kind:"function"})
MERGE (factory:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::createTaskCardCreateRecoveryStore",kind:"function"})
MERGE (maxRecords:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TASK_CARD_CREATE_RECOVERY_MAX_RECORDS",kind:"constant"})
MERGE (roundTrip)-[:TESTS]->(scope)
MERGE (roundTrip)-[:TESTS]->(target)
MERGE (roundTrip)-[:TESTS]->(factory)
MERGE (bounded)-[:TESTS]->(factory)
MERGE (bounded)-[:TESTS]->(maxRecords)
MERGE (missingFocusVersion)-[:TESTS]->(target)
MERGE (hostless)-[:TESTS]->(target)
MERGE (legacy)-[:TESTS]->(factory)
MERGE (legacy)-[:TESTS]->(matchesContext)
@endcypher
*/

import "fake-indexeddb/auto";
import { describe, expect, it } from "vitest";
import {
  createTaskCardCreateRecoveryStore,
  TASK_CARD_CREATE_RECOVERY_MAX_RECORDS,
  taskCardCreateRecoveryScopeKey,
  taskCardCreateRecoveryTarget,
  taskCardCreateRecoveryTargetMatchesContext,
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
    capabilities: { task_cards_available: true, run_owned_apps_available: false, execution_admission_available: false },
  };
}

describe("Task Card create recovery store", () => {
  it("rejects a focus without a context version", () => {
    const context = contextFixture();
    context.focus_version = null;
    expect(taskCardCreateRecoveryTarget(context)).toBeNull();
  });

  it("builds a hostless Run target with no Worktree focus or Branch authorization", () => {
    const context = contextFixture();
    context.engineering_run = { ...context.engineering_run, repository_id: null, branch_id: null };
    context.run_context.repository_id = null;
    context.run_context.branch_id = null;
    context.run_context.authorization.branch = null;
    context.branch = null;
    context.focus = null;
    context.focus_version = null;

    expect(taskCardCreateRecoveryTarget(context)).toMatchObject({
      repository_id: null, branch_id: null, focus_worktree_id: null, focus_version: null,
      focus_repository_id: null, project_binding_id: null, worktree_binding_id: null,
    });
  });

  it("persists one pending command across store instances without replacing its idempotency key", async () => {
    const context = contextFixture();
    const scopeKey = taskCardCreateRecoveryScopeKey(context);
    const target = taskCardCreateRecoveryTarget(context);
    if (!scopeKey || !target) throw new Error("fixture must contain a valid Run context");
    const record: PendingTaskCardCreate = {
      schema_version: 2,
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
      if (!scopeKey || !target) throw new Error("fixture must contain a valid Run context");
      records.push({
        schema_version: 2,
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

  it("loads schema v1 recovery unchanged and matches only its original legacy target", async () => {
    const context = contextFixture();
    const scopeKey = taskCardCreateRecoveryScopeKey(context);
    if (!scopeKey) throw new Error("fixture must contain a Run scope");
    const record: PendingTaskCardCreate = {
      schema_version: 1,
      scope_key: scopeKey,
      created_at_ms: Date.now(),
      input: { title: "Legacy pending", description: "preserve", priority: "medium", correlation_id: "10000000-0000-4000-8000-000000000008" },
      idempotency_key: "10000000-0000-4000-8000-000000000009",
      target: {
        tenant_id: context.run_context.tenant_id, actor_id: context.run_context.actor_id, actor_kind: context.run_context.actor_kind,
        project_id: context.engineering_run.project_id, repository_id: context.engineering_run.repository_id!, branch_id: context.engineering_run.branch_id!,
        engineering_run_id: context.engineering_run.engineering_run_id, worktree_id: context.focus!.worktree_id,
        context_version: context.run_context.context_version, focus_version: context.focus_version!,
        project_authorization_binding_id: context.run_context.authorization.project.binding_id,
        project_authorization_version: context.run_context.authorization.project.version,
        branch_authorization_binding_id: context.run_context.authorization.branch!.binding_id,
        branch_authorization_version: context.run_context.authorization.branch!.version,
        run_authorization_binding_id: context.run_context.authorization.engineering_run.binding_id,
        run_authorization_version: context.run_context.authorization.engineering_run.version,
        project_binding_id: context.focus!.project_binding_id, project_binding_version: context.focus!.project_binding_version,
        worktree_binding_id: context.focus!.binding_id, worktree_binding_version: context.focus!.binding_version,
        branch_version: context.branch!.version, run_version: context.engineering_run.version, worktree_version: context.focus!.version,
      },
    };
    const store = createTaskCardCreateRecoveryStore(`task-card-recovery-v1-${globalThis.crypto.randomUUID()}`);

    expect(taskCardCreateRecoveryTargetMatchesContext(record, context)).toBe(true);
    const changedFocus = contextFixture();
    changedFocus.focus = { ...changedFocus.focus!, worktree_id: "30000000-0000-4000-8000-000000000099" };
    expect(taskCardCreateRecoveryTargetMatchesContext(record, changedFocus)).toBe(false);
    expect(await store.saveIfAbsent(record)).toEqual(record);
    expect(await store.load(scopeKey)).toEqual(record);
  });
});
