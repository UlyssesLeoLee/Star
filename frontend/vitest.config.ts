// =====================================================================
// vitest.config.ts — W5 + W2 + U5 合并配置
//
// W5 store/toast original. W2 Gantt added esbuild.jsx + setupFiles.
// U5 multica-style redirects added `e2e/**\/*.spec.ts` to the include
// list so frontend/e2e/redirects.spec.ts is picked up by `vitest run`.
// =====================================================================
import { defineConfig } from "vitest/config";
import path from "node:path";

/*
@cypher schema=1 source_sha256=81ca156151bc9dc6b78290922f73f66ee1aa9c71f0520e60664e99ada2f79997
MERGE (self:File {path:"frontend/vitest.config.ts"})
MERGE (vitest:ExternalService {name:"Vitest"})
MERGE (config:Symbol {id:"frontend/vitest.config.ts::default", kind:"config"})
MERGE (vitestConfig:ExternalService {name:"vitest/config"})
MERGE (nodePath:ExternalService {name:"node:path"})
MERGE (redirectSpec:File {path:"frontend/e2e/redirects.spec.ts"})
MERGE (self)-[:DEFINES]->(config)
MERGE (self)-[:IMPORTS]->(vitestConfig)
MERGE (self)-[:IMPORTS]->(nodePath)
MERGE (self)-[:CONFIGURES]->(vitest)
MERGE (self)-[:CONFIGURES]->(redirectSpec)
@endcypher
*/

export default defineConfig({
  esbuild: {
    jsx: "automatic",
  },
  test: {
    environment: "jsdom",
    globals: true,
    // Playwright owns the browser suite. Keep only the standalone Vitest
    // redirect contract that currently lives under e2e/ for legacy reasons.
    include: ["src/**/*.{test,spec}.{ts,tsx}", "e2e/redirects.spec.ts"],
    exclude: [
      "node_modules",
      ".next",
      "**/node_modules/**",
    ],
    setupFiles: ["./vitest.setup.ts"],
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
