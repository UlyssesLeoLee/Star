/*
@cypher schema=1 source_sha256=9f30e5f18885de1c0582ab73dbc33125b5a4dac1faee924064b76f4f52b78b61
MERGE (self:File {path:"frontend/src/lib/agent/settings.ts"})
MERGE (agentSettings:Type {id:"frontend/src/lib/agent/settings.ts::AgentSettings", kind:"interface"})
MERGE (agentSession:File {path:"frontend/src/types/ids.ts"})
MERGE (createInitial:Function {name:"createInitialAgentSettings", language:"typescript", visibility:"public"})
MERGE (validateSettings:Function {name:"validateSettings", language:"typescript", visibility:"public"})
MERGE (modelOptionsFor:Function {name:"modelOptionsFor", language:"typescript", visibility:"public"})
MERGE (self)-[:DEFINES]->(agentSettings)
MERGE (self)-[:IMPORTS]->(agentSession)
MERGE (self)-[:DEFINES]->(createInitial)
MERGE (self)-[:DEFINES]->(validateSettings)
MERGE (self)-[:DEFINES]->(modelOptionsFor)
@endcypher
*/

// =====================================================================
// Agent Settings — local configuration draft
// =====================================================================
// The current view is only a local draft. It does not execute an Agent or call a model.
//
// Credentials must be wired to a secure provider before this draft can be used for execution.
// =====================================================================

import type { AgentSession } from "@/types/ids";

/** Agent 设置 (每个 agent 1 份, 跟 store 同步) */
export interface AgentSettings {
  agentId: string;
/** Configured API credential; unavailable until secure storage is integrated. */
  apiKey: string;
  /** model (claude-sonnet / gpt-4o / codex / internal-vibe-coder) */
  model: string;
  /** max tokens per request */
  maxTokens: number;
  /** temperature 0..2 */
  temperature: number;
  /** system prompt (per agent) */
  systemPrompt: string;
  /** base URL (per LLM provider) */
  baseUrl: string;
  /** 是否启用 */
  enabled: boolean;
  /** last update ISO 8601 */
  updatedAt: string;
}

/** 默认 settings (per agent_kind 推荐 model + base URL) */
export const DEFAULT_SETTINGS: Record<string, Partial<AgentSettings>> = {
  "claude-sonnet": {
    model: "claude-sonnet-4.5",
    baseUrl: "https://api.anthropic.com/v1",
    maxTokens: 8000,
    temperature: 0.7,
    systemPrompt: "You are Claude, a helpful AI assistant built by Anthropic.",
  },
  "gpt-4o": {
    model: "gpt-4o",
    baseUrl: "https://api.openai.com/v1",
    maxTokens: 8000,
    temperature: 0.7,
    systemPrompt: "You are a helpful assistant.",
  },
  "codex": {
    model: "codex-1",
    baseUrl: "https://api.openai.com/v1",
    maxTokens: 16000,
    temperature: 0.2,
    systemPrompt: "You are Codex, a code-specialized AI.",
  },
  "internal-vibe-coder": {
    model: "vibe-coder-1",
    baseUrl: "https://internal.star.local/v1",
    maxTokens: 4000,
    temperature: 0.5,
    systemPrompt: "You are Vibe Coder, Star internal AI agent.",
  },
};

/** 初始化 1 个 agent 的 settings (per agent_kind) */
export function createInitialAgentSettings(agent: AgentSession): AgentSettings {
  const defaults = DEFAULT_SETTINGS[agent.agent_kind] ?? {};
  const now = new Date().toISOString();
  return {
    agentId: agent.id,
    apiKey: "", // Credentials are unavailable until secure storage is integrated.
    model: defaults.model ?? agent.agent_kind,
    baseUrl: defaults.baseUrl ?? "https://api.example.com/v1",
    maxTokens: defaults.maxTokens ?? 4000,
    temperature: defaults.temperature ?? 0.7,
    systemPrompt: defaults.systemPrompt ?? "You are a helpful AI agent.",
    enabled: true,
    updatedAt: now,
  };
}

/** 校验 settings (per 拍板, 保存前必检) */
export function validateSettings(s: AgentSettings): { ok: boolean; errors: string[] } {
  const errors: string[] = [];
  if (s.maxTokens < 1 || s.maxTokens > 200000) errors.push(`maxTokens out of range (1..200000), got ${s.maxTokens}`);
  if (s.temperature < 0 || s.temperature > 2) errors.push(`temperature out of range (0..2), got ${s.temperature}`);
  if (s.baseUrl.length < 1) errors.push("baseUrl empty");
  if (!s.baseUrl.startsWith("http://") && !s.baseUrl.startsWith("https://")) errors.push("baseUrl must start with http(s)://");
  return { ok: errors.length === 0, errors };
}

/** 已知 model 列表 (per agent_kind 推) */
export const MODEL_OPTIONS: Record<string, string[]> = {
  "claude-sonnet": ["claude-sonnet-4.5", "claude-sonnet-4", "claude-3-5-sonnet"],
  "gpt-4o": ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
  "codex": ["codex-1", "codex-mini"],
  "internal-vibe-coder": ["vibe-coder-1", "vibe-coder-fast"],
};

/** 取出可用的 model 列表 (per agent_kind) */
export function modelOptionsFor(agentKind: string): string[] {
  return MODEL_OPTIONS[agentKind] ?? [agentKind];
}
