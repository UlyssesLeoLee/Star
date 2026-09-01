import type { CliProfile, ApiKey, TaskWindow } from "@/mocks/schemas/cli";
export { isCliProfile, isApiKey, isTaskWindow } from "@/mocks/schemas/cli";

export const MOCK_CLI_PROFILES: CliProfile[] = [
  { id: "claude", name: "Claude Code", kind: "claude", command: "claude", args: "--model claude-3-5-sonnet", env: "PATH=$PATH", worktreeBinding: "auto", enabled: true, hasApiKey: true },
  { id: "codex", name: "OpenAI Codex", kind: "codex", command: "codex", args: "--model gpt-4", env: "OPENAI_API_KEY", worktreeBinding: "auto", enabled: true, hasApiKey: true },
  { id: "openclaw", name: "OpenClaw", kind: "openclaw", command: "https://api.openclaw.dev/v1", args: "model=gpt-4", env: "OPENCLAW_API_KEY", worktreeBinding: "auto", enabled: false, hasApiKey: false },
  { id: "hermes", name: "Hermes", kind: "hermes", command: "https://api.hermes.dev/v1", args: "model=hermes-2", env: "HERMES_API_KEY", worktreeBinding: "auto", enabled: false, hasApiKey: false },
  { id: "gemini", name: "Google Gemini", kind: "gemini", command: "gemini", args: "", env: "GOOGLE_API_KEY", worktreeBinding: "auto", enabled: true, hasApiKey: true },
  { id: "aider", name: "Aider", kind: "aider", command: "aider", args: "--model gpt-4", env: "OPENAI_API_KEY", worktreeBinding: "auto", enabled: false, hasApiKey: true },
];

export const MOCK_API_KEYS: ApiKey[] = [
  { id: "k1", provider: "anthropic", label: "Primary", mode: "encrypted_rust", preview: "sk-***xyz1", createdAt: "2026-08-15", cli_profile_id: "claude" },
  { id: "k2", provider: "openai", label: "Backup", mode: "environment_var", preview: "env: OPENAI_API_KEY", envVarName: "OPENAI_API_KEY", createdAt: "2026-08-20", cli_profile_id: "codex" },
  // per 2026-09-02 02:49 JST Ulysses 拍板: openai/claude/gemini/minimax 4 必备
  { id: "k3", provider: "minimax", label: "minimax Primary", mode: "encrypted_rust", preview: "mm-***xyz3", createdAt: "2026-09-01" },
  { id: "k4", provider: "gemini", label: "Gemini Backup", mode: "environment_var", preview: "env: GOOGLE_API_KEY", envVarName: "GOOGLE_API_KEY", createdAt: "2026-09-01", cli_profile_id: "gemini" },
];

export const MOCK_TASK_WINDOWS: TaskWindow[] = [
  {
    id: "w1",
    name: "Physis / GVPE",
    worktreeId: "wt-physis-gvpe",
    defaultProfileId: "claude",
    uploadTrigger: "on_success_exit",
    activeTabId: "t1",
    tabs: [
      { id: "t1", windowId: "w1", profileId: "claude", label: "Claude Code", state: "running", kind: "cli", profileName: "Claude Code", lastOutput: "$ claude --model claude-3-5-sonnet\n> Reading worktree state...", filesChanged: 3 },
      { id: "t2", windowId: "w1", profileId: "openclaw", label: "OpenClaw (gpt-4)", state: "completed", kind: "api", profileName: "OpenClaw", lastOutput: "✓ 3 files generated", exitCode: 0, filesChanged: 3 },
      { id: "t3", windowId: "w1", profileId: "codex", label: "Codex", state: "failed", kind: "cli", profileName: "OpenAI Codex", lastOutput: "Error: rate limit exceeded", exitCode: 1 },
    ],
  },
];
