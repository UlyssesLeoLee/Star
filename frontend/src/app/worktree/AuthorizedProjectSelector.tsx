/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/worktree/AuthorizedProjectSelector.tsx",type:"file",language:"tsx"}),
  (selector:Function {name:"AuthorizedProjectSelector",type:"function",signature:"AuthorizedProjectSelector(props): JSX.Element",visibility:"public",complexity:"complex"}),
  (projectAccess:Class {name:"AuthorizedProjectAccess",type:"interface",language:"typescript",visibility:"public"}),
  (directoryState:Class {name:"AuthorizedProjectDirectoryState",type:"interface",language:"typescript",visibility:"public"}),
  (mapPage:Function {name:"mapAuthorizedProjectPage",type:"function",visibility:"private",complexity:"moderate"}),
  (loadPage:Function {name:"loadAuthorizedProjects",type:"function",visibility:"private",complexity:"complex"}),
  (loadMore:Function {name:"loadMoreAuthorizedProjects",type:"function",visibility:"private",complexity:"moderate"}),
  (projectOptions:Variable {name:"authorizedProjectOptions",type:"variable",language:"typescript"}),
  (file)-[:CONTAINS]->(selector),(file)-[:CONTAINS]->(projectAccess),(file)-[:CONTAINS]->(directoryState),(file)-[:CONTAINS]->(mapPage),(file)-[:CONTAINS]->(loadPage),(file)-[:CONTAINS]->(loadMore),(file)-[:CONTAINS]->(projectOptions),
  (selector)-[:CALLS]->(loadPage),(selector)-[:CALLS]->(loadMore),(selector)-[:CALLS]->(projectOptions),(loadPage)-[:CALLS]->(mapPage),(loadMore)-[:CALLS]->(mapPage);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (selector:Function {name:"AuthorizedProjectSelector"}),
      (listProjects:Function {name:"WorktreeGroupApiClient.listAuthorizedProjects"});
CREATE (selector)-[:CALLS]->(listProjects);
*/

"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { WorktreeGroupApiClient } from "@/lib/group/worktreeGroupApi";

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const PROJECT_ROLES = new Set(["tenant_admin", "project_admin", "developer", "viewer", "agent"]);
const PAGE_LIMIT = 200;
const EMPTY_DIRECTORY_STATE: AuthorizedProjectDirectoryState = {
  mode: "loading", projects: [], next_cursor: null, loading_more: false,
};

export interface AuthorizedProjectAccess {
  project_id: string;
  role: string;
}

export interface AuthorizedProjectDirectoryState {
  mode: "loading" | "ready" | "error";
  projects: AuthorizedProjectAccess[];
  next_cursor: string | null;
  loading_more: boolean;
  error?: string;
}

interface AuthorizedProjectsEnvelope {
  projects: unknown[];
  limit: number;
  next_cursor: string | null;
}

interface AuthorizedProjectSelectorProps {
  api: WorktreeGroupApiClient;
  selectedProjectId: string;
  onSelect(projectId: string): void;
  onStateChange(state: AuthorizedProjectDirectoryState): void;
}

function mapAuthorizedProjectPage(raw: unknown, existing: AuthorizedProjectAccess[]): AuthorizedProjectsEnvelope & { projects: AuthorizedProjectAccess[] } {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) throw new Error("授权 Project 目录响应无效");
  const response = raw as Record<string, unknown>;
  if (Object.keys(response).some((key) => !["projects", "limit", "next_cursor"].includes(key))) {
    throw new Error("授权 Project 目录响应包含未识别字段");
  }
  if (!Array.isArray(response.projects) || !Number.isSafeInteger(response.limit) || (response.limit as number) < 1 || (response.limit as number) > PAGE_LIMIT) {
    throw new Error("授权 Project 目录分页信息无效");
  }
  if (response.projects.length > (response.limit as number) || response.projects.length > PAGE_LIMIT) {
    throw new Error("授权 Project 目录超出请求页大小");
  }
  const seen = new Set(existing.map((project) => project.project_id.toLowerCase()));
  const projects = response.projects.map((rawProject) => {
    if (!rawProject || typeof rawProject !== "object" || Array.isArray(rawProject)) throw new Error("授权 Project 条目无效");
    const project = rawProject as Record<string, unknown>;
    if (Object.keys(project).some((key) => !["project_id", "role"].includes(key))) {
      throw new Error("授权 Project 条目包含未识别字段");
    }
    const projectId = project.project_id;
    const role = project.role;
    if (typeof projectId !== "string" || !UUID_PATTERN.test(projectId) || typeof role !== "string" || !PROJECT_ROLES.has(role)) {
      throw new Error("授权 Project 条目包含无效 ID 或角色");
    }
    const normalizedId = projectId.toLowerCase();
    if (seen.has(normalizedId)) throw new Error("授权 Project 目录包含重复条目");
    seen.add(normalizedId);
    return { project_id: projectId, role };
  });
  const nextCursor = response.next_cursor;
  if (nextCursor !== null && (typeof nextCursor !== "string" || !UUID_PATTERN.test(nextCursor) || projects.at(-1)?.project_id.toLowerCase() !== nextCursor.toLowerCase())) {
    throw new Error("授权 Project 目录游标与当前页不匹配");
  }
  return { projects, limit: response.limit as number, next_cursor: nextCursor as string | null };
}

function apiErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  return "授权 Project 目录暂时不可用";
}

export function AuthorizedProjectSelector({ api, selectedProjectId, onSelect, onStateChange }: AuthorizedProjectSelectorProps) {
  const [state, setState] = useState<AuthorizedProjectDirectoryState>({
    mode: "loading", projects: [], next_cursor: null, loading_more: false,
  });
  const requestSequence = useRef(0);
  const stateApi = useRef(api);
  const stateMatchesApi = stateApi.current === api;
  const visibleState = stateMatchesApi ? state : EMPTY_DIRECTORY_STATE;
  const stateRef = useRef(state);
  stateRef.current = state;

  const loadAuthorizedProjects = useCallback(async (cursor?: string, append = false) => {
    const sequence = ++requestSequence.current;
    setState((current) => append
      ? { ...current, loading_more: true, error: undefined }
      : { mode: "loading", projects: [], next_cursor: null, loading_more: false });
    try {
      const response = await api.listAuthorizedProjects<unknown>({ limit: PAGE_LIMIT, cursor });
      if (sequence !== requestSequence.current) return;
      const currentProjects = append ? stateRef.current.projects : [];
      const page = mapAuthorizedProjectPage(response, currentProjects);
      const nextState: AuthorizedProjectDirectoryState = {
        mode: "ready",
        projects: [...currentProjects, ...page.projects],
        next_cursor: page.next_cursor,
        loading_more: false,
      };
      setState(nextState);
    } catch (error) {
      if (sequence !== requestSequence.current) return;
      setState((current) => append
        ? { ...current, loading_more: false, error: apiErrorMessage(error) }
        : { mode: "error", projects: [], next_cursor: null, loading_more: false, error: apiErrorMessage(error) });
    }
  }, [api]);

  useEffect(() => {
    setState({ mode: "loading", projects: [], next_cursor: null, loading_more: false });
    stateApi.current = api;
    void loadAuthorizedProjects();
    return () => { requestSequence.current += 1; };
  }, [loadAuthorizedProjects]);

  useEffect(() => onStateChange(visibleState), [onStateChange, visibleState]);

  const loadMoreAuthorizedProjects = useCallback(() => {
    if (visibleState.next_cursor && !visibleState.loading_more) {
      void loadAuthorizedProjects(visibleState.next_cursor, true);
    }
  }, [loadAuthorizedProjects, visibleState.loading_more, visibleState.next_cursor]);

  const projectOptions = visibleState.projects.map((project) => (
    <option key={project.project_id} value={project.project_id}>Project {project.project_id}</option>
  ));

  return (
    <>
      <select
        id="worktree-project"
        value={selectedProjectId}
        onChange={(event) => onSelect(event.target.value)}
        className="rounded-md border border-line bg-bg-soft px-3 py-2 text-sm"
        data-testid="worktree-project-selector"
        disabled={visibleState.mode === "loading"}
      >
        <option value="" disabled>{visibleState.mode === "loading" ? "正在读取授权 Project…" : "选择 Project"}</option>
        {projectOptions}
      </select>
      {visibleState.mode === "loading" && <span role="status" className="text-xs text-ink-mute">正在读取当前用户授权的 Project…</span>}
      {visibleState.mode === "error" && (
        <span role="alert" className="text-xs text-warning">
          授权 Project 目录读取失败：{visibleState.error}。不会回退到本地 seed。
          <button type="button" className="ml-2 underline" onClick={() => void loadAuthorizedProjects()}>重试</button>
        </span>
      )}
      {visibleState.mode === "ready" && visibleState.projects.length === 0 && (
        <span role="status" className="text-xs text-ink-mute">当前用户没有可访问的 Project。</span>
      )}
      {visibleState.error && visibleState.mode === "ready" && <span role="alert" className="text-xs text-warning">下一页读取失败：{visibleState.error}</span>}
      {visibleState.next_cursor && (
        <button type="button" className="btn btn-sm" onClick={loadMoreAuthorizedProjects} disabled={visibleState.loading_more}>
          {visibleState.loading_more ? "正在加载…" : "加载更多 Project"}
        </button>
      )}
    </>
  );
}
