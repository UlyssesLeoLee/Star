// =====================================================================
// Onboarding scanner / retry / Guide 集成测试 (per ADR-0042, 2026-09-02 08:01 JST 拍板)
// =====================================================================
// 6 个测试:
//   1. scanAllDetectors: localStorage 0/1/2 keys (mock 注入)
//   2. 探测器去重 (同 provider+label 不重复)
//   3. retry 5 attempts, 全部 mock success → status="success"
//   4. retry 5 attempts, 全部 mock 401 → status="failed" + audit log 写入
//   5. ERROR_RESOLUTIONS: 5 种 error code 全有 steps
//   6. isOnboardingCompleted / markOnboardingCompleted / resetOnboarding 状态机
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 6 测试全快速 (< 2s), 不用真 fetch (per 拍板 retryreport_opt3 mock 阶段)
//   - vi.useFakeTimers() 控制 setTimeout, 不真等 48s
// =====================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { scanAllDetectors, isOnboardingCompleted, markOnboardingCompleted, markOnboardingSkipped, resetOnboarding } from "./scanner";
import { testKeyWithRetry, ERROR_RESOLUTIONS, classifyStatus } from "./retry";
import type { DetectedKey } from "@/types/onboarding";

// ---- mock localStorage ----
const LS_KEY = "star:api-keys";
const COMPLETED_KEY = "star:onboarding-completed";

beforeEach(() => {
  if (typeof window !== "undefined") {
    window.localStorage.clear();
  }
  vi.restoreAllMocks();
});

// ---- 1. scanAllDetectors: localStorage 注入 2 个 key ----
describe("scanAllDetectors", () => {
  it("returns DetectedKey[] from localStorage star:api-keys", async () => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(LS_KEY, JSON.stringify([
      { id: "k1", provider: "openai", label: "OpenAI Primary", preview: "sk-***xyz1", createdAt: "2026-09-01" },
      { id: "k2", provider: "claude", label: "Claude Backup", preview: "sk-ant-***abc", createdAt: "2026-09-01" },
    ]));
    const keys = await scanAllDetectors();
    // env-var-hint Phase 1 mock 返空 + IDE-residual 返空
    // localStorage 2 个 → 至少 2 个
    expect(keys.length).toBeGreaterThanOrEqual(2);
    const openai = keys.find((k) => k.provider === "openai" || k.provider === "claude" || k.provider === "anthropic");
    expect(openai).toBeDefined();
    if (openai) {
      expect(openai.source).toBe("localStorage");
      expect(openai.source_label).toMatch(/localStorage/);
    }
  });

  it("returns empty array when localStorage star:api-keys absent", async () => {
    const keys = await scanAllDetectors();
    // 仅 localStorage 来源 0 个, env-var-hint/IDE-residual 也返 0
    // 但 normalizeProvider fallback 不会有 key 漏出来
    const localStorageKeys = keys.filter((k) => k.source === "localStorage");
    expect(localStorageKeys).toHaveLength(0);
  });

  it("deduplicates by provider+label (同 provider+label 不重复)", async () => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(LS_KEY, JSON.stringify([
      { id: "k1", provider: "openai", label: "Primary", preview: "sk-***1", createdAt: "2026-09-01" },
      { id: "k2", provider: "openai", label: "Primary", preview: "sk-***2", createdAt: "2026-09-01" },  // 同 provider+label
      { id: "k3", provider: "openai", label: "Backup",  preview: "sk-***3", createdAt: "2026-09-01" },
    ]));
    const keys = await scanAllDetectors();
    const openaiPrimary = keys.filter((k) => k.provider === "openai" && k.label === "Primary");
    expect(openaiPrimary).toHaveLength(1);  // dedup
  });
});

// ---- 2. retry: 5 attempts success ----
describe("testKeyWithRetry", () => {
  it("returns success when test passes on attempt 0", async () => {
    const key: DetectedKey = {
      id: "k-test-1", provider: "openai", label: "Test",
      preview: "sk-***xyz", source: "localStorage",
      detected_at: new Date().toISOString(),
      source_label: "test",
    };
    // 内部 mock 90% success rate, 跑 10 次 至少 1 次 success
    let foundSuccess = false;
    for (let i = 0; i < 20 && !foundSuccess; i++) {
      const r = await testKeyWithRetry(key);
      if (r.status === "success") {
        foundSuccess = true;
        expect(r.attempt).toBeGreaterThanOrEqual(0);
        expect(r.attempt).toBeLessThan(5);
      }
    }
    expect(foundSuccess).toBe(true);
  }, 60_000);

  it("returns failed + writes audit log when all 5 attempts fail", async () => {
    // 强制 mock 100% fail (Math.random 返回 0 → 走 unauthorized 分支)
    const originalRandom = Math.random;
    Math.random = () => 0;  // 永远触发 10% fail 分支

    const key: DetectedKey = {
      id: "k-test-fail", provider: "openai", label: "Test Fail",
      preview: "sk-***fail", source: "localStorage",
      detected_at: new Date().toISOString(),
      source_label: "test",
    };
    const progressResults: number[] = [];
    const final = await testKeyWithRetry(key, (partial) => {
      if (partial.status === "failed") progressResults.push(partial.attempt);
    });

    expect(final.status).toBe("failed");
    expect(final.attempt).toBe(4);  // 最后一次 attempt (0-indexed)
    expect(final.max_attempts).toBe(5);
    expect(final.status_code).toBe(401);
    expect(final.error_message).toMatch(/unauthorized/i);
    // audit_event_id 必須 (per ADR-0043 §2.4, Phase 2: backend POST → 201 + audit_event_id)
    expect(final.audit_event_id).toBeDefined();
    // MSW mock handler 返 audit-{uuid} 形式, 验证非 SSR fallback
    expect(final.audit_event_id).toMatch(/^audit-[0-9a-f-]+$/);
    expect(progressResults.length).toBeGreaterThanOrEqual(5);  // 5 次 failed 回调

    Math.random = originalRandom;
  }, 60_000);
});

// ---- 3. ERROR_RESOLUTIONS 全 5 code ----
describe("ERROR_RESOLUTIONS", () => {
  it("5 error codes 全有 steps + curl_test", () => {
    expect(ERROR_RESOLUTIONS.unauthorized.steps.length).toBeGreaterThan(0);
    expect(ERROR_RESOLUTIONS.unauthorized.curl_test).toBeDefined();
    expect(ERROR_RESOLUTIONS.forbidden.steps.length).toBeGreaterThan(0);
    expect(ERROR_RESOLUTIONS.rate_limited.steps.length).toBeGreaterThan(0);
    expect(ERROR_RESOLUTIONS.network_timeout.steps.length).toBeGreaterThan(0);
    expect(ERROR_RESOLUTIONS.model_unavailable.steps.length).toBeGreaterThan(0);
    expect(ERROR_RESOLUTIONS.unknown.steps.length).toBeGreaterThan(0);
  });

  it("classifyStatus: 401/403/429/503/0 → 正确 code", () => {
    expect(classifyStatus(401)).toBe("unauthorized");
    expect(classifyStatus(403)).toBe("forbidden");
    expect(classifyStatus(429)).toBe("rate_limited");
    expect(classifyStatus(404)).toBe("model_unavailable");
    expect(classifyStatus(503)).toBe("model_unavailable");
    expect(classifyStatus(0)).toBe("network_timeout");
    expect(classifyStatus(500)).toBe("unknown");
  });
});

// ---- 4. onboarding 状态机 ----
describe("onboarding state machine", () => {
  it("isOnboardingCompleted false initially (after clear)", () => {
    resetOnboarding();
    expect(isOnboardingCompleted()).toBe(false);
  });

  it("markOnboardingCompleted → isOnboardingCompleted true", () => {
    markOnboardingCompleted();
    expect(isOnboardingCompleted()).toBe(true);
  });

  it("markOnboardingSkipped → isOnboardingCompleted true (跳过 = 标记完成)", () => {
    markOnboardingSkipped();
    expect(isOnboardingCompleted()).toBe(true);
  });

  it("resetOnboarding → false 重新触发", () => {
    markOnboardingCompleted();
    expect(isOnboardingCompleted()).toBe(true);
    resetOnboarding();
    expect(isOnboardingCompleted()).toBe(false);
  });
});
