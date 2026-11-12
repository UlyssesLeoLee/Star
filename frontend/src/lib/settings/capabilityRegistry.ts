/*
@cypher schema=1 source_sha256=c71d22e0500d99347249031336ec24ede1e7a2e13cbe2f7a3f3b797f34b47228
MERGE (self:File {path:"frontend/src/lib/settings/capabilityRegistry.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/settings/capabilityRegistry.ts::useCapabilityRegistry",kind:"hook"})
MERGE (api:Symbol {id:"frontend/src/lib/settings/capabilityRegistry.ts::CapabilityRegistryApi",kind:"class"})
MERGE (seed:Symbol {id:"frontend/src/lib/settings/capabilityRegistry.ts::seedCapabilityRegistry",kind:"function"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(api)
MERGE (self)-[:DEFINES]->(seed)
@endcypher
*/
"use client";

// =====================================================================
// capabilityRegistry — Settings 能力注册表 (per WBS §23.33-B, 2026-11-13 用户发令)
//
// 用户发令: "设置里要有 MCP / Skill / Plugin 和类似 Claude Code 的 MCP 相关设置,
// 并且支持这些功能"。
//
// 模型对齐 Claude Code MCP 管理:
//   - 连接类型: stdio (本地子进程) / sse (远端 Server-Sent Events) / http (远端流式)
//   - 作用域: user (全局, ~/.claude.json) / project (项目级, .mcp.json)
//   - 生命周期: enabled/disabled + health (connecting/healthy/error)
//
// 数据层接口形状对齐未来后端 REST CRUD (v0.1+), web 无后端时降级 localStorage
// seed 数据, 保证面板可用可测。
// =====================================================================

import { useSyncExternalStore } from "react";

export type McpTransport = "stdio" | "sse" | "http";
export type CapabilityScope = "user" | "project";
export type CapabilityHealth = "unknown" | "connecting" | "healthy" | "error";

export interface McpServerConfig {
  id: string;
  name: string;
  transport: McpTransport;
  /** stdio: 命令+参数; sse/http: URL */
  command: string;
  args: string[];
  env: Record<string, string>;
  scope: CapabilityScope;
  enabled: boolean;
  health: CapabilityHealth;
  last_error: string | null;
  updated_at_ms: number;
}

export interface SkillEntry {
  id: string;
  name: string;
  version: string;
  scope: CapabilityScope;
  enabled: boolean;
  description: string;
}

export interface PluginEntry {
  id: string;
  name: string;
  version: string;
  scope: CapabilityScope;
  enabled: boolean;
  /** 权限清单 */
  permissions: string[];
  description: string;
}

export interface CapabilityRegistryState {
  mcpServers: McpServerConfig[];
  skills: SkillEntry[];
  plugins: PluginEntry[];
}

const STORAGE_KEY = "star-capability-registry:v1";

export const DEFAULT_CAPABILITY_REGISTRY: CapabilityRegistryState = {
  mcpServers: [
    {
      id: "mcp-seed-filesystem",
      name: "filesystem",
      transport: "stdio",
      command: "npx",
      args: ["-y", "@modelcontextprotocol/server-filesystem"],
      env: {},
      scope: "user",
      enabled: true,
      health: "unknown",
      last_error: null,
      updated_at_ms: 0,
    },
    {
      id: "mcp-seed-github",
      name: "github",
      transport: "http",
      command: "https://api.githubcopilot.com/mcp/",
      args: [],
      env: {},
      scope: "project",
      enabled: false,
      health: "unknown",
      last_error: null,
      updated_at_ms: 0,
    },
  ],
  skills: [
    {
      id: "skill-seed-code-review",
      name: "code-review",
      version: "1.0.0",
      scope: "user",
      enabled: true,
      description: "双轴审查: Standards + Spec",
    },
    {
      id: "skill-seed-tdd",
      name: "tdd",
      version: "1.0.0",
      scope: "user",
      enabled: true,
      description: "红绿循环测试驱动开发",
    },
  ],
  plugins: [
    {
      id: "plugin-seed-kanban-orchestrator",
      name: "kanban-orchestrator",
      version: "0.9.2",
      scope: "project",
      enabled: true,
      permissions: ["work-item:read", "work-item:write"],
      description: "Kanban 任务编排器 (分解 + 防诱惑规则)",
    },
  ],
};

function loadRegistry(): CapabilityRegistryState {
  if (typeof window === "undefined") return DEFAULT_CAPABILITY_REGISTRY;
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_CAPABILITY_REGISTRY;
    const parsed = JSON.parse(raw) as Partial<CapabilityRegistryState>;
    return {
      mcpServers: Array.isArray(parsed.mcpServers) ? parsed.mcpServers : DEFAULT_CAPABILITY_REGISTRY.mcpServers,
      skills: Array.isArray(parsed.skills) ? parsed.skills : DEFAULT_CAPABILITY_REGISTRY.skills,
      plugins: Array.isArray(parsed.plugins) ? parsed.plugins : DEFAULT_CAPABILITY_REGISTRY.plugins,
    };
  } catch {
    return DEFAULT_CAPABILITY_REGISTRY;
  }
}

let registryState: CapabilityRegistryState = loadRegistry();
const listeners = new Set<() => void>();

function emitRegistryChange() {
  for (const listener of listeners) listener();
}

function persistRegistry() {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(registryState));
  } catch {
    // localStorage 不可用 (隐私模式/配额) → 仅内存态, 面板仍可用
  }
}

function subscribeRegistry(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function getCapabilityRegistry(): CapabilityRegistryState {
  return registryState;
}

/** 测试辅助: 重置为 seed (仅 jsdom/test 环境使用) */
export function resetCapabilityRegistryForTest() {
  registryState = structuredClone(DEFAULT_CAPABILITY_REGISTRY);
  emitRegistryChange();
}

export function useCapabilityRegistry(): CapabilityRegistryState {
  return useSyncExternalStore(subscribeRegistry, getCapabilityRegistry, getCapabilityRegistry);
}

// =====================================================================
// CapabilityRegistryApi — 面板操作入口 (形状对齐未来 REST CRUD)
// =====================================================================
export class CapabilityRegistryApi {
  addMcpServer(input: Omit<McpServerConfig, "id" | "health" | "last_error" | "updated_at_ms">): McpServerConfig {
    const id = `mcp-${globalThis.crypto.randomUUID()}`;
    const server: McpServerConfig = {
      ...input,
      id,
      health: "unknown",
      last_error: null,
      updated_at_ms: Date.now(),
    };
    registryState = { ...registryState, mcpServers: [...registryState.mcpServers, server] };
    persistRegistry();
    emitRegistryChange();
    return server;
  }

  removeMcpServer(id: string) {
    registryState = { ...registryState, mcpServers: registryState.mcpServers.filter((s) => s.id !== id) };
    persistRegistry();
    emitRegistryChange();
  }

  toggleMcpServer(id: string) {
    registryState = {
      ...registryState,
      mcpServers: registryState.mcpServers.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s)),
    };
    persistRegistry();
    emitRegistryChange();
  }

  addSkill(input: Omit<SkillEntry, "id">): SkillEntry {
    const skill: SkillEntry = { ...input, id: `skill-${globalThis.crypto.randomUUID()}` };
    registryState = { ...registryState, skills: [...registryState.skills, skill] };
    persistRegistry();
    emitRegistryChange();
    return skill;
  }

  removeSkill(id: string) {
    registryState = { ...registryState, skills: registryState.skills.filter((s) => s.id !== id) };
    persistRegistry();
    emitRegistryChange();
  }

  toggleSkill(id: string) {
    registryState = {
      ...registryState,
      skills: registryState.skills.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s)),
    };
    persistRegistry();
    emitRegistryChange();
  }

  addPlugin(input: Omit<PluginEntry, "id">): PluginEntry {
    const plugin: PluginEntry = { ...input, id: `plugin-${globalThis.crypto.randomUUID()}` };
    registryState = { ...registryState, plugins: [...registryState.plugins, plugin] };
    persistRegistry();
    emitRegistryChange();
    return plugin;
  }

  removePlugin(id: string) {
    registryState = { ...registryState, plugins: registryState.plugins.filter((p) => p.id !== id) };
    persistRegistry();
    emitRegistryChange();
  }

  togglePlugin(id: string) {
    registryState = {
      ...registryState,
      plugins: registryState.plugins.map((p) => (p.id === id ? { ...p, enabled: !p.enabled } : p)),
    };
    persistRegistry();
    emitRegistryChange();
  }
}

export const capabilityRegistryApi = new CapabilityRegistryApi();
