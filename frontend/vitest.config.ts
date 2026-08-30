// =====================================================================
// vitest.config.ts — W5 + W2 + U5 合并配置
//
// W5 store/toast original. W2 Gantt added esbuild.jsx + setupFiles.
// U5 multica-style redirects added `e2e/**\/*.spec.ts` to the include
// list so frontend/e2e/redirects.spec.ts is picked up by `vitest run`.
// =====================================================================
import { defineConfig } from "vitest/config";
import path from "node:path";

export default defineConfig({
  esbuild: {
    jsx: "automatic",
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: [
      "src/**/*.{test,spec}.{ts,tsx}",
      "e2e/**/*.{test,spec}.{ts,tsx}",
    ],
    exclude: [
      "node_modules",
      ".next",
      "**/node_modules/**",
      "e2e/cross-domain-5b.spec.ts", // Playwright-only spec (uses @playwright/test), not a vitest test
    ],
    setupFiles: ["./vitest.setup.ts"],
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
