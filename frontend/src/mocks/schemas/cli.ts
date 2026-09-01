// frontend/src/mocks/schemas/cli.ts
// CLI Profile + API Key schemas (per 2026-08-29 09:07 JST 用户拍板)

export interface CliProfile {
  id: string;
  name: string;
  kind: "claude" | "codex" | "openclaw" | "hermes" | "gemini" | "aider" | "custom";
  command: string;
  args: string;
  env: string;
  worktreeBinding: "auto" | "fixed" | "prompt";
  enabled: boolean;
  hasApiKey: boolean;
}

export interface ApiKey {
  id: string;
  /** LLM 厂商 (per 2026-09-02 02:49 JST Ulysses 拍板: openai/claude/gemini/minimax 4 必备) */
  provider: "anthropic" | "openai" | "openclaw" | "hermes" | "google" | "claude" | "gemini" | "minimax";
  label: string;
  mode: "encrypted_rust" | "environment_var";
  preview: string;
  envVarName?: string;
  createdAt: string;
  lastUsedAt?: string;
  /** 关联 agent tab (per CliTab.id) — 各 agent 分别填不同 key */
  agent_id?: string;
  /** 关联 CLI profile (per CliProfile.id, e.g. "claude" / "codex" / "openclaw") */
  cli_profile_id?: string;
  /** 关联 agent_kind (per types/ids.ts AgentSession.agent_kind) */
  agent_kind?: "claude-sonnet" | "gpt-4o" | "codex" | "internal-vibe-coder" | "gemini-2" | "minimax-v1";
}

export interface TaskWindow {
  id: string;
  name: string;
  worktreeId: string;
  defaultProfileId: string;
  uploadTrigger: "on_success_exit" | "manual" | "polling";
  tabs: CliTab[];
  activeTabId: string | null;
}

export interface CliTab {
  id: string;
  windowId: string;
  profileId: string;
  label: string;
  state: "created" | "running" | "waiting_input" | "completed" | "failed" | "aborted";
  kind: "cli" | "api";
  profileName: string;
  lastOutput?: string;
  exitCode?: number;
  filesChanged?: number;
}

export function isCliProfile(x: unknown): x is CliProfile {
  if (typeof x !== "object" || x === null) return false;
  const o = x as Record<string, unknown>;
  return (
    typeof o.id === "string" &&
    typeof o.name === "string" &&
    typeof o.kind === "string" &&
    ["claude", "codex", "openclaw", "hermes", "gemini", "aider", "custom"].includes(o.kind as string) &&
    typeof o.command === "string" &&
    typeof o.args === "string" &&
    typeof o.env === "string" &&
    ["auto", "fixed", "prompt"].includes(o.worktreeBinding as string) &&
    typeof o.enabled === "boolean" &&
    typeof o.hasApiKey === "boolean"
  );
}

export function isApiKey(x: unknown): x is ApiKey {
  if (typeof x !== "object" || x === null) return false;
  const o = x as Record<string, unknown>;
  return (
    typeof o.id === "string" &&
    typeof o.provider === "string" &&
    typeof o.label === "string" &&
    ["encrypted_rust", "environment_var"].includes(o.mode as string) &&
    typeof o.preview === "string" &&
    typeof o.createdAt === "string" &&
    (o.agent_id === undefined || typeof o.agent_id === "string") &&
    (o.cli_profile_id === undefined || typeof o.cli_profile_id === "string") &&
    (o.agent_kind === undefined || typeof o.agent_kind === "string")
  );
}

export function isTaskWindow(x: unknown): x is TaskWindow {
  if (typeof x !== "object" || x === null) return false;
  const o = x as Record<string, unknown>;
  return (
    typeof o.id === "string" &&
    typeof o.worktreeId === "string" &&
    Array.isArray(o.tabs)
  );
}
