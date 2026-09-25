// =====================================================================
// terminal-split.spec.ts — Playwright E2E (per ULYS-223 P1-D §3.7)
// =====================================================================
// 守门: 8+ E2E test (per description §3.7)
// 模式: 跟 frontend/e2e/remote-mobile.spec.ts 同款 (per P1-D)
// =====================================================================

import { test, expect } from "@playwright/test";

test.describe("Terminal Splits (ULYS-223 P1-D)", () => {
  test.beforeEach(async ({ page }) => {
    // 进入 dev server (假设 vite 已经起来在 :3000)
    await page.goto("/terminal-stack-demo");
    // 等待容器渲染
    await page.waitForSelector('[data-testid="terminal-stack-container"]', {
      timeout: 10_000,
    });
  });

  test("1. toolbar renders + initial pane is single", async ({ page }) => {
    await expect(page.getByTestId("terminal-split-toolbar")).toBeVisible();
    await expect(page.getByTestId("pane-count")).toHaveText(/^1 pane/);
    await expect(page.getByTestId("terminal-pane-pane-root")).toBeVisible();
  });

  test("2. click Split Vertical → 2 panes", async ({ page }) => {
    await page.getByTestId("split-vertical-btn").click();
    await expect(page.getByTestId("pane-count")).toHaveText(/^2 panes/);
    // 2 个 pane (root 仍存在 + 新生成 pane)
    const panes = page.getByRole("region");
    await expect(panes).toHaveCount(2);
  });

  test("3. click Split Horizontal → 2 panes", async ({ page }) => {
    await page.getByTestId("split-horizontal-btn").click();
    await expect(page.getByTestId("pane-count")).toHaveText(/^2 panes/);
  });

  test("4. split 3 times → 4 panes", async ({ page }) => {
    await page.getByTestId("split-vertical-btn").click();
    await page.getByTestId("split-horizontal-btn").click();
    await page.getByTestId("split-horizontal-btn").click();
    await expect(page.getByTestId("pane-count")).toHaveText(/^4 panes/);
  });

  test("5. close button disabled when only 1 pane", async ({ page }) => {
    await expect(page.getByTestId("close-pane-btn")).toBeDisabled();
  });

  test("6. close button enabled when panes > 1", async ({ page }) => {
    await page.getByTestId("split-vertical-btn").click();
    await expect(page.getByTestId("close-pane-btn")).toBeEnabled();
  });

  test("7. close pane removes leaf", async ({ page }) => {
    await page.getByTestId("split-vertical-btn").click();
    await expect(page.getByTestId("pane-count")).toHaveText(/^2 panes/);
    await page.getByTestId("close-pane-btn").click();
    await expect(page.getByTestId("pane-count")).toHaveText(/^1 pane/);
  });

  test("8. drag divider callback fires", async ({ page }) => {
    await page.getByTestId("split-vertical-btn").click();
    const divider = page.getByTestId("split-divider-divider-after");
    await expect(divider).toBeVisible();
    // 模拟拖动
    const box = await divider.boundingBox();
    if (!box) throw new Error("divider not found");
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2 + 50, box.y + box.height / 2);
    await page.mouse.up();
    // 验证 divider 仍存在 (per invariant)
    await expect(divider).toBeVisible();
  });
});