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
  provider: "anthropic" | "openai" | "openclaw" | "hermes" | "google";
  label: string;
  mode: "encrypted_rust" | "environment_var";
  preview: string;
  envVarName?: string;
  createdAt: string;
  lastUsedAt?: string;
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
    typeof o.createdAt === "string"
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
