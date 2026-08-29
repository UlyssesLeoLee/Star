import { describe, it, expect } from "vitest";
import { isCliProfile, isApiKey, isTaskWindow } from "@/mocks/schemas/cli";
import { MOCK_CLI_PROFILES, MOCK_API_KEYS, MOCK_TASK_WINDOWS } from "@/mocks/data/cli";

describe("CLI Schemas", () => {
  it("accepts all 6 built-in profiles", () => {
    for (const p of MOCK_CLI_PROFILES) {
      expect(isCliProfile(p)).toBe(true);
    }
  });

  it("rejects invalid profile", () => {
    expect(isCliProfile({ id: "x" })).toBe(false);
    expect(isCliProfile(null)).toBe(false);
    expect(isCliProfile("string")).toBe(false);
  });

  it("accepts all mock API keys", () => {
    for (const k of MOCK_API_KEYS) {
      expect(isApiKey(k)).toBe(true);
    }
  });

  it("rejects invalid API key", () => {
    expect(isApiKey({ id: "x" })).toBe(false);
  });

  it("accepts all mock task windows", () => {
    for (const w of MOCK_TASK_WINDOWS) {
      expect(isTaskWindow(w)).toBe(true);
    }
  });
});

describe("Mock Data integrity", () => {
  it("6 built-in CLI profiles", () => {
    expect(MOCK_CLI_PROFILES).toHaveLength(6);
  });

  it("CLI kinds: claude/codex/openclaw/hermes/gemini/aider", () => {
    const kinds = MOCK_CLI_PROFILES.map((p) => p.kind);
    expect(kinds).toContain("claude");
    expect(kinds).toContain("codex");
    expect(kinds).toContain("openclaw");
    expect(kinds).toContain("hermes");
    expect(kinds).toContain("gemini");
    expect(kinds).toContain("aider");
  });

  it("API Agent (openclaw/hermes) use https URL", () => {
    const openclaw = MOCK_CLI_PROFILES.find((p) => p.kind === "openclaw");
    const hermes = MOCK_CLI_PROFILES.find((p) => p.kind === "hermes");
    expect(openclaw?.command.startsWith("https://")).toBe(true);
    expect(hermes?.command.startsWith("https://")).toBe(true);
  });

  it("API Key: 2 modes supported", () => {
    const modes = new Set(MOCK_API_KEYS.map((k) => k.mode));
    expect(modes).toContain("encrypted_rust");
    expect(modes).toContain("environment_var");
  });
});
