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
      "e2e/remote-mobile.spec.ts", // Playwright-only spec (per 2026-09-01 PHASE-MOBILE-PWA v0.4 mobile viewport e2e)
      "e2e/debug-mobile.spec.ts", // 临时调试 spec, 仅手动跑
      "e2e/canvas-view.spec.ts", // Playwright-only spec (per 2026-09-04 canvas e2e 守门补齐, uses @playwright/test)
      "e2e/canvas-share-export.spec.ts", // Playwright-only spec (per 2026-09-04 canvas Share/Export PNG, uses @playwright/test)
      "e2e/arg-relationships.spec.ts", // Playwright-only spec (ARG.7 8 E2E, uses @playwright/test)
      "e2e/collaboration-5d-lead-coordination.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/error-404-path-coverage.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/five-domain-feedback-loop.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/interaction-expectation-coverage.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/mcp-16-tool-coverage.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/ops-e2e-4-tab.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/ops-e2e-error-6field.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/ops-e2e-i18n.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/streamable-http-reconnect.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/tmo-merge-task-flow.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/uat-100pct-coverage-assertion.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/worktree-canvas.spec.ts", // Playwright-only spec (per ULYS-57 worktree canvas E2E, uses @playwright/test)
      "e2e/uat-3000-restore.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/uat-business-acceptance.spec.ts", // Playwright-only spec, uses @playwright/test
      "e2e/worktree-creation-flow.spec.ts", // Playwright-only spec, uses @playwright/test
    ],
    setupFiles: ["./vitest.setup.ts"],
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
