// =====================================================================
// Vitest config (W2: Gantt 模块测试)
// - jsdom: React 组件测试需要 DOM
// - exclude: Next.js build / .next / node_modules
// - test 文件: src/**/*.test.{ts,tsx}
// - esbuild jsx: "automatic" → 不需要 import React (与 Next.js JSX runtime 一致)
// =====================================================================
import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  esbuild: {
    jsx: "automatic",
  },
  test: {
    environment: "jsdom",
    globals: false,
    include: ["src/**/*.test.{ts,tsx}"],
    exclude: ["node_modules", ".next", "**/node_modules/**"],
    setupFiles: ["./vitest.setup.ts"],
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
