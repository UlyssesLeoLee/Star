/**
 * UT: AgentCredentialStore per-agent 凭证 CRUD (per ADR-0051)
 *
 * 守门合规: 守门 #5 + #13 c + #14 v2
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { AgentCredentialStore, getAgentCredentialStore } from "../credentials";

describe("AgentCredentialStore", () => {
  let store: AgentCredentialStore;

  beforeEach(() => {
    localStorage.clear();
    store = new AgentCredentialStore();
  });

  it("adds a new credential per-agent", () => {
    const cred = store.add("SA-01", {
      kind: "openai",
      displayName: "Test OpenAI",
      envKey: "OPENAI_API_KEY",
    });
    expect(cred.kind).toBe("openai");
    expect(cred.active).toBe(true);
    expect(cred.envKey).toBe("OPENAI_API_KEY");
    expect(cred.displayName).toBe("Test OpenAI");
    expect(creds().length).toBe(1);

    function creds() {
      return store.list("SA-01");
    }
  });

  it("dedups active credential per agent+kind", () => {
    store.add("SA-01", {
      kind: "openai",
      displayName: "First",
      envKey: "OPENAI_API_KEY_1",
    });
    store.add("SA-01", {
      kind: "openai",
      displayName: "Second",
      envKey: "OPENAI_API_KEY_2",
    });
    const creds = store.list("SA-01");
    expect(creds.length).toBe(2);
    // 仅 1 个 active (新加的)
    const activeCreds = creds.filter((c) => c.active);
    expect(activeCreds.length).toBe(1);
    expect(activeCreds[0].displayName).toBe("Second");
  });

  it("isolates credentials per agent (守门 #13 c RLS 13 类)", () => {
    store.add("SA-01", {
      kind: "openai",
      displayName: "SA-01 OpenAI",
      envKey: "OPENAI_API_KEY_SA01",
    });
    store.add("SA-02", {
      kind: "openai",
      displayName: "SA-02 OpenAI",
      envKey: "OPENAI_API_KEY_SA02",
    });
    // SA-01 跟 SA-02 互不影响
    const sa01Creds = store.list("SA-01");
    const sa02Creds = store.list("SA-02");
    expect(sa01Creds.length).toBe(1);
    expect(sa02Creds.length).toBe(1);
    expect(sa01Creds[0].envKey).toBe("OPENAI_API_KEY_SA01");
    expect(sa02Creds[0].envKey).toBe("OPENAI_API_KEY_SA02");
  });

  it("activates a specific credential", () => {
    const c1 = store.add("SA-01", {
      kind: "openai",
      displayName: "First",
      envKey: "OPENAI_KEY_1",
    });
    store.add("SA-01", {
      kind: "openai",
      displayName: "Second",
      envKey: "OPENAI_KEY_2",
    });
    // 重新激活 First
    const reactivated = store.activate("SA-01", c1.id);
    expect(reactivated?.active).toBe(true);
    // Second 应 inactive
    const creds = store.list("SA-01");
    const inactive = creds.filter((c) => !c.active);
    expect(inactive.length).toBe(1);
    expect(inactive[0].displayName).toBe("Second");
  });

  it("removes a credential", () => {
    const cred = store.add("SA-01", {
      kind: "openai",
      displayName: "Test",
      envKey: "OPENAI_API_KEY",
    });
    const ok = store.remove("SA-01", cred.id);
    expect(ok).toBe(true);
    expect(store.list("SA-01").length).toBe(0);
  });

  it("getActive returns the active credential for kind", () => {
    store.add("SA-01", {
      kind: "openai",
      displayName: "Test",
      envKey: "OPENAI_API_KEY",
    });
    const active = store.getActive("SA-01", "openai");
    expect(active).not.toBeNull();
    expect(active?.envKey).toBe("OPENAI_API_KEY");
    const inactive = store.getActive("SA-01", "claude");
    expect(inactive).toBeNull();
  });

  it("getAgentCredentialStore returns singleton", () => {
    const s1 = getAgentCredentialStore();
    const s2 = getAgentCredentialStore();
    expect(s1).toBe(s2);
  });

  it("allAgentIds returns 10 agent IDs", () => {
    const ids = AgentCredentialStore.allAgentIds();
    expect(ids.length).toBe(10);
    expect(ids).toContain("SA-01");
    expect(ids).toContain("SA-10");
  });
});
