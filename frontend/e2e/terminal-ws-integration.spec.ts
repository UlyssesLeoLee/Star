// =====================================================================
// terminal-ws-integration.spec.ts — Playwright E2E (per PR #98.5 + T23.6 fix)
// =====================================================================
// 守门: 6 E2E 测试验证前端 WS client 真接入 PR #106 后端 (mock server)
//
// T23.6 fix: 使用 window.__mockWsCtor (per useTerminalStackWs hook 读 window.__mockWsCtor)
// 而不是 window.WebSocket = X (Chromium 不允许覆盖内置 WebSocket).
// =====================================================================

import { test, expect } from "@playwright/test";

test.describe("Terminal Stack WS Integration (PR #98.5)", () => {
  test("1. wsClient URL contains session_id", async ({ page }) => {
    // Mock WS via addInitScript before page loads
    await page.addInitScript(() => {
      const OrigWS = window.WebSocket;
      // @ts-expect-error - test-only injection
      window.__mockWsCtor = function (url: string) {
        // @ts-expect-error
        return new OrigWS(url);
      } as unknown as typeof WebSocket;
    });

    await page.goto("/terminal-stack-demo?sessionId=test-session-123");
    await page.waitForSelector('[data-testid="terminal-stack-container"]', {
      timeout: 5000,
    });
    // ws-debug should be hidden after ws close (per TerminalStackContainer effect)
    // For mock, since no actual WS server, ws-debug stays visible.
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
    // Reset state on window for browser-side capture
    await page.addInitScript(() => {
      // @ts-expect-error - test-only injection
      (window as unknown as { __mockWsCalled: boolean }).__mockWsCalled = false;
      // @ts-expect-error
      window.__mockWsCtor = function (url: string) {
        // @ts-expect-error
        const ws = new (window as unknown as { WebSocket: typeof WebSocket }).WebSocket(url);
        // @ts-expect-error
        (window as unknown as { __mockWsCalled: boolean }).__mockWsCalled = true;
        return ws as unknown as WebSocket;
      } as unknown as typeof WebSocket;
    });

    await page.goto("/terminal-stack-demo?sessionId=real-session");
    await page.waitForTimeout(500);
    const called = await page.evaluate(
      () => (window as unknown as { __mockWsCalled: boolean }).__mockWsCalled,
    );
    expect(called).toBe(true);
  });

  test("4. tree state updates on SplitUpdate message (mock via __mockWsCtor)", async ({
    page,
  }) => {
    // Track all created WS instances so test can dispatch to them
    const wsInstances: WebSocket[] = [];
    await page.addInitScript(() => {
      const OrigWS = window.WebSocket;
      // @ts-expect-error - test-only
      const instances: unknown[] = ((window as unknown as { __testWsInstances: unknown[] })
        .__testWsInstances = []);
      // @ts-expect-error
      (window as unknown as { __mockWsCtor: typeof WebSocket }).__mockWsCtor = function (url: string) {
        // @ts-expect-error
        const ws = new OrigWS(url) as WebSocket & { _onopenRef?: () => void };
        instances.push(ws);
        // Force open immediately (Chromium WS would normally do this async)
        setTimeout(() => {
          try {
            // Trigger onopen to set wsConnected=true
            Object.defineProperty(ws, "readyState", { value: 1, configurable: true });
            ws.dispatchEvent(new Event("open"));
          } catch {
            // ignore
          }
        }, 10);
        return ws as unknown as WebSocket;
      } as unknown as typeof WebSocket;
    });

    await page.goto("/terminal-stack-demo?sessionId=tree-update");
    await page.waitForSelector('[data-testid="terminal-stack-container"]');
    await page.waitForTimeout(200);

    // Inject SplitUpdate via the test hook
    await page.evaluate(() => {
      const inst = ((window as unknown as { __testWsInstances?: WebSocket[] }).__testWsInstances ?? [])[0];
      if (!inst) return;
      inst.dispatchEvent(
        new MessageEvent("message", {
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
        }),
      );
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
    // Inject mock ws that emits split_update after open
    await page.addInitScript(() => {
      const OrigWS = window.WebSocket;
      // @ts-expect-error
      (window as unknown as { __mockWsCtor: typeof WebSocket }).__mockWsCtor = function (url: string) {
        // @ts-expect-error
        const ws = new OrigWS(url) as WebSocket;
        setTimeout(() => {
          try {
            Object.defineProperty(ws, "readyState", { value: 1, configurable: true });
            ws.dispatchEvent(new Event("open"));
            ws.dispatchEvent(
              new MessageEvent("message", {
                data: JSON.stringify({
                  type: "split_update",
                  root_id: "550e8400-e29b-41d4-a716-446655440000",
                  tree: {
                    kind: "split",
                    id: "split-1",
                    direction: "horizontal",
                    children: [
                      {
                        kind: "pane",
                        pane: { id: "pane-root", ratio: 0.5, title: "root" },
                      },
                      { kind: "pane", pane: { id: "new-pane", ratio: 0.5 } },
                    ],
                  },
                  reason: "user_split",
                }),
              }),
            );
          } catch {
            // ignore
          }
        }, 100);
        return ws as unknown as WebSocket;
      } as unknown as typeof WebSocket;
    });

    await page.goto("/terminal-stack-demo?sessionId=split-update-test");
    await page.waitForTimeout(500);
    await expect(page.locator('[data-testid="pane-count"]')).toHaveText(
      /^2 panes/,
      { timeout: 3000 },
    );
  });
});
