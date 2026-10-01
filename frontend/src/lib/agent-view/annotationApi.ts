// =====================================================================
// /agent-view annotation persistence (per 2026-10-01 OOB 恢复无限画布 — 任务 #1)
// =====================================================================
// 设计动机:
//  - 跟前端 store 模式一致 (zustand-style 持久化)
//  - 后端需独立 ticket (agent-view annotationApi)
//  - key 范围 = currentAgentId (每个 agent 独立 annotation 空间)
// =====================================================================

import type {
  AgentCanvasAnnotation, AgentCanvasFreeConnector,
} from "@/lib/agent-view/types";

const STORAGE_KEY_PREFIX = "star-agent-view-annotations:";

function keyFor(agentId: string): string {
  return STORAGE_KEY_PREFIX + agentId;
}

interface PersistedState {
  annotations: AgentCanvasAnnotation[];
  freeConnectors: AgentCanvasFreeConnector[];
  savedAt: string;
}

export function loadAgentAnnotations(agentId: string): PersistedState {
  if (typeof window === "undefined") return { annotations: [], freeConnectors: [], savedAt: "" };
  try {
    const raw = window.localStorage.getItem(keyFor(agentId));
    if (!raw) return { annotations: [], freeConnectors: [], savedAt: "" };
    const parsed = JSON.parse(raw) as PersistedState;
    if (!parsed || !Array.isArray(parsed.annotations) || !Array.isArray(parsed.freeConnectors)) {
      return { annotations: [], freeConnectors: [], savedAt: "" };
    }
    return parsed;
  } catch {
    // localStorage 损坏/不可用 — 优雅降级
    return { annotations: [], freeConnectors: [], savedAt: "" };
  }
}

export function saveAgentAnnotations(
  agentId: string,
  annotations: AgentCanvasAnnotation[],
  freeConnectors: AgentCanvasFreeConnector[],
): void {
  if (typeof window === "undefined") return;
  try {
    const payload: PersistedState = {
      annotations,
      freeConnectors,
      savedAt: new Date().toISOString(),
    };
    window.localStorage.setItem(keyFor(agentId), JSON.stringify(payload));
  } catch {
    // quota exceeded — 静默失败 (UI 可继续工作)
  }
}

export function clearAgentAnnotations(agentId: string): void {
  if (typeof window === "undefined") return;
  window.localStorage.removeItem(keyFor(agentId));
}

/** 测试/迁移用 — 列出所有保存的 agent-id (返回 key 后缀) */
export function listPersistedAgentIds(): string[] {
  if (typeof window === "undefined") return [];
  const out: string[] = [];
  for (let i = 0; i < window.localStorage.length; i += 1) {
    const key = window.localStorage.key(i);
    if (key && key.startsWith(STORAGE_KEY_PREFIX)) {
      out.push(key.slice(STORAGE_KEY_PREFIX.length));
    }
  }
  return out;
}
