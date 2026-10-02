/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/lib/run/runDirectoryApi.ts",type:"file",language:"typescript"}),
 (error:Class {name:"RunDirectoryError",type:"class"}),(client:Class {name:"RunDirectoryApiClient",type:"class"}),
 (errorCtor:Function {name:"RunDirectoryError.constructor",type:"function"}),(token:Function {name:"readAccessToken",type:"function"}),
 (stage:Function {name:"awaitAbortable",type:"function"}),(stageCancel:Function {name:"cancelRequestStage",type:"function"}),(bodyCancel:Function {name:"cancelResponseBody",type:"function"}),(tokenProvider:Function {name:"RunAccessTokenProvider",type:"function"}),
 (expire:Function {name:"expireRunDirectoryRequest",type:"function"}),(setTimer:Function {name:"setTimeout",type:"function"}),(clearTimer:Function {name:"clearTimeout",type:"function"}),
 (controllerAbort:Function {name:"AbortController.abort",type:"function"}),(readerCancel:Function {name:"ReadableStreamDefaultReader.cancel",type:"function"}),(readerRead:Function {name:"ReadableStreamDefaultReader.read",type:"function"}),(readerRelease:Function {name:"ReadableStreamDefaultReader.releaseLock",type:"function"}),
 (deadline:Variable {name:"deadlineTimer",type:"variable"}),(timedOut:Variable {name:"timedOut",type:"variable"}),(timeout:Variable {name:"RUN_DIRECTORY_REQUEST_TIMEOUT_MS",type:"variable"}),
 (uuidPattern:Variable {name:"UUID",type:"variable"}),(responseLimit:Variable {name:"MAX_RESPONSE_BYTES",type:"variable"}),(pageSize:Variable {name:"RUN_DIRECTORY_PAGE_SIZE",type:"variable"}),
 (taskPageSize:Variable {name:"RUN_TASK_CARD_PAGE_SIZE",type:"variable"}),(taskResponseLimit:Variable {name:"MAX_RUN_TASK_CARD_RESPONSE_BYTES",type:"variable"}),
 (project:Class {name:"AuthorizedRunProject",type:"class"}),(branch:Class {name:"CloudBranch",type:"class"}),
 (run:Class {name:"EngineeringRun",type:"class"}),(wt:Class {name:"RunWorktree",type:"class"}),
 (taskCard:Class {name:"RunTaskCard",type:"class"}),(taskPage:Class {name:"RunTaskCardPage",type:"class"}),
 (grant:Class {name:"RunAuthorization",type:"class"}),(context:Class {name:"RunContextEnvelope",type:"class"}),
 (page:Class {name:"DirectoryPage",type:"class"}),(options:Class {name:"RunDirectoryClientOptions",type:"class"}),
 (request:Function {name:"RunDirectoryApiClient.request",type:"function"}),
 (acquire:Function {name:"RunDirectoryApiClient.acquire",type:"function"}),(release:Function {name:"RunDirectoryApiClient.release",type:"function"}),
 (dispose:Function {name:"RunDirectoryApiClient.dispose",type:"function"}),(ctor:Function {name:"RunDirectoryApiClient.constructor",type:"function"}),
 (projects:Function {name:"RunDirectoryApiClient.listProjects",type:"function"}),(branches:Function {name:"RunDirectoryApiClient.listBranches",type:"function"}),
 (runs:Function {name:"RunDirectoryApiClient.listRuns",type:"function"}),(worktrees:Function {name:"RunDirectoryApiClient.listWorktrees",type:"function"}),
 (tasks:Function {name:"RunDirectoryApiClient.listRunWorkItems",type:"function"}),
 (resolve:Function {name:"RunDirectoryApiClient.getRunContext",type:"function"}),(compat:Function {name:"RunDirectoryApiClient.getWorktreeRunContext",type:"function"}),
 (record:Function {name:"record",type:"function"}),(text:Function {name:"boundedText",type:"function"}),(rawText:Function {name:"boundedString",type:"function"}),(id:Function {name:"uuid",type:"function"}),
 (nullable:Function {name:"nullableUuid",type:"function"}),(version:Function {name:"positiveVersion",type:"function"}),(boolean:Function {name:"boolean",type:"function"}),
 (projectMap:Function {name:"mapProject",type:"function"}),(branchMap:Function {name:"mapBranch",type:"function"}),(runMap:Function {name:"mapRun",type:"function"}),
 (wtMap:Function {name:"mapWorktree",type:"function"}),(grantMap:Function {name:"mapAuthorization",type:"function"}),(contextMap:Function {name:"mapRunContext",type:"function"}),
 (pageMap:Function {name:"mapPage",type:"function"}),(taskMap:Function {name:"mapRunTaskCard",type:"function"}),(taskBody:Function {name:"mapRunTaskCardBody",type:"function"}),(query:Function {name:"pageQuery",type:"function"}),(origin:Function {name:"trustedBaseUrl",type:"function"}),
 (json:Function {name:"readBoundedJson",type:"function"}),(abort:Function {name:"aborted",type:"function"}),(href:Function {name:"canonicalRunWorktreeHref",type:"function"}),
 (f)-[:CONTAINS]->(error),(error)-[:HAS_METHOD]->(errorCtor),(f)-[:CONTAINS]->(token),(f)-[:CONTAINS]->(stage),(stage)-[:CONTAINS]->(stageCancel),(json)-[:CONTAINS]->(bodyCancel),(stageCancel)-[:CALLS]->(abort),
 (token)-[:CALLS]->(stage),(token)-[:CALLS]->(tokenProvider),(json)-[:CALLS]->(stage),(json)-[:CALLS]->(bodyCancel),(request)-[:CALLS]->(stage),(request)-[:CALLS]->(token),
 (f)-[:CONTAINS]->(timeout),(request)-[:CONTAINS]->(deadline),(request)-[:CONTAINS]->(timedOut),(request)-[:CONTAINS]->(expire),
 (request)-[:CALLS]->(setTimer),(request)-[:CALLS]->(clearTimer),(request)-[:USES]->(deadline),(request)-[:USES]->(timedOut),(request)-[:USES]->(timeout),(expire)-[:USES]->(timedOut),
 (expire)-[:CALLS]->(controllerAbort),(dispose)-[:CALLS]->(controllerAbort),(bodyCancel)-[:CALLS]->(readerCancel),(json)-[:CALLS]->(readerRead),(json)-[:CALLS]->(readerRelease),
 (f)-[:CONTAINS]->(uuidPattern),(f)-[:CONTAINS]->(responseLimit),(f)-[:CONTAINS]->(pageSize),(f)-[:CONTAINS]->(taskPageSize),(f)-[:CONTAINS]->(taskResponseLimit),(id)-[:USES]->(uuidPattern),(json)-[:USES]->(responseLimit),(json)-[:USES]->(taskResponseLimit),(pageMap)-[:USES]->(pageSize),(taskMap)-[:USES]->(taskPageSize),(taskBody)-[:USES]->(taskPageSize),(query)-[:USES]->(pageSize),(tasks)-[:USES]->(taskResponseLimit),(tasks)-[:USES]->(taskPageSize),
 (f)-[:CONTAINS]->(client),(f)-[:CONTAINS]->(project),(f)-[:CONTAINS]->(branch),(f)-[:CONTAINS]->(run),(f)-[:CONTAINS]->(wt),(f)-[:CONTAINS]->(taskCard),(f)-[:CONTAINS]->(taskPage),(f)-[:CONTAINS]->(grant),(f)-[:CONTAINS]->(context),(f)-[:CONTAINS]->(page),(f)-[:CONTAINS]->(options),
 (f)-[:CONTAINS]->(record),(f)-[:CONTAINS]->(text),(f)-[:CONTAINS]->(rawText),(f)-[:CONTAINS]->(id),(f)-[:CONTAINS]->(nullable),(f)-[:CONTAINS]->(version),(f)-[:CONTAINS]->(boolean),(f)-[:CONTAINS]->(projectMap),(f)-[:CONTAINS]->(branchMap),(f)-[:CONTAINS]->(runMap),(f)-[:CONTAINS]->(wtMap),(f)-[:CONTAINS]->(taskMap),(f)-[:CONTAINS]->(taskBody),(f)-[:CONTAINS]->(grantMap),(f)-[:CONTAINS]->(contextMap),(f)-[:CONTAINS]->(pageMap),(f)-[:CONTAINS]->(query),(f)-[:CONTAINS]->(origin),(f)-[:CONTAINS]->(json),(f)-[:CONTAINS]->(abort),(f)-[:CONTAINS]->(href),
 (client)-[:HAS_METHOD]->(ctor),(client)-[:HAS_METHOD]->(request),(client)-[:HAS_METHOD]->(acquire),(client)-[:HAS_METHOD]->(release),(client)-[:HAS_METHOD]->(dispose),(client)-[:HAS_METHOD]->(projects),(client)-[:HAS_METHOD]->(branches),(client)-[:HAS_METHOD]->(runs),(client)-[:HAS_METHOD]->(worktrees),(client)-[:HAS_METHOD]->(tasks),(client)-[:HAS_METHOD]->(resolve),(client)-[:HAS_METHOD]->(compat),
 (ctor)-[:CALLS]->(origin),(request)-[:CALLS]->(acquire),(request)-[:CALLS]->(release),(request)-[:CALLS]->(json),(request)-[:CALLS]->(dispose),(request)-[:CALLS]->(abort),(acquire)-[:CALLS]->(abort),
 (projects)-[:CALLS]->(request),(branches)-[:CALLS]->(request),(runs)-[:CALLS]->(request),(worktrees)-[:CALLS]->(request),(tasks)-[:CALLS]->(request),(tasks)-[:CALLS]->(taskBody),(taskBody)-[:CALLS]->(taskMap),(resolve)-[:CALLS]->(request),(compat)-[:CALLS]->(request),
 (projects)-[:CALLS]->(pageMap),(branches)-[:CALLS]->(pageMap),(runs)-[:CALLS]->(pageMap),(worktrees)-[:CALLS]->(pageMap),(resolve)-[:CALLS]->(contextMap),(compat)-[:CALLS]->(contextMap),
 (pageMap)-[:CALLS]->(record),(pageMap)-[:CALLS]->(text),(branchMap)-[:CALLS]->(record),(branchMap)-[:CALLS]->(id),(runMap)-[:CALLS]->(record),(runMap)-[:CALLS]->(id),(wtMap)-[:CALLS]->(record),(wtMap)-[:CALLS]->(id),(taskMap)-[:CALLS]->(record),(taskMap)-[:CALLS]->(text),(taskMap)-[:CALLS]->(rawText),(taskMap)-[:CALLS]->(id),(taskMap)-[:CALLS]->(nullable),(taskMap)-[:CALLS]->(version),(taskBody)-[:CALLS]->(record),(taskBody)-[:CALLS]->(taskMap),(taskBody)-[:CALLS]->(id),(contextMap)-[:CALLS]->(branchMap),(contextMap)-[:CALLS]->(runMap),(contextMap)-[:CALLS]->(wtMap),(contextMap)-[:CALLS]->(grantMap),(nullable)-[:CALLS]->(id),(href)-[:CALLS]->(id);
*/

export type RunAccessTokenProvider = () => Promise<string | null>;
export interface AuthorizedRunProject { project_id: string; role: string }
export interface CloudBranch {
  branch_id: string; project_id: string; repository_id: string; name: string;
  full_ref: string; head_commit_id: string | null; state: string; version: number;
}
export interface EngineeringRun {
  engineering_run_id: string; project_id: string; repository_id: string; branch_id: string;
  title: string; state: string; owner_user_id: string | null; version: number;
}
export interface RunWorktree {
  worktree_id: string; engineering_run_id: string; project_id: string; repository_id: string;
  workspace_id: string; name: string; branch: string; human_state: string; machine_state: string;
  agent_id: string | null; agent_session_id: string | null; runtime_id: string | null;
  archived: boolean; version: number; binding_id: string; binding_version: number;
  project_binding_id: string; project_binding_version: number;
}
export interface RunAuthorization { binding_id: string; role: string; version: number }
export interface RunContextEnvelope {
  run_context: {
    tenant_id: string; project_id: string; repository_id: string; branch_id: string;
    engineering_run_id: string; actor_id: string; actor_kind: string;
    authorization: { project: RunAuthorization; branch: RunAuthorization; engineering_run: RunAuthorization };
    granted_scopes: string[]; context_version: string; resolved_at: string; correlation_id: string;
  };
  branch: CloudBranch; engineering_run: EngineeringRun; focus: RunWorktree | null; focus_version: string | null;
  capabilities: { run_owned_apps_available: boolean; execution_admission_available: boolean };
}
export interface DirectoryPage<T> { rows: T[]; limit: number; next_cursor: string | null }
export interface RunTaskCard {
  work_item_id: string;
  item_type: string;
  title: string;
  description: string;
  priority: string;
  labels: string[];
  lifecycle: { status: string; review_state: string; active_worktree_id: string | null; version: number; updated_at: string };
  metadata_updated_at: string;
}
export interface RunTaskCardPage { rows: RunTaskCard[]; limit: number; next_after: string | null }
export interface RunDirectoryClientOptions {
  getAccessToken: RunAccessTokenProvider;
  baseUrl?: string;
  /** Supplied by the trusted host, never by route/query input. */
  trustedOrigins?: readonly string[];
  fetcher?: typeof fetch;
  onAuthorizationError?: () => void;
}

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const MAX_RESPONSE_BYTES = 512 * 1024;
export const RUN_DIRECTORY_PAGE_SIZE = 50;
export const RUN_TASK_CARD_PAGE_SIZE = 12;
const MAX_RUN_TASK_CARD_RESPONSE_BYTES = 2 * 1024 * 1024;
export const RUN_DIRECTORY_REQUEST_TIMEOUT_MS = 15_000;

export class RunDirectoryError extends Error {
  constructor(public readonly status: number, public readonly code: string, message: string) {
    super(message);
    this.name = "RunDirectoryError";
  }
}

function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new RunDirectoryError(0, "invalid_projection", "服务端目录响应无效。");
  return value as Record<string, unknown>;
}
function boundedText(value: unknown, max = 256): string {
  if (typeof value !== "string" || !value.trim() || value.length > max || /[\u0000-\u001f\u007f]/.test(value)) {
    throw new RunDirectoryError(0, "invalid_projection", "服务端目录字段无效。");
  }
  return value;
}
function boundedString(value: unknown, max: number): string {
  if (typeof value !== "string" || value.length > max || /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(value)) {
    throw new RunDirectoryError(0, "invalid_projection", "服务端任务字段无效。");
  }
  return value;
}
function uuid(value: unknown): string {
  if (typeof value !== "string" || !UUID.test(value)) throw new RunDirectoryError(0, "invalid_identity", "目录身份无效。");
  return value.toLowerCase();
}
function nullableUuid(value: unknown): string | null { return value === null ? null : uuid(value); }
function positiveVersion(value: unknown): number {
  if (!Number.isSafeInteger(value) || (value as number) < 1) throw new RunDirectoryError(0, "invalid_projection", "目录版本无效。");
  return value as number;
}
function boolean(value: unknown): boolean {
  if (typeof value !== "boolean") throw new RunDirectoryError(0, "invalid_projection", "目录能力字段无效。");
  return value;
}
function mapProject(value: unknown): AuthorizedRunProject {
  const row = record(value);
  return { project_id: uuid(row.project_id), role: boundedText(row.role, 64) };
}
function mapBranch(value: unknown): CloudBranch {
  const row = record(value);
  return {
    branch_id: uuid(row.branch_id), project_id: uuid(row.project_id), repository_id: uuid(row.repository_id),
    name: boundedText(row.name), full_ref: boundedText(row.full_ref, 512),
    head_commit_id: row.head_commit_id === null ? null : boundedText(row.head_commit_id, 128),
    state: boundedText(row.state, 64), version: positiveVersion(row.version),
  };
}
function mapRun(value: unknown): EngineeringRun {
  const row = record(value);
  return {
    engineering_run_id: uuid(row.engineering_run_id), project_id: uuid(row.project_id),
    repository_id: uuid(row.repository_id), branch_id: uuid(row.branch_id), title: boundedText(row.title, 500),
    state: boundedText(row.state, 64), owner_user_id: nullableUuid(row.owner_user_id), version: positiveVersion(row.version),
  };
}
function mapWorktree(value: unknown): RunWorktree {
  const row = record(value);
  return {
    worktree_id: uuid(row.worktree_id), engineering_run_id: uuid(row.engineering_run_id),
    project_id: uuid(row.project_id), repository_id: uuid(row.repository_id), workspace_id: uuid(row.workspace_id),
    name: boundedText(row.name), branch: boundedText(row.branch, 512), human_state: boundedText(row.human_state, 64),
    machine_state: boundedText(row.machine_state, 64), agent_id: nullableUuid(row.agent_id),
    agent_session_id: nullableUuid(row.agent_session_id), runtime_id: nullableUuid(row.runtime_id),
    archived: boolean(row.archived), version: positiveVersion(row.version), binding_id: uuid(row.binding_id),
    binding_version: positiveVersion(row.binding_version), project_binding_id: uuid(row.project_binding_id),
    project_binding_version: positiveVersion(row.project_binding_version),
  };
}
function mapAuthorization(value: unknown): RunAuthorization {
  const row = record(value);
  return { binding_id: uuid(row.binding_id), role: boundedText(row.role, 64), version: positiveVersion(row.version) };
}
function mapRunContext(value: unknown): RunContextEnvelope {
  const envelope = record(value);
  const raw = record(envelope.run_context);
  const authorization = record(raw.authorization);
  const capabilities = record(envelope.capabilities);
  const branch = mapBranch(envelope.branch);
  const run = mapRun(envelope.engineering_run);
  const focus = envelope.focus === null ? null : mapWorktree(envelope.focus);
  if (!Array.isArray(raw.granted_scopes) || raw.granted_scopes.length > 128) throw new RunDirectoryError(0, "invalid_projection", "授权范围响应无效。");
  const context = {
    tenant_id: uuid(raw.tenant_id), project_id: uuid(raw.project_id), repository_id: uuid(raw.repository_id),
    branch_id: uuid(raw.branch_id), engineering_run_id: uuid(raw.engineering_run_id),
    actor_id: uuid(raw.actor_id), actor_kind: boundedText(raw.actor_kind, 32),
    authorization: { project: mapAuthorization(authorization.project), branch: mapAuthorization(authorization.branch), engineering_run: mapAuthorization(authorization.engineering_run) },
    granted_scopes: raw.granted_scopes.map((scope) => boundedText(scope, 128)),
    context_version: boundedText(raw.context_version, 1024), resolved_at: boundedText(raw.resolved_at, 64), correlation_id: uuid(raw.correlation_id),
  };
  if (!Number.isFinite(Date.parse(context.resolved_at)) || context.project_id !== branch.project_id || context.project_id !== run.project_id ||
    context.repository_id !== branch.repository_id || context.repository_id !== run.repository_id || context.branch_id !== branch.branch_id ||
    context.branch_id !== run.branch_id || context.engineering_run_id !== run.engineering_run_id ||
    (focus && (focus.project_id !== context.project_id || focus.repository_id !== context.repository_id || focus.engineering_run_id !== context.engineering_run_id))) {
    throw new RunDirectoryError(0, "scope_mismatch", "RunContext 的 Project、Branch、Run 或 Worktree 关联不一致。");
  }
  const focusVersion = envelope.focus_version === null ? null : boundedText(envelope.focus_version, 1024);
  if ((focus === null) !== (focusVersion === null)) throw new RunDirectoryError(0, "scope_mismatch", "Checkout 焦点版本与 Worktree 不一致。");
  return { run_context: context, branch, engineering_run: run, focus, focus_version: focusVersion, capabilities: {
    run_owned_apps_available: boolean(capabilities.run_owned_apps_available), execution_admission_available: boolean(capabilities.execution_admission_available),
  } };
}
function mapPage<T>(value: unknown, field: string, map: (row: unknown) => T, identity: (row: T) => string): DirectoryPage<T> {
  const envelope = record(value);
  if (!Number.isSafeInteger(envelope.limit) || (envelope.limit as number) < 1 || (envelope.limit as number) > RUN_DIRECTORY_PAGE_SIZE ||
    !Array.isArray(envelope[field]) || (envelope[field] as unknown[]).length > (envelope.limit as number)) {
    throw new RunDirectoryError(0, "invalid_page", "服务端目录页超过请求上限或格式无效。");
  }
  const rows = (envelope[field] as unknown[]).map(map);
  if (new Set(rows.map(identity)).size !== rows.length) throw new RunDirectoryError(0, "duplicate_identity", "服务端目录页包含重复身份。");
  const nextCursor = envelope.next_cursor === null ? null : boundedText(envelope.next_cursor, 1024);
  if (nextCursor !== null && rows.length === 0) throw new RunDirectoryError(0, "invalid_page", "空目录页返回了无效游标。");
  return { rows, limit: envelope.limit as number, next_cursor: nextCursor };
}
function mapRunTaskCard(value: unknown): RunTaskCard {
  const row = record(value);
  const lifecycle = record(row.lifecycle);
  if (!Array.isArray(row.labels) || row.labels.length > 100) throw new RunDirectoryError(0, "invalid_projection", "任务标签列表无效。");
  const labels: string[] = [];
  for (const label of row.labels) labels.push(boundedText(label, 128));
  const task = {
    work_item_id: uuid(row.work_item_id),
    item_type: boundedText(row.item_type, 64),
    title: boundedText(row.title, 500),
    description: boundedString(row.description, 100_000),
    priority: boundedText(row.priority, 32),
    labels,
    lifecycle: {
      status: boundedText(lifecycle.status, 64),
      review_state: boundedText(lifecycle.review_state, 64),
      active_worktree_id: nullableUuid(lifecycle.active_worktree_id),
      version: positiveVersion(lifecycle.version),
      updated_at: boundedText(lifecycle.updated_at, 64),
    },
    metadata_updated_at: boundedText(row.metadata_updated_at, 64),
  };
  if (!Number.isFinite(Date.parse(task.lifecycle.updated_at)) || !Number.isFinite(Date.parse(task.metadata_updated_at))) {
    throw new RunDirectoryError(0, "invalid_projection", "Task Card 时间字段无效。");
  }
  return task;
}
function mapRunTaskCardBody(value: unknown, run: EngineeringRun, after?: string): RunTaskCardPage {
  const envelope = record(value);
  if (uuid(envelope.engineering_run_id) !== run.engineering_run_id || uuid(envelope.project_id) !== run.project_id ||
    uuid(envelope.repository_id) !== run.repository_id || uuid(envelope.branch_id) !== run.branch_id ||
    envelope.limit !== RUN_TASK_CARD_PAGE_SIZE || !Array.isArray(envelope.work_items) || envelope.work_items.length > RUN_TASK_CARD_PAGE_SIZE) {
    throw new RunDirectoryError(0, "scope_mismatch", "Task Card 返回页不属于请求的 Engineering Run 或超过分页上限。");
  }
  const rows: RunTaskCard[] = [];
  for (const item of envelope.work_items) {
    const row = mapRunTaskCard(item);
    if (uuid(record(item).task_card_id) !== row.work_item_id) throw new RunDirectoryError(0, "scope_mismatch", "Task Card 标识与 Work Item 不一致。");
    rows.push(row);
  }
  const identities = new Set<string>();
  for (const row of rows) {
    if (identities.has(row.work_item_id)) throw new RunDirectoryError(0, "duplicate_identity", "Task Card 页面包含重复任务身份。");
    identities.add(row.work_item_id);
  }
  const nextAfter = envelope.next_after === null ? null : uuid(envelope.next_after);
  const lastRow = rows[rows.length - 1];
  if (nextAfter !== null && (rows.length !== RUN_TASK_CARD_PAGE_SIZE || lastRow?.work_item_id !== nextAfter || nextAfter === after)) {
    throw new RunDirectoryError(0, "invalid_page", "Task Card 页面返回了无效游标。");
  }
  return { rows, limit: RUN_TASK_CARD_PAGE_SIZE, next_after: nextAfter };
}
function pageQuery(cursor?: string): string {
  const query = new URLSearchParams({ limit: String(RUN_DIRECTORY_PAGE_SIZE) });
  if (cursor) query.set("cursor", boundedText(cursor, 1024));
  return query.toString();
}
function trustedBaseUrl(baseUrl: string, trustedOrigins: readonly string[]): string {
  const base = baseUrl.trim().replace(/\/+$/, "");
  if (!base) return "";
  if (base.startsWith("/") && !base.startsWith("//") && !/[?#\\]/.test(base)) return base;
  let url: URL;
  try { url = new URL(base); } catch { throw new RunDirectoryError(0, "invalid_origin", "Run Directory API 地址无效。"); }
  const loopback = ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname);
  if ((url.protocol !== "https:" && !(url.protocol === "http:" && loopback)) || url.username || url.password || url.search || url.hash || !trustedOrigins.includes(url.origin)) {
    throw new RunDirectoryError(0, "untrusted_origin", "Run Directory API 必须使用宿主授权的地址。");
  }
  return `${url.origin}${url.pathname.replace(/\/+$/, "")}`;
}
async function readBoundedJson(response: Response, signal: AbortSignal, maxBytes = MAX_RESPONSE_BYTES): Promise<unknown> {
  if (!response.body) throw new RunDirectoryError(0, "invalid_projection", "目录响应为空。");
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let bytes = 0;
  let text = "";
  let complete = false;
  const cancel = function cancelResponseBody() { void reader.cancel().catch(() => undefined); };
  signal.addEventListener("abort", cancel, { once: true });
  try {
    while (true) {
      const chunk = await awaitAbortable(() => reader.read(), signal);
      if (chunk.done) break;
      bytes += chunk.value.byteLength;
      if (bytes > maxBytes) throw new RunDirectoryError(0, "response_limit", "目录响应超过资源上限。");
      text += decoder.decode(chunk.value, { stream: true });
    }
    complete = true;
    return JSON.parse(text + decoder.decode()) as unknown;
  } finally {
    signal.removeEventListener("abort", cancel);
    if (!complete) cancel();
    reader.releaseLock();
  }
}
function aborted(): DOMException { return new DOMException("Run Directory request cancelled", "AbortError"); }
/** Release the request slot even if a trusted host operation ignores AbortSignal. */
function awaitAbortable<T>(operation: () => Promise<T>, signal: AbortSignal): Promise<T> {
  if (signal.aborted) return Promise.reject(aborted());
  return new Promise((resolve, reject) => {
    const cancel = function cancelRequestStage() { signal.removeEventListener("abort", cancel); reject(aborted()); };
    signal.addEventListener("abort", cancel, { once: true });
    Promise.resolve().then(() => {
      if (signal.aborted) throw aborted();
      return operation();
    }).then((result) => {
      signal.removeEventListener("abort", cancel);
      if (signal.aborted) reject(aborted()); else resolve(result);
    }, (error: unknown) => { signal.removeEventListener("abort", cancel); reject(error); });
  });
}
function readAccessToken(provider: RunAccessTokenProvider, signal: AbortSignal): Promise<string | null> { return awaitAbortable(provider, signal); }

export class RunDirectoryApiClient {
  private readonly baseUrl: string;
  private readonly lifetime = new AbortController();
  private running = 0;
  private readonly waiters: Array<{ signal: AbortSignal; start: () => void; cancel: () => void }> = [];
  constructor(private readonly options: RunDirectoryClientOptions) {
    this.baseUrl = trustedBaseUrl(options.baseUrl ?? "", options.trustedOrigins ?? []);
  }
  dispose(): void { this.lifetime.abort(); }
  private acquire(signal: AbortSignal): Promise<void> {
    if (signal.aborted) return Promise.reject(aborted());
    if (this.running < 2) { this.running += 1; return Promise.resolve(); }
    if (this.waiters.length >= 16) return Promise.reject(new RunDirectoryError(0, "queue_limit", "目录请求队列已满，请稍后重试。"));
    return new Promise((resolve, reject) => {
      const waiter = {
        signal,
        start: () => { signal.removeEventListener("abort", waiter.cancel); this.running += 1; resolve(); },
        cancel: () => { const index = this.waiters.indexOf(waiter); if (index >= 0) this.waiters.splice(index, 1); reject(aborted()); },
      };
      this.waiters.push(waiter);
      signal.addEventListener("abort", waiter.cancel, { once: true });
    });
  }
  private release(): void {
    this.running -= 1;
    const next = this.waiters.shift();
    if (next) next.start();
  }
  private async request(path: string, signal?: AbortSignal, maxResponseBytes = MAX_RESPONSE_BYTES): Promise<unknown> {
    const controller = new AbortController();
    const cancel = () => controller.abort();
    const signals = signal ? [signal, this.lifetime.signal] : [this.lifetime.signal];
    for (const source of signals) { source.addEventListener("abort", cancel, { once: true }); if (source.aborted) cancel(); }
    let timedOut = false;
    const deadlineTimer = setTimeout(function expireRunDirectoryRequest() {
      if (!controller.signal.aborted) { timedOut = true; controller.abort(); }
    }, RUN_DIRECTORY_REQUEST_TIMEOUT_MS);
    let acquired = false;
    try {
      await this.acquire(controller.signal);
      acquired = true;
      const token = (await readAccessToken(this.options.getAccessToken, controller.signal))?.trim();
      if (controller.signal.aborted) throw aborted();
      if (!token) {
        this.dispose(); this.options.onAuthorizationError?.();
        throw new RunDirectoryError(401, "session_required", "需要宿主提供已认证的用户会话。");
      }
      const response = await awaitAbortable(() => (this.options.fetcher ?? fetch)(`${this.baseUrl}${path}`, {
        headers: { Accept: "application/json", Authorization: `Bearer ${token}` },
        credentials: "omit", cache: "no-store", redirect: "error", signal: controller.signal,
      }), controller.signal);
      if (response.status === 401 || response.status === 403) {
        this.dispose(); this.options.onAuthorizationError?.();
        throw new RunDirectoryError(response.status, "authorization_failed", "会话或授权已失效，请重新连接宿主会话。");
      }
      if (!response.ok) throw new RunDirectoryError(response.status, "directory_unavailable", "授权目录暂时不可用，请重试。");
      const result = await readBoundedJson(response, controller.signal, maxResponseBytes);
      if (controller.signal.aborted) throw aborted();
      return result;
    } catch (error) {
      if (timedOut) throw new RunDirectoryError(0, "request_timeout", "目录请求超过 15 秒，请重试。");
      throw error;
    } finally {
      clearTimeout(deadlineTimer);
      for (const source of signals) source.removeEventListener("abort", cancel);
      if (acquired) this.release();
    }
  }
  async listProjects(cursor?: string, signal?: AbortSignal): Promise<DirectoryPage<AuthorizedRunProject>> {
    return mapPage(await this.request(`/api/v1/projects?${pageQuery(cursor)}`, signal), "projects", mapProject, (row) => row.project_id);
  }
  async listBranches(projectId: string, cursor?: string, signal?: AbortSignal): Promise<DirectoryPage<CloudBranch>> {
    const id = uuid(projectId);
    const result = record(await this.request(`/api/v1/projects/${id}/branches?${pageQuery(cursor)}`, signal));
    const page = mapPage(result, "branches", mapBranch, (row) => row.branch_id);
    if (uuid(result.project_id) !== id || page.rows.some((row) => row.project_id !== id)) throw new RunDirectoryError(0, "scope_mismatch", "Branch 目录不属于当前 Project。");
    return page;
  }
  async listRuns(branch: CloudBranch, cursor?: string, signal?: AbortSignal): Promise<DirectoryPage<EngineeringRun>> {
    const result = record(await this.request(`/api/v1/branches/${uuid(branch.branch_id)}/engineering-runs?${pageQuery(cursor)}`, signal));
    const page = mapPage(result, "engineering_runs", mapRun, (row) => row.engineering_run_id);
    if (uuid(result.project_id) !== branch.project_id || uuid(result.branch_id) !== branch.branch_id || page.rows.some((row) => row.project_id !== branch.project_id || row.branch_id !== branch.branch_id || row.repository_id !== branch.repository_id)) {
      throw new RunDirectoryError(0, "scope_mismatch", "Engineering Run 目录不属于当前 Branch。");
    }
    return page;
  }
  async listWorktrees(run: EngineeringRun, cursor?: string, signal?: AbortSignal): Promise<DirectoryPage<RunWorktree>> {
    const result = record(await this.request(`/api/v1/engineering-runs/${uuid(run.engineering_run_id)}/worktrees?${pageQuery(cursor)}`, signal));
    const page = mapPage(result, "worktrees", mapWorktree, (row) => row.worktree_id);
    if (uuid(result.project_id) !== run.project_id || uuid(result.branch_id) !== run.branch_id || uuid(result.engineering_run_id) !== run.engineering_run_id || page.rows.some((row) => row.project_id !== run.project_id || row.engineering_run_id !== run.engineering_run_id || row.repository_id !== run.repository_id)) {
      throw new RunDirectoryError(0, "scope_mismatch", "Worktree 目录不属于当前 Engineering Run。");
    }
    return page;
  }
  async listRunWorkItems(run: EngineeringRun, after?: string, signal?: AbortSignal): Promise<RunTaskCardPage> {
    const query = new URLSearchParams({ limit: String(RUN_TASK_CARD_PAGE_SIZE) });
    const cursor = after ? uuid(after) : undefined;
    if (cursor) query.set("after", cursor);
    return mapRunTaskCardBody(
      await this.request(
        `/api/v1/engineering-runs/${uuid(run.engineering_run_id)}/work-items?${query}`,
        signal,
        MAX_RUN_TASK_CARD_RESPONSE_BYTES,
      ),
      run,
      cursor,
    );
  }
  async getRunContext(runId: string, focusWorktreeId?: string, signal?: AbortSignal): Promise<RunContextEnvelope> {
    const run = uuid(runId);
    const focus = focusWorktreeId ? uuid(focusWorktreeId) : undefined;
    const query = focus ? `?${new URLSearchParams({ focus_worktree_id: focus })}` : "";
    const result = mapRunContext(await this.request(`/api/v1/engineering-runs/${run}/context${query}`, signal));
    if (result.run_context.engineering_run_id !== run || (focus ? result.focus?.worktree_id !== focus : result.focus !== null)) {
      throw new RunDirectoryError(0, "scope_mismatch", "RunContext 不属于请求的 Run 或 Worktree。");
    }
    return result;
  }
  async getWorktreeRunContext(worktreeId: string, signal?: AbortSignal): Promise<RunContextEnvelope> {
    const id = uuid(worktreeId);
    const result = mapRunContext(await this.request(`/api/v1/worktrees/${id}/run-context`, signal));
    if (result.focus?.worktree_id !== id) throw new RunDirectoryError(0, "scope_mismatch", "兼容目录返回了其他 Worktree。");
    return result;
  }
}

export function canonicalRunWorktreeHref(projectId: string, branchId: string, runId: string, worktreeId: string): string {
  return `/projects/${uuid(projectId)}/branches/${uuid(branchId)}/runs/${uuid(runId)}/worktrees/${uuid(worktreeId)}`;
}
