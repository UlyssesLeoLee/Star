/*
@cypher schema=1 source_sha256=db801ba36d397966512ad6d735cb91e0baff30f9eb383114422b36ccbe9084e6
CYPHER STRUCTURE MANIFEST
MERGE (self:File {path:"frontend/src/lib/group-api/group-context.ts"});
CREATE
  (context:Class {name:"GroupContextProjection",type:"class"}),
  (worktree:Class {name:"GroupWorktreeProjection",type:"class"}),
  (contextResponse:Class {name:"GroupWorktreeContextResponse",type:"class"}),
  (taskLifecycle:Class {name:"GroupTaskLifecycle",type:"class"}),
  (task:Class {name:"GroupWorkItemProjection",type:"class"}),
  (taskList:Class {name:"GroupWorkItemListResponse",type:"class"}),
  (worktreeIndex:Class {name:"GroupWorktreeIndexResponse",type:"class"}),
  (worktreeRow:Class {name:"GroupWorktreeIndexRow",type:"class"}),
  (managementPlan:Class {name:"GroupWorktreeManagementPlan",type:"class"}),
  (managementResult:Class {name:"GroupWorktreeManagementResult",type:"class"}),
  (apiError:Class {name:"GroupApiError",type:"class"}),
  (client:Class {name:"GroupContextApiClient",type:"class"}),
  (constructor:Function {name:"GroupContextApiClient.constructor",type:"function",signature:"constructor(baseUrl: string = /api/group)",visibility:"public",complexity:"simple"}),
  (request:Function {name:"GroupContextApiClient.request",type:"function",signature:"request<T>(path: string, signal?: AbortSignal): Promise<T>",visibility:"private",complexity:"moderate"}),
  (resolve:Function {name:"GroupContextApiClient.resolveWorktreeContext",type:"function",signature:"resolveWorktreeContext(worktreeId: string, signal?: AbortSignal): Promise<GroupWorktreeContextResponse>",visibility:"public",complexity:"simple"}),
  (list:Function {name:"GroupContextApiClient.listWorkItems",type:"function",signature:"listWorkItems(worktreeId: string, signal?: AbortSignal): Promise<GroupWorkItemListResponse>",visibility:"public",complexity:"simple"}),
  (listProject:Function {name:"GroupContextApiClient.listProjectWorktrees",type:"function",signature:"listProjectWorktrees(projectId: string, cursor?: string | null, signal?: AbortSignal): Promise<GroupWorktreeIndexResponse>",visibility:"public",complexity:"simple"}),
  (managePlan:Function {name:"GroupContextApiClient.createWorktreeManagementPlan",type:"function",signature:"createWorktreeManagementPlan(worktree: GroupWorktreeIndexRow, archived: boolean, signal?: AbortSignal): Promise<GroupWorktreeManagementPlan>",visibility:"public",complexity:"simple"}),
  (confirmPlan:Function {name:"GroupContextApiClient.confirmWorktreeManagementPlan",type:"function",signature:"confirmWorktreeManagementPlan(worktreeId: string, planId: string, signal?: AbortSignal): Promise<GroupWorktreeManagementResult>",visibility:"public",complexity:"simple"}),
  (mutate:Function {name:"GroupContextApiClient.mutate",type:"function",signature:"mutate<T>(path: string, body: unknown, idempotencyKey: string, signal?: AbortSignal): Promise<T>",visibility:"private",complexity:"moderate"}),
  (self)-[:CONTAINS]->(context),(self)-[:CONTAINS]->(worktree),(self)-[:CONTAINS]->(contextResponse),
  (self)-[:CONTAINS]->(taskLifecycle),(self)-[:CONTAINS]->(task),(self)-[:CONTAINS]->(taskList),
  (self)-[:CONTAINS]->(worktreeIndex),(self)-[:CONTAINS]->(worktreeRow),(self)-[:CONTAINS]->(managementPlan),(self)-[:CONTAINS]->(managementResult),
  (self)-[:CONTAINS]->(apiError),(self)-[:CONTAINS]->(client),
  (client)-[:HAS_METHOD]->(constructor),(client)-[:HAS_METHOD]->(request),
  (client)-[:HAS_METHOD]->(resolve),(client)-[:HAS_METHOD]->(list),(client)-[:HAS_METHOD]->(listProject),
  (client)-[:HAS_METHOD]->(managePlan),(client)-[:HAS_METHOD]->(confirmPlan),(client)-[:HAS_METHOD]->(mutate),
  (resolve)-[:CALLS]->(request),(list)-[:CALLS]->(request),(listProject)-[:CALLS]->(request),
  (managePlan)-[:CALLS]->(mutate),(confirmPlan)-[:CALLS]->(mutate);
@endcypher
*/

export interface GroupContextProjection {
  tenant_id: string;
  workspace_id: string;
  project_id: string;
  repository_id: string;
  worktree_id: string;
  actor_id: string;
  actor_kind: "user" | "agent";
  actor_roles: string[];
  granted_scopes: string[];
  context_version: number;
  resolved_at: string;
  permission_snapshot_ref: string;
  correlation_id: string;
}

export interface GroupWorktreeProjection {
  id: string;
  name: string;
  repository_id: string;
  workspace_id: string;
  project_id: string;
  parent_id: string | null;
  owner_user_id: string | null;
  work_item_id: string | null;
  agent_id: string | null;
  agent_session_id: string | null;
  runtime_id: string | null;
  branch: string;
  human_state: string;
  machine_state: string;
  ahead: number;
  behind: number;
  dirty: boolean;
  health_score: number;
  last_activity: string;
  test_state: string;
  risk_count: number;
  locked: boolean;
  archived: boolean;
  version: number;
  pull_request_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface GroupWorktreeContextResponse {
  group_context: GroupContextProjection;
  worktree: GroupWorktreeProjection;
}

export interface GroupTaskLifecycle {
  status: "pending" | "claimed" | "in_progress" | "completed" | "failed" | "cancelled";
  review_state: string;
  active_worktree_id: string | null;
  version: number;
  updated_at: string;
}

export interface GroupWorkItemProjection {
  work_item_id: string;
  task_card_id: string;
  item_type: string;
  title: string;
  description: string;
  priority: string;
  labels: string[];
  ai_task_data: Record<string, unknown> | null;
  reporter_user_id: string;
  lifecycle: GroupTaskLifecycle;
  metadata_updated_at: string;
}

export interface GroupWorkItemListResponse {
  worktree_id: string;
  project_id: string;
  role: string;
  permission_snapshot_ref: string;
  limit: number;
  work_items: GroupWorkItemProjection[];
}

export interface GroupWorktreeIndexRow extends GroupWorktreeProjection {}

export interface GroupWorktreeIndexResponse {
  project_id: string;
  role: string;
  permission_snapshot_ref: string;
  limit: number;
  next_cursor: string | null;
  worktrees: GroupWorktreeIndexRow[];
}

export interface GroupWorktreeManagementResult {
  worktree_id: string;
  plan_id: string;
  status: "confirmed";
  before: { owner_user_id: string | null; archived: boolean; version: number };
  after: { owner_user_id: string | null; archived: boolean; version: number };
  correlation_id: string;
}

export interface GroupWorktreeManagementPlan {
  plan_id: string;
  worktree_id: string;
  operation: "set_archived";
  expected_version: number;
  expires_at: string;
  status: "pending";
  correlation_id: string;
}

export class GroupApiError extends Error {
  constructor(readonly status: number, readonly code: string) {
    super(code);
    this.name = "GroupApiError";
  }
}

export class GroupContextApiClient {
  constructor(private readonly baseUrl = "/api/group") {}

  private async request<T>(path: string, signal?: AbortSignal): Promise<T> {
    const response = await fetch(`${this.baseUrl.replace(/\/+$/, "")}${path}`, {
      method: "GET",
      headers: { Accept: "application/json" },
      credentials: "same-origin",
      cache: "no-store",
      signal,
    });
    if (!response.ok) {
      const payload = await response.json().catch(() => null) as
        | { error?: { code?: unknown } }
        | null;
      const code = typeof payload?.error?.code === "string"
        ? payload.error.code
        : `http_${response.status}`;
      throw new GroupApiError(response.status, code);
    }
    return await response.json() as T;
  }

  private async mutate<T>(path: string, body: unknown, idempotencyKey: string, signal?: AbortSignal): Promise<T> {
    const response = await fetch(`${this.baseUrl.replace(/\/+$/, "")}${path}`, {
      method: "POST",
      headers: { Accept: "application/json", "Content-Type": "application/json", "Idempotency-Key": idempotencyKey },
      credentials: "same-origin",
      cache: "no-store",
      body: JSON.stringify(body),
      signal,
    });
    if (!response.ok) {
      const payload = await response.json().catch(() => null) as { error?: { code?: unknown } } | null;
      const code = typeof payload?.error?.code === "string" ? payload.error.code : `http_${response.status}`;
      throw new GroupApiError(response.status, code);
    }
    return await response.json() as T;
  }

  resolveWorktreeContext(
    worktreeId: string,
    signal?: AbortSignal,
  ): Promise<GroupWorktreeContextResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/group-context`,
      signal,
    );
  }

  listWorkItems(
    worktreeId: string,
    signal?: AbortSignal,
  ): Promise<GroupWorkItemListResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/work-items`,
      signal,
    );
  }

  listProjectWorktrees(
    projectId: string,
    cursor?: string | null,
    signal?: AbortSignal,
  ): Promise<GroupWorktreeIndexResponse> {
    const query = new URLSearchParams({ limit: "50", include_archived: "true" });
    if (cursor) query.set("cursor", cursor);
    return this.request(`/api/v1/projects/${encodeURIComponent(projectId)}/worktrees?${query.toString()}`, signal);
  }

  createWorktreeManagementPlan(
    worktree: GroupWorktreeIndexRow,
    archived: boolean,
    signal?: AbortSignal,
  ): Promise<GroupWorktreeManagementPlan> {
    const base = `/api/v1/worktrees/${encodeURIComponent(worktree.id)}`;
    return this.mutate<GroupWorktreeManagementPlan>(
      `${base}/management-plans`,
      { operation: "set_archived", archived, expected_version: worktree.version },
      crypto.randomUUID(),
      signal,
    );
  }

  confirmWorktreeManagementPlan(
    worktreeId: string,
    planId: string,
    signal?: AbortSignal,
  ): Promise<GroupWorktreeManagementResult> {
    const base = `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/management-plans/${encodeURIComponent(planId)}`;
    return this.mutate<GroupWorktreeManagementResult>(
      `${base}/confirm`,
      {},
      crypto.randomUUID(),
      signal,
    );
  }
}
