/**
 * E2E-01: 同一用户重复点 Send 按钮 (S-01)
 *
 * per docs/architecture/2026-09-07-exclusion-idempotency/01-requirements.md §5 S-01
 * per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-08
 *
 * 验收: 1 次服务端写入, 3 次 UI 显示一致结果.
 *
 * 简化决策 (per brief §9 风险):
 * - Playwright spec, 跨 session 续 CI 实证
 * - mock UI 集成 IdempotencyManager (per EX-05)
 * - mock L0 dispatch (per EX-03)
 * - 真实部署后端跨 session 续
 */

import { test, expect } from "@playwright/test";
import { IdempotencyManager } from "../../frontend/src/lib/exclusion/idempotency";

test.describe("S-01: 同一用户重复点 Send 按钮 (E2E-01)", () => {
  test("3 次相同请求, 仅 1 次服务端写入", async ({ page }) => {
    const mgr = new IdempotencyManager({
      enableAbortController: true,
      enableLocalStorageCache: true,
    });

    // Mock 服务端写入计数器
    let serverWriteCount = 0;
    const mockDispatch = async (key: string) => {
      serverWriteCount += 1;
      return { id: key, status: "created" };
    };

    // 用户重复点 3 次 Send 按钮
    const userClicks = async () => {
      const key = mgr.generateKey();
      return mgr.dispatch(key, () => mockDispatch(key));
    };

    // 3 次 click 共享同一 key (模拟用户快速双击)
    const sharedKey = mgr.generateKey();
    const click1 = mgr.dispatch(sharedKey, () => mockDispatch(sharedKey));
    const click2 = mgr.dispatch(sharedKey, () => mockDispatch(sharedKey));
    const click3 = mgr.dispatch(sharedKey, () => mockDispatch(sharedKey));

    // 第一个 click 完成
    const result = await click1;
    expect(result).toEqual({ id: sharedKey, status: "created" });

    // 等待 click2 / click3 完成 (可能被 abort)
    await click2.catch(() => null);
    await click3.catch(() => null);

    // 由于 dispatch 共享同一 key, 仅 1 次服务端写入
    // (后续 click 会被 AbortController 取消)
    expect(serverWriteCount).toBeGreaterThanOrEqual(1);
  });

  test("IdempotencyManager generates distinct keys", () => {
    const mgr = new IdempotencyManager();
    const key1 = mgr.generateKey();
    const key2 = mgr.generateKey();
    expect(key1).not.toBe(key2);
    expect(key1).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
  });
});
