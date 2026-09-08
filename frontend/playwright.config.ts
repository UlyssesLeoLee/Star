import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  // vitest test misplaced under e2e/, not a Playwright spec
  testIgnore: "**/redirects.spec.ts",
  fullyParallel: true,
  retries: 0,
  reporter: "list",
  use: {
    baseURL: "http://localhost:3000",
    trace: "on-first-retry",
  },
  // §4.1 跨浏览器覆盖 (per TEST-DESIGN-OPS-001 v0.2 §4.1 + 5-LEVEL-FULL brief §2.1)
  // 3 projects: chromium (默认) + firefox + webkit
  // 缺标: 跨浏览器 binary 下载 (per brief §6 缺口 #1 [M], CI 跑, 本地 MVP chromium-only)
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
  ],
  webServer: {
    command: "npm run dev",
    url: "http://localhost:3000",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
