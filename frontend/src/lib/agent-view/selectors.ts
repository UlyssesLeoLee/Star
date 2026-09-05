// =====================================================================
// /agent-view — selector helpers
// =====================================================================
// Per 2026-09-05 11:25 JST 拍板 #2: 默认选最近活跃的 active agent, 用户可手动
//   覆盖 (URL ?agent=ag-XXX 或 dropdown).
//
// 活跃定义: AgentStatus ∈ {
//   "queued", "spawning", "initializing",
//   "compiling_context", "planning", "executing",
//   "awaiting_feedback", "awaiting_human", "awaiting_tool",
//   "validating", "paused"
// }  (排除 completed / failed / cancelled, 这些是终态)
//
// 排序: started_at DESC (最近启动优先), 同 started_at 用 id 升序稳定
// =====================================================================

import type { AgentSession, AgentStatus, WorkItem, Worktree } from "@/types/ids";
import type { CurrentAgentResolution } from "./types";

/** 11 个 "active" 状态 (per 拍板 #2 定义) */
const ACTIVE_STATUSES: ReadonlySet<AgentStatus> = new Set<AgentStatus>([
  "queued",
  "spawning",
  "initializing",
  "compiling_context",
  "planning",
  "executing",
  "awaiting_feedback",
  "awaiting_human",
  "awaiting_tool",
  "validating",
  "paused",
]);

/** 单个 agent 是否算 "active" (用于默认选) */
export function isActiveAgent(a: AgentSession): boolean {
  return ACTIVE_STATUSES.has(a.status);
}

/**
 * 自动选最近活跃的 active agent.
 *   - 第一优先: active 且 started_at 最新的
 *   - 兜底: 任何 agent (按 started_at DESC), 即所有 agent 都终态时
 *   - 都空: 返回 null
 */
export function pickDefaultAgent(agents: ReadonlyArray<AgentSession>): AgentSession | null {
  if (agents.length === 0) return null;

  // 1) active agent 中找 started_at 最大
  const activeAgents = agents.filter(isActiveAgent);
  if (activeAgents.length > 0) {
    return [...activeAgents].sort(compareByStartedDescThenIdAsc)[0];
  }

  // 2) 兜底: 全部 agent 按 started_at DESC
  return [...agents].sort(compareByStartedDescThenIdAsc)[0];
}

/** 解析 URL 参数 + 自动 fallback */
export function resolveCurrentAgent(
  agents: ReadonlyArray<AgentSession>,
  urlAgentId: string | null,
): CurrentAgentResolution | null {
  if (agents.length === 0) return null;

  // URL 参数优先
  if (urlAgentId) {
    const found = agents.find((a) => a.id === urlAgentId);
    if (found) {
      return { agentId: found.id, agent: found, auto: false };
    }
    // URL 给了但找不到, fallback 到默认 + auto=true
  }

  const def = pickDefaultAgent(agents);
  if (!def) return null;
  return { agentId: def.id, agent: def, auto: true };
}

/** 取 agent 关联的 worktree (1:1) */
export function pickAgentWorktree(
  worktrees: ReadonlyArray<Worktree>,
  agent: AgentSession,
): Worktree | null {
  return worktrees.find((w) => w.id === agent.worktree_id) ?? null;
}

/**
 * 取 agent 关联的 work-items (per 拍板 "数据对应kanban等界面的情况").
 *
 * 关联规则 (per docs/frontend/design/dynamic-interaction-design.md §5 + ids.ts):
 *   - 优先: WorkItem.worktree_id === agent.worktree_id (同一 worktree = 同一 agent 责任域)
 *   - 兜底: WorkItem.assignee_id === agent 关联 identity (没有就空)
 *
 * 真实后端会通过 AgentSession 1:N 关联 WorkItem, 当前 mock 走 worktree_id.
 * 未来 DDD 演进: 改为 work_items.agent_session_id 直接 N:1 引用, 当前 schema 没这个字段.
 */
export function pickAgentWorkItems(
  workItems: ReadonlyArray<WorkItem>,
  agent: AgentSession,
  worktree: Worktree | null,
): WorkItem[] {
  if (!worktree) return [];
  return workItems.filter((w) => w.worktree_id === worktree.id);
}

// ---- internal ----

function compareByStartedDescThenIdAsc(a: AgentSession, b: AgentSession): number {
  // started_at 倒序 (newest first)
  if (a.started_at < b.started_at) return 1;
  if (a.started_at > b.started_at) return -1;
  // tie-breaker: id 升序稳定
  if (a.id < b.id) return -1;
  if (a.id > b.id) return 1;
  return 0;
}
