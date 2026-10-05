/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group-api/canvas.ts",type:"file",language:"typescript"}),
  (canvasRow:Class {name:"GroupCanvasRow",type:"class"}),
  (document:Class {name:"GroupCanvasDocument",type:"class"}),
  (frame:Class {name:"GroupCanvasFrame",type:"class"}),
  (documentUpdate:Class {name:"UpdateCanvasDocumentInput",type:"class"}),
  (documentResponse:Class {name:"UpdateCanvasDocumentResponse",type:"class"}),
  (canvasList:Class {name:"GroupCanvasListResponse",type:"class"}),
  (entityRef:Class {name:"GroupCanvasEntityRef",type:"class"}),
  (element:Class {name:"GroupCanvasElement",type:"class"}),
  (elementInput:Class {name:"GroupCanvasElementInput",type:"class"}),
  (elementUpdate:Class {name:"GroupCanvasElementUpdate",type:"class"}),
  (writeContext:Class {name:"CanvasWriteContext",type:"class"}),
  (canvasResponse:Class {name:"CreateCanvasResponse",type:"class"}),
  (elementsResponse:Class {name:"CanvasElementsResponse",type:"class"}),
  (mutationResponse:Class {name:"CanvasMutationResponse",type:"class"}),
  (event:Class {name:"GroupCanvasOutboxEvent",type:"class"}),
  (eventsResponse:Class {name:"GroupCanvasEventsResponse",type:"class"}),
  (workItemInput:Class {name:"CreateCanvasWorkItemInput",type:"class"}),
  (workItemResponse:Class {name:"CreateCanvasWorkItemResponse",type:"class"}),
  (apiError:Class {name:"GroupCanvasApiError",type:"class"}),
  (client:Class {name:"GroupCanvasApiClient",type:"class"}),
  (constructor:Function {name:"GroupCanvasApiClient.constructor",type:"function",signature:"constructor(baseUrl: string = /api/group)",visibility:"public",complexity:"simple"}),
  (errorConstructor:Function {name:"GroupCanvasApiError.constructor",type:"function"}),
  (request:Function {name:"GroupCanvasApiClient.request",type:"function"}),
  (listCanvases:Function {name:"GroupCanvasApiClient.listCanvases",type:"function"}),
  (createCanvas:Function {name:"GroupCanvasApiClient.createCanvas",type:"function"}),
  (updateDocument:Function {name:"GroupCanvasApiClient.updateDocument",type:"function"}),
  (listEvents:Function {name:"GroupCanvasApiClient.listEvents",type:"function"}),
  (createWorkItem:Function {name:"GroupCanvasApiClient.createWorkItem",type:"function"}),
  (listElements:Function {name:"GroupCanvasApiClient.listElements",type:"function"}),
  (createElement:Function {name:"GroupCanvasApiClient.createElement",type:"function"}),
  (updateElement:Function {name:"GroupCanvasApiClient.updateElement",type:"function"}),
  (deleteElement:Function {name:"GroupCanvasApiClient.deleteElement",type:"function"}),
  (setEntityRef:Function {name:"GroupCanvasApiClient.setEntityRef",type:"function"}),
  (clearEntityRef:Function {name:"GroupCanvasApiClient.clearEntityRef",type:"function"}),
  (file)-[:CONTAINS]->(canvasRow),(file)-[:CONTAINS]->(document),(file)-[:CONTAINS]->(frame),(file)-[:CONTAINS]->(documentUpdate),(file)-[:CONTAINS]->(documentResponse),(file)-[:CONTAINS]->(canvasList),(file)-[:CONTAINS]->(entityRef),
  (file)-[:CONTAINS]->(element),(file)-[:CONTAINS]->(elementInput),(file)-[:CONTAINS]->(elementUpdate),
  (file)-[:CONTAINS]->(writeContext),(file)-[:CONTAINS]->(canvasResponse),(file)-[:CONTAINS]->(elementsResponse),
  (file)-[:CONTAINS]->(mutationResponse),(file)-[:CONTAINS]->(event),(file)-[:CONTAINS]->(eventsResponse),
  (file)-[:CONTAINS]->(workItemInput),(file)-[:CONTAINS]->(workItemResponse),
  (file)-[:CONTAINS]->(apiError),(file)-[:CONTAINS]->(client),
  (client)-[:HAS_METHOD]->(constructor),(client)-[:HAS_METHOD]->(request),
  (client)-[:HAS_METHOD]->(listCanvases),(client)-[:HAS_METHOD]->(createCanvas),(client)-[:HAS_METHOD]->(updateDocument),
  (client)-[:HAS_METHOD]->(listEvents),(client)-[:HAS_METHOD]->(createWorkItem),
  (client)-[:HAS_METHOD]->(listElements),(client)-[:HAS_METHOD]->(createElement),
  (client)-[:HAS_METHOD]->(updateElement),(client)-[:HAS_METHOD]->(deleteElement),
  (client)-[:HAS_METHOD]->(setEntityRef),(client)-[:HAS_METHOD]->(clearEntityRef),
  (apiError)-[:HAS_METHOD]->(errorConstructor),
  (listCanvases)-[:CALLS]->(request),(createCanvas)-[:CALLS]->(request),(updateDocument)-[:CALLS]->(request),
  (listEvents)-[:CALLS]->(request),(createWorkItem)-[:CALLS]->(request),
  (listElements)-[:CALLS]->(request),(createElement)-[:CALLS]->(request),
  (updateElement)-[:CALLS]->(request),(deleteElement)-[:CALLS]->(request),
  (setEntityRef)-[:CALLS]->(request),(clearEntityRef)-[:CALLS]->(request),
  (request)-[:CALLS]->(errorConstructor);
*/

import type { CanvasConnector, CanvasElement, CanvasFrame, CanvasViewport } from "@/types/ids";

export type GroupCanvasFrame = Omit<CanvasFrame, "canvas_id">;

export interface GroupCanvasDocument {
  version: number;
  viewport: CanvasViewport;
  frames: GroupCanvasFrame[];
  connectors: CanvasConnector[];
}

export interface GroupCanvasRow {
  canvas_id: string;
  title: string;
  version: number;
  created_by: string;
  created_at: string;
  element_count: number;
  document: GroupCanvasDocument;
}

export interface GroupCanvasListResponse {
  worktree_id: string;
  project_id: string;
  canvases: GroupCanvasRow[];
}

export interface GroupCanvasEntityRef {
  ref_type: "work_item" | "worktree";
  ref_id: string;
  worktree_id: string;
  version?: number;
}

export interface GroupCanvasElement {
  element_id: string;
  kind: "sticky_note" | "text" | "shape" | "image" | "embed" | "work_item_card" | "worktree_node" | "agent_cursor" | "automation_node" | "comment_pin";
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  z_index: number;
  content: CanvasElement["content"];
  locked: boolean;
  hidden: boolean;
  version: number;
  created_by: string;
  created_at: string;
  updated_at: string;
  entity_ref: GroupCanvasEntityRef | null;
}

export interface GroupCanvasElementInput {
  element_id?: string;
  kind: GroupCanvasElement["kind"];
  x: number;
  y: number;
  width: number;
  height: number;
  rotation?: number;
  z_index?: number;
  content?: Record<string, unknown>;
  locked?: boolean;
  hidden?: boolean;
  entity_ref?: Omit<GroupCanvasEntityRef, "version">;
  correlation_id?: string;
}

export interface GroupCanvasElementUpdate {
  expected_version: number;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  z_index: number;
  content: Record<string, unknown>;
  locked: boolean;
  hidden: boolean;
  correlation_id?: string;
}

export interface CanvasWriteContext {
  idempotencyKey: string;
  correlationId?: string;
}

export interface UpdateCanvasDocumentInput {
  expected_version: number;
  viewport: CanvasViewport;
  frames: GroupCanvasFrame[];
  connectors: CanvasConnector[];
  correlation_id?: string;
}

export interface UpdateCanvasDocumentResponse {
  canvas_id: string;
  worktree_id: string;
  document: GroupCanvasDocument;
  correlation_id: string;
}

export interface CreateCanvasResponse {
  canvas: {
    canvas_id: string;
    worktree_id: string;
    project_id: string;
    title: string;
    version: number;
    created_by: string;
  };
}

export interface CanvasElementsResponse {
  canvas_id: string;
  worktree_id: string;
  elements: GroupCanvasElement[];
}

export interface CanvasMutationResponse {
  canvas_id?: string;
  worktree_id?: string;
  element?: GroupCanvasElement;
  element_id?: string;
  version?: number;
  deleted?: boolean;
  correlation_id?: string;
}

export interface GroupCanvasOutboxEvent {
  stream_offset: string;
  event_id: string;
  project_id: string;
  worktree_id: string;
  canvas_id: string;
  element_id: string | null;
  event_type: string;
  schema_version: number;
  aggregate_version: number;
  actor_id: string;
  correlation_id: string;
  payload: Record<string, unknown>;
  occurred_at: string;
}

export interface GroupCanvasEventsResponse {
  worktree_id: string;
  project_id: string;
  events: GroupCanvasOutboxEvent[];
  has_more: boolean;
  next_offset: string;
}

export interface CreateCanvasWorkItemInput {
  work_item: {
    item_type: "epic" | "story" | "task" | "bug" | "subtask" | "ai_task";
    title: string;
    description?: string;
    priority?: "low" | "medium" | "high" | "urgent";
    labels?: string[];
    ai_task_data?: { objective: string; repository_scope: string[] } | null;
    correlation_id?: string;
  };
  x: number;
  y: number;
  width: number;
  height: number;
  rotation?: number;
  z_index?: number;
  content?: Record<string, unknown>;
  locked?: boolean;
  hidden?: boolean;
}

export interface CreateCanvasWorkItemResponse {
  work_item: {
    work_item_id: string;
    task_card_id: string;
    item_type: string;
    title: string;
    description: string;
    priority: string;
    labels: string[];
    lifecycle: { status: string; version: number; active_worktree_id: string | null };
  };
  element: GroupCanvasElement;
  correlation_id: string;
}

export class GroupCanvasApiError extends Error {
  constructor(readonly status: number, readonly code: string) {
    super(code);
    this.name = "GroupCanvasApiError";
  }
}

export class GroupCanvasApiClient {
  constructor(private readonly baseUrl = "/api/group") {}

  private async request<T>(
    path: string,
    init: RequestInit = {},
    idempotencyKey?: string,
  ): Promise<T> {
    const headers = new Headers(init.headers);
    headers.set("Accept", "application/json");
    if (init.body && !headers.has("Content-Type")) {
      headers.set("Content-Type", "application/json");
    }
    if (idempotencyKey !== undefined) {
      if (!idempotencyKey.trim()) {
        throw new GroupCanvasApiError(400, "missing_idempotency_key");
      }
      headers.set("Idempotency-Key", idempotencyKey);
    }

    const apiBaseUrl = this.baseUrl.replace(/\/+$/, "");
    const response = await fetch(`${apiBaseUrl}${path}`, {
      ...init,
      headers,
      credentials: "same-origin",
      cache: "no-store",
    });
    if (!response.ok) {
      const payload = await response.json().catch(() => null) as
        | { error?: { code?: unknown } }
        | null;
      const code = typeof payload?.error?.code === "string"
        ? payload.error.code
        : `http_${response.status}`;
      throw new GroupCanvasApiError(response.status, code);
    }
    if (response.status === 204) return undefined as T;
    return await response.json() as T;
  }

  listCanvases(worktreeId: string, signal?: AbortSignal): Promise<GroupCanvasListResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases`,
      { signal },
    );
  }

  createCanvas(
    worktreeId: string,
    title: string,
    write: CanvasWriteContext,
  ): Promise<CreateCanvasResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases`,
      {
        method: "POST",
        body: JSON.stringify({ title, correlation_id: write.correlationId }),
      },
      write.idempotencyKey,
    );
  }

  updateDocument(
    worktreeId: string,
    canvasId: string,
    document: UpdateCanvasDocumentInput,
    write: CanvasWriteContext,
  ): Promise<UpdateCanvasDocumentResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/document`,
      { method: "PUT", body: JSON.stringify(document) },
      write.idempotencyKey,
    );
  }

  listEvents(
    worktreeId: string,
    afterOffset = "0",
    limit = 50,
  ): Promise<GroupCanvasEventsResponse> {
    const query = new URLSearchParams({ after_offset: afterOffset, limit: String(limit) });
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvas-events?${query.toString()}`,
    );
  }

  createWorkItem(
    worktreeId: string,
    canvasId: string,
    input: CreateCanvasWorkItemInput,
    write: CanvasWriteContext,
  ): Promise<CreateCanvasWorkItemResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/work-items`,
      {
        method: "POST",
        body: JSON.stringify({
          ...input,
          work_item: {
            ...input.work_item,
            correlation_id: input.work_item.correlation_id ?? write.correlationId,
          },
        }),
      },
      write.idempotencyKey,
    );
  }

  listElements(worktreeId: string, canvasId: string, signal?: AbortSignal): Promise<CanvasElementsResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements`,
      { signal },
    );
  }

  createElement(
    worktreeId: string,
    canvasId: string,
    element: GroupCanvasElementInput,
    write: CanvasWriteContext,
  ): Promise<CanvasMutationResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements`,
      {
        method: "POST",
        body: JSON.stringify({ ...element, correlation_id: write.correlationId ?? element.correlation_id }),
      },
      write.idempotencyKey,
    );
  }

  updateElement(
    worktreeId: string,
    canvasId: string,
    elementId: string,
    update: GroupCanvasElementUpdate,
    write: CanvasWriteContext,
  ): Promise<CanvasMutationResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements/${encodeURIComponent(elementId)}`,
      {
        method: "PUT",
        body: JSON.stringify({ ...update, correlation_id: write.correlationId ?? update.correlation_id }),
      },
      write.idempotencyKey,
    );
  }

  deleteElement(
    worktreeId: string,
    canvasId: string,
    elementId: string,
    expectedVersion: number,
    write: CanvasWriteContext,
  ): Promise<CanvasMutationResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements/${encodeURIComponent(elementId)}`,
      {
        method: "DELETE",
        body: JSON.stringify({ expected_version: expectedVersion, correlation_id: write.correlationId }),
      },
      write.idempotencyKey,
    );
  }

  setEntityRef(
    worktreeId: string,
    canvasId: string,
    elementId: string,
    entityRef: Omit<GroupCanvasEntityRef, "version">,
    expectedVersion: number,
    write: CanvasWriteContext,
  ): Promise<CanvasMutationResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements/${encodeURIComponent(elementId)}/entity-ref`,
      {
        method: "PUT",
        body: JSON.stringify({
          ...entityRef,
          expected_version: expectedVersion,
          correlation_id: write.correlationId,
        }),
      },
      write.idempotencyKey,
    );
  }

  clearEntityRef(
    worktreeId: string,
    canvasId: string,
    elementId: string,
    expectedVersion: number,
    write: CanvasWriteContext,
  ): Promise<CanvasMutationResponse> {
    return this.request(
      `/api/v1/worktrees/${encodeURIComponent(worktreeId)}/canvases/${encodeURIComponent(canvasId)}/elements/${encodeURIComponent(elementId)}/entity-ref`,
      {
        method: "DELETE",
        body: JSON.stringify({ expected_version: expectedVersion, correlation_id: write.correlationId }),
      },
      write.idempotencyKey,
    );
  }
}
