/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group/worktreeGroupApi.ts",type:"file",language:"typescript"}),
  (token:Class {name:"GroupAccessTokenProvider",type:"class",signature:"() => Promise<string | null>",visibility:"public"}),
  (cursor:Class {name:"CanvasEventCursor",type:"class",signature:"{ cursor_at: string; cursor_event_id: string }",visibility:"public"}),
  (event:Class {name:"CanvasOutboxEventMetadata",type:"class",visibility:"public"}),
  (eventPage:Class {name:"CanvasOutboxEventPage",type:"class",visibility:"public"}),
  (indexQuery:Class {name:"WorktreeIndexQuery",type:"class",visibility:"public"}),
  (error:Class {name:"GroupApiError",type:"class",visibility:"public"}),
  (errorCtor:Function {name:"GroupApiError.constructor",type:"function",signature:"constructor(status: number, code: string, message: string)",visibility:"public",complexity:"simple"}),
  (client:Class {name:"WorktreeGroupApiClient",type:"class",visibility:"public"}),
  (ctor:Function {name:"WorktreeGroupApiClient.constructor",type:"function",signature:"constructor(getAccessToken: GroupAccessTokenProvider, fetcher?: typeof fetch, baseUrl?: string)",visibility:"public",complexity:"simple"}),
  (request:Function {name:"WorktreeGroupApiClient.request",type:"function",signature:"request<T>(path: string, init?: RequestInit): Promise<T>",visibility:"private",complexity:"moderate"}),
  (groupContext:Function {name:"WorktreeGroupApiClient.getGroupContext",type:"function",signature:"getGroupContext<T>(worktreeId: string): Promise<T>",visibility:"public",complexity:"simple"}),
  (index:Function {name:"WorktreeGroupApiClient.listProjectWorktrees",type:"function",signature:"listProjectWorktrees<T>(projectId: string, query?: WorktreeIndexQuery): Promise<T>",visibility:"public",complexity:"moderate"}),
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
  (file)-[:CONTAINS]->(token),(file)-[:CONTAINS]->(cursor),(file)-[:CONTAINS]->(event),(file)-[:CONTAINS]->(eventPage),(file)-[:CONTAINS]->(indexQuery),(file)-[:CONTAINS]->(error),(file)-[:CONTAINS]->(client),(file)-[:CONTAINS]->(projectMembers),(file)-[:CONTAINS]->(poller),(file)-[:CONTAINS]->(readError),(file)-[:CONTAINS]->(waitPoll),
  (error)-[:HAS_METHOD]->(errorCtor),
  (client)-[:HAS_METHOD]->(ctor),(client)-[:HAS_METHOD]->(request),(client)-[:HAS_METHOD]->(groupContext),(client)-[:HAS_METHOD]->(index),(client)-[:HAS_METHOD]->(projectMembers),(client)-[:HAS_METHOD]->(managementPlan),(client)-[:HAS_METHOD]->(managementConfirm),(client)-[:HAS_METHOD]->(workItems),(client)-[:HAS_METHOD]->(transitionWorkItem),(client)-[:HAS_METHOD]->(canvases),(client)-[:HAS_METHOD]->(createCanvas),(client)-[:HAS_METHOD]->(createCanvasElement),(client)-[:HAS_METHOD]->(updateCanvasElement),(client)-[:HAS_METHOD]->(deleteCanvasElement),(client)-[:HAS_METHOD]->(updateCanvasDocument),(client)-[:HAS_METHOD]->(elements),(client)-[:HAS_METHOD]->(events),(client)-[:HAS_METHOD]->(createItem),
  (poller)-[:HAS_METHOD]->(pollerCtor),(poller)-[:HAS_METHOD]->(pollOnce),(poller)-[:HAS_METHOD]->(runPoller),
  (ctor)-[:CALLS]->(normalizeBaseUrl),(request)-[:CALLS]->(readError),(groupContext)-[:CALLS]->(request),(index)-[:CALLS]->(request),(index)-[:USES]->(indexQuery),(projectMembers)-[:CALLS]->(request),(managementPlan)-[:CALLS]->(request),(managementConfirm)-[:CALLS]->(request),(workItems)-[:CALLS]->(request),(transitionWorkItem)-[:CALLS]->(request),(canvases)-[:CALLS]->(request),(createCanvas)-[:CALLS]->(request),(createCanvasElement)-[:CALLS]->(request),(updateCanvasElement)-[:CALLS]->(request),(updateCanvasDocument)-[:CALLS]->(request),(elements)-[:CALLS]->(request),(events)-[:CALLS]->(request),(createItem)-[:CALLS]->(request),(pollerCtor)-[:USES]->(event),(pollerCtor)-[:USES]->(eventPage),(pollOnce)-[:CALLS]->(events),(runPoller)-[:CALLS]->(pollOnce),(runPoller)-[:CALLS]->(waitPoll);
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
