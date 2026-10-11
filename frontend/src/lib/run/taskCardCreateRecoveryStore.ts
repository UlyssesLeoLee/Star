/*
@cypher schema=1 source_sha256=bddce4420ce74b7f3aa9be7da9ac109f63505708ba9bcc71b24e589508f2a9aa
MERGE (self:File {path:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts"})
MERGE (scope:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryScopeKey",kind:"function"})
MERGE (target:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::taskCardCreateRecoveryTarget",kind:"function"})
MERGE (sameTarget:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::sameTaskCardCreateRecoveryTarget",kind:"function"})
MERGE (store:Type {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore",kind:"interface"})
MERGE (targetType:Type {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryTarget",kind:"interface"})
MERGE (factory:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::createTaskCardCreateRecoveryStore",kind:"function"})
MERGE (databaseProvider:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::createTaskCardCreateRecoveryStore.database",kind:"function"})
MERGE (getStore:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::getTaskCardCreateRecoveryStore",kind:"function"})
MERGE (open:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::openDatabase",kind:"function"})
MERGE (validate:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::validPendingRecord",kind:"function"})
MERGE (load:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore.load",kind:"method"})
MERGE (save:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore.saveIfAbsent",kind:"method"})
MERGE (remove:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryStore.removeIfMatches",kind:"method"})
MERGE (context:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::RunContextEnvelope",kind:"type"})
MERGE (input:Type {id:"frontend/src/lib/run/runDirectoryApi.ts::CreateRunTaskCardInput",kind:"type"})
MERGE (database:Database {name:"IndexedDB star-run-task-card-recovery/pending-create"})
MERGE (errorType:Type {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TaskCardCreateRecoveryError",kind:"class"})
MERGE (maxRecords:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TASK_CARD_CREATE_RECOVERY_MAX_RECORDS",kind:"constant"})
MERGE (maxAge:Symbol {id:"frontend/src/lib/run/taskCardCreateRecoveryStore.ts::TASK_CARD_CREATE_RECOVERY_MAX_AGE_MS",kind:"constant"})
MERGE (apiFile:File {path:"frontend/src/lib/run/runDirectoryApi.ts"})
MERGE (self)-[:DEFINES]->(scope)
MERGE (self)-[:DEFINES]->(target)
MERGE (self)-[:DEFINES]->(sameTarget)
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(targetType)
MERGE (self)-[:DEFINES]->(factory)
MERGE (self)-[:DEFINES]->(errorType)
MERGE (self)-[:DEFINES]->(getStore)
MERGE (self)-[:DEFINES]->(open)
MERGE (self)-[:DEFINES]->(validate)
MERGE (self)-[:DEFINES]->(maxRecords)
MERGE (self)-[:DEFINES]->(maxAge)
MERGE (self)-[:IMPORTS]->(apiFile)
MERGE (scope)-[:USES_TYPE]->(context)
MERGE (target)-[:USES_TYPE]->(context)
MERGE (target)-[:USES_TYPE]->(targetType)
MERGE (sameTarget)-[:USES_TYPE]->(targetType)
MERGE (store)-[:USES_TYPE]->(input)
MERGE (factory)-[:DEFINES]->(databaseProvider)
MERGE (factory)-[:DEFINES]->(load)
MERGE (factory)-[:DEFINES]->(save)
MERGE (factory)-[:DEFINES]->(remove)
MERGE (databaseProvider)-[:CALLS]->(open)
MERGE (load)-[:CALLS]->(databaseProvider)
MERGE (save)-[:CALLS]->(databaseProvider)
MERGE (remove)-[:CALLS]->(databaseProvider)
MERGE (load)-[:CALLS]->(validate)
MERGE (save)-[:CALLS]->(validate)
MERGE (save)-[:READS]->(maxRecords)
MERGE (load)-[:READS]->(database)
MERGE (save)-[:READS]->(database)
MERGE (save)-[:WRITES]->(database)
MERGE (remove)-[:READS]->(database)
MERGE (remove)-[:WRITES]->(database)
MERGE (getStore)-[:CALLS]->(factory)
@endcypher
*/

import type { RunContextEnvelope } from "@/lib/run/runDirectoryApi";
import {
  RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS,
  RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS,
  RUN_TASK_CARD_PRIORITIES,
  type CreateRunTaskCardInput,
  type RunTaskCardPriority,
} from "@/lib/run/runDirectoryApi";

const DATABASE_VERSION = 1;
const OBJECT_STORE = "pending-create";
const DATABASE_NAME = "star-run-task-card-recovery";
export const TASK_CARD_CREATE_RECOVERY_MAX_RECORDS = 32;
export const TASK_CARD_CREATE_RECOVERY_MAX_AGE_MS = 29 * 24 * 60 * 60 * 1000;

export interface TaskCardCreateRecoveryTarget {
  tenant_id: string;
  actor_id: string;
  actor_kind: string;
  project_id: string;
  repository_id: string;
  branch_id: string;
  engineering_run_id: string;
  worktree_id: string;
  context_version: string;
  focus_version: string;
  project_authorization_binding_id: string;
  project_authorization_version: number;
  branch_authorization_binding_id: string;
  branch_authorization_version: number;
  run_authorization_binding_id: string;
  run_authorization_version: number;
  project_binding_id: string;
  project_binding_version: number;
  worktree_binding_id: string;
  worktree_binding_version: number;
  branch_version: number;
  run_version: number;
  worktree_version: number;
}

export interface PendingTaskCardCreate {
  schema_version: 1;
  scope_key: string;
  created_at_ms: number;
  input: CreateRunTaskCardInput;
  idempotency_key: string;
  target: TaskCardCreateRecoveryTarget;
}

export interface TaskCardCreateRecoveryStore {
  load(scopeKey: string): Promise<PendingTaskCardCreate | null>;
  saveIfAbsent(record: PendingTaskCardCreate): Promise<PendingTaskCardCreate | null>;
  removeIfMatches(scopeKey: string, idempotencyKey: string): Promise<void>;
}

export class TaskCardCreateRecoveryError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "TaskCardCreateRecoveryError";
  }
}

export function taskCardCreateRecoveryScopeKey(context: RunContextEnvelope): string | null {
  const { tenant_id, actor_id, project_id } = context.run_context;
  const runId = context.engineering_run.engineering_run_id;
  if (!tenant_id || !actor_id || !project_id || !runId) return null;
  return JSON.stringify(["task-card-create", 1, tenant_id, actor_id, project_id, runId]);
}

export function taskCardCreateRecoveryTarget(context: RunContextEnvelope): TaskCardCreateRecoveryTarget | null {
  const focus = context.focus;
  if (!focus || !context.run_context.context_version || !context.focus_version) return null;
  const authorization = context.run_context.authorization;
  return {
    tenant_id: context.run_context.tenant_id,
    actor_id: context.run_context.actor_id,
    actor_kind: context.run_context.actor_kind,
    project_id: context.engineering_run.project_id,
    repository_id: context.engineering_run.repository_id,
    branch_id: context.engineering_run.branch_id,
    engineering_run_id: context.engineering_run.engineering_run_id,
    worktree_id: focus.worktree_id,
    context_version: context.run_context.context_version,
    focus_version: context.focus_version,
    project_authorization_binding_id: authorization.project.binding_id,
    project_authorization_version: authorization.project.version,
    branch_authorization_binding_id: authorization.branch.binding_id,
    branch_authorization_version: authorization.branch.version,
    run_authorization_binding_id: authorization.engineering_run.binding_id,
    run_authorization_version: authorization.engineering_run.version,
    project_binding_id: focus.project_binding_id,
    project_binding_version: focus.project_binding_version,
    worktree_binding_id: focus.binding_id,
    worktree_binding_version: focus.binding_version,
    branch_version: context.branch.version,
    run_version: context.engineering_run.version,
    worktree_version: focus.version,
  };
}

export function sameTaskCardCreateRecoveryTarget(
  left: TaskCardCreateRecoveryTarget,
  right: TaskCardCreateRecoveryTarget | null,
): boolean {
  if (!right) return false;
  return Object.keys(left).every((key) =>
    left[key as keyof TaskCardCreateRecoveryTarget] === right[key as keyof TaskCardCreateRecoveryTarget]);
}

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const TARGET_UUID_FIELDS: readonly (keyof TaskCardCreateRecoveryTarget)[] = [
  "tenant_id", "actor_id", "project_id", "repository_id", "branch_id", "engineering_run_id", "worktree_id",
  "project_authorization_binding_id", "branch_authorization_binding_id", "run_authorization_binding_id",
  "project_binding_id", "worktree_binding_id",
];
const TARGET_VERSION_FIELDS: readonly (keyof TaskCardCreateRecoveryTarget)[] = [
  "project_authorization_version", "branch_authorization_version", "run_authorization_version",
  "project_binding_version", "worktree_binding_version", "branch_version", "run_version", "worktree_version",
];

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value && typeof value === "object" && !Array.isArray(value));
}

function validPendingRecord(value: unknown, expectedScopeKey: string): value is PendingTaskCardCreate {
  if (!isRecord(value) || value.schema_version !== 1 || value.scope_key !== expectedScopeKey ||
      typeof value.created_at_ms !== "number" || !Number.isSafeInteger(value.created_at_ms) || value.created_at_ms <= 0 ||
      !UUID.test(String(value.idempotency_key ?? "")) || !isRecord(value.input) || !isRecord(value.target)) return false;

  const input = value.input;
  if (typeof input.title !== "string" || !input.title.trim() || input.title.length > RUN_TASK_CARD_CREATE_TITLE_MAX_CHARS ||
      typeof input.description !== "string" || input.description.length > RUN_TASK_CARD_CREATE_DESCRIPTION_MAX_CHARS ||
      typeof input.priority !== "string" || !RUN_TASK_CARD_PRIORITIES.includes(input.priority as RunTaskCardPriority) ||
      typeof input.correlation_id !== "string" || !UUID.test(input.correlation_id)) return false;

  const target = value.target;
  if (TARGET_UUID_FIELDS.some((field) => typeof target[field] !== "string" || !UUID.test(target[field] as string)) ||
      TARGET_VERSION_FIELDS.some((field) => !Number.isSafeInteger(target[field]) || (target[field] as number) < 1) ||
      typeof target.actor_kind !== "string" || target.actor_kind.length === 0 ||
      typeof target.context_version !== "string" || target.context_version.length === 0 ||
      typeof target.focus_version !== "string" || target.focus_version.length === 0) return false;

  const targetScopeKey = JSON.stringify([
    "task-card-create", 1, target.tenant_id, target.actor_id, target.project_id, target.engineering_run_id,
  ]);
  if (targetScopeKey !== expectedScopeKey) return false;

  return true;
}

function openDatabase(databaseName: string): Promise<IDBDatabase> {
  if (!globalThis.indexedDB) return Promise.reject(new TaskCardCreateRecoveryError("此环境不支持本地恢复存储。"));
  return new Promise((resolve, reject) => {
    const request = globalThis.indexedDB.open(databaseName, DATABASE_VERSION);
    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains(OBJECT_STORE)) {
        database.createObjectStore(OBJECT_STORE, { keyPath: "scope_key" });
      }
    };
    request.onsuccess = () => {
      request.result.onversionchange = () => request.result.close();
      resolve(request.result);
    };
    request.onerror = () => reject(new TaskCardCreateRecoveryError("无法打开本地恢复存储。"));
    request.onblocked = () => reject(new TaskCardCreateRecoveryError("本地恢复存储正被其他页面占用。"));
  });
}

export function createTaskCardCreateRecoveryStore(databaseName = DATABASE_NAME): TaskCardCreateRecoveryStore {
  let databasePromise: Promise<IDBDatabase> | null = null;
  function database(): Promise<IDBDatabase> {
    if (!databasePromise) {
      databasePromise = openDatabase(databaseName).catch((error: unknown) => {
        databasePromise = null;
        throw error;
      });
    }
    return databasePromise;
  }

  return {
    async load(scopeKey) {
      const db = await database();
      return new Promise((resolve, reject) => {
        const tx = db.transaction(OBJECT_STORE, "readonly");
        const request = tx.objectStore(OBJECT_STORE).get(scopeKey);
        let result: unknown = null;
        request.onsuccess = () => { result = request.result ?? null; };
        request.onerror = () => reject(new TaskCardCreateRecoveryError("无法读取本地恢复记录。"));
        tx.oncomplete = () => {
          if (result === null) return resolve(null);
          if (!validPendingRecord(result, scopeKey)) return reject(new TaskCardCreateRecoveryError("本地恢复记录格式无效，已阻止重放。"));
          resolve(result);
        };
        tx.onerror = () => reject(new TaskCardCreateRecoveryError("无法读取本地恢复记录。"));
        tx.onabort = () => reject(new TaskCardCreateRecoveryError("本地恢复记录读取事务已中止。"));
      });
    },

    async saveIfAbsent(record) {
      if (!validPendingRecord(record, record.scope_key)) {
        throw new TaskCardCreateRecoveryError("待恢复请求格式无效，已阻止发送。");
      }
      const db = await database();
      return new Promise((resolve, reject) => {
        const tx = db.transaction(OBJECT_STORE, "readwrite");
        const store = tx.objectStore(OBJECT_STORE);
        let result: PendingTaskCardCreate | null = null;
        let abortMessage = "无法保存本地恢复记录，未发送创建请求。";
        const existing = store.get(record.scope_key);
        existing.onsuccess = () => {
          if (existing.result !== undefined) {
            if (!validPendingRecord(existing.result, record.scope_key)) {
              abortMessage = "已有本地恢复记录格式无效，已阻止发送。";
              tx.abort();
              return;
            }
            result = existing.result;
            return;
          }
          const count = store.count();
          count.onsuccess = () => {
            if (count.result >= TASK_CARD_CREATE_RECOVERY_MAX_RECORDS) {
              abortMessage = "本地待恢复记录已达上限，请先处理已有记录。";
              tx.abort();
              return;
            }
            store.add(record);
            result = record;
          };
          count.onerror = () => tx.abort();
        };
        existing.onerror = () => tx.abort();
        tx.oncomplete = () => resolve(result);
        tx.onerror = () => reject(new TaskCardCreateRecoveryError(abortMessage));
        tx.onabort = () => reject(new TaskCardCreateRecoveryError(abortMessage));
      });
    },

    async removeIfMatches(scopeKey, idempotencyKey) {
      const db = await database();
      return new Promise((resolve, reject) => {
        const tx = db.transaction(OBJECT_STORE, "readwrite");
        const store = tx.objectStore(OBJECT_STORE);
        const request = store.get(scopeKey);
        request.onsuccess = () => {
          const value: unknown = request.result;
          if (isRecord(value) && value.idempotency_key === idempotencyKey) store.delete(scopeKey);
        };
        request.onerror = () => tx.abort();
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(new TaskCardCreateRecoveryError("无法清理已处理的本地恢复记录。"));
        tx.onabort = () => reject(new TaskCardCreateRecoveryError("无法清理已处理的本地恢复记录。"));
      });
    },
  };
}

let defaultRecoveryStore: TaskCardCreateRecoveryStore | null = null;
export function getTaskCardCreateRecoveryStore(): TaskCardCreateRecoveryStore {
  if (!defaultRecoveryStore) defaultRecoveryStore = createTaskCardCreateRecoveryStore();
  return defaultRecoveryStore;
}
