// Star Frontend - Credentials API Client Tests
// V2-2 完整版 - vitest
// per 守门 #5: secret 不入 log, 不在测试中用真实 key

import { describe, it, expect, vi, beforeEach } from "vitest";
import { credentialsApi } from "@/lib/api/credentials";

describe("credentialsApi", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("list() 调 GET /api/v2/credentials", async () => {
    const mock = [{ id: "1", provider: "openclaw", display_name: "Test", status: "active", created_at_ms: 1000, updated_at_ms: 1000, deprecated_at_ms: null, revoked_at_ms: null }];
    global.fetch = vi.fn().mockResolvedValue({ ok: true, status: 200, json: async () => mock });
    const result = await credentialsApi.list();
    expect(result).toEqual(mock);
    expect(fetch).toHaveBeenCalledWith("/api/v2/credentials", expect.objectContaining({ headers: expect.objectContaining({ "Content-Type": "application/json" }) }));
  });

  it("list(provider) 调 GET /api/v2/credentials?provider=...", async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: true, status: 200, json: async () => [] });
    await credentialsApi.list("hermes");
    expect(fetch).toHaveBeenCalledWith("/api/v2/credentials?provider=hermes", expect.any(Object));
  });

  it("create() 调 POST /api/v2/credentials with body", async () => {
    const mock = { id: "1", provider: "openclaw", display_name: "X", status: "active", created_at_ms: 1000, updated_at_ms: 1000, deprecated_at_ms: null, revoked_at_ms: null };
    global.fetch = vi.fn().mockResolvedValue({ ok: true, status: 201, json: async () => mock });
    const result = await credentialsApi.create({
      provider: "openclaw", display_name: "X", description: "d", secret: "s", base_url: "https://x.com",
    });
    expect(result).toEqual(mock);
    expect(fetch).toHaveBeenCalledWith("/api/v2/credentials", expect.objectContaining({ method: "POST", body: JSON.stringify({ provider: "openclaw", display_name: "X", description: "d", secret: "s", base_url: "https://x.com" }) }));
  });

  it("revoke() 调 POST /api/v2/credentials/{id}/revoke", async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: true, status: 204, json: async () => undefined });
    await credentialsApi.revoke("abc-123");
    expect(fetch).toHaveBeenCalledWith("/api/v2/credentials/abc-123/revoke", expect.objectContaining({ method: "POST" }));
  });

  it("audit() 调 GET /api/v2/credentials/{id}/audit", async () => {
    const mock = [{ id: "1", credential_id: "x", user_id: "u1", event_type: "store", event_at_ms: 1000, display_name_snapshot: "T" }];
    global.fetch = vi.fn().mockResolvedValue({ ok: true, status: 200, json: async () => mock });
    const result = await credentialsApi.audit("x");
    expect(result).toEqual(mock);
  });

  it("失败时抛错 (守门 #5: 错误消息不含 secret)", async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: false, status: 500, statusText: "Internal", text: async () => "internal: db error" });
    await expect(credentialsApi.list()).rejects.toThrow("API 500: internal: db error");
  });
});
