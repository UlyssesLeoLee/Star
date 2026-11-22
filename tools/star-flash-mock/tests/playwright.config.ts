import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './',
  testMatch: /.*\.spec\.ts$/,
  fullyParallel: true,
  retries: 0,
  reporter: 'list',
  use: {
    baseURL: 'http://localhost:3002',
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
  // webServer 已禁用：依赖已启动的前端 dev server (http://localhost:3000)
  // 如需自动启动前端，请取消注释下方并设置 PLAYWRIGHT_SKIP_WEBSERVER 未定义
  // webServer: {
  //   command: 'cd D:/Star/frontend && npm run dev',
  //   url: 'http://localhost:3000',
  //   reuseExistingServer: !process.env.CI,
  //   timeout: 180000,
  // },
});
