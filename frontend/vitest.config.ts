// =====================================================================
// vitest.config.ts — W5 单元测试配置
// =====================================================================
// jsdom 环境 → 测试 DOM 相关 hook (useBoardSync 等)
// happy-dom / jsdom 二选一;这里选 jsdom 兼容性更广
// =====================================================================
import { defineConfig } from "vitest/config";
import path from "node:path";

export default defineConfig({
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
