/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts",type:"file",language:"typescript"}),
  (token:Class {name:"GroupAccessTokenProvider",type:"class",signature:"() => Promise<string | null>",visibility:"public"}),
  (cursor:Class {name:"CanvasEventCursor",type:"class",signature:"{ cursor_at: string; cursor_event_id: string }",visibility:"public"}),
  (event:Class {name:"CanvasOutboxEventMetadata",type:"class",visibility:"public"}),
  (eventPage:Class {name:"CanvasOutboxEventPage",type:"class",visibility:"public"}),
  (indexQuery:Class {name:"WorktreeIndexQuery",type:"class",visibility:"public"}),
  (projectsQuery:Class {name:"AuthorizedProjectsQuery",type:"interface",visibility:"public"}),
  (authorizedProject:Class {name:"AuthorizedProjectAccess",type:"interface",visibility:"public"}),
  (repository:Class {name:"ProjectWorktreeRepository",type:"interface",visibility:"public"}),
  (candidate:Class {name:"WorktreeImportCandidate",type:"interface",visibility:"public"}),
  (lifecycleReceipt:Class {name:"ProjectWorktreeLifecycleReceipt",type:"interface",visibility:"public"}),
  (error:Class {name:"GroupApiError",type:"class",visibility:"public"}),
  (errorCtor:Function {name:"GroupApiError.constructor",type:"function",signature:"constructor(status: number, code: string, message: string)",visibility:"public",complexity:"simple"}),
  (client:Class {name:"WorktreeGroupApiClient",type:"class",visibility:"public"}),
  (ctor:Function {name:"WorktreeGroupApiClient.constructor",type:"function",signature:"constructor(getAccessToken: GroupAccessTokenProvider, fetcher?: typeof fetch, baseUrl?: string)",visibility:"public",complexity:"simple"}),
  (request:Function {name:"WorktreeGroupApiClient.request",type:"function",signature:"request<T>(path: string, init?: RequestInit): Promise<T>",visibility:"private",complexity:"moderate"}),
  (groupContext:Function {name:"WorktreeGroupApiClient.getGroupContext",type:"function",signature:"getGroupContext<T>(worktreeId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (index:Function {name:"WorktreeGroupApiClient.listProjectWorktrees",type:"function",signature:"listProjectWorktrees<T>(projectId: string, query?: WorktreeIndexQuery): Promise<T>",visibility:"public",complexity:"moderate"}),
  (listAuthorizedProjects:Function {name:"WorktreeGroupApiClient.listAuthorizedProjects",type:"function",signature:"listAuthorizedProjects<T>(query?: AuthorizedProjectsQuery): Promise<T>",visibility:"public",complexity:"moderate"}),
  (projectRepositories:Function {name:"WorktreeGroupApiClient.listProjectWorktreeRepositories",type:"function",signature:"listProjectWorktreeRepositories<T>(projectId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (importCandidates:Function {name:"WorktreeGroupApiClient.listWorktreeImportCandidates",type:"function",signature:"listWorktreeImportCandidates<T>(projectId: string, repositoryId: string, limit?: number): Promise<T>",visibility:"public",complexity:"simple"}),
  (createWorktree:Function {name:"WorktreeGroupApiClient.createProjectWorktree",type:"function",signature:"createProjectWorktree<T>(projectId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (importWorktree:Function {name:"WorktreeGroupApiClient.importProjectWorktree",type:"function",signature:"importProjectWorktree<T>(projectId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (projectMembers:Function {name:"WorktreeGroupApiClient.listProjectMembers",type:"function",signature:"listProjectMembers<T>(projectId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (managementPlan:Function {name:"WorktreeGroupApiClient.createManagementPlan",type:"function",signature:"createManagementPlan<T>(worktreeId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (managementConfirm:Function {name:"WorktreeGroupApiClient.confirmManagementPlan",type:"function",signature:"confirmManagementPlan<T>(worktreeId: string, planId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (workItems:Function {name:"WorktreeGroupApiClient.listWorkItems",type:"function",signature:"listWorkItems<T>(worktreeId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (transitionWorkItem:Function {name:"WorktreeGroupApiClient.transitionWorkItem",type:"function",signature:"transitionWorkItem<T>(worktreeId: string, workItemId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (canvases:Function {name:"WorktreeGroupApiClient.listCanvases",type:"function",signature:"listCanvases<T>(worktreeId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (createCanvas:Function {name:"WorktreeGroupApiClient.createCanvas",type:"function",signature:"createCanvas<T>(worktreeId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (createCanvasElement:Function {name:"WorktreeGroupApiClient.createCanvasElement",type:"function",signature:"createCanvasElement<T>(worktreeId: string, canvasId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (updateCanvasElement:Function {name:"WorktreeGroupApiClient.updateCanvasElement",type:"function",signature:"updateCanvasElement<T>(worktreeId: string, canvasId: string, elementId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (deleteCanvasElement:Function {name:"WorktreeGroupApiClient.deleteCanvasElement",type:"function",signature:"deleteCanvasElement<T>(worktreeId: string, canvasId: string, elementId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (updateCanvasDocument:Function {name:"WorktreeGroupApiClient.updateCanvasDocument",type:"function",signature:"updateCanvasDocument<T>(worktreeId: string, canvasId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (elements:Function {name:"WorktreeGroupApiClient.listCanvasElements",type:"function",signature:"listCanvasElements<T>(worktreeId: string, canvasId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (events:Function {name:"WorktreeGroupApiClient.listCanvasEvents",type:"function",signature:"listCanvasEvents<T>(worktreeId: string, canvasId: string, cursor?: CanvasEventCursor, limit?: number): Promise<T>",visibility:"public",complexity:"moderate"}),
  (createItem:Function {name:"WorktreeGroupApiClient.createWorkItemOnCanvas",type:"function",signature:"createWorkItemOnCanvas<T>(worktreeId: string, canvasId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (poller:Class {name:"CanvasOutboxPoller",type:"class",visibility:"public"}),
  (pollerCtor:Function {name:"CanvasOutboxPoller.constructor",type:"function",signature:"constructor(client, worktreeId, canvasId, refreshProjection, options)",visibility:"public",complexity:"simple"}),
  (pollOnce:Function {name:"CanvasOutboxPoller.pollOnce",type:"function",signature:"pollOnce(): Promise<number>",visibility:"public",complexity:"moderate"}),
  (runPoller:Function {name:"CanvasOutboxPoller.run",type:"function",signature:"run(signal: AbortSignal, onError?): Promise<void>",visibility:"public",complexity:"moderate"}),
  (readError:Function {name:"readApiError",type:"function",signature:"readApiError(response: Response): Promise<GroupApiError>",visibility:"private",complexity:"simple"}),
  (normalizeBaseUrl:Function {name:"normalizeBaseUrl",type:"function",signature:"normalizeBaseUrl(baseUrl: string): string",visibility:"private",complexity:"moderate"}),
  (waitPoll:Function {name:"waitForPoll",type:"function",signature:"waitForPoll(ms: number, signal: AbortSignal): Promise<void>",visibility:"private",complexity:"simple"}),
  (file)-[:CONTAINS]->(token),(file)-[:CONTAINS]->(cursor),(file)-[:CONTAINS]->(event),(file)-[:CONTAINS]->(eventPage),(file)-[:CONTAINS]->(indexQuery),(file)-[:CONTAINS]->(projectsQuery),(file)-[:CONTAINS]->(authorizedProject),(file)-[:CONTAINS]->(repository),(file)-[:CONTAINS]->(candidate),(file)-[:CONTAINS]->(lifecycleReceipt),(file)-[:CONTAINS]->(error),(file)-[:CONTAINS]->(client),(file)-[:CONTAINS]->(projectMembers),(file)-[:CONTAINS]->(poller),(file)-[:CONTAINS]->(readError),(file)-[:CONTAINS]->(waitPoll),
  (error)-[:HAS_METHOD]->(errorCtor),
  (client)-[:HAS_METHOD]->(ctor),(client)-[:HAS_METHOD]->(request),(client)-[:HAS_METHOD]->(groupContext),(client)-[:HAS_METHOD]->(index),(client)-[:HAS_METHOD]->(listAuthorizedProjects),(client)-[:HAS_METHOD]->(projectRepositories),(client)-[:HAS_METHOD]->(importCandidates),(client)-[:HAS_METHOD]->(createWorktree),(client)-[:HAS_METHOD]->(importWorktree),(client)-[:HAS_METHOD]->(projectMembers),(client)-[:HAS_METHOD]->(managementPlan),(client)-[:HAS_METHOD]->(managementConfirm),(client)-[:HAS_METHOD]->(workItems),(client)-[:HAS_METHOD]->(transitionWorkItem),(client)-[:HAS_METHOD]->(canvases),(client)-[:HAS_METHOD]->(createCanvas),(client)-[:HAS_METHOD]->(createCanvasElement),(client)-[:HAS_METHOD]->(updateCanvasElement),(client)-[:HAS_METHOD]->(deleteCanvasElement),(client)-[:HAS_METHOD]->(updateCanvasDocument),(client)-[:HAS_METHOD]->(elements),(client)-[:HAS_METHOD]->(events),(client)-[:HAS_METHOD]->(createItem),
  (poller)-[:HAS_METHOD]->(pollerCtor),(poller)-[:HAS_METHOD]->(pollOnce),(poller)-[:HAS_METHOD]->(runPoller),
  (ctor)-[:CALLS]->(normalizeBaseUrl),(request)-[:CALLS]->(readError),(groupContext)-[:CALLS]->(request),(index)-[:CALLS]->(request),(index)-[:USES]->(indexQuery),(listAuthorizedProjects)-[:CALLS]->(request),(listAuthorizedProjects)-[:USES]->(projectsQuery),(projectRepositories)-[:CALLS]->(request),(importCandidates)-[:CALLS]->(request),(createWorktree)-[:CALLS]->(request),(importWorktree)-[:CALLS]->(request),(projectMembers)-[:CALLS]->(request),(managementPlan)-[:CALLS]->(request),(managementConfirm)-[:CALLS]->(request),(workItems)-[:CALLS]->(request),(transitionWorkItem)-[:CALLS]->(request),(canvases)-[:CALLS]->(request),(createCanvas)-[:CALLS]->(request),(createCanvasElement)-[:CALLS]->(request),(updateCanvasElement)-[:CALLS]->(request),(updateCanvasDocument)-[:CALLS]->(request),(elements)-[:CALLS]->(request),(events)-[:CALLS]->(request),(createItem)-[:CALLS]->(request),(pollerCtor)-[:USES]->(event),(pollerCtor)-[:USES]->(eventPage),(pollOnce)-[:CALLS]->(events),(runPoller)-[:CALLS]->(pollOnce),(runPoller)-[:CALLS]->(waitPoll);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"}),
      (client:Class {name:"WorktreeGroupApiClient"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (groupApp:Class {name:"GroupAppNavigationEntry",type:"interface",language:"typescript",visibility:"public"}),
       (groupAppProjection:Class {name:"GroupAppRegistryProjection",type:"interface",language:"typescript",visibility:"public"}),
       (listGroupApps:Function {name:"WorktreeGroupApiClient.listGroupApps",type:"function",language:"typescript",visibility:"public",complexity:"simple"});
CREATE (file)-[:CONTAINS]->(groupApp),(file)-[:CONTAINS]->(groupAppProjection),
       (client)-[:HAS_METHOD]->(listGroupApps),(listGroupApps)-[:CALLS]->(request),
       (groupAppProjection)-[:CONTAINS]->(groupApp);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (deleteElement:Function {name:"WorktreeGroupApiClient.deleteCanvasElement"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (deleteElement)-[:CALLS]->(request);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (review:Function {name:"WorktreeGroupApiClient.reviewWorkItem"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (review)-[:CALLS]->(request);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"}),
      (client:Class {name:"WorktreeGroupApiClient"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (startCli:Function {name:"WorktreeGroupApiClient.startTaskCliSession",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (receipt:Class {name:"TaskCliSessionReceipt",type:"interface",language:"typescript",visibility:"public"});
CREATE (file)-[:CONTAINS]->(startCli),
       (file)-[:CONTAINS]->(receipt),
       (client)-[:HAS_METHOD]->(startCli),
       (startCli)-[:CALLS]->(request);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (poller:Class {name:"CanvasOutboxPoller"}),
      (cursorStore:Class {name:"BrowserCanvasEventCursorStore"}),
      (loadCursor:Function {name:"BrowserCanvasEventCursorStore.load"}),
      (saveCursor:Function {name:"BrowserCanvasEventCursorStore.save"}),
      (validCursor:Function {name:"isCanvasEventCursor"});
CREATE (poller)-[:USES]->(cursorStore),
       (cursorStore)-[:HAS_METHOD]->(loadCursor),
       (cursorStore)-[:HAS_METHOD]->(saveCursor),
       (loadCursor)-[:CALLS]->(validCursor);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"}),
      (client:Class {name:"WorktreeGroupApiClient"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (chatScope:Class {name:"ScopedChatScope",type:"class",language:"typescript",visibility:"public"}),
       (chatRef:Class {name:"ScopedChatEntityRef",type:"interface",language:"typescript",visibility:"public"}),
       (chatBody:Class {name:"ScopedChatMessageBody",type:"interface",language:"typescript",visibility:"public"}),
       (chatReceipt:Class {name:"ScopedChatReceipt",type:"interface",language:"typescript",visibility:"public"}),
       (submitChat:Function {name:"WorktreeGroupApiClient.submitScopedChatMessage",type:"function",language:"typescript",visibility:"public",complexity:"simple"});
CREATE (file)-[:CONTAINS]->(chatScope),(file)-[:CONTAINS]->(chatRef),(file)-[:CONTAINS]->(chatBody),(file)-[:CONTAINS]->(chatReceipt),
       (client)-[:HAS_METHOD]->(submitChat),(submitChat)-[:CALLS]->(request);
 */

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"}),
      (client:Class {name:"WorktreeGroupApiClient"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (cliState:Class {name:"TaskCliSessionState",type:"class",language:"typescript",visibility:"public"}),
       (cliStatus:Class {name:"TaskCliSessionStatus",type:"interface",language:"typescript",visibility:"public"}),
       (cliAttachment:Class {name:"TaskCliSessionAttachmentReceipt",type:"class",language:"typescript",visibility:"public"}),
       (cliPage:Class {name:"TaskCliSessionPage",type:"class",language:"typescript",visibility:"public"}),
       (listCliSessions:Function {name:"WorktreeGroupApiClient.listTaskCliSessions",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (getCliStatus:Function {name:"WorktreeGroupApiClient.getTaskCliSessionStatus",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (cancelCli:Function {name:"WorktreeGroupApiClient.cancelTaskCliSession",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (reattachCli:Function {name:"WorktreeGroupApiClient.reattachTaskCliSession",type:"function",language:"typescript",visibility:"public",complexity:"simple"});
CREATE (file)-[:CONTAINS]->(cliState),(file)-[:CONTAINS]->(cliStatus),(file)-[:CONTAINS]->(cliAttachment),(file)-[:CONTAINS]->(cliPage),
       (client)-[:HAS_METHOD]->(getCliStatus),(client)-[:HAS_METHOD]->(cancelCli),(client)-[:HAS_METHOD]->(reattachCli),(client)-[:HAS_METHOD]->(listCliSessions),
       (getCliStatus)-[:CALLS]->(request),(cancelCli)-[:CALLS]->(request),(reattachCli)-[:CALLS]->(request),(listCliSessions)-[:CALLS]->(request);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"}),
      (client:Class {name:"WorktreeGroupApiClient"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (chatTarget:Class {name:"ScopedChatTarget",type:"interface",language:"typescript",visibility:"public"}),
       (chatTargetPage:Class {name:"ScopedChatTargetPage",type:"interface",language:"typescript",visibility:"public"}),
       (listChatTargets:Function {name:"WorktreeGroupApiClient.listGlobalChatTargets",type:"function",language:"typescript",visibility:"public",complexity:"simple"});
CREATE (file)-[:CONTAINS]->(chatTarget),(file)-[:CONTAINS]->(chatTargetPage),
       (client)-[:HAS_METHOD]->(listChatTargets),(listChatTargets)-[:CALLS]->(request);
*/

export type GroupAccessTokenProvider = () => Promise<string | null>;

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"});
CREATE (gitLockState:Class {name:"GitLockState",type:"class",language:"typescript",visibility:"public"}),
       (gitLockView:Class {name:"WorktreeGitLockView",type:"class",language:"typescript",visibility:"public"}),
       (normalizeGitLock:Function {name:"normalizeWorktreeGitLock",type:"function",language:"typescript",visibility:"public",complexity:"moderate"});
CREATE (file)-[:CONTAINS]->(gitLockState),(file)-[:CONTAINS]->(gitLockView),(file)-[:CONTAINS]->(normalizeGitLock);
*/

export type GitLockState = "locked" | "unlocked" | "unknown";

export interface WorktreeGitLockView {
  state: GitLockState;
  observedAt: string | null;
}

export function normalizeWorktreeGitLock(value: unknown, nowMs = Date.now()): WorktreeGitLockView {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return { state: "unknown", observedAt: null };
  }

  const observation = value as Record<string, unknown>;
  const observedAt = typeof observation.observed_at === "string"
    && Number.isFinite(Date.parse(observation.observed_at))
    ? observation.observed_at
    : null;
  if (observation.source !== "host_runtime" || observedAt === null) {
    return { state: "unknown", observedAt };
  }

  const ageMs = nowMs - Date.parse(observedAt);
  if (ageMs < 0 || ageMs > 30_000) return { state: "unknown", observedAt };
  if (observation.state !== "locked" && observation.state !== "unlocked") {
    return { state: "unknown", observedAt };
  }

  return { state: observation.state, observedAt };
}

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"}),
      (client:Class {name:"WorktreeGroupApiClient"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (runSummary:Class {name:"TaskRunSummary",type:"interface",language:"typescript",visibility:"public"}),
       (runPage:Class {name:"TaskRunPage",type:"interface",language:"typescript",visibility:"public"}),
       (runEvent:Class {name:"TaskRunEvent",type:"interface",language:"typescript",visibility:"public"}),
       (runEvidence:Class {name:"TaskRunEvidence",type:"interface",language:"typescript",visibility:"public"}),
       (runDetail:Class {name:"TaskRunDetail",type:"interface",language:"typescript",visibility:"public"}),
       (listRuns:Function {name:"WorktreeGroupApiClient.listTaskRuns",type:"function",language:"typescript",visibility:"public",complexity:"moderate"}),
       (getRun:Function {name:"WorktreeGroupApiClient.getTaskRunDetail",type:"function",language:"typescript",visibility:"public",complexity:"simple"});
CREATE (file)-[:CONTAINS]->(runSummary),(file)-[:CONTAINS]->(runPage),(file)-[:CONTAINS]->(runEvent),(file)-[:CONTAINS]->(runEvidence),(file)-[:CONTAINS]->(runDetail),
       (client)-[:HAS_METHOD]->(listRuns),(client)-[:HAS_METHOD]->(getRun),(listRuns)-[:CALLS]->(request),(getRun)-[:CALLS]->(request);
*/

export type ScopedChatScope = "WORKTREE" | "GLOBAL";

export interface ScopedChatEntityRef {
  ref_type: string;
  ref_id: string;
  worktree_id: string;
}

export interface ScopedChatMessageBody {
  scope: ScopedChatScope;
  target_worktree_ids?: string[];
  session_id?: string | null;
  message: string;
  entity_refs?: ScopedChatEntityRef[];
  correlation_id?: string;
}

export interface ScopedChatReceipt {
  session_id: string;
  run_id: string;
  scope: ScopedChatScope;
  target_worktree_ids: string[];
  correlation_id: string;
  status: "accepted";
}

export interface ScopedChatTarget {
  worktree_id: string;
  project_id: string;
  name: string;
}

export interface ScopedChatTargetPage {
  targets: ScopedChatTarget[];
  limit: number;
  next_cursor: string | null;
}

export interface GroupAppNavigationEntry {
  plugin_id: string;
  manifest_version: string;
  label: string;
  sort_order: number;
}

export interface GroupAppRegistryProjection {
  worktree_id: string;
  registry_version: number;
  correlation_id: string;
  apps: GroupAppNavigationEntry[];
}

export interface CanvasEventCursor {
  cursor_at: string;
  cursor_event_id: string;
}

export interface CanvasOutboxEventMetadata {
  event_id: string;
  occurred_at: string;
}

export interface CanvasOutboxEventPage {
  events: CanvasOutboxEventMetadata[];
  next_cursor: CanvasEventCursor | null;
}

export interface WorktreeIndexQuery {
  limit?: number;
  cursor?: string;
  owner_user_id?: string;
  human_state?: string;
  include_archived?: boolean;
}

export interface AuthorizedProjectsQuery {
  limit?: number;
  cursor?: string;
}

export type HookDecision = "Allow" | "Deny" | "RequireHuman" | "Defer";
export type HookReasonCode =
  | "AllowedByBuiltinBaseline"
  | "IncompleteScope"
  | "EventSchemaUnsupported"
  | "ActorNotAuthorized"
  | "LifecycleVersionStale"
  | "RuntimeUnhealthyOrUnknown"
  | "RetentionLockUnusable"
  | "ExecutionNotDrained"
  | "PolicyUnavailable"
  | "PolicyInvalid"
  | "RuleDenied"
  | "HumanApprovalRequired"
  | "ExternalConditionPending";
export type HookFactField =
  | "ActorAuthorized"
  | "LifecycleVersionMatches"
  | "RuntimeHealthy"
  | "RetentionLock"
  | "ActiveRunCount"
  | "ActiveAgentLeaseCount"
  | "FileClaimCount"
  | "OwnedProcessCount";
export type HookOperator =
  | "Equal"
  | "NotEqual"
  | "GreaterThan"
  | "GreaterThanOrEqual"
  | "LessThan"
  | "LessThanOrEqual";
export type HookRetentionLockState = "Fresh" | "Missing" | "Stale" | "Conflict" | "Unknown";
export type HookValue =
  | { Boolean: boolean }
  | { Count: number }
  | { RetentionLock: HookRetentionLockState };

export interface HookCondition {
  field: HookFactField;
  operator: HookOperator;
  expected: HookValue;
}

export interface HookRule {
  rule_id: number[];
  priority: number;
  enabled: boolean;
  decision: Exclude<HookDecision, "Allow">;
  reason_code: Extract<HookReasonCode, "RuleDenied" | "HumanApprovalRequired" | "ExternalConditionPending">;
  conditions: HookCondition[];
}

export interface HookPolicyDocument {
  schema_version: number;
  evaluator_api_version: number;
  tenant_id: number[];
  project_id: number[];
  worktree_id: number[] | null;
  project_version: number;
  worktree_version: number | null;
  digest: number[];
  project_rules: HookRule[];
  worktree_rules: HookRule[];
}

export interface HookPolicyAuditEvent {
  event_id: string;
  event_type: string;
  policy_set_id: string | null;
  draft_id: string | null;
  policy_version: number | null;
  correlation_id: string;
  details: unknown;
  occurred_at: string;
}

export interface HookPolicyDraftView {
  draft_id: string;
  draft_version: number;
  base_policy_set_id: string | null;
  inherited_project_policy_set_id: string | null;
  expires_at: string;
  is_stale: boolean;
  policy_document: HookPolicyDocument;
}

export interface HookPolicyResponse {
  scope: { kind: "project" | "worktree"; project_id: string; worktree_id: string | null };
  policy_set_id: string | null;
  policy_document: HookPolicyDocument | null;
  effective_policy_document: HookPolicyDocument | null;
  inherited_project_policy_set_id: string | null;
  draft: HookPolicyDraftView | null;
  audit: HookPolicyAuditEvent[];
}

export interface HookPolicyDraftBody {
  expected_draft_version: number;
  expected_current_policy_set_id: string | null;
  correlation_id: string;
  policy_document: HookPolicyDocument;
}

export interface AuthorizedProjectAccess {
  project_id: string;
  role: string;
}

export interface ProjectWorktreeRepository {
  repository_id: string;
  name: string;
  default_branch: string;
}

export interface WorktreeImportCandidate {
  candidate_id: string;
  repository_id: string;
  name: string;
  branch: string;
  head_commit: string;
  dirty: boolean;
  observed_at: string;
}

export interface ProjectWorktreeLifecycleReceipt {
  operation_id: string;
  worktree_id: string;
  project_id: string;
  repository_id: string;
  branch: string;
  state: "provisioning" | "ready";
  accepted_at: string;
  correlation_id: string;
}

export interface TaskCliSessionReceipt {
  session_id: string;
  worktree_id: string;
  work_item_id: string;
  runtime_id: string;
  status: "running";
  attachment_ticket: string;
  attachment_ticket_expires_at: string;
  correlation_id: string;
}

export type TaskCliSessionState =
  | "starting"
  | "running"
  | "disconnected"
  | "cancelling"
  | "completed"
  | "failed"
  | "cancelled"
  | "timed_out"
  | "lost";

export interface TaskCliSessionStatus {
  session_id: string;
  state: TaskCliSessionState;
  exit_code: number | null;
  updated_at: string;
}

export interface TaskCliSessionPage {
  sessions: TaskCliSessionStatus[];
}

export interface TaskRunSummary {
  run_id: string;
  worktree_id: string | null;
  repository_id: string | null;
  work_item_id: string;
  initiated_by: string;
  execution_channel: string;
  run_origin: string;
  start_ref: string | null;
  start_commit_ref: string | null;
  task_contract_version: number | null;
  agent_id: string | null;
  model_version: string | null;
  skill_version: string | null;
  orchestrator_version: string | null;
  strategy_version: string | null;
  execution_profile_id: string | null;
  execution_profile_version: number | null;
  execution_profile_digest: string | null;
  execution_state: string | null;
  verification_state: string | null;
  human_acceptance_state: string | null;
  started_at: string;
  correlation_id: string;
}

export interface TaskRunPage {
  project_id: string;
  repository_id: string;
  worktree_id: string;
  work_item_id: string;
  permission_snapshot_ref: string;
  limit: number;
  runs: TaskRunSummary[];
  next_cursor: { cursor_started_at: string; cursor_run_id: string } | null;
}

export interface TaskRunEvent {
  event_id: string;
  event_type: string;
  execution_state: string | null;
  verification_state: string | null;
  human_acceptance_state: string | null;
  failure_category: string | null;
  loop_iteration_no: number | null;
  loop_phase: string | null;
  loop_decision: string | null;
  hook_phase: string | null;
  hook_decision: string | null;
  hook_reason_class: string | null;
  peak_rss_bytes: number | null;
  cpu_time_ms: number | null;
  occurred_at: string;
}

export interface TaskRunEvidence {
  evidence_id: string;
  evidence_kind: string;
  summary: string | null;
  sha256_digest: string | null;
  media_type: string | null;
  byte_length: number | null;
  sensitivity: string;
  captured_at: string;
}

export interface TaskRunDetail {
  project_id: string;
  worktree_id: string;
  work_item_id: string;
  permission_snapshot_ref: string;
  run: TaskRunSummary;
  events: TaskRunEvent[];
  events_truncated: boolean;
  evidence: TaskRunEvidence[];
  evidence_truncated: boolean;
}

export type TaskCliSessionAttachmentReceipt = Omit<TaskCliSessionReceipt, "status"> & {
  status: "attachment_ticket_issued";
};

export class GroupApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = "GroupApiError";
  }
}

export class WorktreeGroupApiClient {
  private readonly baseUrl: string;

  constructor(
    private readonly getAccessToken: GroupAccessTokenProvider,
    private readonly fetcher: typeof fetch = fetch,
    baseUrl = "",
  ) {
    this.baseUrl = normalizeBaseUrl(baseUrl);
  }

  private async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const token = (await this.getAccessToken())?.trim();
    if (!token) {
      throw new GroupApiError(401, "session_required", "Group API requires an authenticated user session.");
    }

    const headers = new Headers(init.headers);
    headers.set("Accept", "application/json");
    headers.set("Authorization", `Bearer ${token}`);
    if (init.body !== undefined && !headers.has("Content-Type")) {
      headers.set("Content-Type", "application/json");
    }

    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      ...init,
      headers,
      credentials: "omit",
      cache: "no-store",
    });

    if (!response.ok) {
      const error = await readApiError(response);
      throw error;
    }
    if (response.status === 204) return undefined as T;
    return (await response.json()) as T;
  }

  getGroupContext<T>(worktreeId: string): Promise<T> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/group-context`);
  }

  listGroupApps(worktreeId: string): Promise<GroupAppRegistryProjection> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/group-apps`);
  }

  listProjectWorktrees<T>(projectId: string, filters: WorktreeIndexQuery = {}): Promise<T> {
    const query = new URLSearchParams();
    if (filters.limit !== undefined) query.set("limit", String(filters.limit));
    if (filters.cursor) query.set("cursor", filters.cursor);
    if (filters.owner_user_id) query.set("owner_user_id", filters.owner_user_id);
    if (filters.human_state) query.set("human_state", filters.human_state);
    if (filters.include_archived !== undefined) query.set("include_archived", String(filters.include_archived));
    const serialized = query.toString();
    const suffix = serialized ? `?${serialized}` : "";
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/worktrees${suffix}`);
  }

  getProjectHookPolicy(projectId: string): Promise<HookPolicyResponse> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/hook-policy`);
  }

  getWorktreeHookPolicy(worktreeId: string): Promise<HookPolicyResponse> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/hook-policy/effective`);
  }

  saveProjectHookDraft(projectId: string, body: HookPolicyDraftBody): Promise<{ draft_id: string; draft_version: number }> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/hook-policy/draft`, {
      method: "PUT",
      body: JSON.stringify(body),
    });
  }

  saveWorktreeHookDraft(worktreeId: string, body: HookPolicyDraftBody): Promise<{ draft_id: string; draft_version: number }> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/hook-policy/draft`, {
      method: "PUT",
      body: JSON.stringify(body),
    });
  }

  publishProjectHookDraft(projectId: string, body: { expected_draft_version: number; correlation_id: string }): Promise<{ policy_set_id: string }> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/hook-policy/publish`, {
      method: "POST",
      body: JSON.stringify(body),
    });
  }

  publishWorktreeHookDraft(worktreeId: string, body: { expected_draft_version: number; correlation_id: string }): Promise<{ policy_set_id: string }> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/hook-policy/publish`, {
      method: "POST",
      body: JSON.stringify(body),
    });
  }

  rollbackProjectHookPolicy(projectId: string, body: { target_policy_set_id: string; expected_current_policy_set_id: string; correlation_id: string }): Promise<{ policy_set_id: string; rolled_back_from: string }> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/hook-policy/rollback`, {
      method: "POST",
      body: JSON.stringify(body),
    });
  }

  rollbackWorktreeHookPolicy(worktreeId: string, body: { target_policy_set_id: string; expected_current_policy_set_id: string; correlation_id: string }): Promise<{ policy_set_id: string; rolled_back_from: string }> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/hook-policy/rollback`, {
      method: "POST",
      body: JSON.stringify(body),
    });
  }

  listAuthorizedProjects<T>(query: AuthorizedProjectsQuery = {}): Promise<T> {
    const params = new URLSearchParams();
    if (query.limit !== undefined) params.set("limit", String(query.limit));
    if (query.cursor) params.set("cursor", query.cursor);
    const serialized = params.toString();
    const suffix = serialized ? `?${serialized}` : "";
    return this.request(`/api/v1/projects${suffix}`);
  }

  listProjectWorktreeRepositories<T>(projectId: string): Promise<T> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/worktree-repositories`);
  }

  listWorktreeImportCandidates<T>(projectId: string, repositoryId: string, limit = 25): Promise<T> {
    const query = new URLSearchParams({ repository_id: repositoryId, limit: String(limit) });
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/worktree-import-candidates?${query.toString()}`);
  }

  createProjectWorktree<T>(projectId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/worktrees`, {
      method: "POST",
      headers: { "Idempotency-Key": idempotencyKey },
      body: JSON.stringify(body),
    });
  }

  importProjectWorktree<T>(projectId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/worktrees/import`, {
      method: "POST",
      headers: { "Idempotency-Key": idempotencyKey },
      body: JSON.stringify(body),
    });
  }

  listProjectMembers<T>(projectId: string): Promise<T> {
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/members`);
  }

  createManagementPlan<T>(
    worktreeId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/management-plans`, {
      method: "POST",
      headers: { "Idempotency-Key": idempotencyKey },
      body: JSON.stringify(body),
    });
  }

  confirmManagementPlan<T>(worktreeId: string, planId: string): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/management-plans/${encodeURIComponent(planId)}/confirm`,
      { method: "POST" },
    );
  }

  listWorkItems<T>(worktreeId: string): Promise<T> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items`);
  }

  listTaskRuns(
    worktreeId: string,
    workItemId: string,
    query: {
      limit?: number;
      cursor?: { cursor_started_at: string; cursor_run_id: string };
    } = {},
  ): Promise<TaskRunPage> {
    if (query.limit !== undefined && (!Number.isInteger(query.limit) || query.limit < 1 || query.limit > 50)) {
      throw new GroupApiError(400, "invalid_request", "Task Run list limit must be between 1 and 50.");
    }
    const params = new URLSearchParams();
    if (query.limit !== undefined) params.set("limit", String(query.limit));
    if (query.cursor) {
      params.set("cursor_started_at", query.cursor.cursor_started_at);
      params.set("cursor_run_id", query.cursor.cursor_run_id);
    }
    const serialized = params.toString();
    const suffix = serialized ? `?${serialized}` : "";
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/runs${suffix}`,
    );
  }

  getTaskRunDetail(worktreeId: string, workItemId: string, runId: string): Promise<TaskRunDetail> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/runs/${encodeURIComponent(runId)}`,
    );
  }

  transitionWorkItem<T>(
    worktreeId: string,
    workItemId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/lifecycle`,
      {
        method: "POST",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  startTaskCliSession(
    worktreeId: string,
    workItemId: string,
    body: {
      expected_lifecycle_version: number;
      approved_launch_profile_id: string;
      correlation_id: string;
    },
    idempotencyKey: string,
  ): Promise<TaskCliSessionReceipt> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/cli-sessions`,
      {
        method: "POST",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  listTaskCliSessions(
    worktreeId: string,
    workItemId: string,
    correlationId: string,
    limit = 20,
  ): Promise<TaskCliSessionPage> {
    if (!Number.isInteger(limit) || limit < 1 || limit > 50) {
      throw new GroupApiError(400, "invalid_request", "Task CLI session list limit must be between 1 and 50.");
    }
    const query = new URLSearchParams({ limit: String(limit) });
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/cli-sessions?${query}`,
      { headers: { "X-Correlation-ID": correlationId } },
    );
  }

  getTaskCliSessionStatus(
    worktreeId: string,
    workItemId: string,
    sessionId: string,
    correlationId: string,
  ): Promise<TaskCliSessionStatus> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/cli-sessions/${encodeURIComponent(sessionId)}`,
      { headers: { "X-Correlation-ID": correlationId } },
    );
  }

  cancelTaskCliSession(
    worktreeId: string,
    workItemId: string,
    sessionId: string,
    correlationId: string,
  ): Promise<TaskCliSessionStatus> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/cli-sessions/${encodeURIComponent(sessionId)}`,
      { method: "DELETE", headers: { "X-Correlation-ID": correlationId } },
    );
  }

  reattachTaskCliSession(
    worktreeId: string,
    workItemId: string,
    sessionId: string,
    correlationId: string,
  ): Promise<TaskCliSessionAttachmentReceipt> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/cli-sessions/${encodeURIComponent(sessionId)}/attachment-tickets`,
      { method: "POST", headers: { "X-Correlation-ID": correlationId } },
    );
  }

  submitScopedChatMessage(
    worktreeId: string,
    body: ScopedChatMessageBody,
    idempotencyKey: string,
  ): Promise<ScopedChatReceipt> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/chat/messages`,
      {
        method: "POST",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  listGlobalChatTargets(
    worktreeId: string,
    query: { limit?: number; cursor?: string } = {},
  ): Promise<ScopedChatTargetPage> {
    const params = new URLSearchParams();
    if (query.limit !== undefined) params.set("limit", String(query.limit));
    if (query.cursor) params.set("cursor", query.cursor);
    const serialized = params.toString();
    const suffix = serialized ? `?${serialized}` : "";
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/chat/targets${suffix}`,
    );
  }

  reviewWorkItem<T>(
    worktreeId: string,
    workItemId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items/${encodeURIComponent(workItemId)}/review`,
      {
        method: "POST",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  listCanvases<T>(worktreeId: string): Promise<T> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases`);
  }

  createCanvas<T>(worktreeId: string, body: Record<string, unknown>, idempotencyKey: string): Promise<T> {
    return this.request(`/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases`, {
      method: "POST",
      headers: { "Idempotency-Key": idempotencyKey },
      body: JSON.stringify(body),
    });
  }

  createCanvasElement<T>(
    worktreeId: string,
    canvasId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements`,
      {
        method: "POST",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  updateCanvasElement<T>(
    worktreeId: string,
    canvasId: string,
    elementId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements/${encodeURIComponent(elementId)}`,
      {
        method: "PUT",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  deleteCanvasElement<T>(
    worktreeId: string,
    canvasId: string,
    elementId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements/${encodeURIComponent(elementId)}`,
      {
        method: "DELETE",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  updateCanvasDocument<T>(
    worktreeId: string,
    canvasId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/document`,
      {
        method: "PUT",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  listCanvasElements<T>(worktreeId: string, canvasId: string): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements`,
    );
  }

  listCanvasEvents<T>(
    worktreeId: string,
    canvasId: string,
    cursor?: CanvasEventCursor,
    limit = 100,
  ): Promise<T> {
    const query = new URLSearchParams({ limit: String(limit) });
    if (cursor) {
      query.set("cursor_at", cursor.cursor_at);
      query.set("cursor_event_id", cursor.cursor_event_id);
    }
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/events?${query}`,
    );
  }

  createWorkItemOnCanvas<T>(
    worktreeId: string,
    canvasId: string,
    body: Record<string, unknown>,
    idempotencyKey: string,
  ): Promise<T> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/work-items`,
      {
        method: "POST",
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }
}

export interface CanvasOutboxPollerOptions {
  initialCursor?: CanvasEventCursor;
  intervalMs?: number;
  pageSize?: number;
  cursorStore?: CanvasEventCursorStore;
}

export interface CanvasEventCursorStore {
  load(scopeKey: string): CanvasEventCursor | undefined;
  save(scopeKey: string, cursor: CanvasEventCursor): void;
}

/** Stores an untrusted resume hint; authorization is checked on every poll. */
export class BrowserCanvasEventCursorStore implements CanvasEventCursorStore {
  load(scopeKey: string): CanvasEventCursor | undefined {
    try {
      const raw = globalThis.localStorage?.getItem(cursorStorageKey(scopeKey));
      if (!raw) return undefined;
      const parsed: unknown = JSON.parse(raw);
      return isCanvasEventCursor(parsed) ? parsed : undefined;
    } catch {
      return undefined;
    }
  }

  save(scopeKey: string, cursor: CanvasEventCursor): void {
    try {
      globalThis.localStorage?.setItem(cursorStorageKey(scopeKey), JSON.stringify(cursor));
    } catch {
      // Storage can be unavailable; the in-memory cursor still works this session.
    }
  }
}

const browserCanvasEventCursorStore = new BrowserCanvasEventCursorStore();

export class CanvasOutboxPoller {
  private cursor?: CanvasEventCursor;
  private cursorLoaded: boolean;
  private readonly intervalMs: number;
  private readonly pageSize: number;
  private readonly cursorStore: CanvasEventCursorStore;
  private readonly cursorScopeKey: string;

  constructor(
    private readonly client: WorktreeGroupApiClient,
    private readonly worktreeId: string,
    private readonly canvasId: string,
    private readonly refreshProjection: () => Promise<void>,
    options: CanvasOutboxPollerOptions = {},
  ) {
    this.cursor = isCanvasEventCursor(options.initialCursor) ? options.initialCursor : undefined;
    this.cursorLoaded = this.cursor !== undefined;
    this.intervalMs = options.intervalMs ?? 1_000;
    this.pageSize = options.pageSize ?? 100;
    this.cursorStore = options.cursorStore ?? browserCanvasEventCursorStore;
    this.cursorScopeKey = `${worktreeId}:${canvasId}`;
    if (this.intervalMs < 100 || this.intervalMs > 30_000 || this.pageSize < 1 || this.pageSize > 200) {
      throw new RangeError("Canvas Outbox poll settings are outside the supported bounds.");
    }
  }

  async pollOnce(): Promise<number> {
    if (!this.cursorLoaded) {
      const storedCursor = this.cursorStore.load(this.cursorScopeKey);
      this.cursor = isCanvasEventCursor(storedCursor) ? storedCursor : undefined;
      this.cursorLoaded = true;
    }
    const page = await this.client.listCanvasEvents<CanvasOutboxEventPage>(
      this.worktreeId,
      this.canvasId,
      this.cursor,
      this.pageSize,
    );
    if (page.events.length === 0) return 0;

    // Refresh the current authorized projection before persisting or advancing the hint.
    await this.refreshProjection();
    const lastEvent = page.events[page.events.length - 1];
    const nextCursor = page.next_cursor ?? {
      cursor_at: lastEvent.occurred_at,
      cursor_event_id: lastEvent.event_id,
    };
    this.cursorStore.save(this.cursorScopeKey, nextCursor);
    this.cursor = nextCursor;
    return page.events.length;
  }

  async run(signal: AbortSignal, onError: (error: unknown) => void = () => undefined): Promise<void> {
    let retryMs = this.intervalMs;
    while (!signal.aborted) {
      try {
        const consumed = await this.pollOnce();
        retryMs = this.intervalMs;
        if (consumed < this.pageSize) await waitForPoll(this.intervalMs, signal);
      } catch (error) {
        try {
          onError(error);
        } catch {
          // An observer must not alter cursor or retry behavior.
        }
        if (error instanceof GroupApiError && (error.status === 401 || error.status === 403)) return;
        await waitForPoll(retryMs, signal);
        retryMs = Math.min(retryMs * 2, 30_000);
      }
    }
  }
}

function cursorStorageKey(scopeKey: string): string {
  return `star:worktree-group:canvas-outbox:v1:${encodeURIComponent(scopeKey)}`;
}

function isCanvasEventCursor(value: unknown): value is CanvasEventCursor {
  if (!value || typeof value !== "object") return false;
  const cursor = value as Partial<CanvasEventCursor>;
  return typeof cursor.cursor_at === "string"
    && Number.isFinite(Date.parse(cursor.cursor_at))
    && Date.parse(cursor.cursor_at) <= Date.now() + 30_000
    && typeof cursor.cursor_event_id === "string"
    && /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(cursor.cursor_event_id);
}

async function readApiError(response: Response): Promise<GroupApiError> {
  const payload = (await response.json().catch(() => null)) as
    | { error?: { code?: unknown; message?: unknown } }
    | null;
  const code = typeof payload?.error?.code === "string" ? payload.error.code : "group_api_error";
  const message = typeof payload?.error?.message === "string" ? payload.error.message : "Group API request failed.";
  return new GroupApiError(response.status, code, message);
}

function normalizeBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.trim().replace(/\/+$/, "");
  if (!normalized) return "";
  if (normalized.startsWith("/") && !normalized.startsWith("//")) {
    if (normalized.includes("?") || normalized.includes("#")) {
      throw new GroupApiError(0, "invalid_api_origin", "Group API base path cannot include query or fragment.");
    }
    return normalized;
  }
  if (!/^https?:\/\//i.test(normalized)) {
    throw new GroupApiError(0, "invalid_api_origin", "Group API base URL must be a same-origin path or HTTP(S) URL.");
  }

  const url = new URL(normalized);
  const isLoopback = ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname);
  if (url.protocol !== "https:" && !isLoopback) {
    throw new GroupApiError(0, "insecure_api_origin", "Group API must use HTTPS outside localhost development.");
  }
  if (url.username || url.password || url.search || url.hash) {
    throw new GroupApiError(0, "invalid_api_origin", "Group API base URL cannot include credentials, query, or fragment.");
  }
  return `${url.origin}${url.pathname.replace(/\/+$/, "")}`;
}

function waitForPoll(ms: number, signal: AbortSignal): Promise<void> {
  if (signal.aborted) return Promise.resolve();
  return new Promise((resolve) => {
    const finish = () => {
      clearTimeout(timer);
      signal.removeEventListener("abort", finish);
      resolve();
    };
    const timer = setTimeout(finish, ms);
    signal.addEventListener("abort", finish, { once: true });
  });
}

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts"}),
      (client:Class {name:"WorktreeGroupApiClient"}),
      (request:Function {name:"WorktreeGroupApiClient.request"});
CREATE (hookDecision:Class {name:"HookDecision",type:"class",language:"typescript",visibility:"public"}),
       (hookReason:Class {name:"HookReasonCode",type:"class",language:"typescript",visibility:"public"}),
       (hookFact:Class {name:"HookFactField",type:"class",language:"typescript",visibility:"public"}),
       (hookOperator:Class {name:"HookOperator",type:"class",language:"typescript",visibility:"public"}),
       (hookValue:Class {name:"HookValue",type:"class",language:"typescript",visibility:"public"}),
       (hookCondition:Class {name:"HookCondition",type:"interface",language:"typescript",visibility:"public"}),
       (hookRule:Class {name:"HookRule",type:"interface",language:"typescript",visibility:"public"}),
       (hookDocument:Class {name:"HookPolicyDocument",type:"interface",language:"typescript",visibility:"public"}),
       (hookAudit:Class {name:"HookPolicyAuditEvent",type:"interface",language:"typescript",visibility:"public"}),
       (hookDraft:Class {name:"HookPolicyDraftView",type:"interface",language:"typescript",visibility:"public"}),
       (hookResponse:Class {name:"HookPolicyResponse",type:"interface",language:"typescript",visibility:"public"}),
       (hookDraftBody:Class {name:"HookPolicyDraftBody",type:"interface",language:"typescript",visibility:"public"}),
       (getProjectPolicy:Function {name:"WorktreeGroupApiClient.getProjectHookPolicy",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (getWorktreePolicy:Function {name:"WorktreeGroupApiClient.getWorktreeHookPolicy",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (saveProjectDraft:Function {name:"WorktreeGroupApiClient.saveProjectHookDraft",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (saveWorktreeDraft:Function {name:"WorktreeGroupApiClient.saveWorktreeHookDraft",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (publishProject:Function {name:"WorktreeGroupApiClient.publishProjectHookDraft",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (publishWorktree:Function {name:"WorktreeGroupApiClient.publishWorktreeHookDraft",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (rollbackProject:Function {name:"WorktreeGroupApiClient.rollbackProjectHookPolicy",type:"function",language:"typescript",visibility:"public",complexity:"simple"}),
       (rollbackWorktree:Function {name:"WorktreeGroupApiClient.rollbackWorktreeHookPolicy",type:"function",language:"typescript",visibility:"public",complexity:"simple"});
CREATE (file)-[:CONTAINS]->(hookDecision),(file)-[:CONTAINS]->(hookReason),
       (file)-[:CONTAINS]->(hookFact),(file)-[:CONTAINS]->(hookOperator),(file)-[:CONTAINS]->(hookValue),
       (file)-[:CONTAINS]->(hookCondition),(file)-[:CONTAINS]->(hookRule),(file)-[:CONTAINS]->(hookDocument),
       (file)-[:CONTAINS]->(hookAudit),(file)-[:CONTAINS]->(hookDraft),(file)-[:CONTAINS]->(hookResponse),
       (file)-[:CONTAINS]->(hookDraftBody),
       (client)-[:HAS_METHOD]->(getProjectPolicy),(client)-[:HAS_METHOD]->(getWorktreePolicy),
       (client)-[:HAS_METHOD]->(saveProjectDraft),(client)-[:HAS_METHOD]->(saveWorktreeDraft),
       (client)-[:HAS_METHOD]->(publishProject),(client)-[:HAS_METHOD]->(publishWorktree),
       (client)-[:HAS_METHOD]->(rollbackProject),(client)-[:HAS_METHOD]->(rollbackWorktree),
       (getProjectPolicy)-[:CALLS]->(request),
       (getWorktreePolicy)-[:CALLS]->(request),(saveProjectDraft)-[:CALLS]->(request),
       (saveWorktreeDraft)-[:CALLS]->(request),(publishProject)-[:CALLS]->(request),
       (publishWorktree)-[:CALLS]->(request),(rollbackProject)-[:CALLS]->(request),
       (rollbackWorktree)-[:CALLS]->(request),(hookDraft)-[:CONTAINS]->(hookDocument),
       (hookResponse)-[:CONTAINS]->(hookDraft),(hookResponse)-[:CONTAINS]->(hookAudit),
       (hookRule)-[:CONTAINS]->(hookCondition),(hookCondition)-[:USES]->(hookValue);
*/
