/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group/groupProjection.tsx",type:"file",language:"tsx"}),
  (provider:Function {name:"WorktreeGroupApiProvider",type:"function",signature:"WorktreeGroupApiProvider({children,getAccessToken,sessionKey,baseUrl,fetcher})",visibility:"public",complexity:"simple"}),
  (useApi:Function {name:"useWorktreeGroupApi",type:"function",signature:"useWorktreeGroupApi()",visibility:"public",complexity:"simple"}),
  (useProjection:Function {name:"useWorktreeGroupProjection",type:"function",signature:"useWorktreeGroupProjection(worktreeId, subscribeCanvas, refreshKey, selectedCanvasId)",visibility:"public",complexity:"complex"}),
  (load:Function {name:"loadProjection",type:"function",signature:"loadProjection(client, worktreeId, includeCanvas, selectedCanvasId)",visibility:"private",complexity:"complex"}),
  (mapWorkItems:Function {name:"mapWorkItems",type:"function",signature:"mapWorkItems(response, worktreeId)",visibility:"private",complexity:"moderate"}),
  (mapCanvas:Function {name:"mapCanvas",type:"function",signature:"mapCanvas(row, worktreeId)",visibility:"private",complexity:"moderate"}),
  (canvasRows:Variable {name:"canvasRows",type:"variable",language:"typescript"}),
  (projectedCanvases:Variable {name:"projectedCanvases",type:"variable",language:"typescript"}),
  (mapElements:Function {name:"mapElements",type:"function",signature:"mapElements(response, canvasId, worktreeId)",visibility:"private",complexity:"moderate"}),
  (mapWorktree:Function {name:"mapWorktree",type:"function",signature:"mapWorktree(response, worktreeId)",visibility:"private",complexity:"simple"}),
  (file)-[:CONTAINS]->(provider),(file)-[:CONTAINS]->(useApi),(file)-[:CONTAINS]->(useProjection),(file)-[:CONTAINS]->(load),(file)-[:CONTAINS]->(mapWorkItems),(file)-[:CONTAINS]->(mapCanvas),(file)-[:CONTAINS]->(canvasRows),(file)-[:CONTAINS]->(projectedCanvases),(file)-[:CONTAINS]->(mapElements),(file)-[:CONTAINS]->(mapWorktree),
  (provider)-[:CREATES]->(client:Class {name:"WorktreeGroupApiClient",type:"class"}),(useApi)-[:READS]->(provider),(useProjection)-[:CALLS]->(load),(useProjection)-[:CALLS]->(poller:Class {name:"CanvasOutboxPoller",type:"class"}),(load)-[:CALLS]->(mapWorktree),(load)-[:CALLS]->(mapWorkItems),(load)-[:CALLS]->(mapCanvas),(load)-[:USES]->(canvasRows),(load)-[:USES]->(projectedCanvases),(load)-[:CALLS]->(mapElements);
*/

"use client";

import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";

import type { CanvasConnector, CanvasElement, CanvasFrame, CanvasViewport, WorkItemKind, WorkItemPriority, WorkItemStatus, WorktreeStatus } from "@/types/ids";
import {
  CanvasOutboxPoller,
  GroupApiError,
  WorktreeGroupApiClient,
  type GroupAccessTokenProvider,
} from "./worktreeGroupApi";

const GroupApiContext = createContext<WorktreeGroupApiClient | null>(null);
const clientIdentities = new WeakMap<WorktreeGroupApiClient, number>();
let nextClientIdentity = 1;

export function WorktreeGroupApiProvider({
  children,
  getAccessToken,
  sessionKey,
  baseUrl,
  fetcher,
}: {
  children: ReactNode;
  getAccessToken: GroupAccessTokenProvider;
  /** Non-secret session generation; change it whenever auth principal/session changes. */
  sessionKey: string;
  baseUrl?: string;
  fetcher?: typeof fetch;
}) {
  const client = useMemo(
    () => {
      if (!sessionKey.trim()) throw new Error("Group API provider requires a non-secret session generation key.");
      return new WorktreeGroupApiClient(getAccessToken, fetcher ?? fetch, baseUrl);
    },
    [getAccessToken, baseUrl, fetcher, sessionKey],
  );
  return <GroupApiContext.Provider value={client}>{children}</GroupApiContext.Provider>;
}

export function useWorktreeGroupApi(): WorktreeGroupApiClient | null {
  return useContext(GroupApiContext);
}

export interface GroupWorktreeView {
  id: string;
  project_id: string;
  name: string;
  branch: string;
  status: WorktreeStatus;
}

export interface GroupWorkItemView {
  id: string;
  project_id: string;
  worktree_id: string;
  key: string;
  title: string;
  description: string;
  kind: WorkItemKind;
  status: WorkItemStatus;
  priority: WorkItemPriority;
  lifecycle_status: WorkItemLifecycleStatus;
  lifecycle_version: number;
  review_state: string;
  active_worktree_id: string | null;
  labels: string[];
  reporter_id: string;
}

export type WorkItemLifecycleStatus = "pending" | "claimed" | "in_progress" | "completed" | "failed" | "cancelled";

export interface GroupCanvasView {
  id: string;
  version: number;
  title: string;
  ref_kind: "worktree";
  ref_id: string;
  viewport: CanvasViewport;
  frames: CanvasFrame[];
}

export interface WorktreeGroupProjection {
  worktree: GroupWorktreeView;
  project_id: string;
  work_items: GroupWorkItemView[];
  canvases: GroupCanvasView[];
  canvas?: GroupCanvasView;
  elements: Array<Pick<CanvasElement, "id" | "canvas_id" | "kind" | "x" | "y" | "width" | "height" | "rotation" | "z_index" | "entity_ref" | "content" | "locked" | "hidden"> & { version: number }>;
  connectors: CanvasConnector[];
}

export type WorktreeGroupProjectionState =
  | { mode: "preview" }
  | { mode: "loading" }
  | { mode: "error"; message: string }
  | ({ mode: "live"; refreshError?: string } & WorktreeGroupProjection);

export function useWorktreeGroupProjection(
  worktreeId: string,
  subscribeCanvas: boolean,
  refreshKey = 0,
  selectedCanvasId?: string,
): WorktreeGroupProjectionState {
  const client = useWorktreeGroupApi();
  if (client && !clientIdentities.has(client)) clientIdentities.set(client, nextClientIdentity++);
  const requestKey = client ? `${clientIdentities.get(client)}:${worktreeId}:${refreshKey}:${selectedCanvasId ?? ""}` : "preview";
  const [internalState, setInternalState] = useState<{ requestKey: string; state: WorktreeGroupProjectionState }>(
    { requestKey: "preview", state: { mode: "preview" } },
  );

  const publish = (next: WorktreeGroupProjectionState | ((current: WorktreeGroupProjectionState) => WorktreeGroupProjectionState)) => {
    setInternalState((current) => {
      const base = current.requestKey === requestKey ? current.state : { mode: "loading" as const };
      return { requestKey, state: typeof next === "function" ? next(base) : next };
    });
  };

  useEffect(() => {
    if (!client) {
      publish({ mode: "preview" });
      return;
    }

    const controller = new AbortController();
    let active = true;
    publish({ mode: "loading" });

    const refresh = async () => {
      const projection = await loadProjection(client, worktreeId, subscribeCanvas, selectedCanvasId);
      if (active) publish({ mode: "live", ...projection });
      return projection;
    };

    void refresh().then((projection) => {
      if (!active || !subscribeCanvas || !projection.canvas) return;
      const poller = new CanvasOutboxPoller(
        client,
        worktreeId,
        projection.canvas.id,
        async () => { await refresh(); },
      );
      void poller.run(controller.signal, (error) => {
        if (!active) return;
        const message = errorMessage(error);
        publish((current) => {
          if (error instanceof GroupApiError && (error.status === 401 || error.status === 403)) {
            return { mode: "error", message };
          }
          return current.mode === "live" ? { ...current, refreshError: message } : current;
        });
      });
    }).catch((error: unknown) => {
      if (active) publish({ mode: "error", message: errorMessage(error) });
    });

    return () => {
      active = false;
      controller.abort();
    };
  }, [client, requestKey, refreshKey, selectedCanvasId, subscribeCanvas, worktreeId]);

  if (!client) return { mode: "preview" };
  return internalState.requestKey === requestKey ? internalState.state : { mode: "loading" };
}

async function loadProjection(
  client: WorktreeGroupApiClient,
  worktreeId: string,
  includeCanvas: boolean,
  selectedCanvasId?: string,
): Promise<WorktreeGroupProjection> {
  const [context, workItems] = await Promise.all([
    client.getGroupContext<GroupContextEnvelope>(worktreeId),
    client.listWorkItems<WorkItemEnvelope>(worktreeId),
  ]);
  const worktree = mapWorktree(context, worktreeId);
  if (workItems.worktree_id !== worktreeId) {
    throw new Error("Group API returned a projection outside the active Worktree.");
  }
  if (workItems.project_id !== worktree.project_id) {
    throw new Error("Group API returned mismatched Project scope.");
  }
  const canvases = includeCanvas ? await client.listCanvases<CanvasEnvelope>(worktreeId) : undefined;
  if (canvases && (canvases.worktree_id !== worktreeId || canvases.project_id !== worktree.project_id)) {
    throw new Error("Group API returned Canvas data outside the active Project or Worktree.");
  }
  const canvasRows = canvases?.canvases ?? [];
  const projectedCanvases = canvasRows.map((row) => mapCanvas(row, worktreeId));
  const selectedCanvas = selectedCanvasId
    ? projectedCanvases.find((item) => item.id === selectedCanvasId)
    : undefined;
  const canvas = selectedCanvas ?? projectedCanvases[0];
  const canvasRow = canvasRows.find((row) => row.canvas_id === canvas?.id);
  const elementResponse = canvas
    ? await client.listCanvasElements<ElementEnvelope>(worktreeId, canvas.id)
    : undefined;
  if (elementResponse && elementResponse.worktree_id !== worktreeId) {
    throw new Error("Group API returned Canvas elements outside the active Worktree.");
  }
  return {
    worktree,
    project_id: worktree.project_id,
    work_items: mapWorkItems(workItems, worktreeId),
    canvases: projectedCanvases,
    canvas,
    elements: canvas && elementResponse ? mapElements(elementResponse, canvas.id, worktreeId) : [],
    connectors: canvasRow ? canvasConnectors(canvasRow) : [],
  };
}

interface GroupContextEnvelope {
  group_context: { worktree_id: string; project_id: string };
  worktree: Record<string, unknown>;
}

interface WorkItemEnvelope {
  worktree_id: string;
  project_id: string;
  work_items: Array<Record<string, unknown>>;
}

interface CanvasEnvelope {
  worktree_id: string;
  project_id: string;
  canvases: Array<Record<string, unknown>>;
}

interface ElementEnvelope {
  worktree_id: string;
  canvas_id: string;
  elements: Array<Record<string, unknown>>;
}

function mapWorktree(envelope: GroupContextEnvelope, expectedId: string): GroupWorktreeView {
  const raw = envelope.worktree;
  const id = requiredString(raw.id, "worktree.id");
  if (id !== expectedId || envelope.group_context.worktree_id !== expectedId) {
    throw new Error("GroupContext does not match the requested Worktree.");
  }
  const allowedStatuses: WorktreeStatus[] = ["initializing", "cloning", "syncing", "active", "dirty", "behind", "diverged", "conflict", "committing", "pushing", "ci_running", "review_requested", "merged", "closed", "abandoned", "archived", "reverted"];
  const humanState = typeof raw.human_state === "string" ? raw.human_state : "";
  const status = raw.archived === true ? "archived" : allowedStatuses.includes(humanState as WorktreeStatus) ? humanState as WorktreeStatus : "active";
  const projectId = requiredString(raw.project_id, "worktree.project_id");
  if (envelope.group_context.project_id !== projectId) {
    throw new Error("GroupContext Project does not match the requested Worktree.");
  }
  return {
    id,
    project_id: projectId,
    name: requiredString(raw.name, "worktree.name"),
    branch: requiredString(raw.branch, "worktree.branch"),
    status,
  };
}

function mapWorkItems(response: WorkItemEnvelope, worktreeId: string): GroupWorkItemView[] {
  return response.work_items.map((item) => {
    const id = requiredString(item.work_item_id, "work_item.work_item_id");
    const lifecycle = asRecord(item.lifecycle, "work_item.lifecycle");
    const statuses: Record<string, WorkItemStatus> = {
      pending: "todo", claimed: "in_progress", in_progress: "in_progress",
      completed: "done", failed: "blocked", cancelled: "wontfix",
    };
    const priorities: Record<string, WorkItemPriority> = { low: "p3", medium: "p2", high: "p1", urgent: "p0" };
    const itemType = requiredString(item.item_type, "work_item.item_type");
    const kind: WorkItemKind = ["story", "task", "bug", "spike", "epic"].includes(itemType)
      ? itemType as WorkItemKind
      : itemType === "epic" ? "epic" : "task";
    const status = statuses[String(lifecycle.status)];
    const priority = priorities[String(item.priority)];
    const lifecycleVersion = Number(lifecycle.version);
    const activeWorktreeId = lifecycle.active_worktree_id;
    if (!status || !priority || !Number.isSafeInteger(lifecycleVersion) || lifecycleVersion < 1) {
      throw new Error("Group API returned an unsupported WorkItem state.");
    }
    return {
      id,
      project_id: response.project_id,
      worktree_id: worktreeId,
      key: id,
      title: requiredString(item.title, "work_item.title"),
      description: typeof item.description === "string" ? item.description : "",
      kind,
      status,
      priority,
      lifecycle_status: String(lifecycle.status) as WorkItemLifecycleStatus,
      lifecycle_version: lifecycleVersion,
      review_state: typeof lifecycle.review_state === "string" ? lifecycle.review_state : "none",
      active_worktree_id: typeof activeWorktreeId === "string" ? activeWorktreeId : null,
      labels: Array.isArray(item.labels) ? item.labels.filter((label): label is string => typeof label === "string") : [],
      reporter_id: requiredString(item.reporter_user_id, "work_item.reporter_user_id"),
    };
  });
}

function mapCanvas(row: Record<string, unknown>, worktreeId: string): GroupCanvasView {
  const document = asRecord(row.document, "canvas.document");
  const viewport = asRecord(document.viewport, "canvas.document.viewport");
  if (!Array.isArray(document.frames)) throw new Error("Canvas document frames are missing.");
  return {
    id: requiredString(row.canvas_id, "canvas.canvas_id"),
    version: finiteNumber(row.version, "canvas.version"),
    title: requiredString(row.title, "canvas.title"),
    ref_kind: "worktree",
    ref_id: worktreeId,
    viewport: {
      x: finiteNumber(viewport.x, "canvas.viewport.x"),
      y: finiteNumber(viewport.y, "canvas.viewport.y"),
      zoom: finiteNumber(viewport.zoom, "canvas.viewport.zoom"),
    },
    frames: document.frames as CanvasFrame[],
  };
}

function canvasConnectors(row: Record<string, unknown>): CanvasConnector[] {
  const document = asRecord(row.document, "canvas.document");
  if (!Array.isArray(document.connectors)) throw new Error("Canvas document connectors are missing.");
  return document.connectors as CanvasConnector[];
}

function mapElements(response: ElementEnvelope, canvasId: string, worktreeId: string): WorktreeGroupProjection["elements"] {
  if (response.canvas_id !== canvasId) throw new Error("Canvas API returned elements from another Canvas.");
  return response.elements.map((raw) => {
    const entityRef = raw.entity_ref == null ? undefined : asRecord(raw.entity_ref, "element.entity_ref");
    if (entityRef && entityRef.worktree_id !== worktreeId) {
      throw new Error("Canvas EntityRef is outside the active Worktree.");
    }
    const version = finiteNumber(raw.version, "element.version");
    if (!Number.isSafeInteger(version) || version < 1) throw new Error("Invalid Group API response: element.version.");
    return {
      id: requiredString(raw.element_id, "element.element_id"),
      canvas_id: canvasId,
      version,
      kind: requiredString(raw.kind, "element.kind") as CanvasElement["kind"],
      x: finiteNumber(raw.x, "element.x"),
      y: finiteNumber(raw.y, "element.y"),
      width: finiteNumber(raw.width, "element.width"),
      height: finiteNumber(raw.height, "element.height"),
      rotation: finiteNumber(raw.rotation, "element.rotation"),
      z_index: finiteNumber(raw.z_index, "element.z_index"),
      entity_ref: entityRef ? {
        ref_type: requiredString(entityRef.ref_type, "element.entity_ref.ref_type") as NonNullable<CanvasElement["entity_ref"]>["ref_type"],
        ref_id: requiredString(entityRef.ref_id, "element.entity_ref.ref_id"),
        worktree_id: requiredString(entityRef.worktree_id, "element.entity_ref.worktree_id"),
      } : undefined,
      content: asRecord(raw.content, "element.content") as CanvasElement["content"],
      locked: raw.locked === true,
      hidden: raw.hidden === true,
    };
  });
}

function asRecord(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`Invalid Group API response: ${label}.`);
  return value as Record<string, unknown>;
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) throw new Error(`Invalid Group API response: ${label}.`);
  return value;
}

function finiteNumber(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`Invalid Group API response: ${label}.`);
  return value;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "Group API request failed.";
}
