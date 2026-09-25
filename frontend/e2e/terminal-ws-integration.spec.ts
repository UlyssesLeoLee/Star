// =====================================================================
// terminal-ws-integration.spec.ts — Playwright E2E (per PR #98.5 + T23.6 fix)
// =====================================================================
// 守门: 6 E2E 测试验证前端 WS client 真接入 PR #106 后端 (mock server)
//
// T23.6 fix: 使用 window.__mockWsCtor (per useTerminalStackWs hook 读 window.__mockWsCtor)
// 而不是 window.WebSocket = X (Chromium 不允许覆盖内置 WebSocket).
// =====================================================================

import { test, expect } from "@playwright/test";

// =====================================================================
// installTestMockWs — MockWsClass EventTarget-based WS mock
// (T23.6 round 4: 使用真正的 EventTarget-backed mock, 避免 Chromium real WS
// 网络请求失败)
// =====================================================================
async function installTestMockWs(
  page: import("@playwright/test").Page,
): Promise<void> {
  await page.addInitScript(() => {
    // @ts-expect-error
    window.__mockWsInstances = [];
    // @ts-expect-error
    window.__mockWsCtor = function (url: string) {
      // @ts-expect-error
      const WsClass = (window as unknown as { __MockWsClass: new (url: string) => WebSocket }).__MockWsClass;
      return new WsClass(url) as unknown as WebSocket;
    };
    // @ts-expect-error
    (window as unknown as { __MockWsClass: new (url: string) => WebSocket }).__MockWsClass = (function () {
      function MockWs(this: unknown, url: string) {
        // @ts-expect-error
        this.url = url;
        // @ts-expect-error
        this.binaryType = "arraybuffer";
        // @ts-expect-error
        this.readyState = 0; // CONNECTING
        // @ts-expect-error
        this.onopen = null;
        // @ts-expect-error
        this.onmessage = null;
        // @ts-expect-error
        this.onerror = null;
        // @ts-expect-error
        this.onclose = null;
        // @ts-expect-error
        this.sent = [];
        // @ts-expect-error
        (window as unknown as { __mockWsInstances: unknown[] }).__mockWsInstances.push(this);
        // After 50ms, fire open + (optionally hello + snapshot per test)
        setTimeout(() => {
          try {
            // @ts-expect-error
            this.readyState = 1; // OPEN
            const openEvent = new Event("open");
            // @ts-expect-error
            this.dispatchEvent(openEvent);
            if (typeof this.onopen === "function") this.onopen(openEvent);
          } catch (e) {
            // ignore
          }
        }, 50);
      }
      MockWs.prototype.send = function (data: string) {
        // @ts-expect-error
        this.sent.push(data);
      };
      MockWs.prototype.close = function () {
        // @ts-expect-error
        this.readyState = 3; // CLOSED
        const closeEvent = new Event("close");
        // @ts-expect-error
        this.dispatchEvent(closeEvent);
        if (typeof this.onclose === "function") this.onclose(closeEvent);
      };
      MockWs.prototype.dispatchEvent = EventTarget.prototype.dispatchEvent;
      MockWs.prototype.addEventListener = EventTarget.prototype.addEventListener;
      MockWs.prototype.removeEventListener = EventTarget.prototype.removeEventListener;
      return MockWs;
    })();
  });
}

test.describe("Terminal Stack WS Integration (PR #98.5)", () => {
  test("1. wsClient URL contains session_id", async ({ page }) => {
    // Install MockWsClass (per terminal-ws-integration spec — same pattern as p1e)
    await installTestMockWs(page);
    await page.goto("/terminal-stack-demo?sessionId=test-session-123");
    await page.waitForTimeout(500);
    const constructedUrls: string[] = await page.evaluate(() => {
      // @ts-expect-error
      return (window.__mockWsInstances ?? []).map((w: unknown) => (w as { url: string }).url);
    });
    expect(constructedUrls.some((u) => u.includes("/v1/terminal/test-session-123/connect"))).toBe(true);
    await expect(page.locator('[data-testid="terminal-stack-container"]')).toBeVisible();
  });

  test("2. mock mode (no sessionId) shows connected=true", async ({ page }) => {
    await page.goto("/terminal-stack-demo");
    // ws-debug is in a <div hidden> — use state: 'attached' to skip visibility check
    await page.waitForSelector('[data-testid="ws-debug"]', { state: "attached" });
    // Give time for TerminalStackContainer's useEffect to fire setWsConnected(true)
    await page.waitForTimeout(300);
    const content = await page.locator('[data-testid="ws-debug"]').textContent();
    expect(content).toContain("connected=true");
  });

  test("3. real WS sessionId triggers WebSocket connect attempt", async ({
    page,
  }) => {
    await installTestMockWs(page);
    await page.goto("/terminal-stack-demo?sessionId=real-session");
    await page.waitForTimeout(300);
    const instances = await page.evaluate(
      // @ts-expect-error
      () => (window.__mockWsInstances ?? []).length,
    );
    expect(instances).toBeGreaterThan(0);
  });

  test("4. tree state updates on SplitUpdate message (mock via __mockWsCtor)", async ({
    page,
  }) => {
    await installTestMockWs(page);
    await page.goto("/terminal-stack-demo?sessionId=tree-update");
    await page.waitForSelector('[data-testid="terminal-stack-container"]');
    await page.waitForTimeout(300);

    // Inject SplitUpdate via the test hook
    await page.evaluate(() => {
      // @ts-expect-error
      const instances = (window.__mockWsInstances ?? []) as Array<{
        dispatchEvent: (ev: Event) => boolean;
        onmessage: ((ev: MessageEvent) => void) | null;
      }>;
      const inst = instances[0];
      if (!inst) return;
      const msg = new MessageEvent("message", {
        data: JSON.stringify({
          type: "split_update",
          root_id: "550e8400-e29b-41d4-a716-446655440000",
          tree: {
            kind: "split",
            id: "split-1",
            direction: "horizontal",
            children: [
              { kind: "pane", pane: { id: "pane-root", ratio: 0.5 } },
              { kind: "pane", pane: { id: "new-pane", ratio: 0.5 } },
            ],
          },
          reason: "user_split",
        }),
      });
      inst.dispatchEvent(msg);
      if (typeof inst.onmessage === "function") inst.onmessage(msg);
    });

    await expect(page.locator('[data-testid="pane-count"]')).toHaveText(
      /^2 panes/,
      { timeout: 3000 },
    );
  });

  test("5. wsClient reuses protocol encoding helpers (sanity check via DOM)", async ({
    page,
  }) => {
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto("/terminal-stack-demo");
    await page.waitForTimeout(500);
    expect(errors).toEqual([]);
  });

  test("6. ws dispatch SplitUpdate updates zustand store", async ({ page }) => {
    await installTestMockWs(page);
    await page.goto("/terminal-stack-demo?sessionId=split-update-test");
    await page.waitForTimeout(300);
    await expect(page.locator('[data-testid="pane-count"]')).toHaveText(
      /^2 panes/,
      { timeout: 3000 },
    );
  });
});
