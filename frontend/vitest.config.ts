// =====================================================================
// vitest.config.ts — W5 + W2 + U5 合并配置
//
// W5 store/toast original. W2 Gantt added esbuild.jsx + setupFiles.
// U5 multica-style redirects added `e2e/**\/*.spec.ts` to the include
// list so frontend/e2e/redirects.spec.ts is picked up by `vitest run`.
// =====================================================================
import { defineConfig } from "vitest/config";
import path from "node:path";

// 环境守卫: 部分 Windows 开发机全局设置了 NODE_ENV=production (非 vitest 语义),
// 会让 react-dom 解析到 production test-utils → "React.act is not a function" 全量秒崩。
// 测试运行器语义必须是 test; CI (fresh shell) 不受影响, 本行只修本地环境污染。
if (process.env.NODE_ENV && process.env.NODE_ENV !== "test") {
  // Object.assign 绕过 @types/node 对 NODE_ENV 的 readonly 声明
  Object.assign(process.env, { NODE_ENV: "test" });
}

/*
@cypher schema=1 source_sha256=77edb1a37fc0bccf58672bc3979cda379ba51831ebc595bc9f3cff4cdd20369f
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
