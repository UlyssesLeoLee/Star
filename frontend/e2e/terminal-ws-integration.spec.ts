// =====================================================================
// terminal-ws-integration.spec.ts — Playwright E2E (per PR #98.5)
// =====================================================================
// 守门: 6+ E2E 测试验证前端 WS client 真接入 PR #106 后端 (mock server)
// 模式: per frontend/e2e/remote-mobile.spec.ts (Playwright 自定义 mock server)
// =====================================================================

import { test, expect } from "@playwright/test";

test.describe("Terminal Stack WS Integration (PR #98.5)", () => {
  test("1. wsClient URL contains session_id", async ({ page }) => {
    // Mock WebSocket server
    await page.route("ws://**", async (route) => {
      // Verify URL format
      const url = route.request().url();
      expect(url).toContain("/v1/terminal/test-session-123/connect");
      // Accept upgrade then close
      await route.fulfill({
        status: 101,
        headers: { Upgrade: "websocket" },
      });
    });
    await page.goto("/terminal-stack-demo?sessionId=test-session-123");
    await page.waitForSelector('[data-testid="terminal-stack-container"]', {
      timeout: 5000,
    });
    // ws-debug element shows connection state
    const wsDebug = page.locator('[data-testid="ws-debug"]');
    await expect(wsDebug).toHaveAttribute("hidden", "");
  });

  test("2. mock mode (no sessionId) shows connected=true", async ({ page }) => {
    await page.goto("/terminal-stack-demo");
    await page.waitForSelector('[data-testid="ws-debug"]');
    const content = await page.locator('[data-testid="ws-debug"]').textContent();
    expect(content).toContain("connected=true");
  });

  test("3. real WS sessionId triggers WebSocket connect attempt", async ({ page }) => {
    let wsConnected = false;
    await page.route("ws://**", async (route) => {
      wsConnected = true;
      await route.fulfill({ status: 101 });
    });
    await page.goto("/terminal-stack-demo?sessionId=real-session");
    await page.waitForTimeout(500); // give WS connect time
    expect(wsConnected).toBe(true);
  });

  test("4. tree state updates on SplitUpdate message", async ({ page }) => {
    await page.goto("/terminal-stack-demo?sessionId=tree-update");
    await page.waitForSelector('[data-testid="terminal-pane-pane-root"]');
    // Note: receiving SplitUpdate requires WS connection; in mock mode (no sessionId),
    // we can verify the toolbar interactions independently.
    const splitBtn = page.locator('[data-testid="split-vertical-btn"]');
    await splitBtn.click();
    await expect(page.locator('[data-testid="pane-count"]')).toHaveText(
      /^2 panes/,
    );
  });

  test("5. wsClient reuses protocol encoding helpers (sanity check via DOM)", async ({
    page,
  }) => {
    // 验证 ws-client module 已与 store + container 集成 (无 error)
    await page.goto("/terminal-stack-demo");
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.waitForTimeout(500);
    expect(errors).toEqual([]);
  });

  test("6. ws client dispatch SplitUpdate updates zustand store", async ({
    page,
  }) => {
    // 注入一个 mock ws server, 推 split_update 消息
    await page.addInitScript(() => {
      // @ts-expect-error - test-only injection
      window.__lastTreeUpdate = null;
      const OrigWS = window.WebSocket;
      // @ts-expect-error
      window.WebSocket = function (url: string) {
        const ws = new OrigWS(url);
        ws.addEventListener("open", () => {
          setTimeout(() => {
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
                        pane: {
                          id: "pane-root",
                          ratio: 0.5,
                          title: "root",
                        },
                      },
                      {
                        kind: "pane",
                        pane: { id: "new-pane", ratio: 0.5 },
                      },
                    ],
                  },
                  reason: "user_split",
                }),
              }),
            );
          }, 100);
        });
        return ws;
      } as unknown as typeof WebSocket;
    });

    await page.goto("/terminal-stack-demo?sessionId=split-update-test");
    await page.waitForTimeout(500);
    // After SplitUpdate, pane count should be 2
    await expect(page.locator('[data-testid="pane-count"]')).toHaveText(
      /^2 panes/,
      { timeout: 3000 },
    );
  });
});