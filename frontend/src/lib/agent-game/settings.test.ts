// =====================================================================
// settings.test.ts — Agent Settings 纯函数
// =====================================================================

import { describe, it, expect } from "vitest";
import { createInitialAgentSettings, validateSettings, modelOptionsFor, DEFAULT_SETTINGS } from "./settings";
import type { AgentSession } from "@/types/ids";

const baseAgent: AgentSession = {
  id: "ag-001",
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  worktree_id: "wt-001",
  agent_kind: "claude-sonnet",
  status: "executing",
  current_step: "test",
  token_usage: { input: 0, output: 0, total: 0 },
  cost_summary: { usd: 0, budget_usd: 5.0 },
  started_at: "2026-09-05T10:00:00Z",
};

describe("createInitialAgentSettings", () => {
  it("claude-sonnet 推默认 model + baseUrl + systemPrompt", () => {
    const s = createInitialAgentSettings(baseAgent);
    expect(s.model).toBe("claude-sonnet-4.5");
    expect(s.baseUrl).toBe("https://api.anthropic.com/v1");
    expect(s.systemPrompt).toContain("Claude");
    expect(s.maxTokens).toBe(8000);
  });

  it("gpt-4o 推 OpenAI defaults", () => {
    const s = createInitialAgentSettings({ ...baseAgent, agent_kind: "gpt-4o" });
    expect(s.model).toBe("gpt-4o");
    expect(s.baseUrl).toBe("https://api.openai.com/v1");
  });

  it("codex 推 codex-1, maxTokens 16000", () => {
    const s = createInitialAgentSettings({ ...baseAgent, agent_kind: "codex" });
    expect(s.model).toBe("codex-1");
    expect(s.maxTokens).toBe(16000);
    expect(s.temperature).toBe(0.2);
  });

  it("internal-vibe-coder 推 internal API", () => {
    const s = createInitialAgentSettings({ ...baseAgent, agent_kind: "internal-vibe-coder" });
    expect(s.model).toBe("vibe-coder-1");
    expect(s.baseUrl).toContain("internal.star.local");
  });

  it("未知 kind 走 fallback (kind 当 model, 通用 baseUrl)", () => {
    const s = createInitialAgentSettings({ ...baseAgent, agent_kind: "unknown-llm" });
    expect(s.model).toBe("unknown-llm");
    expect(s.baseUrl).toBe("https://api.example.com/v1");
  });

  it("初始 enabled = true, updatedAt 有值", () => {
    const s = createInitialAgentSettings(baseAgent);
    expect(s.enabled).toBe(true);
    expect(s.updatedAt).toBeTruthy();
  });

  it("apiKey 是 mock (per agentId)", () => {
    const s = createInitialAgentSettings(baseAgent);
    expect(s.apiKey).toContain("ag-001");
    expect(s.apiKey).toContain("mock");
  });
});

describe("validateSettings", () => {
  it("合法 settings 通过", () => {
    const s = createInitialAgentSettings(baseAgent);
    const r = validateSettings(s);
    expect(r.ok).toBe(true);
    expect(r.errors).toEqual([]);
  });

  it("apiKey 短 (< 8) 失败", () => {
    const s = { ...createInitialAgentSettings(baseAgent), apiKey: "abc" };
    const r = validateSettings(s);
    expect(r.ok).toBe(false);
    expect(r.errors.some((e) => e.includes("apiKey"))).toBe(true);
  });

  it("maxTokens 越界 (< 1 或 > 200000) 失败", () => {
    const s1 = { ...createInitialAgentSettings(baseAgent), maxTokens: 0 };
    expect(validateSettings(s1).ok).toBe(false);
    const s2 = { ...createInitialAgentSettings(baseAgent), maxTokens: 200001 };
    expect(validateSettings(s2).ok).toBe(false);
  });

  it("temperature 越界 (< 0 或 > 2) 失败", () => {
    const s1 = { ...createInitialAgentSettings(baseAgent), temperature: -0.1 };
    expect(validateSettings(s1).ok).toBe(false);
    const s2 = { ...createInitialAgentSettings(baseAgent), temperature: 2.1 };
    expect(validateSettings(s2).ok).toBe(false);
  });

  it("baseUrl 不以 http(s):// 开头 失败", () => {
    const s = { ...createInitialAgentSettings(baseAgent), baseUrl: "ftp://example.com" };
    const r = validateSettings(s);
    expect(r.ok).toBe(false);
    expect(r.errors.some((e) => e.includes("baseUrl"))).toBe(true);
  });
});

describe("modelOptionsFor", () => {
  it("claude-sonnet 推 3 个 model", () => {
    expect(modelOptionsFor("claude-sonnet")).toContain("claude-sonnet-4.5");
    expect(modelOptionsFor("claude-sonnet")).toHaveLength(3);
  });

  it("gpt-4o 推 3 个 model", () => {
    expect(modelOptionsFor("gpt-4o")).toHaveLength(3);
  });

  it("未知 kind 推 1 个 (kind 当 model)", () => {
    expect(modelOptionsFor("unknown")).toEqual(["unknown"]);
  });
});

describe("DEFAULT_SETTINGS sanity", () => {
  it("4 个 agent_kind 都覆盖", () => {
    expect(Object.keys(DEFAULT_SETTINGS).sort()).toEqual([
      "claude-sonnet",
      "codex",
      "gpt-4o",
      "internal-vibe-coder",
    ]);
  });
});
